use quote::quote;
use proc_macro2::TokenStream;
use std::str::FromStr;
use rustc_ast_pretty::pprust;
use syn::{
    bracketed,
    parse2,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, Token, Type,
};
use std::path::{Path, PathBuf};
use std::io;
use std::sync::Arc;
use std::fs::File;
use crate::CompileMocks;
use rustc_driver::{Compilation, run_compiler};








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
    let mock = match parse2::<MockDef>(input) {
        Ok(m) => m,
        Err(e) => return e.to_compile_error(),
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



pub struct MockDefsLoader {
    pub mockdefs: String,
}

impl rustc_span::source_map::FileLoader for MockDefsLoader {
    fn file_exists(&self, path: &Path) -> bool {
        path == Path::new("main.rs")
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        Ok(self.mockdefs.clone())
  
    }

    fn read_binary_file(&self, _path: &Path) -> io::Result<Arc<[u8]>> {
        Err(io::Error::other("oops"))
    }

    fn current_directory(&self) -> Result<PathBuf, std::io::Error> {
        Ok(PathBuf::from("."))
    }
}



impl CompileMocks {
    pub fn handleMacCall(&mut self, tokens: rustc_ast::tokenstream::TokenStream) {
        let syn_ts = TokenStream::from_str(&pprust::tts_to_string(&tokens))
        .expect("failed to parse token stream");
        let result = expand_mock(syn_ts);
        self.compileMacCall(result.to_string());
    }

    fn compileMacCall(&mut self, program: String) {
        let mut mockedFuns = CompileMocks {mocks: Vec::new(), inline: Some(program)};
        run_compiler(
            &[
                "ignored".to_string(),
                "mock_defs.rs".to_string(),
                "--crate-type".to_string(),
                "bin".to_string(),
                "-o".to_string(),
                "./target/mocked_main".to_string(),
            ],
            &mut mockedFuns,
        );
        for foo in mockedFuns.mocks{
            self.mocks.push(foo);
        }
    }
}
