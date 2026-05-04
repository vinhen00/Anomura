use context::time_mod::TimeModifier;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, ExprClosure, Ident, Token, Type, parse::Parse, parse_macro_input, parse_quote,
    punctuated::Punctuated, spanned::Spanned, token::Comma,
};

struct Expectation {
    condition: ExprClosure,
    time: TimeModifier,
    ret: syn::Expr,
    exit: bool,
}

impl Expectation {
    fn from_exprs(input_idents: &[Ident], exprs: Punctuated<Expr, Comma>) -> Self {
        let mut exit = false;
        let mut ret: Expr = parse_quote! { None };
        let mut time = TimeModifier::Once;
        let mut exprs = exprs.into_iter();
        let Some(expr) = exprs.next() else {
            panic!("no expr");
        };

        let input_tuple = quote! { (#(#input_idents),*) };
        let error_format = input_idents
            .iter()
            .map(|_| quote! { {}})
            .collect::<Vec<_>>();
        let error_format = quote! { #(#error_format),*};
        let error_format_args = quote! { #(#input_idents),*};
        let error_string = format!("condition {} failed for {error_format}", quote! {#expr });
        let condition: ExprClosure = parse_quote! {
            |#input_tuple| if #expr {Ok(())} else { Err(format!( #error_string , #error_format_args ).into())}
        };
        for expr in exprs {
            match expr {
                Expr::Call(expr_call) => {
                    if let Expr::Path(expr_path) = *expr_call.func.clone()
                        && let Some(ident) = expr_path.path.get_ident()
                    {
                        if ident == "with_return" {
                            expr_call.args.first().cloned().inspect(|expr| {
                                ret = parse_quote! { Some(Box::new(|#input_tuple| #expr)) }
                            });
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
                return None;
            };
            let ident = expr_path.path.get_ident()?;
            if ident == "default_return" {
                let e = expr_call.args.first();
                e.cloned()
            } else {
                None
            }
        });
        let input_idents: Vec<Ident> = fun
            .sig
            .inputs
            .iter()
            .map(|a| match a {
                syn::FnArg::Receiver(receiver) => todo!(),
                syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                    syn::Pat::Ident(pat_ident) => pat_ident.ident,
                    _ => todo!(),
                },
            })
            .collect();
        let expectations: Vec<Expectation> = fun
            .block
            .stmts
            .into_iter()
            .filter_map(|stmt| {
                let syn::Stmt::Expr(expr, _) = stmt else {
                    return None;
                };
                let Expr::Call(expr_call) = expr else {
                    return None;
                };
                let maybe_expr_lit = *expr_call.func.clone();
                let Expr::Path(expr_path) = maybe_expr_lit else {
                    return None;
                };
                let ident = expr_path.path.get_ident()?;
                if ident == "expect" {
                    let e = Expectation::from_exprs(&input_idents, expr_call.args);
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
            input_idents,
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
    input_idents: Vec<Ident>,
    input_types: Vec<Type>,
    return_type: Type,
    default_return_val: Option<syn::Expr>,
    expectations: Vec<Expectation>,
}

fn path_to_ident(path: &syn::Path) -> Ident {
    let parts: Vec<String> = path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect();

    let combined_name = parts.join("_");
    format_ident!("{}", combined_name, span = path.span())
}
fn combine_path_and_ident(path: &syn::Path, ident: &Ident) -> Ident {
    let mut parts: Vec<String> = path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect();

    parts.push(ident.to_string());

    let combined_name = parts.join("_");
    format_ident!("{}", combined_name, span = ident.span())
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
    let input_idents = mock_fn_data.input_idents;
    let input_tuple = quote! { (#(#input_idents),*) };
    //let return_type = quote! {#return_type};
    let default_return_val = mock_fn_data.default_return_val;
    let default_return: Expr = match &default_return_val {
        Some(expr) => parse_quote! {
            Some( Box::new(|#input_tuple| #default_return_val) )
        },
        None => parse_quote! { None },
    };
    //let default_return_val = quote! { #default_return_val };
    let input_type = quote! { (#(#input_types),*) };
    let mut setup_mock = quote! {
        let #mock_id_ident = context::MockId::new(#mock_id_string);

        if let Err(e) = context::add_mock::<#input_type, #return_type>(#mock_id_ident.clone(), #default_return) {
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
    time: TimeModifier,
) {
    let append = quote! {
    if let Err(e) = context::add_expectation::<#input_type, #return_type>(&#mock_id_ident, Box::new( #cond ), #ret, #time) {
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
        let time = e.time;
        add_expectation_to_context(
            &mut expects,
            fn_data.return_type.clone(),
            mock_id_ident.clone(),
            input_type.clone(),
            ret,
            cond,
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
/*
struct ExpectData {
    path: syn::Path,
    expectation: Expectation,
    input_types: Vec<Type>,
    return_type: Type,
}
impl Parse for ExpectData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path = input.parse::<syn::Path>()?;
        let krate = path.segments.first().unwrap().ident.to_string();
        let name = path.segments.last().unwrap().ident.to_string();
        input.parse::<Token![,]>()?;
        let exprs = Punctuated::<Expr, Token![,]>::parse_terminated(&input)?;
        let program = std::env::var(driver_test::SUBSTITUTION_MOCK_PATHS).unwrap();
        let map = driver_test::mock_map_from_program(program);
        let mocks = map
            .get(&krate)
            .expect(&format!("krate with string id {:?} wasnt found", krate));
        let Some((input_idents, input_types, return_type)) = mocks.into_iter().find_map(|m| {
            let m_name = m.get_name();
            if m_name == name {
                Some((
                    m.get_idents()
                        .into_iter()
                        .map(|s| Ident::new(&s, path.span()))
                        .collect::<Vec<_>>(),
                    m.input_types(),
                    m.return_type(),
                ))
            } else {
                None
            }
        }) else {
            let err = syn::Error::new(
                path.span(),
                "did not find a matching mock definition in expectdata",
            );
            return Err(err);
        };

        let expectation = Expectation::from_exprs(&input_idents, exprs);
        Ok(ExpectData {
            path,
            expectation,
            input_types,
            return_type,
        })
    }
}
#[proc_macro]
pub fn expect(item: TokenStream) -> TokenStream {
    let expect_data = parse_macro_input!(item as ExpectData);
    let input_types = expect_data.input_types;
    let input_type = quote! { (#(#input_types),*) };
    let return_type = expect_data.return_type;
    let mock_id = path_to_ident(&expect_data.path);
    let mock_id_string = format!("{}", quote! {#mock_id});
    let cond = expect_data.expectation.condition;
    let ret = expect_data.expectation.ret;
    let time = expect_data.expectation.time;
    quote! {
    if let Err(e) = context::add_expectation::<#input_type, #return_type>(&context::MockId::new(#mock_id_string), Box::new( #cond ), #ret, #time) {
        panic!("failed to add mock, got error {:?}", e);
        };
    }.into()
}*/

#[proc_macro]
pub fn mock_method(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
