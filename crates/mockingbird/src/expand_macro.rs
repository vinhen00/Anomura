use core::panic;

use mock_macro::{mock_method, mock_struct};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, Ident, Path, PathSegment, Token, Type, bracketed,
    parse::{Parse, ParseStream},
    parse_quote, parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{VisitMut, visit_expr_mut},
};

//Don't know if these are used, gives warning about non use, so might delete in future
pub fn _parse_struct_field_value<P: Parse>(
    field_name: &str,
    input: &ParseStream,
    postfix_comma: bool,
) -> syn::Result<P> {
    let field: Ident = input
        .parse::<Ident>()
        .map_err(|e| syn::Error::new(e.span(), "failed to parse MockFun name ident"))?;
    if field != field_name {
        return Err(syn::Error::new(
            field.span(),
            format!("expected field {field_name}. Got {} instead", field),
        ));
    }
    input.parse::<Token![:]>()?;
    let res = input.parse::<P>().map_err(|e| {
        syn::Error::new(
            e.span(),
            format!("failed to parse value of field_name {:?}", field_name),
        )
    })?;
    if postfix_comma {
        input.parse::<Token![,]>()?;
    }
    Ok(res)
}

//Don't know if these are used, gives warning about non use, so might delete in future
pub fn _parse_struct_field_value_array<P: Parse>(
    field_name: &str,
    input: &ParseStream,
    postfix_comma: bool,
) -> syn::Result<Vec<P>> {
    let field: Ident = input
        .parse::<Ident>()
        .map_err(|e| syn::Error::new(e.span(), "failed to parse MockFun name ident"))?;
    if field != field_name {
        return Err(syn::Error::new(
            field.span(),
            format!(
                "expected field_name {field_name} got {} instead",
                field_name
            ),
        ));
    }
    input.parse::<Token![:]>()?;
    let inner;
    bracketed!(inner in input);
    let res = Punctuated::<P, Token![,]>::parse_terminated(&inner)
        .map_err(|e| {
            syn::Error::new(
                e.span(),
                format!("failed to parse value of field_name {:?}", field_name),
            )
        })?
        .into_iter()
        .collect();
    if postfix_comma {
        input.parse::<Token![,]>()?;
    }
    Ok(res)
}

struct MockFun {
    name: Ident,
    path: Path,
    input_types: Vec<Type>,
    input_ident: Vec<Ident>,
    ret_type: Type,
    ret_val: Expr,
}

// mock_fn mocks a function, write more stuff
//
// example use:
// mock_fn!(
//     crate, //crate name
//     fn example (input : i32) -> i32 {
//         default_return( input * 2 )
//     }
// )

