use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemFn};

/// A derive macro that generates a `describe()` method on the type.
///
/// For a struct `Foo`, generates:
/// ```ignore
/// impl Foo {
///     pub fn describe(&self) -> &'static str {
///         "Foo"
///     }
/// }
/// ```
#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    let expanded = quote! {
        impl #name {
            pub fn describe(&self) -> &'static str {
                #name_str
            }
        }
    };

    TokenStream::from(expanded)
}

/// An attribute macro that generates a companion function `{name}_doubled`
/// that calls the original function and doubles the result.
///
/// ```ignore
/// #[generate_doubled]
/// pub fn magic_number() -> u32 { 21 }
///
/// // Generates:
/// pub fn magic_number_doubled() -> u32 {
///     magic_number() * 2
/// }
/// ```
#[proc_macro_attribute]
pub fn generate_doubled(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let doubled_name = syn::Ident::new(&format!("{}_doubled", name), name.span());
    let ret_ty = &input.sig.output;

    let expanded = quote! {
        #input

        pub fn #doubled_name() #ret_ty {
            #name() * 2
        }
    };

    TokenStream::from(expanded)
}
