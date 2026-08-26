//! Code generator for `mock_crate!` — generates Rust source strings from `CrateApiModel`
//! that implement the full mock infrastructure.
//!
//! The generated code is parsed back into AST items using `rustc_parse` and injected
//! into the target crate's AST within the same compiler session.

use rustc_ast_pretty::pprust;
use rustc_ast as ast;
use rustc_span::symbol::Symbol;

use crate::crate_api::*;

/// Generate mock dispatch body for a free function.
///
/// Returns a function source string with the mock dispatch body.
/// The original function is preserved as `{name}_original` by the caller.
pub fn gen_mock_fn_body(
    crate_name: &str,
    func: &FunctionModel,
) -> String {
    let fn_name = func.name.as_str();
    let mock_id = format!("{}_{}", crate_name, fn_name);

    let params_str = func.params.iter()
        .map(|p| format!("{}: {}", p.name.as_str(), ty_to_string(&p.ty)))
        .collect::<Vec<_>>()
        .join(", ");

    let input_types_str = func.params.iter()
        .map(|p| ty_to_string(&p.ty))
        .collect::<Vec<_>>()
        .join(", ");

    let input_idents_str = func.params.iter()
        .map(|p| p.name.as_str().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = match &func.return_type {
        Some(ty) => ty_to_string(ty),
        None => "()".to_string(),
    };

    // Wrap input_idents for the run_mock call (needs trailing comma for single-element tuples)
    let input_idents_tuple = if func.params.len() == 1 {
        format!("{},", input_idents_str)
    } else {
        input_idents_str.clone()
    };

    // Type tuple also needs trailing comma for single elements
    let input_types_tuple = if func.params.len() == 1 {
        format!("{},", input_types_str)
    } else {
        input_types_str.clone()
    };

    format!(
        r#"pub fn {fn_name}({params_str}) -> {ret_type_str} {{
    let {mock_id}_mock_id = context::MockId::new(stringify!({mock_id}));
    if context::ctx_built_and_contains_id(&{mock_id}_mock_id) {{
        match context::run_mock::<({input_types_tuple}), {ret_type_str}>({mock_id}_mock_id, ({input_idents_tuple})) {{
            Ok(res) => res,
            Err(e) => match e {{
                context::MockError::Other(e) => panic!("unexpected Error: {{:?}}", e),
                context::MockError::PredicateError(e) => panic!("{{:?}}", e.0),
                context::MockError::NoMatchingId => {{
                    panic!("failed to find mock id");
                }}
            }}
        }}
    }} else {{
        panic!("mock_crate: no mock context built for {fn_name}");
    }}
}}"#,
        fn_name = fn_name,
        params_str = params_str,
        ret_type_str = ret_type_str,
        mock_id = mock_id,
        input_types_tuple = input_types_tuple,
        input_idents_tuple = input_idents_tuple,
    )
}

/// Generate mock dispatch body for a method within an impl block.
pub fn gen_mock_method_body(
    crate_name: &str,
    struct_name: &str,
    method: &MethodSigModel,
) -> String {
    let method_name = method.name.as_str();
    let mock_id = format!("{}_{}_{}", crate_name, struct_name, method_name);

    let params_str = method.params.iter()
        .map(|p| format!("{}: {}", p.name.as_str(), ty_to_string(&p.ty)))
        .collect::<Vec<_>>()
        .join(", ");

    let input_types_str = method.params.iter()
        .map(|p| ty_to_string(&p.ty))
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = match &method.return_type {
        Some(ty) => ty_to_string(ty),
        None => "()".to_string(),
    };

    let (receiver_str, self_type_prefix, self_value_prefix) = match method.receiver {
        ReceiverKind::Ref => ("&self, ".to_string(), format!("&{}, ", struct_name), "self, ".to_string()),
        ReceiverKind::RefMut => ("&mut self, ".to_string(), format!("&mut {}, ", struct_name), "self, ".to_string()),
        ReceiverKind::Owned => ("self, ".to_string(), format!("{}, ", struct_name), "self, ".to_string()),
        ReceiverKind::None => ("".to_string(), "".to_string(), "".to_string()),
    };

    // Full type tuple for run_mock
    let full_input_types = if self_type_prefix.is_empty() {
        input_types_str.clone()
    } else if input_types_str.is_empty() {
        self_type_prefix.trim_end_matches(", ").to_string()
    } else {
        format!("{}{}", self_type_prefix, input_types_str)
    };

    // Full value tuple for run_mock
    let input_idents_str = method.params.iter()
        .map(|p| p.name.as_str().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let full_input_values = if self_value_prefix.is_empty() {
        input_idents_str.clone()
    } else if input_idents_str.is_empty() {
        self_value_prefix.trim_end_matches(", ").to_string()
    } else {
        format!("{}{}", self_value_prefix, input_idents_str)
    };

    // Trailing comma for single-element tuples
    let full_input_values_tuple = match (method.receiver != ReceiverKind::None, method.params.len()) {
        (true, 0) => format!("{},", full_input_values),
        (false, 1) => format!("{},", full_input_values),
        _ => full_input_values.clone(),
    };

    format!(
        r#"pub fn {method_name}({receiver_str}{params_str}) -> {ret_type_str} {{
    let {mock_id}_mock_id = context::MockId::new(stringify!({mock_id}));
    if context::ctx_built_and_contains_id(&{mock_id}_mock_id) {{
        match context::run_mock::<({full_input_types}), {ret_type_str}>({mock_id}_mock_id, ({full_input_values_tuple})) {{
            Ok(res) => res,
            Err(e) => match e {{
                context::MockError::Other(e) => panic!("unexpected Error: {{:?}}", e),
                context::MockError::PredicateError(e) => panic!("{{:?}}", e.0),
                context::MockError::NoMatchingId => panic!("failed to find mock id"),
            }}
        }}
    }} else {{
        panic!("mock_crate: no mock context built for {method_name}");
    }}
}}"#,
        method_name = method_name,
        receiver_str = receiver_str,
        params_str = params_str,
        ret_type_str = ret_type_str,
        mock_id = mock_id,
        full_input_types = full_input_types,
        full_input_values_tuple = full_input_values_tuple,
    )
}

/// Generate a full source string with all mock infrastructure for the crate.
/// This includes: mock dispatch bodies wrapped in a dummy crate structure that
/// can be parsed by `rustc_parse`.
///
/// For now: only generates function replacements (Phase 1).
pub fn gen_full_mock_source(api: &CrateApiModel) -> String {
    let mut source = String::new();

    // Generate mock fn bodies as top-level items (will be parsed and used to replace originals)
    // These are NOT injected directly — they're used as templates for body replacement.
    // Instead, we generate standalone helper items to inject.

    source
}

/// Helper: convert an ast::Ty to a string representation
fn ty_to_string(ty: &ast::Ty) -> String {
    pprust::ty_to_string(ty)
}
