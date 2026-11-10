//! Read-only analysis pass for the two-pass mock architecture.
//!
//! This module implements the **first pass** of the two-pass compilation strategy.
//! It runs `rustc` through macro expansion to obtain the fully-expanded AST, then
//! walks the public API surface — functions, structs, enums, traits, impls, and
//! submodules — collecting everything into a [`CrateApiModel`].
//!
//! Unlike [`CrateIntercept`](crate::crate_intercept::CrateIntercept), this pass
//! does **not** modify the AST. It stops compilation after `after_expansion`,
//! producing only the model. The second pass (substitution) then uses that model
//! to rewrite function bodies and inject mock infrastructure.
//!
//! ## Usage
//!
//! ```ignore
//! let mut pass = AnalysisPass::new("my_crate".to_string());
//! rustc_driver::run_compiler(&args, &mut pass);
//! let api = pass.into_model();
//! ```

use rustc_ast as ast;
use rustc_ast_pretty::pprust;
use rustc_driver::Compilation;
use rustc_interface::interface::Compiler;
use rustc_middle::ty::TyCtxt;

use crate::crate_api::*;
use crate::generics_model::*;
use crate::type_repr::TypeRepr;

/// The analysis pass — collects the crate's public API from the expanded AST
/// without modifying it.
pub struct AnalysisPass {
    crate_name: String,
    model: Option<CrateApiModel>,
}

impl AnalysisPass {
    pub fn new(crate_name: String) -> Self {
        Self {
            crate_name,
            model: None,
        }
    }

    pub fn into_model(self) -> CrateApiModel {
        self.model
            .expect("analysis pass did not produce a model — was after_expansion called?")
    }
}

impl rustc_driver::Callbacks for AnalysisPass {
    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        log::debug!(
            "AnalysisPass: collecting expanded API for crate '{}'",
            self.crate_name
        );

        // Access the expanded AST via the resolver_for_lowering_raw query
        let (_, krate_steal, _) = tcx.resolver_for_lowering_raw(());
        let krate = krate_steal.borrow();

        // Collect the full public API from the expanded AST
        let root = collect_module(&self.crate_name, &krate.items);

        let api = CrateApiModel {
            crate_name: self.crate_name.clone(),
            root,
        };

        log::debug!("AnalysisPass: collected API:");
        log::debug!("  {} structs", api.root.structs.len());
        log::debug!("  {} enums", api.root.enums.len());
        log::debug!("  {} traits", api.root.traits.len());
        log::debug!("  {} functions", api.root.functions.len());
        log::debug!("  {} impls", api.root.impls.len());
        log::debug!("  {} submodules", api.root.children.len());

        self.model = Some(api);

