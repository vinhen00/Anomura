use core::panic;

use proc_macro2::TokenStream;
use quote::quote;

use syn::{
    Expr, Fields, FnArg, Ident, Path, QSelf, Receiver, Token, 
    Type, bracketed, parse::{Parse, ParseStream}, parse_quote, 
    parse2, punctuated::Punctuated, spanned::Spanned, token::Token,
    visit_mut::{VisitMut, visit_expr_mut},
};

struct MockFun {
    name: Ident,
    path: Path,
    input_types: Vec<Type>,
    input_ident: Vec<Ident>,
    ret_type: Type,
    ret_val: Expr,
}

pub fn parse_struct_field_value<P: Parse>(
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
pub fn parse_struct_field_value_array<P: Parse>(
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

impl Parse for MockFun {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        /*  let name = parse_struct_field_value::<Path>("name", &input, true)?;
                let path = parse_struct_field_value::<Path>("path", &input, true)?;
                let input_types = parse_struct_field_value_array("input_types", &input, true)?;
                let input_ident = parse_struct_field_value_array("input_ident", &input, true)?;
                let ret_type = parse_struct_field_value("ret_type", &input, true)?;
                let ret_val = parse_struct_field_value("ret_val", &input, false)?;
        */
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
            /*let syn::Lit::Str(str) = expr_lit.lit else {
                log::debug!("expected lit_str found {:?}", quote! {expr_lit.lit});
                return None;
            };*/
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
                    syn::FnArg::Receiver(receiver) => todo!(),
                    syn::FnArg::Typed(pat_type) => *pat_type.ty.clone(),
                })
                .collect(),

            input_ident: fn_body
                .sig
                .inputs
                .iter()
                .map(|a| match a {
                    syn::FnArg::Receiver(receiver) => todo!(),
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

pub fn expand_mock_fn(input: TokenStream) -> TokenStream {
    //println!("Inside syn {}", input);
    let mock = match parse2::<MockFun>(input.clone()) {
        Ok(m) => m,
        Err(e) => panic!("invalid mock_def! input for function: {} with error:  {e} ", &input),
    };

    let name = mock.name;
    let path = mock.path;
    let name_str = quote! {#name}.to_string();
    let ret_type = mock.ret_type;
    let ret_val = mock.ret_val;

    let params = mock
        .input_ident
        .iter()
        .zip(mock.input_types.iter())
        .map(|(ident, ty)| quote! { #ident: #ty });

    let expanded = quote! {
        #[mocked( #path )]
        fn #name(#(#params),*) -> #ret_type {
            
            std::println!("Mocked version of function {} was used", #name_str);
            #ret_val
        }
    };

    expanded
}

enum SelfReceiver {
    None,
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
            /*let syn::Lit::Str(str) = expr_lit.lit else {
                log::debug!("expected lit_str found {:?}", quote! {expr_lit.lit});
                return None;
            };*/
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
                    if let Some(_) = receiver.mutability { self_receiver = SelfReceiver::RefMut }
                    else { self_receiver = SelfReceiver::Ref }
                }
                syn::FnArg::Typed(pat_type) => {
                    input_types.push(*pat_type.ty.clone());
                    match *pat_type.pat.clone() {
                        syn::Pat::Ident(pat_ident) => input_ident.push(pat_ident.ident),
                        _ => {}
                    }
                },

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

//Can only mock
pub fn expand_mock_method(input: TokenStream) -> TokenStream {
    //println!("Inside syn {}", input);
    let mock = match parse2::<MockMethod>(input) {
        Ok(m) => m,
        Err(e) => panic!("invalid mock_def! input for method: {}", e),
    };

    let struct_name = mock.struct_name.segments.last();
    let name = mock.name;
    let name_str = quote! {#name}.to_string();
    let path = mock.path;
    let ret_type = mock.ret_type;
    let ret_val = mock.ret_val;

    let receiver = match mock.self_receiver {
        SelfReceiver::Ref    => quote! { &self, },
        SelfReceiver::RefMut => quote! { &mut self, },
        SelfReceiver::None   => quote! {},
    };

    let params = mock
        .input_ident
        .iter()
        .zip(mock.input_types.iter())
        .map(|(ident, ty)| quote! { #ident: #ty });

    let expanded = quote! {
        impl #struct_name {
            #[mocked( #path )]
            fn #name(#receiver #(#params),*) -> #ret_type {
                println!("Mocked version of method {} was used", #name_str);
                #ret_val
            }
        }
    };

    expanded
}



struct MockMethod2 {
    name: Ident,
    self_receiver: SelfReceiver,
    input_types: Vec<Type>,
    input_ident: Vec<Ident>,
    ret_type: Type,
    ret_val: Expr,
}

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
            /*let syn::Lit::Str(str) = expr_lit.lit else {
                log::debug!("expected lit_str found {:?}", quote! {expr_lit.lit});
                return None;
            };*/
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
                    if let Some(_) = receiver.mutability { self_receiver = SelfReceiver::RefMut }
                    else { self_receiver = SelfReceiver::Ref }
                }
                syn::FnArg::Typed(pat_type) => {
                    input_types.push(*pat_type.ty.clone());
                    match *pat_type.pat.clone() {
                        syn::Pat::Ident(pat_ident) => input_ident.push(pat_ident.ident),
                        _ => {}
                    }
                },

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



struct MockStruct {
    name: Ident,
    path: Path,
    field_types: Vec<Type>,
    field_ident: Vec<Ident>,
    constructor: MockMethod2,
    methods: Vec<MockMethod2>,
}



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

        match(data.fields) {
            syn::Fields::Named(fields) => {
                for field in fields.named.iter() {
                    if let Some(ident) = &field.ident {
                        field_ident.push(ident.clone());
                        field_types.push(field.ty.clone());
                    } 
                    
                }
            }
            syn::Fields::Unnamed(fields) => {todo!()}
            syn::Fields::Unit => {todo!()}
        }
        input.parse::<Token![,]>()?;


        let constructor: MockMethod2 = input.parse()?;

        input.parse::<Token![,]>()?;
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


//Can only mock
pub fn expand_mock_struct(input: TokenStream) -> TokenStream {
    //println!("Inside syn {}", input);
    let mock = match parse2::<MockStruct>(input) {
        Ok(m) => m,
        Err(e) => panic!("invalid mock_def! input for struct: {}", e),
    };

    let name = mock.name;
    let name_str = quote! {#name}.to_string();
    let path = mock.path;

    let constructor = quote_method(mock.constructor, name_str.clone(), path.clone(), true);

    let methods: Vec<TokenStream> = mock.methods.into_iter().map(|m| quote_method(m, name_str.clone(), path.clone(), false)).collect();


    let fields = mock
        .field_ident
        .iter()
        .zip(mock.field_types.iter())
        .map(|(ident, ty)| quote! { #ident: #ty });

    let expanded = quote! { 
        #[mocked( #path )]       
        struct #name { #(#fields),* , mock_hash: String }
        impl #name {
            #constructor
            #(#methods)*
        }
    };

    expanded
}


fn quote_method(mock: MockMethod2, struct_string: String, path: Path, is_constructor: bool) -> TokenStream {
    let name = mock.name;
    let name_str = quote! {#name}.to_string();
    let ret_type = mock.ret_type;
    let mut ret_val;
    let mut hash_id_getter;
    if (is_constructor) {
        let mut visitor = RetvalFinder {name: "test".into()};
        ret_val = mock.ret_val.clone();
        visitor.visit_expr_mut(&mut ret_val);

        hash_id_getter = quote!{
            let Some(ctx) = context::GLOBAL_CONTEXT
                .get()
                else {
                    panic!{"Context not init"};
                };
            let mut guard = ctx.lock().expect("failed to fetch guard");

            let mock_hash = guard.get_incr_id();
            println!{"New instance of {} initialized with id {}", #struct_string, mock_hash};
        };
    }
    else {
        ret_val = mock.ret_val;
        hash_id_getter = quote! { println!{"{} object with id {} called function {}", #struct_string, self.mock_hash, #name_str } };
    }

    let receiver = match mock.self_receiver {
        SelfReceiver::Ref    => quote! { &self, },
        SelfReceiver::RefMut => quote! { &mut self, },
        SelfReceiver::None   => quote! {},
    };

    let params = mock
        .input_ident
        .iter()
        .zip(mock.input_types.iter())
        .map(|(ident, ty)| quote! { #ident: #ty });

    let expanded = quote! {
        #[mocked( #path )]
        fn #name(#receiver #(#params),*) -> #ret_type {
            #hash_id_getter
            #ret_val
        }
    };

    expanded
}

struct RetvalFinder {
    name: String
}

impl VisitMut for RetvalFinder {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::Struct(inner) = node {
            let hashval = syn::parse_str("mock_hash: mock_hash.to_string()").unwrap();
            inner.fields.push(hashval)
        }
        visit_expr_mut(self, node)
    }
}
