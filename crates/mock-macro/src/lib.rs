use proc_macro::TokenStream;

#[proc_macro]
pub fn mock_fn(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro]
pub fn mock_method(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn mocked(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