        // Stop compilation — we only needed the expanded AST
        Compilation::Stop
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Collection functions — walk the expanded AST and build the model
// ═══════════════════════════════════════════════════════════════════════════════

/// Walk a list of items (module contents) and collect the public API.
fn collect_module(name: &str, items: &[Box<ast::Item>]) -> ModuleModel {
    let mut module = ModuleModel::new(name);

    for item in items {
        match &item.kind {
            ast::ItemKind::Fn(fn_data) if is_pub(&item.vis) => {
                if let Some(func) = collect_function(fn_data) {
                    module.functions.push(func);
                }
            }
            ast::ItemKind::Struct(ident, _generics, fields) if is_pub(&item.vis) => {
                if let Some(s) = collect_struct(ident.name.as_str(), fields) {
                    module.structs.push(s);
                }
            }
            ast::ItemKind::Enum(ident, _generics, enum_def) if is_pub(&item.vis) => {
                if let Some(e) = collect_enum(ident.name.as_str(), &enum_def.variants) {
                    module.enums.push(e);
                }
            }
            ast::ItemKind::Trait(trait_data) if is_pub(&item.vis) => {
                if let Some(t) = collect_trait(trait_data) {
                    module.traits.push(t);
                }
            }
            ast::ItemKind::Impl(impl_data) => {
                if let Some(imp) = collect_impl(impl_data) {
                    module.impls.push(imp);
                }
            }
            ast::ItemKind::Mod(_safety, ident, mod_kind) if is_pub(&item.vis) => {
                if let ast::ModKind::Loaded(mod_items, ..) = mod_kind {
                    let child = collect_module(ident.name.as_str(), mod_items);
                    module.children.push(child);
                }
            }
            _ => {}
        }
    }

    module
}

/// Collect a public function's signature into a [`FunctionModel`].
fn collect_function(fn_data: &ast::Fn) -> Option<FunctionModel> {
    let name = fn_data.ident.name.as_str().to_string();
    let params = collect_params(&fn_data.sig.decl.inputs);
    let return_type = collect_return_type(&fn_data.sig.decl.output);

    Some(FunctionModel {
        name,
        generics: GenericsModel::empty(),
        params,
        return_type,
        is_async: false,
        is_unsafe: false,
        abi: None,
    })
}

/// Collect a named struct's fields into a [`StructModel`].
///
/// Returns `None` for tuple structs and unit structs.
fn collect_struct(name: &str, fields: &ast::VariantData) -> Option<StructModel> {
    let ast::VariantData::Struct { fields: field_list, .. } = fields else {
        // Skip tuple structs and unit structs for now
        return None;
    };

    let fields = field_list
        .iter()
        .map(|f| FieldModel {
            name: f
                .ident
                .map(|i| i.name.as_str().to_string())
                .unwrap_or_else(|| "_".to_string()),
            ty: ty_to_type_repr(&f.ty),
            is_pub: matches!(f.vis.kind, ast::VisibilityKind::Public),
        })
        .collect::<Vec<_>>();

    let trackable = fields.iter().any(|f: &FieldModel| !f.is_pub);

    Some(StructModel {
        name: name.to_string(),
        generics: GenericsModel::empty(),
        fields,
        trackable,
        repr: None,
        trait_impls: Vec::new(),
    })
}

/// Collect an enum's variants into an [`EnumModel`].
fn collect_enum(name: &str, variants: &[ast::Variant]) -> Option<EnumModel> {
    let variants = variants
        .iter()
        .map(|v| {
            let fields = match &v.data {
                ast::VariantData::Tuple(fields, ..) => {
                    fields.iter().map(|f| ty_to_type_repr(&f.ty)).collect()
                }
                ast::VariantData::Struct { fields, .. } => {
                    fields.iter().map(|f| ty_to_type_repr(&f.ty)).collect()
                }
                ast::VariantData::Unit(..) => Vec::new(),
            };
            VariantModel {
                name: v.ident.name.as_str().to_string(),
                fields,
            }
        })
        .collect();

    Some(EnumModel {
        name: name.to_string(),
        generics: GenericsModel::empty(),
        variants,
        repr: None,
        trait_impls: Vec::new(),
    })
}

/// Collect a trait definition into a [`TraitModel`].
fn collect_trait(trait_data: &ast::Trait) -> Option<TraitModel> {
    let name = trait_data.ident.name.as_str().to_string();
    let methods = trait_data
        .items
        .iter()
        .filter_map(|item| {
            if let ast::AssocItemKind::Fn(fn_data) = &item.kind {
                Some(collect_method_sig(fn_data, &item.vis))
            } else {
                None
            }
        })
        .collect();

    Some(TraitModel {
        name,
        generics: GenericsModel::empty(),
        supertraits: Vec::new(),
        methods,
        associated_types: Vec::new(),
    })
}

/// Collect an impl block into an [`ImplModel`].
fn collect_impl(impl_data: &ast::Impl) -> Option<ImplModel> {
    // Get the self type name
    let self_type_name = extract_type_name(&impl_data.self_ty)?;

    // Get the trait name if this is a trait impl
    let trait_name = impl_data.of_trait.as_ref().and_then(|trait_header| {
        trait_header
            .trait_ref
            .path
            .segments
            .last()
            .map(|seg| seg.ident.name.as_str().to_string())
    });

    let methods = impl_data
        .items
        .iter()
        .filter_map(|item| {
            if let ast::AssocItemKind::Fn(fn_data) = &item.kind {
                // Only collect public methods for inherent impls;
                // for trait impls, collect all methods
                if trait_name.is_some() || is_pub(&item.vis) {
                    Some(collect_method_sig(fn_data, &item.vis))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let self_type = TypeRepr::simple(&self_type_name);
    let trait_ref = trait_name.as_ref().map(|n| TypeRepr::simple(n));

    Some(ImplModel {
        self_type,
        self_type_name,
        trait_ref,
        trait_name,
        generics: GenericsModel::empty(),
        methods,
        associated_types: Vec::new(),
    })
}

/// Collect a method signature into a [`MethodSigModel`].
fn collect_method_sig(fn_data: &ast::Fn, vis: &ast::Visibility) -> MethodSigModel {
    let name = fn_data.ident.name.as_str().to_string();
    let receiver = extract_receiver(&fn_data.sig.decl.inputs);
    let params = collect_method_params(&fn_data.sig.decl.inputs);
    let return_type = collect_return_type(&fn_data.sig.decl.output);
    let is_pub = is_pub(vis);

    MethodSigModel {
        name,
        generics: GenericsModel::empty(),
        receiver,
        params,
        return_type,
        is_pub,
        is_async: false,
        is_unsafe: false,
    }
}

/// Collect all function parameters into [`ParamModel`]s.
fn collect_params(inputs: &[ast::Param]) -> Vec<ParamModel> {
    inputs
        .iter()
        .filter_map(|param| {
            let name = extract_param_name(&param.pat)?;
            Some(ParamModel {
                name,
                ty: ty_to_type_repr(&param.ty),
            })
        })
        .collect()
}

/// Collect method parameters into [`ParamModel`]s, skipping the self receiver.
fn collect_method_params(inputs: &[ast::Param]) -> Vec<ParamModel> {
    inputs
        .iter()
        .filter(|param| !param.is_self())
        .filter_map(|param| {
            let name = extract_param_name(&param.pat)?;
            Some(ParamModel {
                name,
                ty: ty_to_type_repr(&param.ty),
            })
        })
        .collect()
}

/// Determine the receiver kind from the first parameter of a method.
fn extract_receiver(inputs: &[ast::Param]) -> ReceiverKind {
    let Some(first) = inputs.first() else {
        return ReceiverKind::None;
    };

    if !first.is_self() {
        return ReceiverKind::None;
    }

    // Determine if it's &self, &mut self, or self
    match &first.ty.kind {
        #[cfg(rustc_tyref_three_fields)]
        ast::TyKind::Ref(_, _, mutbl) => {
            if *mutbl == ast::Mutability::Mut {
                ReceiverKind::RefMut
            } else {
                ReceiverKind::Ref
            }
        }
        #[cfg(not(rustc_tyref_three_fields))]
        ast::TyKind::Ref(_, mut_ty) => {
            if mut_ty.mutbl == ast::Mutability::Mut {
                ReceiverKind::RefMut
            } else {
                ReceiverKind::Ref
            }
        }
        _ => ReceiverKind::Owned,
    }
}

/// Convert the return type declaration to a [`TypeRepr`].
///
/// Returns `TypeRepr::unit()` for the default (no explicit return type).
fn collect_return_type(output: &ast::FnRetTy) -> TypeRepr {
    match output {
        ast::FnRetTy::Ty(ty) => ty_to_type_repr(ty),
        ast::FnRetTy::Default(..) => TypeRepr::unit(),
    }
}

/// Extract the type name from a type AST node (last path segment).
fn extract_type_name(ty: &ast::Ty) -> Option<String> {
    if let ast::TyKind::Path(_, path) = &ty.kind {
        path.segments
            .last()
            .map(|seg| seg.ident.name.as_str().to_string())
    } else {
        None
    }
}

/// Extract a parameter name from a pattern node.
fn extract_param_name(pat: &ast::Pat) -> Option<String> {
    match &pat.kind {
        ast::PatKind::Ident(_, ident, _) => Some(ident.name.as_str().to_string()),
        _ => Some("_".to_string()),
    }
}

/// Check whether a visibility node represents `pub`.
fn is_pub(vis: &ast::Visibility) -> bool {
    matches!(vis.kind, ast::VisibilityKind::Public)
}

/// Convert an `ast::Ty` to a [`TypeRepr`].
///
/// This is a temporary bridge that uses `pprust::ty_to_string` to pretty-print
/// the type and wraps it in `TypeRepr::simple`. It will be replaced with proper
/// structural conversion in a later phase.
fn ty_to_type_repr(ty: &ast::Ty) -> TypeRepr {
    let ty_str = pprust::ty_to_string(ty);
    TypeRepr::simple(&ty_str)
}
