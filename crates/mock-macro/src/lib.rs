use context::TimeModifier;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, ExprClosure, Ident, Token, Type, parse::Parse, parse_macro_input, parse_quote,
    punctuated::Punctuated, token::Comma,
};

struct Expectation {
    condition: ExprClosure,
    time: TimeModifier,
    ret: Option<syn::Expr>,
    exit: bool,
}

impl Expectation {
    fn from_exprs(exprs: Punctuated<Expr, Comma>) -> Self {
        let mut exit = false;
        let mut ret = None;
        let mut time = TimeModifier::Once;
        let mut exprs = exprs.into_iter();
        let Some(syn::Expr::Closure(condition)) = exprs.next() else {
            panic!("no expr");
        };
        for expr in exprs {
            if let Expr::Call(expr_call) = expr
                && let Expr::Path(expr_path) = *expr_call.func.clone()
                && let Some(ident) = expr_path.path.get_ident()
            {
                if ident == "with_return" {
                    ret = expr_call.args.first().cloned();
                } else if ident == "once" {
                    time = TimeModifier::Once
                } else if ident == "any" {
                    time = TimeModifier::Any
                } else if ident == "at_least_once" {
                    time = TimeModifier::AtLeastOnce
                } else if ident == "at_most_once" {
                    time = TimeModifier::AtMostOnce
                } else if ident == "exit" {
                    exit = true;
                };
            }
        }

        Self {
            condition,
            time,
            ret,
            exit,
        }
    }
}

impl Parse for MockFnData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path = input.parse::<syn::Path>()?;
        input.parse::<Token![,]>()?;
        let fun = input.parse::<syn::ItemFn>()?;
        let ident = fun.sig.ident;
        let return_type = match fun.sig.output {
            syn::ReturnType::Default => parse_quote!(()),
            syn::ReturnType::Type(_, t) => *t,
        };
        let default_return_val = fun.block.stmts.iter().find_map(|d| {
            let syn::Stmt::Expr(expr, _) = d else {
                //panic!("expected stmtexpr found {:?}", quote! {d});
                return None;
            };

            let Expr::Call(expr_call) = expr else {
                //panic!("expect expr call found {:?}", quote! {expr});
                return None;
            };
            let maybe_expr_lit = *expr_call.func.clone();
            let Expr::Path(expr_path) = maybe_expr_lit else {
                //panic!("expected expr_path found {:?}", quote! {maybe_expr_lit});
                return None;
            };
            let Some(ident) = expr_path.path.get_ident() else {
                //panic!("could not get ident from path");
                return None;
            };
            if ident == "default_return" {
                let e = expr_call.args.first();
                if e.is_none() {
                    //panic!("first arg for default_return not found");
                }
                e.cloned()
            } else {
                //panic!("expected id default_return, found {:?}", ident);
                None
            }
        });
        let expectations: Vec<Expectation> = fun
            .block
            .stmts
            .into_iter()
            .filter_map(|stmt| {
                let syn::Stmt::Expr(expr, _) = stmt else {
                    //panic!("expected stmtexpr found {:?}", quote! {d});
                    return None;
                };

                let Expr::Call(expr_call) = expr else {
                    //panic!("expect expr call found {:?}", quote! {expr});
                    return None;
                };
                let maybe_expr_lit = *expr_call.func.clone();
                let Expr::Path(expr_path) = maybe_expr_lit else {
                    //panic!("expected expr_path found {:?}", quote! {maybe_expr_lit});
                    return None;
                };
                let Some(ident) = expr_path.path.get_ident() else {
                    //panic!("could not get ident from path");
                    return None;
                };
                if ident == "expect" {
                    let e = Expectation::from_exprs(expr_call.args);
                    Some(e)
                } else {
                    None
                }
            })
            .collect();
        let input_types: Vec<Type> = fun
            .sig
            .inputs
            .iter()
            .map(|a| match a {
                syn::FnArg::Receiver(receiver) => todo!(),
                syn::FnArg::Typed(pat_type) => *pat_type.ty.clone(),
            })
            .collect();
        Ok(Self {
            path,
            ident,
            input_types,
            return_type,
            default_return_val,
            expectations,
        })
    }
}

struct MockFnData {
    path: syn::Path,
    ident: Ident,
    input_types: Vec<Type>,
    return_type: Type,
    default_return_val: Option<syn::Expr>,
    expectations: Vec<Expectation>,
}

#[proc_macro]
pub fn mock_fn(item: TokenStream) -> TokenStream {
    let MockFnData = parse_macro_input!(item as MockFnData);
    TokenStream::new()
}

#[proc_macro]
pub fn mock_method(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro]
pub fn start_mock_setup(_item: TokenStream) -> TokenStream {
    quote! {
        let mut context_builder = context::ContextBuilder::new();
    }
    .into()
}

#[proc_macro]
pub fn end_mock_setup(_item: TokenStream) -> TokenStream {
    quote! {  ;
        let binding = context::GLOBAL_CONTEXT.get_or_init(|| Mutex::new(context_builder.finish()));
        drop(binding);
    }
    .into()
}
