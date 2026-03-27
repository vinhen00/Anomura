use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, Path, Token, Type, bracketed,
    parse::{Parse, ParseStream},
    parse_quote, parse2,
    punctuated::Punctuated,
    spanned::Spanned,
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
        Err(e) => panic!("invalid mock_def! input: {} with error:  {e} ", &input),
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
    name: Path,
    path: Path,
    self_receiver: SelfReceiver,
    input_types: Vec<Type>,
    input_ident: Vec<Ident>,
    ret_type: Type,
    ret_val: Expr,
}



impl Parse for MockMethod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_name = parse_struct_field_value::<Path>("struct_name", &input, true)?;
        let name = parse_struct_field_value::<Path>("name", &input, true)?;
        let path = parse_struct_field_value::<Path>("path", &input, true)?;
        let self_receiver_ident = parse_struct_field_value::<Path>("self_receiver", &input, true)?;
        let self_receiver = match self_receiver_ident.segments.last().map(|s| s.ident.to_string()).as_deref() {
            Some("Ref") => SelfReceiver::Ref,
            Some("RefMut") => SelfReceiver::RefMut,
            _ => SelfReceiver::None,
        };
        let input_types = parse_struct_field_value_array("input_types", &input, true)?;
        let input_ident = parse_struct_field_value_array("input_ident", &input, true)?;
        let ret_type = parse_struct_field_value("ret_type", &input, true)?;
        let ret_val = parse_struct_field_value("ret_val", &input, false)?;

        

        Ok(MockMethod {
            struct_name,
            name,
            path,
            self_receiver,
            input_types,
            input_ident,
            ret_type,
            ret_val,
        })
    }
}

//Can only mock
pub fn expand_mock_method(input: TokenStream) -> TokenStream {
    //println!("Inside syn {}", input);
    let mock = match parse2::<MockMethod>(input) {
        Ok(m) => m,
        Err(e) => panic!("invalid mock_def! input: {}", e),
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