impl Parse for MockFun {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<Path>()?;
        input.parse::<Token![,]>()?;
        let fn_body: syn::ItemFn = input.parse()?;
        let Some(default_return_val) = fn_body.block.stmts.iter().find_map(|d| {
            let syn::Stmt::Expr(expr, _) = d else {
                log::debug!("expected stmtexpr found {:?}", quote! {d});
                return None;
            };

            let Expr::Call(expr_call) = expr else {
                log::debug!("expect expr call found {:?}", quote! {expr});
                return None;
            };
            let maybe_expr_lit = *expr_call.func.clone();
            let Expr::Path(expr_path) = maybe_expr_lit else {
                log::debug!("expected expr_path found {:?}", quote! {maybe_expr_lit});
                return None;
            };
            let Some(ident) = expr_path.path.get_ident() else {
                log::debug!("could not get ident from path");
                return None;
            };
            if ident == "default_return" {
                let e = expr_call.args.first();
                if e.is_none() {
                    log::debug!("first arg for default_return not found");
                }
                e
            } else {
                log::debug!("expected id default_return, found {:?}", ident);
                None
            }
        }) else {
            return Err(syn::Error::new(
                fn_body.span(),
                "no default return value found",
            ));
        };
        Ok(MockFun {
            name: fn_body.sig.ident,
            path,
            input_types: fn_body
                .sig
                .inputs
                .iter()
                .map(|a| match a {
                    syn::FnArg::Receiver(_receiver) => todo!(),
                    syn::FnArg::Typed(pat_type) => *pat_type.ty.clone(),
                })
                .collect(),

            input_ident: fn_body
                .sig
                .inputs
                .iter()
                .map(|a| match a {
                    syn::FnArg::Receiver(_receiver) => todo!(),
                    syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                        syn::Pat::Ident(pat_ident) => pat_ident.ident,
                        _ => todo!(),
                    },
                })
                .collect(),
            ret_type: match fn_body.sig.output {
                syn::ReturnType::Default => parse_quote!(()),
                syn::ReturnType::Type(_, t) => *t,
            },
            ret_val: default_return_val.to_owned(),
        })
    }
}
/// Strip a single-segment crate prefix from a type so it can be used inside that crate.
/// e.g. `fns::Foo` with crate_path `fns` → `Foo`, `&fns::Foo` → `&Foo`.
/// Types that don't start with the crate name are returned unchanged.
fn strip_crate_prefix(ty: Type, crate_path: &Path) -> Type {
    let Some(crate_seg) = crate_path.segments.first() else {
        return ty;
    };
    if crate_path.segments.len() != 1 {
        return ty;
    }
    match ty {
        Type::Path(mut type_path) if type_path.qself.is_none() => {
            let segs = &type_path.path.segments;
            if segs.len() > 1
                && segs
                    .first()
                    .map(|s| s.ident == crate_seg.ident)
                    .unwrap_or(false)
            {
                let new_segs: Punctuated<PathSegment, Token![::]> =
                    segs.iter().skip(1).cloned().collect();
                type_path.path.segments = new_segs;
            }
            Type::Path(type_path)
        }
        Type::Reference(mut type_ref) => {
            *type_ref.elem = strip_crate_prefix(*type_ref.elem, crate_path);
            Type::Reference(type_ref)
        }
        other => other,
    }
}

fn combine_path_struct_and_method(
    path: &syn::Path,
    struct_name: &syn::Path,
    method: &Ident,
) -> Ident {
    let mut parts: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    parts.extend(struct_name.segments.iter().map(|s| s.ident.to_string()));
    parts.push(method.to_string());
    format_ident!("{}", parts.join("_"), span = method.span())
}

