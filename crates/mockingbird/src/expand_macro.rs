use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    Expr, Ident, Path, Token, Type,
};

struct MockFun {
    name: Path,
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
        let name = parse_struct_field_value::<Path>("name", &input, true)?;
        let path = parse_struct_field_value::<Path>("path", &input, true)?;
        let input_types = parse_struct_field_value_array("input_types", &input, true)?;
        let input_ident = parse_struct_field_value_array("input_ident", &input, true)?;
        let ret_type = parse_struct_field_value("ret_type", &input, true)?;
        let ret_val = parse_struct_field_value("ret_val", &input, false)?;

        Ok(MockFun {
            name,
            path,
            input_types,
            input_ident,
            ret_type,
            ret_val,
        })
    }
}

pub fn expand_mock_fn(input: TokenStream) -> TokenStream {
    //println!("Inside syn {}", input);
    let mock = match parse2::<MockFun>(input.clone()) {
        Ok(m) => m,
        Err(e) => panic!("invalid mock_def! input: {} with error:  {e} ", &input),
    };

    let name = mock.name.segments.last();
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
            use std::println;
            println!("Mocked version of function {} was used", #name_str);
            #ret_val
        }
    };

    expanded
}

struct MockMethod {
    struct_name: Path,
    name: Path,
    path: Path,
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
        let input_types = parse_struct_field_value_array("input_types", &input, true)?;
        let input_ident = parse_struct_field_value_array("input_ident", &input, true)?;
        let ret_type = parse_struct_field_value("ret_type", &input, true)?;
        let ret_val = parse_struct_field_value("ret_val", &input, false)?;

        Ok(MockMethod {
            struct_name,
            name,
            path,
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

    let params = mock
        .input_ident
        .iter()
        .zip(mock.input_types.iter())
        .map(|(ident, ty)| quote! { #ident: #ty });

    let expanded = quote! {
        impl #struct_name {
            #[mocked( #path )]
            fn #name(&mut self, #(#params),*) -> #ret_type {
                println!("Mocked version of method {} was used", #name_str);
                #ret_val
            }
        }
    };

    expanded
}
