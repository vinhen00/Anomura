use quote::quote;
use proc_macro2::TokenStream;
use syn::{
    bracketed,
    parse2,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, Token, Type,
};


struct MockDef {
    name: Ident,
    input_types: Vec<Type>,
    input_ident: Vec<Ident>,
    ret_type: Type,
    ret_val: Expr,
}

impl Parse for MockDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut input_types = None;
        let mut input_ident = None;
        let mut ret_type = None;
        let mut ret_val = None;

        while !input.is_empty() {
            let field: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match field.to_string().as_str() {
                "name" => {
                    name = Some(input.parse()?);
                }
                "input_types" => {
                    let inner;
                    bracketed!(inner in input);
                    input_types = Some(
                        Punctuated::<Type, Token![,]>::parse_terminated(&inner)?
                            .into_iter()
                            .collect(),
                    );
                }
                "input_ident" => {
                    let inner;
                    bracketed!(inner in input);
                    input_ident = Some(
                        Punctuated::<Ident, Token![,]>::parse_terminated(&inner)?
                            .into_iter()
                            .collect(),
                    );
                }
                "ret_type" => {
                    ret_type = Some(input.parse()?);
                }
                "ret_val" => {
                    ret_val = Some(input.parse()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        field.span(),
                        "Unknown field",
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(MockDef {
            name: name.unwrap(),
            input_types: input_types.unwrap(),
            input_ident: input_ident.unwrap(),
            ret_type: ret_type.unwrap(),
            ret_val: ret_val.unwrap(),
        })
    }
}



pub fn expand_mock(input: TokenStream) -> TokenStream {
    //println!("Inside syn {}", input);
    let mock = match parse2::<MockDef>(input) {
        Ok(m) => m,
        Err(e) => panic!("invalid mock_def! input: {}", e),
    };
    

    let name = mock.name;
    let ret_type = mock.ret_type;
    let ret_val = mock.ret_val;

    let params = mock
        .input_ident
        .iter()
        .zip(mock.input_types.iter())
        .map(|(ident, ty)| quote! { #ident: #ty });


    let expanded = quote! {
        #[mocked]
        
        fn #name(#(#params),*) -> #ret_type {

            #ret_val
        }
    };

    TokenStream::from(expanded)
}








    