fn combine_path_and_ident(path: &syn::Path, ident: &Ident) -> Ident {
    let mut parts: Vec<String> = path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect();

    parts.push(ident.to_string());

    let combined_name = parts.join("_");
    format_ident!("{}", combined_name, span = ident.span())
}
pub fn expand_mock_fn(input: TokenStream) -> TokenStream {
    //println!("Inside syn {}", input);
    let mock = match parse2::<MockFun>(input.clone()) {
        Ok(m) => m,
        Err(e) => panic!("invalid mock_def! input: {} with error:  {e} ", &input),
    };

    let name = mock.name;
    let original_name = format_ident!("{name}_original");
    let path = mock.path;
    let name_str = quote! {#name}.to_string();
    let ret_type = mock.ret_type;
    let mock_id = combine_path_and_ident(&path, &name);
    let mock_id_ident = format_ident!("{}_mock_id", mock_id);
    let input_types = mock.input_types;
    let input_idents = mock.input_ident;
    let input_ident_tuple = quote! { (#(#input_idents),*) };
    let input_idents_no_tuple = quote! { #(#input_idents),* };
    let input_type_tuple = quote! { (#(#input_types),*) };

    let params = input_idents
        .iter()
        .zip(input_types.iter())
        .map(|(ident, ty)| quote! { #ident: #ty });

    let expanded = quote! {

        #[mocked( #path )]
        fn #name(#(#params),*) -> #ret_type {

            std::println!("Mocked version of function {} was used", #name_str);
            let #mock_id_ident = context::MockId::new(stringify!(#mock_id));
            if context::ctx_built_and_contains_id(&#mock_id_ident) {
                match context::run_mock::<#input_type_tuple, #ret_type>(#mock_id_ident, #input_ident_tuple) {
                    Ok(res) => res,
                    Err(e) => match e {
                        context::MockError::Other(e) => panic!("unexpected Error: {:?}",e),
                        context::MockError::PredicateError(e) => panic!("{:?}", e.0),
                        context::MockError::NoMatchingId => {
                            panic!("failed to find mock id");
                        }
                    }
                }
            } else {return #original_name(#input_idents_no_tuple);}

        }
    };
    expanded
}

enum SelfReceiver {
    None,
    Owned,
    Ref,
    RefMut,
}

struct MockMethod {
    struct_name: Path,
    name: Ident,
    path: Path,
    self_receiver: SelfReceiver,
    input_types: Vec<Type>,
    input_ident: Vec<Ident>,
    ret_type: Type,
    ret_val: Expr,
}

// MockMethod works very similiar to mockfun. Use the macro to define how you want to mock the method
// it will replace the method in the implementation
//
// mock_method!(
//     crate, //crate name
//     Foo, //name of parent struct
//     fn example (&mut self, input : i32) -> i32 {
//         default_return( self.field + input )
//     }
// )
impl Parse for MockMethod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<Path>()?;
        input.parse::<Token![,]>()?;
        let struct_name = input.parse::<Path>()?;
        input.parse::<Token![,]>()?;
        let fn_body: syn::ItemFn = input.parse()?;
        let Some(default_return_val) = fn_body.block.stmts.iter().find_map(|d| {
            let syn::Stmt::Expr(expr, _) = d else {
                log::debug!("expected stmtexpr found {:?}", quote! {d});
                return None;
            };

            let Expr::Call(expr_call) = expr else {
                log::debug!("expect expr call found {:?}", quote! {expr});
                return None;
            };
            let maybe_expr_lit = *expr_call.func.clone();
            let Expr::Path(expr_path) = maybe_expr_lit else {
                log::debug!("expected expr_path found {:?}", quote! {maybe_expr_lit});
                return None;
            };
            let Some(ident) = expr_path.path.get_ident() else {
                log::debug!("could not get ident from path");
                return None;
            };
            if ident == "default_return" {
                let e = expr_call.args.first();
                if e.is_none() {
                    log::debug!("first arg for default_return not found");
                }
                e
            } else {
                log::debug!("expected id default_return, found {:?}", ident);
                None
            }
        }) else {
            return Err(syn::Error::new(
                fn_body.span(),
                "no default return value found",
            ));
        };

        let mut self_receiver = SelfReceiver::None;
        let mut input_types = Vec::new();
        let mut input_ident = Vec::new();

        for i in fn_body.sig.inputs.iter() {
            match i {
                //If there is a reciever (&self or &mut self) then set it correctly, otherwise stays as none
                syn::FnArg::Receiver(receiver) => {
                    self_receiver = if receiver.reference.is_none() {
                        SelfReceiver::Owned
                    } else if receiver.mutability.is_some() {
                        SelfReceiver::RefMut
                    } else {
                        SelfReceiver::Ref
                    };
                }
                syn::FnArg::Typed(pat_type) => {
                    input_types.push(*pat_type.ty.clone());
                    match *pat_type.pat.clone() {
                        syn::Pat::Ident(pat_ident) => input_ident.push(pat_ident.ident),
                        _ => {}
                    }
                }
            }
        }

        Ok(MockMethod {
            struct_name,
            name: fn_body.sig.ident,
            path,
            self_receiver,
            input_types,
            input_ident,
            ret_type: match fn_body.sig.output {
                syn::ReturnType::Default => parse_quote!(()),
                syn::ReturnType::Type(_, t) => *t,
            },
            ret_val: default_return_val.to_owned(),
        })
    }
}

pub fn expand_mock_method(input: TokenStream) -> TokenStream {
    let mock = match parse2::<MockMethod>(input) {
        Ok(m) => m,
        Err(e) => panic!("invalid mock_def! input: {}", e),
    };

    let struct_name = mock.struct_name;
    let name = mock.name;
    let original_name = format_ident!("{}_original", name);
    let name_str = quote! {#name}.to_string();
    let path = mock.path;
    let ret_type = mock.ret_type;
    let input_types = mock.input_types;
    let input_idents = mock.input_ident;

    let mock_id = combine_path_struct_and_method(&path, &struct_name, &name);
    let mock_id_ident = format_ident!("{}_mock_id", mock_id);

    let input_idents_no_tuple = quote! { #(#input_idents),* };
    let is_static = matches!(mock.self_receiver, SelfReceiver::None);

    let (receiver_quote, self_in_tuple, self_type_in_tuple) = match mock.self_receiver {
        SelfReceiver::None => (quote! {}, quote! {}, quote! {}),
        SelfReceiver::Owned => (quote! { self, }, quote! { self, }, quote! { #struct_name, }),
        SelfReceiver::Ref => (
            quote! { &self, },
            quote! { self, },
            quote! { &#struct_name, },
        ),
        SelfReceiver::RefMut => (
            quote! { &mut self, },
            quote! { self, },
            quote! { &mut #struct_name, },
        ),
    };

    let input_ident_tuple = quote! { (#self_in_tuple #(#input_idents),*) };
    let input_type_tuple = quote! { (#self_type_in_tuple #(#input_types),*) };

    // Strip the crate prefix for types used inside the target crate's compiled body.
    // e.g. `fns::Foo` (valid in test crate) → `Foo` (valid inside `fns` crate).
    let driver_ret_type = strip_crate_prefix(ret_type.clone(), &path);
    let driver_input_types: Vec<Type> = input_types
        .iter()
        .map(|ty| strip_crate_prefix(ty.clone(), &path))
        .collect();
    let driver_input_type_tuple = quote! { (#self_type_in_tuple #(#driver_input_types),*) };

    let fallback = if is_static {
        quote! { return #struct_name::#original_name(#input_idents_no_tuple); }
    } else {
        quote! { return self.#original_name(#input_idents_no_tuple); }
    };

    let driver_params = input_idents
        .iter()
        .zip(driver_input_types.iter())
        .map(|(ident, ty)| quote! { #ident: #ty });

    let expanded = quote! {
        impl #struct_name {
            #[mocked( #path )]
            fn #name(#receiver_quote #(#driver_params),*) -> #driver_ret_type {
                std::println!("Mocked version of method {} was used", #name_str);
                let mock_id_ident_string : String = format!("{}{}", stringify!(#mock_id), self.)
                let #mock_id_ident = context::MockId::new(stringify!(#mock_id));
                if context::ctx_built_and_contains_id(&#mock_id_ident) {
                    match context::run_mock::<#driver_input_type_tuple, #driver_ret_type>(#mock_id_ident, #input_ident_tuple) {
                        Ok(res) => res,
                        Err(e) => match e {
                            context::MockError::Other(e) => panic!("unexpected Error: {:?}", e),
                            context::MockError::PredicateError(e) => panic!("{:?}", e.0),
                            context::MockError::NoMatchingId => panic!("failed to find mock id"),
                        }
                    }
                } else { #fallback }
            }
        }
    };

    expanded
}

// MockMethod2 is methods defined in a mock struct. They need be a seperate struct in order to parse without needing path and struct name included
struct MockMethod2 {
    name: Ident,
    self_receiver: SelfReceiver,
    input_types: Vec<Type>,
    input_ident: Vec<Ident>,
    ret_type: Type,
    ret_val: Expr,
}

// Same parse as original method, but without parsing path and struct name
impl Parse for MockMethod2 {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fn_body: syn::ItemFn = input.parse()?;

        let Some(default_return_val) = fn_body.block.stmts.iter().find_map(|d| {
            let syn::Stmt::Expr(expr, _) = d else {
                log::debug!("expected stmtexpr found {:?}", quote! {d});
                return None;
            };

            let Expr::Call(expr_call) = expr else {
                log::debug!("expect expr call found {:?}", quote! {expr});
                return None;
            };

            let maybe_expr_lit = *expr_call.func.clone();
            let Expr::Path(expr_path) = maybe_expr_lit else {
                log::debug!("expected expr_path found {:?}", quote! {maybe_expr_lit});
                return None;
            };

            let Some(ident) = expr_path.path.get_ident() else {
                log::debug!("could not get ident from path");
                return None;
            };
            if ident == "default_return" {
                let e = expr_call.args.first();
                if e.is_none() {
                    log::debug!("first arg for default_return not found");
                }
                e
            } else {
                log::debug!("expected id default_return, found {:?}", ident);
                None
            }
        }) else {
            return Err(syn::Error::new(
                fn_body.span(),
                "no default return value found",
            ));
        };

        let mut self_receiver = SelfReceiver::None;
        let mut input_types = Vec::new();
        let mut input_ident = Vec::new();

        for i in fn_body.sig.inputs.iter() {
            match i {
                syn::FnArg::Receiver(receiver) => {
                    self_receiver = if receiver.reference.is_none() {
                        SelfReceiver::Owned
                    } else if receiver.mutability.is_some() {
                        SelfReceiver::RefMut
                    } else {
                        SelfReceiver::Ref
                    };
                }
                syn::FnArg::Typed(pat_type) => {
                    input_types.push(*pat_type.ty.clone());
                    match *pat_type.pat.clone() {
                        syn::Pat::Ident(pat_ident) => input_ident.push(pat_ident.ident),
                        _ => {}
                    }
                }
            }
        }

        Ok(MockMethod2 {
            name: fn_body.sig.ident,
            self_receiver,
            input_types,
            input_ident,
            ret_type: match fn_body.sig.output {
                syn::ReturnType::Default => parse_quote!(()),
                syn::ReturnType::Type(_, t) => *t,
            },
            ret_val: default_return_val.to_owned(),
        })
    }
}

//Mock struct, because of our requirements it requires a constructor method to be included first in the mock definition.
//Constructor method is a method that returns an instance of its parent struct.
//Methods field is a list of the other methods that the user wants to mock.
struct MockStruct {
    name: Ident,
    path: Path,
    field_types: Vec<Type>,
    field_ident: Vec<Ident>,
    constructor: MockMethod2,
    methods: Vec<MockMethod2>,
}

//Parse function for structs, example of how to use:
// mock_struct!(
//     crate, //Crate name
//     struct Example {
//         additional_field: u32
//     }

//     fn constructor() -> Example {
//         default_return ( Example { additional_field : 10 } )
//     }

//     //Other methods we want to mock
//     [
//         fn foo(input: i32) -> i32 {
//             default_return ( input * 2 )
//         }, //note comma

//         fn bar(input1: String, input2: String) -> String {
//             default_return ( format!("{input1} + {input2}") )
//         }
//     ]
// );

impl Parse for MockStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<Path>()?;
        input.parse::<Token![,]>()?;

        let input2: syn::DeriveInput = input.parse()?;

        let data = match input2.data {
            syn::Data::Struct(data_struct) => data_struct,
            _ => return Err(syn::Error::new_spanned(input2.ident, "expected a struct")),
        };

        let name = input2.ident;

        let mut field_ident = Vec::new();
        let mut field_types = Vec::new();

        match data.fields {
            syn::Fields::Named(fields) => {
                for field in fields.named.iter() {
                    if let Some(ident) = &field.ident {
                        field_ident.push(ident.clone());
                        field_types.push(field.ty.clone());
                    }
                }
            }
            syn::Fields::Unnamed(_fields) => {
                todo!()
            }
            syn::Fields::Unit => { todo!() }
        }



        let methods: Vec<MockMethod2> = {
            let content;
            syn::bracketed!(content in input);
            let methods_terminated: Punctuated<MockMethod2, Token![,]> =
                content.parse_terminated(MockMethod2::parse, syn::Token![,])?;
            methods_terminated.into_iter().collect()
        };

        Ok(MockStruct {
            name,
            path,
            field_types,
            field_ident,
            constructor,
            methods,
        })
    }
}

pub fn expand_mock_struct(input: TokenStream) -> TokenStream {
    let mock = match parse2::<MockStruct>(input) {
        Ok(m) => m,
        Err(e) => panic!("invalid mock_def! input for struct: {}", e),
    };

    let name = mock.name;
    let name_str = quote! {#name}.to_string();
    let path = mock.path;

    let constructor = quote_method(mock.constructor, name_str.clone(), path.clone(), true);

    let methods: Vec<TokenStream> = mock
        .methods
        .into_iter()
        .map(|m| quote_method(m, name_str.clone(), path.clone(), false))
        .collect();

    let fields = mock
        .field_ident
        .iter()
        .zip(mock.field_types.iter())
        .map(|(ident, ty)| quote! { pub #ident: #ty });

    let expanded = quote! {
        #[mocked( #path )]
        struct #name { #(#fields),* , pub mock_id: String }
        impl #name {
            #constructor
            #(#methods)*
        }
    };
    
    expanded
}

//Expands methods within a struct mock
fn quote_method(
    mock: MockMethod2,
    struct_string: String,
    path: Path,
    is_constructor: bool,
) -> TokenStream {
    let name = mock.name;
    let name_str = quote! {#name}.to_string();
    let ret_type = mock.ret_type;
    let ret_val;
    let hash_id_getter;
    let is_constructor = if let Type::Path(type_path) = mock.ret_type {
        if let Some(ident) = type_path.path.get_ident() {ident == "Self" || return ident == mock.name}
    }
    false
}
    //if our method is a constructor we need to fetch a new id for the initialization 
    if is_constructor {
        let mut visitor = RetvalFinder;
        ret_val = mock.ret_val.clone();
        //The retval declared by the user needs to be modified
        visitor.visit_expr_mut(&mut ret_val);

        //Get the context to assign a mock_hash id
        hash_id_getter = quote! {
            let mock_hash = context::get_new_id();
            eprintln!{"[INFO] New instance of {} initialized with id {}", #struct_string, mock_hash};
            mock_hash
        };
    } else {
        ret_val = mock.ret_val;

        //Currently just a print but should be some form of logging in context
        hash_id_getter = quote! { println!{"{} object with id {} called function {}", #struct_string, self.mock_hash, #name_str } };
    }

    let receiver = match mock.self_receiver {
        SelfReceiver::Ref => quote! { &self, },
        SelfReceiver::RefMut => quote! { &mut self, },
        SelfReceiver::None => quote! {},
        SelfReceiver::Owned => quote! { self, },
    };

    let params = mock
        .input_ident
        .iter()
        .zip(mock.input_types.iter())
        .map(|(ident, ty)| quote! { #ident: #ty });
    
    let expanded = quote! {
            fn #name(#receiver_quote #(#driver_params),*) -> #driver_ret_type {
                std::println!("Mocked version of method {} was used", #name_str);
                let mock_id_ident_string : String = format!("{}{}", stringify!(#mock_id), self.mock_id)
                let #mock_id_ident = context::MockId::new(stringify!(#mock_id));
                if context::ctx_built_and_contains_id(&#mock_id_ident) {
                    match context::run_mock::<#driver_input_type_tuple, #driver_ret_type>(#mock_id_ident, #input_ident_tuple) {
                        Ok(res) => res,
                        Err(e) => match e {
                            context::MockError::Other(e) => panic!("unexpected Error: {:?}", e),
                            context::MockError::PredicateError(e) => panic!("{:?}", e.0),
                            context::MockError::NoMatchingId => panic!("failed to find mock id"),
                        }
                    }
                } else { #fallback }
            }
        };
    

    /*let expanded = quote! {
        #[mocked( #path )]
        fn #name(#receiver #(#params),*) -> #ret_type {
            #hash_id_getter
            #ret_val
        }
    };*/

    expanded
}

struct RetvalFinder;

//Finds expressions and modifies the struct return value to add our custom fields
impl VisitMut for RetvalFinder {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::Struct(inner) = node {
            //the variable mock_hash has already been initialized in quote_method
            let hashval = syn::parse_str("mock_hash: mock_hash.to_string()").unwrap();
            inner.fields.push(hashval)
        }
        visit_expr_mut(self, node)
    }
}
