//! `mock_adt!` — procedural macro that takes a crate identifier and module definitions, then
//! generates the full mock infrastructure for each struct: the mock struct (with PhantomData for
//! private fields), Drop impl, Predicate/Expectation/Return wrappers, mock method bodies,
//! on_call/expect helpers, constructors, and sequence helpers.
//!
//! # Input Syntax
//!
//! ```text
//! mock_adt! {
//!     krate,   // crate name for mock ID prefix
//!
//!     mod Mod {
//!         pub struct Example {
//!             a: f32,
//!             pub b: f32,
//!         }
//!
//!         pub trait ExTrait {
//!             fn meth2(&mut self, text: String) -> bool;
//!         }
//!         impl ExTrait for Example {}
//!
//!         trait From<(f32, f32)> {
//!             fn from(value: (f32, f32)) -> Self;
//!         }
//!         impl From for Example {}
//!
//!         impl Example {
//!             fn meth1(&self, a: f32, b: f32) -> usize;
//!             fn new(a: f32, b: f32) -> Self;
//!         }
//!     }
//!
//!     mod Other {
//!         mod Nested {
//!             // structs, traits, impls...
//!         }
//!     }
//! }
//! ```

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{
    Ident, Token, Type, Visibility,
    braced,
    parse::{Parse, ParseStream},
    token,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Data model
// ═══════════════════════════════════════════════════════════════════════════════

/// A field in the input struct definition.
#[derive(Clone)]
pub struct StructField {
    pub vis: Visibility,
    pub name: Ident,
    pub ty: Type,
}

/// A method signature in a trait or impl block.
#[derive(Clone)]
pub struct MethodSig {
    pub vis: Visibility,
    pub name: Ident,
    pub receiver: Receiver,
    pub params: Vec<(Ident, Type)>,
    pub ret_type: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Receiver {
    None,           // static / constructor
    Ref,            // &self
    RefMut,         // &mut self
}

/// A trait definition with its methods.
#[derive(Clone)]
pub struct TraitDef {
    pub vis: Visibility,
    pub name: Ident,
    pub generics: Vec<Type>,
    pub methods: Vec<MethodSig>,
}

/// A trait impl for the struct.
#[derive(Clone, Debug)]
pub struct TraitImpl {
    pub trait_name: Ident,
}

/// An inherent impl block with method signatures.
#[derive(Clone)]
pub struct InherentImpl {
    pub methods: Vec<MethodSig>,
}

/// The full input to `mock_adt!`.
#[derive(Clone)]
pub struct MockAdtInput {
    pub krate: Ident,
    pub modules: Vec<ModuleDef>,
}

/// A module definition, which can contain items and nested modules.
#[derive(Clone)]
pub struct ModuleDef {
    pub name: Ident,
    pub items: ModuleItems,
    pub children: Vec<ModuleDef>,
}

/// A variant in an enum definition.
#[derive(Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub fields: Vec<Type>,  // empty for unit variants
}

/// Items within a module (structs, traits, impls).
#[derive(Clone)]
pub struct ModuleItems {
    pub vis: Visibility,
    pub struct_name: Option<Ident>,
    pub fields: Vec<StructField>,
    pub enum_name: Option<Ident>,
    pub variants: Vec<EnumVariant>,
    pub traits: Vec<TraitDef>,
    pub trait_impls: Vec<TraitImpl>,
    pub inherent_impl: Option<InherentImpl>,
}

/// A flattened struct entry with its full path computed from crate + module nesting.
/// Used internally for code generation.
#[derive(Clone)]
struct FlattenedStruct {
    path: syn::Path,
    vis: Visibility,
    struct_name: Ident,
    fields: Vec<StructField>,
    traits: Vec<TraitDef>,
    trait_impls: Vec<TraitImpl>,
    inherent_impl: Option<InherentImpl>,
    /// Whether all fields are public. If true, no `adt_mock_id` field is added
    /// and mock IDs are derived solely from path+method (no instance suffix).
    all_public: bool,
}

/// A flattened enum entry with its full path computed from crate + module nesting.
/// Used internally for code generation.
#[derive(Clone)]
struct FlattenedEnum {
    path: syn::Path,
    vis: Visibility,
    enum_name: Ident,
    variants: Vec<EnumVariant>,
    traits: Vec<TraitDef>,
    trait_impls: Vec<TraitImpl>,
    inherent_impl: Option<InherentImpl>,
    /// Whether this enum is instance-trackable (can have per-instance mock IDs)
    trackable: bool,
    /// For trackable enums: maps each variant index to the index of the first trackable field within it
    trackable_field_indices: Vec<Option<usize>>,
    /// For trackable enums: whether the trackable field at each variant is an enum (true) or struct (false)
    trackable_field_is_enum: Vec<bool>,
}

/// A flattened trait entry with its full path computed from crate + module nesting.
/// Used internally for code generation.
#[derive(Clone)]
struct FlattenedTrait {
    path: syn::Path,
    vis: Visibility,
    name: Ident,
    generics: Vec<Type>,
    /// Only the public methods (Receiver != None, and vis is pub)
    methods: Vec<MethodSig>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Parsing
// ═══════════════════════════════════════════════════════════════════════════════

impl Parse for MockAdtInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // 1. Parse the crate identifier
        let krate: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // 2. Parse module definitions
        let mut modules = Vec::new();
        while !input.is_empty() {
            modules.push(parse_module_def(input)?);
        }

        if modules.is_empty() {
            return Err(input.error("expected at least one `mod` block"));
        }

        Ok(MockAdtInput { krate, modules })
    }
}

/// Parse a `mod Name { ... }` block, which can contain items and nested modules.
fn parse_module_def(input: ParseStream) -> syn::Result<ModuleDef> {
    // Expect `mod`
    input.parse::<Token![mod]>()?;
    let name: Ident = input.parse()?;
    let content;
    braced!(content in input);

    let mut items = ModuleItems {
        vis: Visibility::Inherited,
        struct_name: None,
        fields: Vec::new(),
        enum_name: None,
        variants: Vec::new(),
        traits: Vec::new(),
        trait_impls: Vec::new(),
        inherent_impl: None,
    };
    let mut children = Vec::new();

    // Parse contents: items and nested modules
    while !content.is_empty() {
        if content.peek(Token![mod]) {
            // Nested module
            children.push(parse_module_def(&content)?);
        } else {
            // Parse an item (struct, trait, impl)
            parse_module_item(&content, &mut items)?;
        }
    }

    Ok(ModuleDef { name, items, children })
}

/// Parse a single item within a module (struct, trait, or impl).
fn parse_module_item(input: ParseStream, items: &mut ModuleItems) -> syn::Result<()> {
    let item_vis: Visibility = input.parse()?;

    if input.peek(Token![struct]) {
        input.parse::<Token![struct]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let parsed_fields = parse_struct_fields(&content)?;
        items.vis = item_vis;
        items.struct_name = Some(name);
        items.fields = parsed_fields;
    } else if input.peek(Token![enum]) {
        input.parse::<Token![enum]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let parsed_variants = parse_enum_variants(&content)?;
        items.vis = item_vis;
        items.enum_name = Some(name);
        items.variants = parsed_variants;
    } else if input.peek(Token![trait]) {
        input.parse::<Token![trait]>()?;
        let trait_name: Ident = input.parse()?;

        // Optionally parse generic type args: <Type, Type, ...>
        let mut generics = Vec::new();
        if input.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            loop {
                if input.peek(Token![>]) {
                    break;
                }
                let ty: Type = input.parse()?;
                generics.push(ty);
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
            input.parse::<Token![>]>()?;
        }

        let content;
        braced!(content in input);
        let methods = parse_method_sigs(&content)?;
        items.traits.push(TraitDef {
            vis: item_vis,
            name: trait_name,
            generics,
            methods,
        });
    } else if input.peek(Token![impl]) {
        input.parse::<Token![impl]>()?;

        let first_ident: Ident = input.parse()?;

        if input.peek(Token![for]) {
            // `impl TraitName for StructName { }`
            input.parse::<Token![for]>()?;
            let _struct_ref: Ident = input.parse()?;
            let content;
            braced!(content in input);
            let _ = content.parse::<TokenStream>()?;
            items.trait_impls.push(TraitImpl { trait_name: first_ident });
        } else if input.peek(token::Brace) {
            // inherent impl: `impl Example { fn meth1(...) -> ...; }`
            let content;
            braced!(content in input);
            let methods = parse_method_sigs(&content)?;
            items.inherent_impl = Some(InherentImpl { methods });
        } else {
            return Err(input.error("expected `{` or `for` after impl identifier"));
        }
    } else if input.peek(Token![mod]) {
        return Err(input.error("nested `mod` must appear before or after items, not interleaved with visibility"));
    } else {
        return Err(input.error("expected `struct`, `trait`, `impl`, or `mod`"));
    }

    Ok(())
}

