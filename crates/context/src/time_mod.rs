use proc_macro2::TokenStream;
use quote::quote;

pub use crate::new_expectations::TimesModifier;

impl quote::ToTokens for TimesModifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let append = match self {
            TimesModifier::Once => quote! { context::TimesModifier::Once },
            TimesModifier::Times(n) => quote! { context::TimesModifier::Times(#n) },
            TimesModifier::AtLeast(n) => quote! { context::TimesModifier::AtLeast(#n) },
            TimesModifier::AtMost(n) => quote! { context::TimesModifier::AtMost(#n) },
            TimesModifier::Any => quote! { context::TimesModifier::Any },
            TimesModifier::Never => quote! { context::TimesModifier::Never },
        };
        tokens.extend(append);
    }
}
