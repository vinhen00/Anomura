use proc_macro::TokenStream;

#[proc_macro]
pub fn mock_fn(item: TokenStream) -> TokenStream {
    //do we need to generate items here? would not TokenStream::new() suffice?
    TokenStream::new()
}

#[proc_macro]
pub fn mock_method(item: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn mocked(attr: TokenStream, item: TokenStream) -> TokenStream {
    TokenStream::new()
}
