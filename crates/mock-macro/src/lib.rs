use context::time_mod::TimeModifier;
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
            match expr {
                Expr::Call(expr_call) => {
                    if let Expr::Path(expr_path) = *expr_call.func.clone()
                        && let Some(ident) = expr_path.path.get_ident()
                    {
                        if ident == "with_return" {
                            expr_call
                                .args
                                .first()
                                .cloned()
                                .inspect(|expr| ret = parse_quote! { Some(Box::new(|| #expr)) });
                        } else if ident == "once" {
                            time = context::time_mod::TimeModifier::Once
                        } else if ident == "any" {
                            time = context::time_mod::TimeModifier::Any
                        } else if ident == "at_least_once" {
                            time = context::time_mod::TimeModifier::AtLeastOnce
                        } else if ident == "at_most_once" {
                            time = context::time_mod::TimeModifier::AtMostOnce
                        } else if ident == "exit" {
                            exit = true;
                        };
                    }
                }
                Expr::Path(expr_path) => {
                    if let Some(ident) = expr_path
                        .path
                        .segments
                        .iter()
                        .last()
                        .map(|i| i.ident.clone())
                    {
                        if ident == "Once" {
                            time = context::time_mod::TimeModifier::Once
                        } else if ident == "Any" {
                            time = context::time_mod::TimeModifier::Any
                        } else if ident == "AtLeastOnce" {
                            time = context::time_mod::TimeModifier::AtLeastOnce
                        } else if ident == "AtMostOnce" {
                            time = context::time_mod::TimeModifier::AtMostOnce
                        }
                    }
                }
                _ => {}
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
    let mock_fn_data = parse_macro_input!(item as MockFnData);
    let input_types = mock_fn_data.input_types;
    let return_type = mock_fn_data.return_type;
    let path = mock_fn_data.path;
    let ident = mock_fn_data.ident;
    let mock_id = combine_path_and_ident(&path, &ident);
    let mock_id_string = format!("{}", quote! {#mock_id});
    let mock_id_ident = format_ident!("{}_mock_id", mock_id);
    //let return_type = quote! {#return_type};
    let default_return_val = mock_fn_data.default_return_val;
    let default_return: Expr = match &default_return_val {
        Some(expr) => parse_quote! {
            Some( Box::new(|| #default_return_val) )
        },
        None => parse_quote! { None },
    };
    //let default_return_val = quote! { #default_return_val };
    let input_type = quote! { (#(#input_types),*) };
    let mut setup_mock = quote! {
        let #mock_id_ident = context::MockId::new(#mock_id_string);

        if let Err(e) = context_builder.add_mock::<#return_type>(#mock_id_ident.clone(), #default_return) {
            panic!("failed to add mock, got error {:?}", e);
        }

    };
    mock_fn_data.expectations.into_iter().for_each(|e| {
        let ret = e.ret;
        let cond = e.condition;
        let exit = e.exit;
        let time = e.time;
        add_expectation_to_context(
            &mut setup_mock,
            return_type.clone(),
            mock_id_ident.clone(),
            input_type.clone(),
            ret,
            cond,
            exit,
            time,
        );
    });
    setup_mock.into()
}

fn add_expectation_to_context(
    appended: &mut proc_macro2::TokenStream,
    return_type: Type,
    mock_id_ident: Ident,
    input_type: proc_macro2::TokenStream,
    ret: Expr,
    cond: ExprClosure,
    exit: bool,
    time: TimeModifier,
) {
    let append = quote! {
    if let Err(e) = context_builder.add_expectation::<#input_type, #return_type>(&#mock_id_ident, Box::new( #cond ), #ret, #time, #exit) {
        panic!("failed to add mock, got error {:?}", e);
        };
    };
    appended.extend(append);
}

struct SliceData {
    name: Ident,
    expectations: Vec<Expectation>,
    time_mod: TimeModifier,
}

#[proc_macro]
pub fn slice(item: TokenStream) -> TokenStream {
    // should look like expect!(path, fnSig {expectation}, modifiers...)
    // so similar to mock_fn but without initializing the mock
    let fn_data = parse_macro_input!(item as MockFnData);
    assert!(
        fn_data.default_return_val.is_none(),
        "default return value not allowed in expect"
    );
    let input_types = fn_data.input_types;
    let input_type = quote! { (#(#input_types),*) };
    let mock_id = combine_path_and_ident(&fn_data.path, &fn_data.ident);
    let mock_id_string = format!("{}", quote! {#mock_id});
    let mock_id_ident = format_ident!("{}_mock_id", mock_id);
    let mut expects = quote! {};
    fn_data.expectations.into_iter().for_each(|e| {
        let ret = e.ret;
        let cond = e.condition;
        let exit = e.exit;
        let time = e.time;
        add_expectation_to_context(
            &mut expects,
            fn_data.return_type.clone(),
            mock_id_ident.clone(),
            input_type.clone(),
            ret,
            cond,
            exit,
            time,
        );
    });
    expects.into()
}
struct SequenceData {
    name: Ident,
    expectations: Vec<MockFnData>,
}
impl Parse for SequenceData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;

        let expectations = Punctuated::<MockFnData, Token![,]>::parse_terminated(&input)?
            .into_iter()
            .collect();
        Ok(Self { name, expectations })
    }
}
/*
pub fn sequence(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as SequenceData);

    TokenStream::new()
}*/

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
