use proc_macro::TokenStream;


#[proc_macro]
pub fn mock_fn(item: TokenStream) -> TokenStream {
    item
}

#[proc_macro]
pub fn mock_method(item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn mocked(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}