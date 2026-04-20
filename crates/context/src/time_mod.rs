use std::collections::HashMap;

use derive_more::{Display, FromStr};
use proc_macro2::TokenStream;
use quote::quote;

pub struct ExpectationRef(u32);

#[derive(Debug, Clone, Display, FromStr)]
pub struct EnvId(String);
pub enum EnvVal {
    ExpectationRef(ExpectationRef),
    Int(u32),
}
pub type Environment = HashMap<EnvId, EnvVal>;

#[derive(Display, Debug, Clone)]
pub enum TimesValue {
    Explicit(u32),
    Implicit(EnvId),
}
#[derive(Display, Debug, Clone)]
pub enum TimeModifier {
    Once,
    AtMostOnce,
    Any,
    AtLeastOnce,
    Until(EnvId),
    Times(TimesValue),
    After(EnvId),
}

impl quote::ToTokens for TimeModifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let append = match self {
            TimeModifier::Once => quote! { context::TimeModifier::Once },
            TimeModifier::AtMostOnce => quote! { context::TimeModifier::AtMostOnce },
            TimeModifier::Any => quote! { context::TimeModifier::Any },
            TimeModifier::AtLeastOnce => quote! { context::TimeModifier::AtLeastOnce },
            TimeModifier::Until(env_id) => quote! { context::TimeModifier::Until },
            TimeModifier::Times(times_value) => quote! { context::TimeModifier::Times},
            TimeModifier::After(env_id) => quote! { context::TimeModifier::After },
        };
        tokens.extend(append);
    }
}