/// Parse struct fields: `name: Type` or `pub name: Type`
fn parse_struct_fields(input: ParseStream) -> syn::Result<Vec<StructField>> {
    let mut fields = Vec::new();
    while !input.is_empty() {
        let field_vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        fields.push(StructField { vis: field_vis, name, ty });
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(fields)
}

/// Parse enum variants: `Variant1(Type1, Type2), Variant2, Variant3(Type3)`
fn parse_enum_variants(input: ParseStream) -> syn::Result<Vec<EnumVariant>> {
    let mut variants = Vec::new();
    while !input.is_empty() {
        let name: Ident = input.parse()?;
        let mut fields = Vec::new();
        if input.peek(token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            while !content.is_empty() {
                let ty: Type = content.parse()?;
                fields.push(ty);
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
        }
        variants.push(EnumVariant { name, fields });
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(variants)
}

/// Parse method signatures: `fn name(&self, ...) -> RetType;`
fn parse_method_sigs(input: ParseStream) -> syn::Result<Vec<MethodSig>> {
    let mut methods = Vec::new();
    while !input.is_empty() {
        // optional visibility
        let vis: Visibility = input.parse()?;
        input.parse::<Token![fn]>()?;
        let name: Ident = input.parse()?;

        // Parse params
        let params_content;
        syn::parenthesized!(params_content in input);

        let mut receiver = Receiver::None;
        let mut params: Vec<(Ident, Type)> = Vec::new();

        if !params_content.is_empty() {
            // Check for self receiver
            if params_content.peek(Token![&]) {
                let fork = params_content.fork();
                fork.parse::<Token![&]>()?;
                if fork.peek(Token![mut]) && fork.peek2(Token![self]) {
                    // &mut self
                    params_content.parse::<Token![&]>()?;
                    params_content.parse::<Token![mut]>()?;
                    params_content.parse::<Token![self]>()?;
                    receiver = Receiver::RefMut;
                    if params_content.peek(Token![,]) {
                        params_content.parse::<Token![,]>()?;
                    }
                } else if fork.peek(Token![self]) {
                    // &self
                    params_content.parse::<Token![&]>()?;
                    params_content.parse::<Token![self]>()?;
                    receiver = Receiver::Ref;
                    if params_content.peek(Token![,]) {
                        params_content.parse::<Token![,]>()?;
                    }
                }
            } else if params_content.peek(Token![self]) {
                params_content.parse::<Token![self]>()?;
                receiver = Receiver::Ref; // treat bare `self` as owned, but for mock purposes &self
                if params_content.peek(Token![,]) {
                    params_content.parse::<Token![,]>()?;
                }
            }

            // Parse remaining typed params
            while !params_content.is_empty() {
                let param_name: Ident = params_content.parse()?;
                params_content.parse::<Token![:]>()?;
                let param_ty: Type = params_content.parse()?;
                params.push((param_name, param_ty));
                if params_content.peek(Token![,]) {
                    params_content.parse::<Token![,]>()?;
                }
            }
        }

        // Return type
        let ret_type: Type = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            input.parse()?
        } else {
            syn::parse_quote!(())
        };

        // Consume trailing semicolon or empty braces
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        } else if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            let _ = content.parse::<TokenStream>()?;
        }

        methods.push(MethodSig { vis, name, receiver, params, ret_type });
    }
    Ok(methods)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Code Generation
// ═══════════════════════════════════════════════════════════════════════════════

/// A constructor method, possibly from a trait impl.
#[derive(Clone)]
struct ConstructorMethod {
    sig: MethodSig,
    /// If this constructor comes from a trait impl (e.g. From), store the trait name and generics.
    trait_name: Option<Ident>,
    /// Generic type args from the trait definition (e.g. `(f32, f32)` for `From<(f32, f32)>`).
    trait_generics: Vec<Type>,
}

/// Categorize methods into constructors vs mockable methods.
#[derive(Clone)]
struct ClassifiedMethods {
    /// Constructor methods (return Self, no self receiver)
    constructors: Vec<ConstructorMethod>,
    /// Regular methods that get mocked
    mockable: Vec<MockableMethod>,
}

#[derive(Clone)]
struct MockableMethod {
    sig: MethodSig,
    /// The trait this method comes from, if any
    trait_name: Option<Ident>,
}

impl MockableMethod {
    /// Generate the mock_id prefix string for this method.
    fn mock_id_prefix(&self, path: &syn::Path, struct_name: &Ident) -> String {
        let path_str = path.segments.iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("_");
        match &self.trait_name {
            Some(trait_name) => format!("{}_{}_{}_{}", path_str, struct_name, trait_name, self.sig.name),
            None => format!("{}_{}_{}", path_str, struct_name, self.sig.name),
        }
    }

    /// Generate the suffix for wrapper struct names.
    fn wrapper_suffix(&self, struct_name: &Ident) -> Ident {
        let method_capitalized = capitalize_first(&self.sig.name.to_string());
        match &self.trait_name {
            Some(trait_name) => format_ident!("{}Impl{}{}", struct_name, trait_name, method_capitalized),
            None => format_ident!("{}{}", struct_name, method_capitalized),
        }
    }

    /// The input type tuple: (*const StructName, param1_ty, param2_ty, ...)
    fn input_type_tuple(&self, struct_name: &Ident) -> TokenStream {
        let param_types: Vec<&Type> = self.sig.params.iter().map(|(_, ty)| ty).collect();
        quote! { (*const #struct_name, #(#param_types),*) }
    }

    /// The return type
    fn ret_type(&self) -> &Type {
        &self.sig.ret_type
    }
}

/// Capitalize the first character of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

pub fn expand_mock_adt(input: MockAdtInput) -> TokenStream {
    // Flatten the module tree into individual structs, enums, and traits with computed paths
    let (flattened_structs, mut flattened_enums, flattened_traits) = flatten_modules(&input.krate, &input.modules);

    // Build trackability map for structs
    let mut trackable_types: HashMap<String, bool> = HashMap::new();
    for entry in &flattened_structs {
        trackable_types.insert(entry.struct_name.to_string(), !entry.all_public);
    }

    // Resolve enum trackability using fixpoint iteration
    resolve_enum_trackability(&mut flattened_enums, &mut trackable_types);

    let mut all_output = TokenStream::new();
    for entry in &flattened_structs {
        all_output.extend(expand_single_struct(entry));
    }
    for entry in &flattened_enums {
        all_output.extend(expand_single_enum(entry, &trackable_types));
    }
    for entry in &flattened_traits {
        all_output.extend(expand_trait_mock(entry));
    }
    all_output
}

/// Extract the last ident segment from a syn::Type path.
/// For `foo::bar::Baz`, returns `Baz`. For non-path types, returns None.
fn type_last_ident(ty: &Type) -> Option<Ident> {
    if let Type::Path(type_path) = ty {
        type_path.path.segments.last().map(|seg| seg.ident.clone())
    } else {
        None
    }
}

/// Resolve enum trackability using fixpoint iteration.
/// An enum is trackable if:
/// - It has no unit variants (all variants have at least one field)
/// - Every variant has at least one field whose type is trackable
fn resolve_enum_trackability(
    enums: &mut Vec<FlattenedEnum>,
    trackable_types: &mut HashMap<String, bool>,
) {
    // Start optimistic: all enums are trackable
    for e in enums.iter() {
        trackable_types.insert(e.enum_name.to_string(), true);
    }

    // Fixpoint iteration
    loop {
        let mut changed = false;
        for e in enums.iter() {
            let name = e.enum_name.to_string();
            let currently_trackable = *trackable_types.get(&name).unwrap_or(&false);

            // Check if enum is still trackable
            let still_trackable = e.variants.iter().all(|v| {
                // Unit variants make enum non-trackable
                if v.fields.is_empty() {
                    return false;
                }
                // At least one field must be trackable
                v.fields.iter().any(|field_ty| {
                    if let Some(ident) = type_last_ident(field_ty) {
                        *trackable_types.get(&ident.to_string()).unwrap_or(&false)
                    } else {
                        false
                    }
                })
            });

            if currently_trackable != still_trackable {
                trackable_types.insert(name, still_trackable);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // Set trackable and compute trackable_field_indices for each enum
    let enum_names: Vec<String> = enums.iter().map(|en| en.enum_name.to_string()).collect();
    for e in enums.iter_mut() {
        let name = e.enum_name.to_string();
        e.trackable = *trackable_types.get(&name).unwrap_or(&false);

        if e.trackable {
            e.trackable_field_indices = e.variants.iter().map(|v| {
                v.fields.iter().position(|field_ty| {
                    if let Some(ident) = type_last_ident(field_ty) {
                        *trackable_types.get(&ident.to_string()).unwrap_or(&false)
                    } else {
                        false
                    }
                })
            }).collect();

            // Determine whether each variant's trackable field is an enum or struct
            e.trackable_field_is_enum = e.variants.iter().zip(e.trackable_field_indices.iter()).map(|(v, idx)| {
                if let Some(field_idx) = idx {
                    if let Some(ident) = type_last_ident(&v.fields[*field_idx]) {
                        enum_names.contains(&ident.to_string())
                    } else {
                        false
                    }
                } else {
                    false
                }
            }).collect();
        } else {
            e.trackable_field_indices = vec![None; e.variants.len()];
            e.trackable_field_is_enum = vec![false; e.variants.len()];
        }
    }
}

/// Recursively flatten the module tree into FlattenedStructs, FlattenedEnums, and FlattenedTraits with full paths.
fn flatten_modules(krate: &Ident, modules: &[ModuleDef]) -> (Vec<FlattenedStruct>, Vec<FlattenedEnum>, Vec<FlattenedTrait>) {
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut traits = Vec::new();
    for module in modules {
        let mut path_segments = vec![krate.clone()];
        flatten_module_recursive(module, &mut path_segments, &mut structs, &mut enums, &mut traits);
    }
    (structs, enums, traits)
}

fn flatten_module_recursive(
    module: &ModuleDef,
    path_segments: &mut Vec<Ident>,
    structs: &mut Vec<FlattenedStruct>,
    enums: &mut Vec<FlattenedEnum>,
    traits: &mut Vec<FlattenedTrait>,
) {
    path_segments.push(module.name.clone());

    // If this module has a struct definition, produce a FlattenedStruct
    if let Some(ref struct_name) = module.items.struct_name {
        let path = build_path(path_segments);
        let all_public = module.items.fields.iter().all(|f| matches!(f.vis, Visibility::Public(_)));
        structs.push(FlattenedStruct {
            path,
            vis: module.items.vis.clone(),
            struct_name: struct_name.clone(),
            fields: module.items.fields.clone(),
            traits: module.items.traits.clone(),
            trait_impls: module.items.trait_impls.clone(),
            inherent_impl: module.items.inherent_impl.clone(),
            all_public,
        });
    }

    // If this module has an enum definition, produce a FlattenedEnum
    if let Some(ref enum_name) = module.items.enum_name {
        let path = build_path(path_segments);
        enums.push(FlattenedEnum {
            path,
            vis: module.items.vis.clone(),
            enum_name: enum_name.clone(),
            variants: module.items.variants.clone(),
            traits: module.items.traits.clone(),
            trait_impls: module.items.trait_impls.clone(),
            inherent_impl: module.items.inherent_impl.clone(),
            trackable: false,  // resolved later
            trackable_field_indices: Vec::new(),  // resolved later
            trackable_field_is_enum: Vec::new(),  // resolved later
        });
    }

    // Collect public traits that are NOT only used as struct/enum trait impls
    // but stand alone as mockable trait entities
    for trait_def in &module.items.traits {
        if matches!(trait_def.vis, Visibility::Public(_)) {
            let path = build_path(path_segments);
            // Filter to only public methods with a receiver
            let public_methods: Vec<MethodSig> = trait_def.methods.iter()
                .filter(|m| matches!(m.vis, Visibility::Public(_)) && m.receiver != Receiver::None)
                .cloned()
                .collect();
            if !public_methods.is_empty() {
                traits.push(FlattenedTrait {
                    path,
                    vis: trait_def.vis.clone(),
                    name: trait_def.name.clone(),
                    generics: trait_def.generics.clone(),
                    methods: public_methods,
                });
            }
        }
    }

    // Recurse into child modules
    for child in &module.children {
        flatten_module_recursive(child, path_segments, structs, enums, traits);
    }

    path_segments.pop();
}

/// Build a `syn::Path` from a list of idents.
fn build_path(segments: &[Ident]) -> syn::Path {
    let segs: Vec<syn::PathSegment> = segments.iter().map(|id| {
        syn::PathSegment {
            ident: id.clone(),
            arguments: syn::PathArguments::None,
        }
    }).collect();
    syn::Path {
        leading_colon: None,
        segments: segs.into_iter().collect(),
    }
}

/// Generate all code for a single struct entry.
fn expand_single_struct(entry: &FlattenedStruct) -> TokenStream {
    let struct_name = &entry.struct_name;

    // Classify all methods
    let classified = classify_methods(entry);

    // Generate each piece
    let struct_def = gen_struct_def(entry);
    let drop_impl = gen_drop_impl(entry, &classified);
    let wrapper_structs = gen_wrapper_structs(struct_name, &classified);
    let return_from_fn_impls = gen_return_from_fn(struct_name, &classified);
    let predicate_from_fn_impls = gen_predicate_from_fn(entry, struct_name, &classified);
    let mock_methods = gen_mock_method_bodies(entry, struct_name, &classified);
    let on_call_methods = gen_on_call_methods(entry, struct_name, &classified);
    let create_predicate_methods = gen_create_predicate_methods(entry, struct_name, &classified);
    let times_methods = gen_times_methods(entry, struct_name, &classified);
    let expect_methods = gen_expect_methods(entry, struct_name, &classified);
    let (inherent_constructors, trait_constructor_impls) = gen_constructors(entry, &classified);
    let sequence_helpers = gen_sequence_helpers(entry, struct_name, &classified);

    quote! {
        #struct_def
        #drop_impl
        #wrapper_structs
        #return_from_fn_impls
        #predicate_from_fn_impls
        impl #struct_name {
            #mock_methods
            #on_call_methods
            #create_predicate_methods
            #times_methods
            #expect_methods
            #inherent_constructors
            #sequence_helpers
        }
        #trait_constructor_impls
    }
}

/// Classify methods from the input into constructors vs mockable.
fn classify_methods(input: &FlattenedStruct) -> ClassifiedMethods {
    let mut constructors = Vec::new();
    let mut mockable = Vec::new();

    // Inherent impl methods
    if let Some(ref inh) = input.inherent_impl {
        for method in &inh.methods {
            if method.receiver == Receiver::None {
                // Constructor (returns Self)
                constructors.push(ConstructorMethod {
                    sig: method.clone(),
                    trait_name: None,
                    trait_generics: Vec::new(),
                });
            } else {
                mockable.push(MockableMethod {
                    sig: method.clone(),
                    trait_name: None,
                });
            }
        }
    }

    // Trait methods — only include public methods
    for trait_impl in &input.trait_impls {
        // Find the matching trait definition
        if let Some(trait_def) = input.traits.iter().find(|t| t.name == trait_impl.trait_name) {
            for method in &trait_def.methods {
                // Skip private trait methods (not marked pub)
                if !matches!(method.vis, Visibility::Public(_)) {
                    continue;
                }
                if method.receiver == Receiver::None {
                    // Constructor from a trait (e.g. From::from)
                    constructors.push(ConstructorMethod {
                        sig: method.clone(),
                        trait_name: Some(trait_impl.trait_name.clone()),
                        trait_generics: trait_def.generics.clone(),
                    });
                } else {
                    mockable.push(MockableMethod {
                        sig: method.clone(),
                        trait_name: Some(trait_impl.trait_name.clone()),
                    });
                }
            }
        }
    }

    ClassifiedMethods { constructors, mockable }
}

// ─── Struct definition ───────────────────────────────────────────────────────

fn gen_struct_def(input: &FlattenedStruct) -> TokenStream {
    let vis = &input.vis;
    let struct_name = &input.struct_name;

    let fields: Vec<TokenStream> = input.fields.iter().map(|f| {
        let name = &f.name;
        let ty = &f.ty;
        match &f.vis {
            Visibility::Public(_) => quote! { pub #name: #ty },
            _ => quote! { #name: std::marker::PhantomData<#ty> },
        }
    }).collect();

    if input.all_public {
        quote! {
            #vis struct #struct_name {
                #(#fields,)*
            }
        }
    } else {
        quote! {
            #vis struct #struct_name {
                #(#fields,)*
                adt_mock_id: context::AdtMockId,
            }
        }
    }
}

// ─── Drop impl ──────────────────────────────────────────────────────────────

fn gen_drop_impl(input: &FlattenedStruct, classified: &ClassifiedMethods) -> TokenStream {
    let struct_name = &input.struct_name;
    let all_public = input.all_public;

    // Generate mock_id bindings for each mockable method
    let mock_id_bindings: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let prefix = m.mock_id_prefix(&input.path, struct_name);
        let var_name = format_ident!("{}_mock_id", m.sig.name);
        if all_public {
            quote! {
                let #var_name = context::MockId::new(#prefix);
            }
        } else {
            quote! {
                let #var_name = context::MockId::new(format!("{}{}", #prefix, self.adt_mock_id.0));
            }
        }
    }).collect();

    // Generate drop_predicate helper functions for each method
    let drop_predicate_fns: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let fn_name = format_ident!("drop_predicate_{}", m.sig.name);
        let input_tuple = m.input_type_tuple(struct_name);
        quote! {
            unsafe fn #fn_name(pred: context::Predicate) {
                match pred.kind {
                    context::new_expectations::PredicateKind::Single(single) => {
                        unsafe { single.condition.id_drop::<(#input_tuple)>() };
                    }
                    context::new_expectations::PredicateKind::And(children)
                    | context::new_expectations::PredicateKind::Or(children)
                    | context::new_expectations::PredicateKind::Xor(children) => {
                        for child in children {
                            unsafe { #fn_name(child) };
                        }
                    }
                    context::new_expectations::PredicateKind::Not(inner)
                    | context::new_expectations::PredicateKind::After { then: inner, .. } => {
                        unsafe { #fn_name(*inner) };
                    }
                    context::new_expectations::PredicateKind::Times { inner, .. } => {
                        unsafe { #fn_name(*inner) };
                    }
                }
            }
        }
    }).collect();

    // Generate the cleanup blocks for each method
    let cleanup_blocks: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let var_name = format_ident!("{}_mock_id", m.sig.name);
        let fn_name = format_ident!("drop_predicate_{}", m.sig.name);
        let input_tuple = m.input_type_tuple(struct_name);
        let ret_type = m.ret_type();
        quote! {
            if let Some(expectations) = cp.expectations.remove(&#var_name) {
                for exp in expectations {
                    if let Some(ret) = exp.return_val {
                        unsafe {
                            ret.id_drop::<(#input_tuple), #ret_type>();
                        }
                    }
                    if let Some(pred) = cp.arena.take(exp.predicate) {
                        unsafe {
                            #fn_name(pred);
                        }
                    }
                }
            }
        }
    }).collect();

    quote! {
        impl Drop for #struct_name {
            fn drop(&mut self) {
                #(#mock_id_bindings)*
                #(#drop_predicate_fns)*

                context::active_or_latest_checkpoint_mut(|cp| {
                    #(#cleanup_blocks)*
                });
            }
        }
    }
}

// ─── Wrapper structs ─────────────────────────────────────────────────────────

fn gen_wrapper_structs(struct_name: &Ident, classified: &ClassifiedMethods) -> TokenStream {
    let wrappers: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let suffix = m.wrapper_suffix(struct_name);
        let pred_name = format_ident!("Predicate{}", suffix);
        let exp_name = format_ident!("Expectation{}", suffix);
        let ret_name = format_ident!("Return{}", suffix);
        quote! {
            pub struct #pred_name(context::Predicate);
            pub struct #exp_name(context::Expectation);
            pub struct #ret_name(context::ReturnValDoublePointer);
        }
    }).collect();

    quote! { #(#wrappers)* }
}

// ─── Return::from_fn ─────────────────────────────────────────────────────────

fn gen_return_from_fn(struct_name: &Ident, classified: &ClassifiedMethods) -> TokenStream {
    let impls: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let suffix = m.wrapper_suffix(struct_name);
        let ret_name = format_ident!("Return{}", suffix);
        let input_tuple = m.input_type_tuple(struct_name);
        let ret_type = m.ret_type();

        // Generate closure params for destructuring
        let param_count = m.sig.params.len() + 1; // +1 for *const Self
        let param_names: Vec<Ident> = (0..param_count)
            .map(|i| format_ident!("_{}", i))
            .collect();

        // Generate the closure parameter types for the public API
        let closure_param_types: Vec<TokenStream> = std::iter::once(quote! { *const #struct_name })
            .chain(m.sig.params.iter().map(|(_, ty)| quote! { #ty }))
            .collect();

        quote! {
            impl #ret_name {
                pub fn from_fn(closure: impl Fn(#(#closure_param_types),*) -> #ret_type + 'static) -> Self {
                    Self(context::ReturnValDoublePointer::from_fn::<
                        (#input_tuple),
                        #ret_type,
                    >(Box::new(move |(#(#param_names,)*)| closure(#(#param_names),*))))
                }
            }
        }
    }).collect();

    quote! { #(#impls)* }
}

// ─── Predicate::from_fn ──────────────────────────────────────────────────────

fn gen_predicate_from_fn(input: &FlattenedStruct, struct_name: &Ident, classified: &ClassifiedMethods) -> TokenStream {
    let impls: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let suffix = m.wrapper_suffix(struct_name);
        let pred_name = format_ident!("Predicate{}", suffix);
        let mock_id_prefix = m.mock_id_prefix(&input.path, struct_name);
        let input_tuple = m.input_type_tuple(struct_name);

        // Generate closure parameter types
        let closure_param_types: Vec<TokenStream> = std::iter::once(quote! { *const #struct_name })
            .chain(m.sig.params.iter().map(|(_, ty)| quote! { #ty }))
            .collect();

        // For String params, clone them
        let closure_args: Vec<TokenStream> = std::iter::once(quote! { input.0 })
            .chain(m.sig.params.iter().enumerate().map(|(i, (_, ty))| {
                let idx = syn::Index::from(i + 1);
                let ty_str = quote! { #ty }.to_string();
                if ty_str == "String" {
                    quote! { input.#idx.clone() }
                } else {
                    quote! { input.#idx }
                }
            }))
            .collect();

        quote! {
            impl #pred_name {
                pub fn from_fn(
                    closure: impl Fn(#(#closure_param_types),*) -> context::errors::PredicateResult<()> + 'static,
                ) -> Self {
                    let mock_id = context::MockId::new(#mock_id_prefix);
                    let cond = context::ConditionDoublePointer::from_fn::<(#input_tuple)>(Box::new(
                        move |input: &(#input_tuple)| closure(#(#closure_args),*),
                    ));
                    Self(context::Predicate::create_single::<(#input_tuple)>(&mock_id, cond))
                }
            }
        }
    }).collect();

    quote! { #(#impls)* }
}

// ─── Mock method bodies ──────────────────────────────────────────────────────

fn gen_mock_method_bodies(input: &FlattenedStruct, struct_name: &Ident, classified: &ClassifiedMethods) -> TokenStream {
    let all_public = input.all_public;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let mock_id_prefix = m.mock_id_prefix(&input.path, struct_name);
        let input_tuple = m.input_type_tuple(struct_name);
        let ret_type = m.ret_type();
        let name_str = name.to_string();

        let receiver = match m.sig.receiver {
            Receiver::Ref => quote! { &self },
            Receiver::RefMut => quote! { &mut self },
            Receiver::None => quote! {},
        };

        let params: Vec<TokenStream> = m.sig.params.iter().map(|(name, ty)| {
            quote! { #name: #ty }
        }).collect();

        let param_names: Vec<&Ident> = m.sig.params.iter().map(|(name, _)| name).collect();

        let receiver_comma = if m.sig.receiver != Receiver::None && !params.is_empty() {
            quote! { , }
        } else {
            quote! {}
        };

        let panic_msg = format!("no id found in context matching {}", mock_id_prefix);

        let mock_id_expr = if all_public {
            quote! { context::MockId::new(#mock_id_prefix) }
        } else {
            quote! { context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id.0)) }
        };

        quote! {
            pub fn #name(#receiver #receiver_comma #(#params),*) -> #ret_type {
                std::eprintln!("INFO: Mocked version of method {} was used", #name_str);
                let mock_id = #mock_id_expr;
                if context::ctx_built_and_contains_id(&mock_id) {
                    match context::run_mock::<(#input_tuple), #ret_type>(
                        mock_id,
                        (self as *const Self, #(#param_names),*),
                    ) {
                        Ok(res) => res,
                        Err(e) => match e {
                            context::MockError::Other(e) => panic!("unexpected Error: {:?}", e),
                            context::MockError::PredicateError(e) => panic!("{:?}", e.0),
                            context::MockError::NoMatchingId => panic!("failed to find mock id"),
                        },
                    }
                } else {
                    panic!(#panic_msg)
                }
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── on_call methods ─────────────────────────────────────────────────────────

fn gen_on_call_methods(input: &FlattenedStruct, struct_name: &Ident, classified: &ClassifiedMethods) -> TokenStream {
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let on_call_name = format_ident!("on_call_{}", name);
        let suffix = m.wrapper_suffix(struct_name);
        let ret_wrapper = format_ident!("Return{}", suffix);
        let mock_id_prefix = m.mock_id_prefix(&input.path, struct_name);
        let input_tuple = m.input_type_tuple(struct_name);
        let ret_type = m.ret_type();

        quote! {
            pub fn #on_call_name(ret: impl Into<#ret_wrapper>) {
                let inner: #ret_wrapper = ret.into();
                let cond = context::ConditionDoublePointer::from_fn::<(#input_tuple)>(Box::new(|_| Ok(())));
                context::add_expectation::<(#input_tuple), #ret_type>(
                    &context::MockId::new(#mock_id_prefix),
                    cond,
                    Some(inner.0),
                    None,
                    context::TimesModifier::Any,
                )
                .unwrap();
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── create_predicate methods ────────────────────────────────────────────────

fn gen_create_predicate_methods(input: &FlattenedStruct, struct_name: &Ident, classified: &ClassifiedMethods) -> TokenStream {
    let all_public = input.all_public;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let create_pred_name = format_ident!("create_predicate_{}", name);
        let suffix = m.wrapper_suffix(struct_name);
        let pred_wrapper = format_ident!("Predicate{}", suffix);
        let mock_id_prefix = m.mock_id_prefix(&input.path, struct_name);
        let input_tuple = m.input_type_tuple(struct_name);

        // Condition closure params: (&StructName, param_types...)
        // Use references for the condition
        let condition_param_types: Vec<TokenStream> = std::iter::once(quote! { &#struct_name })
            .chain(m.sig.params.iter().map(|(_, ty)| {
                let ty_str = quote! { #ty }.to_string();
                if ty_str == "String" {
                    quote! { &str }
                } else {
                    quote! { #ty }
                }
            }))
            .collect();

        // Generate the closure body that dereferences self_ptr and passes args
        let param_accesses: Vec<TokenStream> = m.sig.params.iter().enumerate().map(|(i, (_, ty))| {
            let idx = syn::Index::from(i + 1);
            let ty_str = quote! { #ty }.to_string();
            if ty_str == "String" {
                quote! { &input.#idx }
            } else {
                quote! { input.#idx }
            }
        }).collect();

        let failure_msg = format!("failed to uphold condition for {}", name);

        let mock_id_expr = if all_public {
            quote! { context::MockId::new(#mock_id_prefix) }
        } else {
            quote! { context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id.0)) }
        };

        quote! {
            pub fn #create_pred_name(
                &self,
                condition: impl Fn(#(#condition_param_types),*) -> bool + 'static,
                on_failure: Option<String>,
            ) -> #pred_wrapper {
                let mock_id = #mock_id_expr;
                let cond: context::ConditionDoublePointer =
                    context::ConditionDoublePointer::from_fn::<(#input_tuple)>(Box::new(
                        move |input: &(#input_tuple)| {
                            let self_ref = unsafe { &*input.0 };
                            if condition(self_ref, #(#param_accesses),*) {
                                Ok(())
                            } else {
                                Err(on_failure
                                    .clone()
                                    .unwrap_or(#failure_msg.into())
                                    .into())
                            }
                        },
                    ));
                let single = context::Predicate::create_single::<(#input_tuple)>(&mock_id, cond);
                #pred_wrapper(single)
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── times methods ───────────────────────────────────────────────────────────

fn gen_times_methods(_input: &FlattenedStruct, struct_name: &Ident, classified: &ClassifiedMethods) -> TokenStream {
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let times_name = format_ident!("{}_times", name);
        let suffix = m.wrapper_suffix(struct_name);
        let pred_wrapper = format_ident!("Predicate{}", suffix);

        quote! {
            pub fn #times_name(
                checkpoint: Option<impl Into<context::CheckpointName>>,
                condition: impl Into<#pred_wrapper>,
                tmod: context::TimesModifier,
            ) -> #pred_wrapper {
                let pred: #pred_wrapper = condition.into();

                let result = std::cell::Cell::new(None);
                let do_times = |cp: &mut context::Checkpoint| {
                    result.set(Some(#pred_wrapper(cp.times(pred.0, tmod))));
                };

                if let Some(name) = checkpoint {
                    let name: context::CheckpointName = name.into();
                    context::checkpoint_by_name_mut(&name.0, do_times)
                        .expect("failed to resolve checkpoint by name");
                } else {
                    context::latest_checkpoint_mut(do_times);
                }

                result.into_inner().expect("checkpoint closure did not run")
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── expect methods ──────────────────────────────────────────────────────────

fn gen_expect_methods(input: &FlattenedStruct, struct_name: &Ident, classified: &ClassifiedMethods) -> TokenStream {
    let all_public = input.all_public;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let expect_name = format_ident!("expect_{}", name);
        let suffix = m.wrapper_suffix(struct_name);
        let pred_wrapper = format_ident!("Predicate{}", suffix);
        let ret_wrapper = format_ident!("Return{}", suffix);
        let mock_id_prefix = m.mock_id_prefix(&input.path, struct_name);
        let input_tuple = m.input_type_tuple(struct_name);
        let ret_type = m.ret_type();

        let mock_id_expr = if all_public {
            quote! { context::MockId::new(#mock_id_prefix) }
        } else {
            quote! { context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id.0)) }
        };

        quote! {
            pub fn #expect_name(
                &self,
                checkpoint: Option<impl Into<context::CheckpointName>>,
                condition: impl Into<#pred_wrapper>,
                ret: impl Into<#ret_wrapper>,
                tmod: Option<context::TimesModifier>,
            ) {
                let mock_id = #mock_id_expr;

                let mut pred: #pred_wrapper = condition.into();
                let ret_val: #ret_wrapper = ret.into();

                // Patch the predicate's mock_id to be instance-specific
                if let context::new_expectations::PredicateKind::Single(ref mut single) = pred.0.kind {
                    single.mock_id = mock_id.clone();
                }

                let do_expect = |cp: &mut context::Checkpoint| {
                    let pred_idx = cp.arena.insert(pred.0);
                    let final_pred_idx = if let Some(tmod) = tmod {
                        cp.times_arena(pred_idx, tmod)
                    } else {
                        pred_idx
                    };
                    cp.expect::<(#input_tuple), #ret_type>(
                        &mock_id,
                        final_pred_idx,
                        Some(ret_val.0),
                    );
                };

                if let Some(name) = checkpoint {
                    let name: context::CheckpointName = name.into();
                    context::checkpoint_by_name_mut(&name.0, do_expect)
                        .expect("failed to resolve checkpoint by name");
                } else {
                    context::latest_checkpoint_mut(do_expect);
                }
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Constructors ────────────────────────────────────────────────────────────

/// Generate constructors. Returns (inherent_constructors, trait_constructor_impls).
/// - inherent_constructors: goes inside `impl StructName { ... }`
/// - trait_constructor_impls: goes outside as separate `impl Trait for StructName { ... }` blocks
fn gen_constructors(input: &FlattenedStruct, classified: &ClassifiedMethods) -> (TokenStream, TokenStream) {
    let struct_name = &input.struct_name;
    let all_public = input.all_public;

    // Generate mock registration for all mockable methods
    let mock_registrations: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let mock_id_prefix = m.mock_id_prefix(&input.path, struct_name);
        let var_name = format_ident!("{}_mock_id", m.sig.name);
        let input_tuple = m.input_type_tuple(struct_name);
        let ret_type = m.ret_type();
        if all_public {
            // Shared ID: ignore duplicate registration (multiple instances share one mock)
            quote! {
                let #var_name = context::MockId::new(#mock_id_prefix);
                let _ = context::add_mock::<(#input_tuple), #ret_type>(#var_name, None);
            }
        } else {
            quote! {
                let #var_name = context::MockId::new(format!("{}{}", #mock_id_prefix, slf.adt_mock_id.0));
                context::add_mock::<(#input_tuple), #ret_type>(#var_name, None).unwrap();
            }
        }
    }).collect();

    let mut inherent_fns: Vec<TokenStream> = Vec::new();
    let mut trait_impl_blocks: Vec<TokenStream> = Vec::new();

    for ctor in &classified.constructors {
        let ctor_name = &ctor.sig.name;
        let params: Vec<TokenStream> = ctor.sig.params.iter().map(|(name, ty)| {
            quote! { #name: #ty }
        }).collect();

        // For each struct field, try to match a constructor parameter by name.
        // - Public fields: use matching param if available, otherwise Default::default().
        // - Private fields: always PhantomData (they're erased in the mock struct).
        let field_inits: Vec<TokenStream> = input.fields.iter().map(|f| {
            let field_name = &f.name;
            match &f.vis {
                Visibility::Public(_) => {
                    // Check if any constructor param has the same name as this field
                    let has_matching_param = ctor.sig.params.iter().any(|(p, _)| p == field_name);
                    if has_matching_param {
                        quote! { #field_name }
                    } else {
                        quote! { #field_name: Default::default() }
                    }
                }
                _ => quote! { #field_name: std::marker::PhantomData },
            }
        }).collect();

        match &ctor.trait_name {
            None => {
                // Inherent constructor — goes inside `impl StructName { ... }`
                if all_public {
                    inherent_fns.push(quote! {
                        pub fn #ctor_name(#(#params),*) -> Self {
                            let slf = Self {
                                #(#field_inits,)*
                            };
                            #(#mock_registrations)*
                            slf
                        }
                    });
                } else {
                    inherent_fns.push(quote! {
                        pub fn #ctor_name(#(#params),*) -> Self {
                            let slf = Self {
                                #(#field_inits,)*
                                adt_mock_id: context::new_id(),
                            };
                            #(#mock_registrations)*
                            slf
                        }
                    });
                }
            }
            Some(trait_name) => {
                // Trait constructor — generate `impl TraitName<Generics> for StructName { ... }`
                let generics_tokens = if ctor.trait_generics.is_empty() {
                    quote! {}
                } else {
                    let generics = &ctor.trait_generics;
                    quote! { <#(#generics),*> }
                };

                // For From-style trait constructors, use tuple field indexing for public fields
                // since the parameter is typically a tuple (e.g. `value: (f32, f32)`).
                let from_field_inits: Vec<TokenStream> = input.fields.iter().enumerate().map(|(i, field)| {
                    let name = &field.name;
                    let idx = syn::Index::from(i);
                    match &field.vis {
                        Visibility::Public(_) => {
                            // Use first param name with tuple indexing
                            if let Some((param_name, _)) = ctor.sig.params.first() {
                                quote! { #name: #param_name.#idx }
                            } else {
                                quote! { #name: Default::default() }
                            }
                        }
                        _ => quote! { #name: std::marker::PhantomData },
                    }
                }).collect();

                if all_public {
                    trait_impl_blocks.push(quote! {
                        impl #trait_name #generics_tokens for #struct_name {
                            fn #ctor_name(#(#params),*) -> Self {
                                let slf = Self {
                                    #(#from_field_inits,)*
                                };
                                #(#mock_registrations)*
                                slf
                            }
                        }
                    });
                } else {
                    trait_impl_blocks.push(quote! {
                        impl #trait_name #generics_tokens for #struct_name {
                            fn #ctor_name(#(#params),*) -> Self {
                                let slf = Self {
                                    #(#from_field_inits,)*
                                    adt_mock_id: context::new_id(),
                                };
                                #(#mock_registrations)*
                                slf
                            }
                        }
                    });
                }
            }
        }
    }

    let inherent = quote! { #(#inherent_fns)* };
    let trait_impls = quote! { #(#trait_impl_blocks)* };
    (inherent, trait_impls)
}

// ─── Sequence helpers ────────────────────────────────────────────────────────

fn gen_sequence_helpers(input: &FlattenedStruct, struct_name: &Ident, classified: &ClassifiedMethods) -> TokenStream {
    let all_public = input.all_public;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let seq_name = format_ident!("expect_{}_in_sequence", name);
        let mock_id_prefix = m.mock_id_prefix(&input.path, struct_name);
        let input_tuple = m.input_type_tuple(struct_name);
        let ret_type = m.ret_type();

        // Condition closure params
        let closure_param_types: Vec<TokenStream> = std::iter::once(quote! { *const #struct_name })
            .chain(m.sig.params.iter().map(|(_, ty)| quote! { #ty }))
            .collect();

        // Input field accesses for condition
        let cond_field_accesses: Vec<TokenStream> = (0..m.sig.params.len() + 1)
            .map(|i| {
                let idx = syn::Index::from(i);
                let ty_str = if i > 0 {
                    let ty = &m.sig.params[i - 1].1;
                    quote! { #ty }.to_string()
                } else {
                    String::new()
                };
                if ty_str == "String" {
                    quote! { input.#idx.clone() }
                } else {
                    quote! { input.#idx }
                }
            })
            .collect();

        // Destructure for return closure
        let param_count = m.sig.params.len() + 1;
        let param_names: Vec<Ident> = (0..param_count)
            .map(|i| format_ident!("_{}", i))
            .collect();

        let mock_id_expr = if all_public {
            quote! { context::MockId::new(#mock_id_prefix) }
        } else {
            quote! { context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id.0)) }
        };

        quote! {
            pub fn #seq_name(
                &self,
                sequence_name: impl Into<context::SequenceName>,
                sequence_index: usize,
                condition: impl Fn(#(#closure_param_types),*) -> context::errors::PredicateResult<()> + 'static,
                ret: impl Fn(#(#closure_param_types),*) -> #ret_type + 'static,
                checkpoint: Option<impl Into<context::CheckpointName>>,
            ) {
                let mock_id = #mock_id_expr;
                let cond = context::ConditionDoublePointer::from_fn::<(#input_tuple)>(
                    Box::new(move |input: &(#input_tuple)| {
                        condition(#(#cond_field_accesses),*)
                    }),
                );
                let ret_closure: Box<dyn Fn((#input_tuple)) -> #ret_type> =
                    Box::new(move |(#(#param_names,)*)| ret(#(#param_names),*));

                context::add_expectation_to_sequence::<(#input_tuple), #ret_type>(
                    &mock_id,
                    cond,
                    Some(ret_closure),
                    sequence_name,
                    sequence_index,
                    checkpoint.map(|c| c.into()),
                )
                .expect(concat!("failed to add ", stringify!(#name), " to sequence"));
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Enum Code Generation
// ═══════════════════════════════════════════════════════════════════════════════

/// Classify methods for an enum (same logic as structs).
fn classify_enum_methods(entry: &FlattenedEnum) -> ClassifiedMethods {
    let mut constructors = Vec::new();
    let mut mockable = Vec::new();

    // Inherent impl methods
    if let Some(ref inh) = entry.inherent_impl {
        for method in &inh.methods {
            if method.receiver == Receiver::None {
                constructors.push(ConstructorMethod {
                    sig: method.clone(),
                    trait_name: None,
                    trait_generics: Vec::new(),
                });
            } else {
                mockable.push(MockableMethod {
                    sig: method.clone(),
                    trait_name: None,
                });
            }
        }
    }

    // Trait methods — only include public methods
    for trait_impl in &entry.trait_impls {
        if let Some(trait_def) = entry.traits.iter().find(|t| t.name == trait_impl.trait_name) {
            for method in &trait_def.methods {
                // Skip private trait methods (not marked pub)
                if !matches!(method.vis, Visibility::Public(_)) {
                    continue;
                }
                if method.receiver == Receiver::None {
                    constructors.push(ConstructorMethod {
                        sig: method.clone(),
                        trait_name: Some(trait_impl.trait_name.clone()),
                        trait_generics: trait_def.generics.clone(),
                    });
                } else {
                    mockable.push(MockableMethod {
                        sig: method.clone(),
                        trait_name: Some(trait_impl.trait_name.clone()),
                    });
                }
            }
        }
    }

    ClassifiedMethods { constructors, mockable }
}

/// Generate all code for a single enum entry.
fn expand_single_enum(entry: &FlattenedEnum, trackable_types: &HashMap<String, bool>) -> TokenStream {
    let enum_name = &entry.enum_name;

    // Classify all methods
    let classified = classify_enum_methods(entry);

    // Generate each piece
    let enum_def = gen_enum_def(entry);
    let drop_impl = gen_enum_drop_impl(entry, &classified);
    let wrapper_structs = gen_wrapper_structs(enum_name, &classified);
    let return_from_fn_impls = gen_return_from_fn(enum_name, &classified);
    let predicate_from_fn_impls = gen_enum_predicate_from_fn(entry, &classified);
    let mock_methods = gen_enum_mock_method_bodies(entry, &classified);
    let on_call_methods = gen_enum_on_call_methods(entry, &classified);
    let create_predicate_methods = gen_enum_create_predicate_methods(entry, &classified);
    let times_methods = gen_enum_times_methods(entry, &classified);
    let expect_methods = gen_enum_expect_methods(entry, &classified);
    let inherent_constructors = gen_enum_constructors(entry, &classified, trackable_types);
    let sequence_helpers = gen_enum_sequence_helpers(entry, &classified);
    let adt_mock_id_method = if entry.trackable {
        gen_enum_adt_mock_id_method(entry)
    } else {
        quote! {}
    };

    quote! {
        #enum_def
        #drop_impl
        #wrapper_structs
        #return_from_fn_impls
        #predicate_from_fn_impls
        impl #enum_name {
            #adt_mock_id_method
            #mock_methods
            #on_call_methods
            #create_predicate_methods
            #times_methods
            #expect_methods
            #inherent_constructors
            #sequence_helpers
        }
    }
}

// ─── Enum definition ─────────────────────────────────────────────────────────

fn gen_enum_def(entry: &FlattenedEnum) -> TokenStream {
    let vis = &entry.vis;
    let enum_name = &entry.enum_name;

    let variants: Vec<TokenStream> = entry.variants.iter().map(|v| {
        let name = &v.name;
        if v.fields.is_empty() {
            quote! { #name }
        } else {
            let fields = &v.fields;
            quote! { #name(#(#fields),*) }
        }
    }).collect();

    quote! {
        #vis enum #enum_name {
            #(#variants,)*
        }
    }
}

// ─── Enum adt_mock_id() method ───────────────────────────────────────────────

fn gen_enum_adt_mock_id_method(entry: &FlattenedEnum) -> TokenStream {
    let match_arms: Vec<TokenStream> = entry.variants.iter().enumerate().map(|(i, v)| {
        let variant_name = &v.name;
        let trackable_idx = entry.trackable_field_indices[i];
        let is_enum = entry.trackable_field_is_enum[i];

        // Use method call for enum inner types, field access for structs
        let id_access = if is_enum {
            quote! { .adt_mock_id() }
        } else {
            quote! { .adt_mock_id }
        };

        match trackable_idx {
            Some(_idx) if v.fields.len() == 1 => {
                // Single field variant — simple destructure
                quote! {
                    Self::#variant_name(inner) => &inner #id_access,
                }
            }
            Some(idx) => {
                // Multi-field variant — positional destructuring
                let bindings: Vec<TokenStream> = (0..v.fields.len()).map(|j| {
                    if j == idx {
                        quote! { trackable_field }
                    } else {
                        quote! { _ }
                    }
                }).collect();
                quote! {
                    Self::#variant_name(#(#bindings),*) => &trackable_field #id_access,
                }
            }
            None => {
                // This shouldn't happen for a trackable enum, but handle gracefully
                quote! {
                    Self::#variant_name(..) => unreachable!("non-trackable variant in trackable enum"),
                }
            }
        }
    }).collect();

    quote! {
        fn adt_mock_id(&self) -> &context::AdtMockId {
            match self {
                #(#match_arms)*
            }
        }
    }
}

// ─── Enum Drop impl ─────────────────────────────────────────────────────────

fn gen_enum_drop_impl(entry: &FlattenedEnum, classified: &ClassifiedMethods) -> TokenStream {
    let enum_name = &entry.enum_name;
    let trackable = entry.trackable;

    // Generate mock_id bindings for each mockable method
    let mock_id_bindings: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let prefix = m.mock_id_prefix(&entry.path, enum_name);
        let var_name = format_ident!("{}_mock_id", m.sig.name);
        if !trackable {
            quote! {
                let #var_name = context::MockId::new(#prefix);
            }
        } else {
            quote! {
                let #var_name = context::MockId::new(format!("{}{}", #prefix, self.adt_mock_id().0));
            }
        }
    }).collect();

    // Generate drop_predicate helper functions for each method
    let drop_predicate_fns: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let fn_name = format_ident!("drop_predicate_{}", m.sig.name);
        let input_tuple = m.input_type_tuple(enum_name);
        quote! {
            unsafe fn #fn_name(pred: context::Predicate) {
                match pred.kind {
                    context::new_expectations::PredicateKind::Single(single) => {
                        unsafe { single.condition.id_drop::<(#input_tuple)>() };
                    }
                    context::new_expectations::PredicateKind::And(children)
                    | context::new_expectations::PredicateKind::Or(children)
                    | context::new_expectations::PredicateKind::Xor(children) => {
                        for child in children {
                            unsafe { #fn_name(child) };
                        }
                    }
                    context::new_expectations::PredicateKind::Not(inner)
                    | context::new_expectations::PredicateKind::After { then: inner, .. } => {
                        unsafe { #fn_name(*inner) };
                    }
                    context::new_expectations::PredicateKind::Times { inner, .. } => {
                        unsafe { #fn_name(*inner) };
                    }
                }
            }
        }
    }).collect();

    // Generate the cleanup blocks for each method
    let cleanup_blocks: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let var_name = format_ident!("{}_mock_id", m.sig.name);
        let fn_name = format_ident!("drop_predicate_{}", m.sig.name);
        let input_tuple = m.input_type_tuple(enum_name);
        let ret_type = m.ret_type();
        quote! {
            if let Some(expectations) = cp.expectations.remove(&#var_name) {
                for exp in expectations {
                    if let Some(ret) = exp.return_val {
                        unsafe {
                            ret.id_drop::<(#input_tuple), #ret_type>();
                        }
                    }
                    if let Some(pred) = cp.arena.take(exp.predicate) {
                        unsafe {
                            #fn_name(pred);
                        }
                    }
                }
            }
        }
    }).collect();

    quote! {
        impl Drop for #enum_name {
            fn drop(&mut self) {
                #(#mock_id_bindings)*
                #(#drop_predicate_fns)*

                context::active_or_latest_checkpoint_mut(|cp| {
                    #(#cleanup_blocks)*
                });
            }
        }
    }
}

// ─── Enum Predicate::from_fn ─────────────────────────────────────────────────

fn gen_enum_predicate_from_fn(entry: &FlattenedEnum, classified: &ClassifiedMethods) -> TokenStream {
    let enum_name = &entry.enum_name;
    let impls: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let suffix = m.wrapper_suffix(enum_name);
        let pred_name = format_ident!("Predicate{}", suffix);
        let mock_id_prefix = m.mock_id_prefix(&entry.path, enum_name);
        let input_tuple = m.input_type_tuple(enum_name);

        // Generate closure parameter types
        let closure_param_types: Vec<TokenStream> = std::iter::once(quote! { *const #enum_name })
            .chain(m.sig.params.iter().map(|(_, ty)| quote! { #ty }))
            .collect();

        // For String params, clone them
        let closure_args: Vec<TokenStream> = std::iter::once(quote! { input.0 })
            .chain(m.sig.params.iter().enumerate().map(|(i, (_, ty))| {
                let idx = syn::Index::from(i + 1);
                let ty_str = quote! { #ty }.to_string();
                if ty_str == "String" {
                    quote! { input.#idx.clone() }
                } else {
                    quote! { input.#idx }
                }
            }))
            .collect();

        quote! {
            impl #pred_name {
                pub fn from_fn(
                    closure: impl Fn(#(#closure_param_types),*) -> context::errors::PredicateResult<()> + 'static,
                ) -> Self {
                    let mock_id = context::MockId::new(#mock_id_prefix);
                    let cond = context::ConditionDoublePointer::from_fn::<(#input_tuple)>(Box::new(
                        move |input: &(#input_tuple)| closure(#(#closure_args),*),
                    ));
                    Self(context::Predicate::create_single::<(#input_tuple)>(&mock_id, cond))
                }
            }
        }
    }).collect();

    quote! { #(#impls)* }
}

// ─── Enum mock method bodies ─────────────────────────────────────────────────

fn gen_enum_mock_method_bodies(entry: &FlattenedEnum, classified: &ClassifiedMethods) -> TokenStream {
    let enum_name = &entry.enum_name;
    let trackable = entry.trackable;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let mock_id_prefix = m.mock_id_prefix(&entry.path, enum_name);
        let input_tuple = m.input_type_tuple(enum_name);
        let ret_type = m.ret_type();
        let name_str = name.to_string();

        let receiver = match m.sig.receiver {
            Receiver::Ref => quote! { &self },
            Receiver::RefMut => quote! { &mut self },
            Receiver::None => quote! {},
        };

        let params: Vec<TokenStream> = m.sig.params.iter().map(|(name, ty)| {
            quote! { #name: #ty }
        }).collect();

        let param_names: Vec<&Ident> = m.sig.params.iter().map(|(name, _)| name).collect();

        let receiver_comma = if m.sig.receiver != Receiver::None && !params.is_empty() {
            quote! { , }
        } else {
            quote! {}
        };

        let panic_msg = format!("no id found in context matching {}", mock_id_prefix);

        let mock_id_expr = if !trackable {
            quote! { context::MockId::new(#mock_id_prefix) }
        } else {
            quote! { context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id().0)) }
        };

        quote! {
            pub fn #name(#receiver #receiver_comma #(#params),*) -> #ret_type {
                std::eprintln!("INFO: Mocked version of method {} was used", #name_str);
                let mock_id = #mock_id_expr;
                if context::ctx_built_and_contains_id(&mock_id) {
                    match context::run_mock::<(#input_tuple), #ret_type>(
                        mock_id,
                        (self as *const Self, #(#param_names),*),
                    ) {
                        Ok(res) => res,
                        Err(e) => match e {
                            context::MockError::Other(e) => panic!("unexpected Error: {:?}", e),
                            context::MockError::PredicateError(e) => panic!("{:?}", e.0),
                            context::MockError::NoMatchingId => panic!("failed to find mock id"),
                        },
                    }
                } else {
                    panic!(#panic_msg)
                }
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Enum on_call methods ────────────────────────────────────────────────────

fn gen_enum_on_call_methods(entry: &FlattenedEnum, classified: &ClassifiedMethods) -> TokenStream {
    let enum_name = &entry.enum_name;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let on_call_name = format_ident!("on_call_{}", name);
        let suffix = m.wrapper_suffix(enum_name);
        let ret_wrapper = format_ident!("Return{}", suffix);
        let mock_id_prefix = m.mock_id_prefix(&entry.path, enum_name);
        let input_tuple = m.input_type_tuple(enum_name);
        let ret_type = m.ret_type();

        quote! {
            pub fn #on_call_name(ret: impl Into<#ret_wrapper>) {
                let inner: #ret_wrapper = ret.into();
                let cond = context::ConditionDoublePointer::from_fn::<(#input_tuple)>(Box::new(|_| Ok(())));
                context::add_expectation::<(#input_tuple), #ret_type>(
                    &context::MockId::new(#mock_id_prefix),
                    cond,
                    Some(inner.0),
                    None,
                    context::TimesModifier::Any,
                )
                .unwrap();
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Enum create_predicate methods ───────────────────────────────────────────

fn gen_enum_create_predicate_methods(entry: &FlattenedEnum, classified: &ClassifiedMethods) -> TokenStream {
    let enum_name = &entry.enum_name;
    let trackable = entry.trackable;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let create_pred_name = format_ident!("create_predicate_{}", name);
        let suffix = m.wrapper_suffix(enum_name);
        let pred_wrapper = format_ident!("Predicate{}", suffix);
        let mock_id_prefix = m.mock_id_prefix(&entry.path, enum_name);
        let input_tuple = m.input_type_tuple(enum_name);

        let condition_param_types: Vec<TokenStream> = std::iter::once(quote! { &#enum_name })
            .chain(m.sig.params.iter().map(|(_, ty)| {
                let ty_str = quote! { #ty }.to_string();
                if ty_str == "String" {
                    quote! { &str }
                } else {
                    quote! { #ty }
                }
            }))
            .collect();

        let param_accesses: Vec<TokenStream> = m.sig.params.iter().enumerate().map(|(i, (_, ty))| {
            let idx = syn::Index::from(i + 1);
            let ty_str = quote! { #ty }.to_string();
            if ty_str == "String" {
                quote! { &input.#idx }
            } else {
                quote! { input.#idx }
            }
        }).collect();

        let failure_msg = format!("failed to uphold condition for {}", name);

        let mock_id_expr = if !trackable {
            quote! { context::MockId::new(#mock_id_prefix) }
        } else {
            quote! { context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id().0)) }
        };

        quote! {
            pub fn #create_pred_name(
                &self,
                condition: impl Fn(#(#condition_param_types),*) -> bool + 'static,
                on_failure: Option<String>,
            ) -> #pred_wrapper {
                let mock_id = #mock_id_expr;
                let cond: context::ConditionDoublePointer =
                    context::ConditionDoublePointer::from_fn::<(#input_tuple)>(Box::new(
                        move |input: &(#input_tuple)| {
                            let self_ref = unsafe { &*input.0 };
                            if condition(self_ref, #(#param_accesses),*) {
                                Ok(())
                            } else {
                                Err(on_failure
                                    .clone()
                                    .unwrap_or(#failure_msg.into())
                                    .into())
                            }
                        },
                    ));
                let single = context::Predicate::create_single::<(#input_tuple)>(&mock_id, cond);
                #pred_wrapper(single)
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Enum times methods ──────────────────────────────────────────────────────

fn gen_enum_times_methods(entry: &FlattenedEnum, classified: &ClassifiedMethods) -> TokenStream {
    let enum_name = &entry.enum_name;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let times_name = format_ident!("{}_times", name);
        let suffix = m.wrapper_suffix(enum_name);
        let pred_wrapper = format_ident!("Predicate{}", suffix);

        quote! {
            pub fn #times_name(
                checkpoint: Option<impl Into<context::CheckpointName>>,
                condition: impl Into<#pred_wrapper>,
                tmod: context::TimesModifier,
            ) -> #pred_wrapper {
                let pred: #pred_wrapper = condition.into();

                let result = std::cell::Cell::new(None);
                let do_times = |cp: &mut context::Checkpoint| {
                    result.set(Some(#pred_wrapper(cp.times(pred.0, tmod))));
                };

                if let Some(name) = checkpoint {
                    let name: context::CheckpointName = name.into();
                    context::checkpoint_by_name_mut(&name.0, do_times)
                        .expect("failed to resolve checkpoint by name");
                } else {
                    context::latest_checkpoint_mut(do_times);
                }

                result.into_inner().expect("checkpoint closure did not run")
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Enum expect methods ─────────────────────────────────────────────────────

fn gen_enum_expect_methods(entry: &FlattenedEnum, classified: &ClassifiedMethods) -> TokenStream {
    let enum_name = &entry.enum_name;
    let trackable = entry.trackable;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let expect_name = format_ident!("expect_{}", name);
        let suffix = m.wrapper_suffix(enum_name);
        let pred_wrapper = format_ident!("Predicate{}", suffix);
        let ret_wrapper = format_ident!("Return{}", suffix);
        let mock_id_prefix = m.mock_id_prefix(&entry.path, enum_name);
        let input_tuple = m.input_type_tuple(enum_name);
        let ret_type = m.ret_type();

        let mock_id_expr = if !trackable {
            quote! { context::MockId::new(#mock_id_prefix) }
        } else {
            quote! { context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id().0)) }
        };

        quote! {
            pub fn #expect_name(
                &self,
                checkpoint: Option<impl Into<context::CheckpointName>>,
                condition: impl Into<#pred_wrapper>,
                ret: impl Into<#ret_wrapper>,
                tmod: Option<context::TimesModifier>,
            ) {
                let mock_id = #mock_id_expr;

                let mut pred: #pred_wrapper = condition.into();
                let ret_val: #ret_wrapper = ret.into();

                // Patch the predicate's mock_id to be instance-specific
                if let context::new_expectations::PredicateKind::Single(ref mut single) = pred.0.kind {
                    single.mock_id = mock_id.clone();
                }

                let do_expect = |cp: &mut context::Checkpoint| {
                    let pred_idx = cp.arena.insert(pred.0);
                    let final_pred_idx = if let Some(tmod) = tmod {
                        cp.times_arena(pred_idx, tmod)
                    } else {
                        pred_idx
                    };
                    cp.expect::<(#input_tuple), #ret_type>(
                        &mock_id,
                        final_pred_idx,
                        Some(ret_val.0),
                    );
                };

                if let Some(name) = checkpoint {
                    let name: context::CheckpointName = name.into();
                    context::checkpoint_by_name_mut(&name.0, do_expect)
                        .expect("failed to resolve checkpoint by name");
                } else {
                    context::latest_checkpoint_mut(do_expect);
                }
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Enum constructors ───────────────────────────────────────────────────────

/// Generate enum constructors.
/// For each constructor, find a param whose type matches a variant's field type,
/// then generate `Self::Variant(param)` + mock registrations.
fn gen_enum_constructors(
    entry: &FlattenedEnum,
    classified: &ClassifiedMethods,
    _trackable_types: &HashMap<String, bool>,
) -> TokenStream {
    let enum_name = &entry.enum_name;
    let trackable = entry.trackable;

    // Generate mock registration for all mockable methods
    let mock_registrations: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let mock_id_prefix = m.mock_id_prefix(&entry.path, enum_name);
        let var_name = format_ident!("{}_mock_id", m.sig.name);
        let input_tuple = m.input_type_tuple(enum_name);
        let ret_type = m.ret_type();
        if !trackable {
            // Non-trackable: shared ID, ignore duplicate registration errors
            quote! {
                let #var_name = context::MockId::new(#mock_id_prefix);
                let _ = context::add_mock::<(#input_tuple), #ret_type>(#var_name, None);
            }
        } else {
            quote! {
                let #var_name = context::MockId::new(format!("{}{}", #mock_id_prefix, slf.adt_mock_id().0));
                context::add_mock::<(#input_tuple), #ret_type>(#var_name, None).unwrap();
            }
        }
    }).collect();

    let mut inherent_fns: Vec<TokenStream> = Vec::new();

    for ctor in &classified.constructors {
        if ctor.trait_name.is_some() {
            // Trait constructors for enums are not supported in this initial implementation
            continue;
        }

        let ctor_name = &ctor.sig.name;
        let params: Vec<TokenStream> = ctor.sig.params.iter().map(|(name, ty)| {
            quote! { #name: #ty }
        }).collect();

        // Find a param whose type matches a variant's field type
        let mut ctor_body = None;

        // Strategy: for each variant, check if we can match all its fields to constructor params
        'variant_search: for variant in &entry.variants {
            if variant.fields.is_empty() {
                // Unit variant — body is just Self::Variant
                if ctor.sig.params.is_empty() {
                    let variant_name = &variant.name;
                    ctor_body = Some(quote! { let slf = Self::#variant_name; });
                    break;
                }
                continue;
            }

            // Try to match each field type to a param by type name
            let mut matched_params: Vec<Option<&Ident>> = vec![None; variant.fields.len()];
            let mut used_params: Vec<bool> = vec![false; ctor.sig.params.len()];

            for (field_idx, field_ty) in variant.fields.iter().enumerate() {
                if let Some(field_ident) = type_last_ident(field_ty) {
                    for (param_idx, (param_name, param_ty)) in ctor.sig.params.iter().enumerate() {
                        if used_params[param_idx] {
                            continue;
                        }
                        if let Some(param_ident) = type_last_ident(param_ty) {
                            if param_ident == field_ident {
                                matched_params[field_idx] = Some(param_name);
                                used_params[param_idx] = true;
                                break;
                            }
                        }
                    }
                }
            }

            // Check if all fields are matched
            if matched_params.iter().all(|m| m.is_some()) {
                let variant_name = &variant.name;
                let args: Vec<&Ident> = matched_params.into_iter().map(|m| m.unwrap()).collect();
                ctor_body = Some(quote! {
                    let slf = Self::#variant_name(#(#args),*);
                });
                break 'variant_search;
            }

            // Partial match: if at least the first field matched (single-field shortcut)
            if variant.fields.len() == 1 {
                if let Some(param_name) = matched_params[0] {
                    let variant_name = &variant.name;
                    ctor_body = Some(quote! {
                        let slf = Self::#variant_name(#param_name);
                    });
                    break 'variant_search;
                }
            }
        }

        // Fallback: try matching any single param to any single-field variant
        if ctor_body.is_none() {
            for (param_name, param_ty) in &ctor.sig.params {
                if let Some(param_ident) = type_last_ident(param_ty) {
                    for variant in &entry.variants {
                        if variant.fields.len() == 1 {
                            if let Some(field_ident) = type_last_ident(&variant.fields[0]) {
                                if field_ident == param_ident {
                                    let variant_name = &variant.name;
                                    ctor_body = Some(quote! {
                                        let slf = Self::#variant_name(#param_name);
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
                if ctor_body.is_some() {
                    break;
                }
            }
        }

        // If no match found, use first variant with Default::default()
        let body = ctor_body.unwrap_or_else(|| {
            if let Some(first_variant) = entry.variants.first() {
                let variant_name = &first_variant.name;
                if first_variant.fields.is_empty() {
                    quote! { let slf = Self::#variant_name; }
                } else {
                    let defaults: Vec<TokenStream> = first_variant.fields.iter().map(|_| {
                        quote! { Default::default() }
                    }).collect();
                    quote! { let slf = Self::#variant_name(#(#defaults),*); }
                }
            } else {
                quote! { unreachable!("enum has no variants"); }
            }
        });

        inherent_fns.push(quote! {
            pub fn #ctor_name(#(#params),*) -> Self {
                #body
                #(#mock_registrations)*
                slf
            }
        });
    }

    quote! { #(#inherent_fns)* }
}

// ─── Enum sequence helpers ───────────────────────────────────────────────────

fn gen_enum_sequence_helpers(entry: &FlattenedEnum, classified: &ClassifiedMethods) -> TokenStream {
    let enum_name = &entry.enum_name;
    let trackable = entry.trackable;
    let methods: Vec<TokenStream> = classified.mockable.iter().map(|m| {
        let name = &m.sig.name;
        let seq_name = format_ident!("expect_{}_in_sequence", name);
        let mock_id_prefix = m.mock_id_prefix(&entry.path, enum_name);
        let input_tuple = m.input_type_tuple(enum_name);
        let ret_type = m.ret_type();

        let closure_param_types: Vec<TokenStream> = std::iter::once(quote! { *const #enum_name })
            .chain(m.sig.params.iter().map(|(_, ty)| quote! { #ty }))
            .collect();

        let cond_field_accesses: Vec<TokenStream> = (0..m.sig.params.len() + 1)
            .map(|i| {
                let idx = syn::Index::from(i);
                let ty_str = if i > 0 {
                    let ty = &m.sig.params[i - 1].1;
                    quote! { #ty }.to_string()
                } else {
                    String::new()
                };
                if ty_str == "String" {
                    quote! { input.#idx.clone() }
                } else {
                    quote! { input.#idx }
                }
            })
            .collect();

        let param_count = m.sig.params.len() + 1;
        let param_names: Vec<Ident> = (0..param_count)
            .map(|i| format_ident!("_{}", i))
            .collect();

        let mock_id_expr = if !trackable {
            quote! { context::MockId::new(#mock_id_prefix) }
        } else {
            quote! { context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id().0)) }
        };

        quote! {
            pub fn #seq_name(
                &self,
                sequence_name: impl Into<context::SequenceName>,
                sequence_index: usize,
                condition: impl Fn(#(#closure_param_types),*) -> context::errors::PredicateResult<()> + 'static,
                ret: impl Fn(#(#closure_param_types),*) -> #ret_type + 'static,
                checkpoint: Option<impl Into<context::CheckpointName>>,
            ) {
                let mock_id = #mock_id_expr;
                let cond = context::ConditionDoublePointer::from_fn::<(#input_tuple)>(
                    Box::new(move |input: &(#input_tuple)| {
                        condition(#(#cond_field_accesses),*)
                    }),
                );
                let ret_closure: Box<dyn Fn((#input_tuple)) -> #ret_type> =
                    Box::new(move |(#(#param_names,)*)| ret(#(#param_names),*));

                context::add_expectation_to_sequence::<(#input_tuple), #ret_type>(
                    &mock_id,
                    cond,
                    Some(ret_closure),
                    sequence_name,
                    sequence_index,
                    checkpoint.map(|c| c.into()),
                )
                .expect(concat!("failed to add ", stringify!(#name), " to sequence"));
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Trait Mock Code Generation
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate all code for a single trait mock entry.
fn expand_trait_mock(entry: &FlattenedTrait) -> TokenStream {
    let trait_name = &entry.name;
    let mock_struct_name = format_ident!("Mock{}", trait_name);

    // Generate trait definition (only public methods)
    let trait_def = gen_trait_def(entry);

    // Generate mock struct
    let mock_struct_def = gen_trait_mock_struct(entry, &mock_struct_name);

    // Generate trait impl for mock struct
    let trait_impl = gen_trait_mock_impl(entry, &mock_struct_name);

    // Generate wrapper structs, Return::from_fn, Predicate::from_fn
    let wrapper_structs = gen_trait_mock_wrapper_structs(entry, &mock_struct_name);
    let return_from_fn = gen_trait_mock_return_from_fn(entry, &mock_struct_name);
    let predicate_from_fn = gen_trait_mock_predicate_from_fn(entry, &mock_struct_name);

    // Generate expect/on_call/create_predicate/times/sequence helpers
    let on_call_methods = gen_trait_mock_on_call_methods(entry, &mock_struct_name);
    let create_predicate_methods = gen_trait_mock_create_predicate_methods(entry, &mock_struct_name);
    let times_methods = gen_trait_mock_times_methods(entry, &mock_struct_name);
    let expect_methods = gen_trait_mock_expect_methods(entry, &mock_struct_name);
    let sequence_helpers = gen_trait_mock_sequence_helpers(entry, &mock_struct_name);

    // Generate constructor
    let constructor = gen_trait_mock_constructor(entry, &mock_struct_name);

    // Generate Drop impl
    let drop_impl = gen_trait_mock_drop(entry, &mock_struct_name);

    quote! {
        #trait_def
        #mock_struct_def
        #drop_impl
        #wrapper_structs
        #return_from_fn
        #predicate_from_fn
        impl #mock_struct_name {
            #constructor
            #on_call_methods
            #create_predicate_methods
            #times_methods
            #expect_methods
            #sequence_helpers
        }
        #trait_impl
    }
}

/// Helper: compute mock_id prefix for a trait method.
fn trait_mock_id_prefix(path: &syn::Path, trait_name: &Ident, method_name: &Ident) -> String {
    let path_str = path.segments.iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("_");
    format!("{}_{}_{}", path_str, trait_name, method_name)
}

/// Helper: compute wrapper suffix for a trait mock method.
fn trait_wrapper_suffix(mock_struct_name: &Ident, method_name: &Ident) -> Ident {
    let method_capitalized = capitalize_first(&method_name.to_string());
    format_ident!("{}{}", mock_struct_name, method_capitalized)
}

/// Helper: compute input type tuple for a trait mock method.
fn trait_input_type_tuple(mock_struct_name: &Ident, method: &MethodSig) -> TokenStream {
    let param_types: Vec<&Type> = method.params.iter().map(|(_, ty)| ty).collect();
    quote! { (*const #mock_struct_name, #(#param_types),*) }
}

// ─── Trait definition ────────────────────────────────────────────────────────

fn gen_trait_def(entry: &FlattenedTrait) -> TokenStream {
    let vis = &entry.vis;
    let trait_name = &entry.name;

    let methods: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let name = &m.name;
        let ret_type = &m.ret_type;

        let receiver = match m.receiver {
            Receiver::Ref => quote! { &self },
            Receiver::RefMut => quote! { &mut self },
            Receiver::None => quote! {},
        };

        let params: Vec<TokenStream> = m.params.iter().map(|(name, ty)| {
            quote! { #name: #ty }
        }).collect();

        let receiver_comma = if m.receiver != Receiver::None && !params.is_empty() {
            quote! { , }
        } else {
            quote! {}
        };

        quote! {
            fn #name(#receiver #receiver_comma #(#params),*) -> #ret_type;
        }
    }).collect();

    quote! {
        #vis trait #trait_name {
            #(#methods)*
        }
    }
}

// ─── Mock struct definition ──────────────────────────────────────────────────

fn gen_trait_mock_struct(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let vis = &entry.vis;
    quote! {
        #vis struct #mock_struct_name {
            adt_mock_id: context::AdtMockId,
        }
    }
}

// ─── Trait impl for mock struct ──────────────────────────────────────────────

fn gen_trait_mock_impl(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let trait_name = &entry.name;

    let methods: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let name = &m.name;
        let mock_id_prefix = trait_mock_id_prefix(&entry.path, trait_name, name);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);
        let ret_type = &m.ret_type;
        let name_str = name.to_string();

        let receiver = match m.receiver {
            Receiver::Ref => quote! { &self },
            Receiver::RefMut => quote! { &mut self },
            Receiver::None => quote! {},
        };

        let params: Vec<TokenStream> = m.params.iter().map(|(name, ty)| {
            quote! { #name: #ty }
        }).collect();

        let param_names: Vec<&Ident> = m.params.iter().map(|(name, _)| name).collect();

        let receiver_comma = if m.receiver != Receiver::None && !params.is_empty() {
            quote! { , }
        } else {
            quote! {}
        };

        let panic_msg = format!("no id found in context matching {}", mock_id_prefix);

        quote! {
            fn #name(#receiver #receiver_comma #(#params),*) -> #ret_type {
                std::eprintln!("INFO: Mocked version of trait method {} was used", #name_str);
                let mock_id = context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id.0));
                if context::ctx_built_and_contains_id(&mock_id) {
                    match context::run_mock::<(#input_tuple), #ret_type>(
                        mock_id,
                        (self as *const Self, #(#param_names),*),
                    ) {
                        Ok(res) => res,
                        Err(e) => match e {
                            context::MockError::Other(e) => panic!("unexpected Error: {:?}", e),
                            context::MockError::PredicateError(e) => panic!("{:?}", e.0),
                            context::MockError::NoMatchingId => panic!("failed to find mock id"),
                        },
                    }
                } else {
                    panic!(#panic_msg)
                }
            }
        }
    }).collect();

    quote! {
        impl #trait_name for #mock_struct_name {
            #(#methods)*
        }
    }
}

// ─── Trait mock wrapper structs ──────────────────────────────────────────────

fn gen_trait_mock_wrapper_structs(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let wrappers: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let suffix = trait_wrapper_suffix(mock_struct_name, &m.name);
        let pred_name = format_ident!("Predicate{}", suffix);
        let exp_name = format_ident!("Expectation{}", suffix);
        let ret_name = format_ident!("Return{}", suffix);
        quote! {
            pub struct #pred_name(context::Predicate);
            pub struct #exp_name(context::Expectation);
            pub struct #ret_name(context::ReturnValDoublePointer);
        }
    }).collect();

    quote! { #(#wrappers)* }
}

// ─── Trait mock Return::from_fn ──────────────────────────────────────────────

fn gen_trait_mock_return_from_fn(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let impls: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let suffix = trait_wrapper_suffix(mock_struct_name, &m.name);
        let ret_name = format_ident!("Return{}", suffix);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);
        let ret_type = &m.ret_type;

        let param_count = m.params.len() + 1; // +1 for *const Self
        let param_names: Vec<Ident> = (0..param_count)
            .map(|i| format_ident!("_{}", i))
            .collect();

        let closure_param_types: Vec<TokenStream> = std::iter::once(quote! { *const #mock_struct_name })
            .chain(m.params.iter().map(|(_, ty)| quote! { #ty }))
            .collect();

        quote! {
            impl #ret_name {
                pub fn from_fn(closure: impl Fn(#(#closure_param_types),*) -> #ret_type + 'static) -> Self {
                    Self(context::ReturnValDoublePointer::from_fn::<
                        (#input_tuple),
                        #ret_type,
                    >(Box::new(move |(#(#param_names,)*)| closure(#(#param_names),*))))
                }
            }
        }
    }).collect();

    quote! { #(#impls)* }
}

// ─── Trait mock Predicate::from_fn ───────────────────────────────────────────

fn gen_trait_mock_predicate_from_fn(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let trait_name = &entry.name;
    let impls: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let suffix = trait_wrapper_suffix(mock_struct_name, &m.name);
        let pred_name = format_ident!("Predicate{}", suffix);
        let mock_id_prefix = trait_mock_id_prefix(&entry.path, trait_name, &m.name);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);

        let closure_param_types: Vec<TokenStream> = std::iter::once(quote! { *const #mock_struct_name })
            .chain(m.params.iter().map(|(_, ty)| quote! { #ty }))
            .collect();

        let closure_args: Vec<TokenStream> = std::iter::once(quote! { input.0 })
            .chain(m.params.iter().enumerate().map(|(i, (_, ty))| {
                let idx = syn::Index::from(i + 1);
                let ty_str = quote! { #ty }.to_string();
                if ty_str == "String" {
                    quote! { input.#idx.clone() }
                } else {
                    quote! { input.#idx }
                }
            }))
            .collect();

        quote! {
            impl #pred_name {
                pub fn from_fn(
                    closure: impl Fn(#(#closure_param_types),*) -> context::errors::PredicateResult<()> + 'static,
                ) -> Self {
                    let mock_id = context::MockId::new(#mock_id_prefix);
                    let cond = context::ConditionDoublePointer::from_fn::<(#input_tuple)>(Box::new(
                        move |input: &(#input_tuple)| closure(#(#closure_args),*),
                    ));
                    Self(context::Predicate::create_single::<(#input_tuple)>(&mock_id, cond))
                }
            }
        }
    }).collect();

    quote! { #(#impls)* }
}

// ─── Trait mock constructor ──────────────────────────────────────────────────

fn gen_trait_mock_constructor(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let trait_name = &entry.name;

    let mock_registrations: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let mock_id_prefix = trait_mock_id_prefix(&entry.path, trait_name, &m.name);
        let var_name = format_ident!("{}_mock_id", m.name);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);
        let ret_type = &m.ret_type;
        quote! {
            let #var_name = context::MockId::new(format!("{}{}", #mock_id_prefix, slf.adt_mock_id.0));
            context::add_mock::<(#input_tuple), #ret_type>(#var_name, None).unwrap();
        }
    }).collect();

    quote! {
        pub fn new() -> Self {
            let slf = Self { adt_mock_id: context::new_id() };
            #(#mock_registrations)*
            slf
        }
    }
}

// ─── Trait mock Drop impl ────────────────────────────────────────────────────

fn gen_trait_mock_drop(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let trait_name = &entry.name;

    let mock_id_bindings: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let prefix = trait_mock_id_prefix(&entry.path, trait_name, &m.name);
        let var_name = format_ident!("{}_mock_id", m.name);
        quote! {
            let #var_name = context::MockId::new(format!("{}{}", #prefix, self.adt_mock_id.0));
        }
    }).collect();

    let drop_predicate_fns: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let fn_name = format_ident!("drop_predicate_{}", m.name);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);
        quote! {
            unsafe fn #fn_name(pred: context::Predicate) {
                match pred.kind {
                    context::new_expectations::PredicateKind::Single(single) => {
                        unsafe { single.condition.id_drop::<(#input_tuple)>() };
                    }
                    context::new_expectations::PredicateKind::And(children)
                    | context::new_expectations::PredicateKind::Or(children)
                    | context::new_expectations::PredicateKind::Xor(children) => {
                        for child in children {
                            unsafe { #fn_name(child) };
                        }
                    }
                    context::new_expectations::PredicateKind::Not(inner)
                    | context::new_expectations::PredicateKind::After { then: inner, .. } => {
                        unsafe { #fn_name(*inner) };
                    }
                    context::new_expectations::PredicateKind::Times { inner, .. } => {
                        unsafe { #fn_name(*inner) };
                    }
                }
            }
        }
    }).collect();

    let cleanup_blocks: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let var_name = format_ident!("{}_mock_id", m.name);
        let fn_name = format_ident!("drop_predicate_{}", m.name);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);
        let ret_type = &m.ret_type;
        quote! {
            if let Some(expectations) = cp.expectations.remove(&#var_name) {
                for exp in expectations {
                    if let Some(ret) = exp.return_val {
                        unsafe {
                            ret.id_drop::<(#input_tuple), #ret_type>();
                        }
                    }
                    if let Some(pred) = cp.arena.take(exp.predicate) {
                        unsafe {
                            #fn_name(pred);
                        }
                    }
                }
            }
        }
    }).collect();

    quote! {
        impl Drop for #mock_struct_name {
            fn drop(&mut self) {
                #(#mock_id_bindings)*
                #(#drop_predicate_fns)*

                context::active_or_latest_checkpoint_mut(|cp| {
                    #(#cleanup_blocks)*
                });
            }
        }
    }
}

// ─── Trait mock on_call methods ──────────────────────────────────────────────

fn gen_trait_mock_on_call_methods(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let trait_name = &entry.name;
    let methods: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let name = &m.name;
        let on_call_name = format_ident!("on_call_{}", name);
        let suffix = trait_wrapper_suffix(mock_struct_name, name);
        let ret_wrapper = format_ident!("Return{}", suffix);
        let mock_id_prefix = trait_mock_id_prefix(&entry.path, trait_name, name);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);
        let ret_type = &m.ret_type;

        quote! {
            pub fn #on_call_name(ret: impl Into<#ret_wrapper>) {
                let inner: #ret_wrapper = ret.into();
                let cond = context::ConditionDoublePointer::from_fn::<(#input_tuple)>(Box::new(|_| Ok(())));
                context::add_expectation::<(#input_tuple), #ret_type>(
                    &context::MockId::new(#mock_id_prefix),
                    cond,
                    Some(inner.0),
                    None,
                    context::TimesModifier::Any,
                )
                .unwrap();
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Trait mock create_predicate methods ─────────────────────────────────────

fn gen_trait_mock_create_predicate_methods(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let trait_name = &entry.name;
    let methods: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let name = &m.name;
        let create_pred_name = format_ident!("create_predicate_{}", name);
        let suffix = trait_wrapper_suffix(mock_struct_name, name);
        let pred_wrapper = format_ident!("Predicate{}", suffix);
        let mock_id_prefix = trait_mock_id_prefix(&entry.path, trait_name, name);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);

        let condition_param_types: Vec<TokenStream> = std::iter::once(quote! { &#mock_struct_name })
            .chain(m.params.iter().map(|(_, ty)| {
                let ty_str = quote! { #ty }.to_string();
                if ty_str == "String" {
                    quote! { &str }
                } else {
                    quote! { #ty }
                }
            }))
            .collect();

        let param_accesses: Vec<TokenStream> = m.params.iter().enumerate().map(|(i, (_, ty))| {
            let idx = syn::Index::from(i + 1);
            let ty_str = quote! { #ty }.to_string();
            if ty_str == "String" {
                quote! { &input.#idx }
            } else {
                quote! { input.#idx }
            }
        }).collect();

        let failure_msg = format!("failed to uphold condition for {}", name);

        quote! {
            pub fn #create_pred_name(
                &self,
                condition: impl Fn(#(#condition_param_types),*) -> bool + 'static,
                on_failure: Option<String>,
            ) -> #pred_wrapper {
                let mock_id = context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id.0));
                let cond: context::ConditionDoublePointer =
                    context::ConditionDoublePointer::from_fn::<(#input_tuple)>(Box::new(
                        move |input: &(#input_tuple)| {
                            let self_ref = unsafe { &*input.0 };
                            if condition(self_ref, #(#param_accesses),*) {
                                Ok(())
                            } else {
                                Err(on_failure
                                    .clone()
                                    .unwrap_or(#failure_msg.into())
                                    .into())
                            }
                        },
                    ));
                let single = context::Predicate::create_single::<(#input_tuple)>(&mock_id, cond);
                #pred_wrapper(single)
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Trait mock times methods ────────────────────────────────────────────────

fn gen_trait_mock_times_methods(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let methods: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let name = &m.name;
        let times_name = format_ident!("{}_times", name);
        let suffix = trait_wrapper_suffix(mock_struct_name, name);
        let pred_wrapper = format_ident!("Predicate{}", suffix);

        quote! {
            pub fn #times_name(
                checkpoint: Option<impl Into<context::CheckpointName>>,
                condition: impl Into<#pred_wrapper>,
                tmod: context::TimesModifier,
            ) -> #pred_wrapper {
                let pred: #pred_wrapper = condition.into();

                let result = std::cell::Cell::new(None);
                let do_times = |cp: &mut context::Checkpoint| {
                    result.set(Some(#pred_wrapper(cp.times(pred.0, tmod))));
                };

                if let Some(name) = checkpoint {
                    let name: context::CheckpointName = name.into();
                    context::checkpoint_by_name_mut(&name.0, do_times)
                        .expect("failed to resolve checkpoint by name");
                } else {
                    context::latest_checkpoint_mut(do_times);
                }

                result.into_inner().expect("checkpoint closure did not run")
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Trait mock expect methods ───────────────────────────────────────────────

fn gen_trait_mock_expect_methods(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let trait_name = &entry.name;
    let methods: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let name = &m.name;
        let expect_name = format_ident!("expect_{}", name);
        let suffix = trait_wrapper_suffix(mock_struct_name, name);
        let pred_wrapper = format_ident!("Predicate{}", suffix);
        let ret_wrapper = format_ident!("Return{}", suffix);
        let mock_id_prefix = trait_mock_id_prefix(&entry.path, trait_name, name);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);
        let ret_type = &m.ret_type;

        quote! {
            pub fn #expect_name(
                &self,
                checkpoint: Option<impl Into<context::CheckpointName>>,
                condition: impl Into<#pred_wrapper>,
                ret: impl Into<#ret_wrapper>,
                tmod: Option<context::TimesModifier>,
            ) {
                let mock_id = context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id.0));

                let mut pred: #pred_wrapper = condition.into();
                let ret_val: #ret_wrapper = ret.into();

                // Patch the predicate's mock_id to be instance-specific
                if let context::new_expectations::PredicateKind::Single(ref mut single) = pred.0.kind {
                    single.mock_id = mock_id.clone();
                }

                let do_expect = |cp: &mut context::Checkpoint| {
                    let pred_idx = cp.arena.insert(pred.0);
                    let final_pred_idx = if let Some(tmod) = tmod {
                        cp.times_arena(pred_idx, tmod)
                    } else {
                        pred_idx
                    };
                    cp.expect::<(#input_tuple), #ret_type>(
                        &mock_id,
                        final_pred_idx,
                        Some(ret_val.0),
                    );
                };

                if let Some(name) = checkpoint {
                    let name: context::CheckpointName = name.into();
                    context::checkpoint_by_name_mut(&name.0, do_expect)
                        .expect("failed to resolve checkpoint by name");
                } else {
                    context::latest_checkpoint_mut(do_expect);
                }
            }
        }
    }).collect();

    quote! { #(#methods)* }
}

// ─── Trait mock sequence helpers ─────────────────────────────────────────────

fn gen_trait_mock_sequence_helpers(entry: &FlattenedTrait, mock_struct_name: &Ident) -> TokenStream {
    let trait_name = &entry.name;
    let methods: Vec<TokenStream> = entry.methods.iter().map(|m| {
        let name = &m.name;
        let seq_name = format_ident!("expect_{}_in_sequence", name);
        let mock_id_prefix = trait_mock_id_prefix(&entry.path, trait_name, name);
        let input_tuple = trait_input_type_tuple(mock_struct_name, m);
        let ret_type = &m.ret_type;

        let closure_param_types: Vec<TokenStream> = std::iter::once(quote! { *const #mock_struct_name })
            .chain(m.params.iter().map(|(_, ty)| quote! { #ty }))
            .collect();

        let cond_field_accesses: Vec<TokenStream> = (0..m.params.len() + 1)
            .map(|i| {
                let idx = syn::Index::from(i);
                let ty_str = if i > 0 {
                    let ty = &m.params[i - 1].1;
                    quote! { #ty }.to_string()
                } else {
                    String::new()
                };
                if ty_str == "String" {
                    quote! { input.#idx.clone() }
                } else {
                    quote! { input.#idx }
                }
            })
            .collect();

        let param_count = m.params.len() + 1;
        let param_names: Vec<Ident> = (0..param_count)
            .map(|i| format_ident!("_{}", i))
            .collect();

        quote! {
            pub fn #seq_name(
                &self,
                sequence_name: impl Into<context::SequenceName>,
                sequence_index: usize,
                condition: impl Fn(#(#closure_param_types),*) -> context::errors::PredicateResult<()> + 'static,
                ret: impl Fn(#(#closure_param_types),*) -> #ret_type + 'static,
                checkpoint: Option<impl Into<context::CheckpointName>>,
            ) {
                let mock_id = context::MockId::new(format!("{}{}", #mock_id_prefix, self.adt_mock_id.0));
                let cond = context::ConditionDoublePointer::from_fn::<(#input_tuple)>(
                    Box::new(move |input: &(#input_tuple)| {
                        condition(#(#cond_field_accesses),*)
                    }),
                );
                let ret_closure: Box<dyn Fn((#input_tuple)) -> #ret_type> =
                    Box::new(move |(#(#param_names,)*)| ret(#(#param_names),*));

                context::add_expectation_to_sequence::<(#input_tuple), #ret_type>(
                    &mock_id,
                    cond,
                    Some(ret_closure),
                    sequence_name,
                    sequence_index,
                    checkpoint.map(|c| c.into()),
                )
                .expect(concat!("failed to add ", stringify!(#name), " to sequence"));
            }
        }
    }).collect();

    quote! { #(#methods)* }
}
