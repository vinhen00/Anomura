extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{
    ExprReturn, LitStr,
    parse::{Parse, ParseStream},
    token::Comma,
};

///  We can't dedicate a rust specific identifier for our mocks, so we can determine our mocks instead by generating a special SHA256 hash. the source is project_mockingbird_hash.
///  17075fa291fd4e8398464656284672a3572383ef59ccbbc126df1fbdbab6538f
const MOCK_HASH_IDENTIFIER: &str =
    "17075fa291fd4e8398464656284672a3572383ef59ccbbc126df1fbdbab6538f";

enum Modifier {
    Return(ExprReturn),
}
impl Parse for Modifier {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr: syn::Expr = input.parse()?;
        let syn::Expr::Return(expr_return) = expr else {
            return Err(input.error("failed to parse mock modifier"));
        };
        Ok(Modifier::Return(expr_return))
    }
}

struct MockDefInput {
    ident: LitStr,
    modifiers: Vec<Modifier>,
}

impl Parse for MockDefInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::LitStr = input.parse()?;

        let modifiers = input.parse_terminated(Modifier::parse, Comma)?;
        let modifiers = modifiers.into_iter().collect();

        Ok(Self { ident, modifiers })
    }
}

#[proc_macro]
///for now this macro is completely magical -- It generates nothing, but is parsed before expansion to find functions to mock
pub fn mock(input: TokenStream) -> TokenStream {
    TokenStream::new()
}
