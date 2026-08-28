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
                context::MockError::AlreadyRegistered => unreachable!(),
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
    trackable: bool,
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

    // Trailing comma for single-element type tuples (must match value tuple)
    let full_input_types = match (method.receiver != ReceiverKind::None, method.params.len()) {
        (true, 0) => format!("{},", full_input_types),   // just self type → (T,)
        (false, 1) => format!("{},", full_input_types),  // single param → (T,)
        _ => full_input_types,
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

    // Mock ID: instance-specific for trackable structs with self, static otherwise
    let mock_id_expr = if trackable && method.receiver != ReceiverKind::None {
        format!("format!(\"{{}}{{}}\", \"{}\", self.adt_mock_id.0)", mock_id)
    } else {
        format!("stringify!({})", mock_id)
    };

    format!(
        r#"pub fn {method_name}({receiver_str}{params_str}) -> {ret_type_str} {{
    let {mock_id}_mock_id = context::MockId::new({mock_id_expr});
    if context::ctx_built_and_contains_id(&{mock_id}_mock_id) {{
        match context::run_mock::<({full_input_types}), {ret_type_str}>({mock_id}_mock_id, ({full_input_values_tuple})) {{
            Ok(res) => res,
            Err(e) => match e {{
                context::MockError::Other(e) => panic!("unexpected Error: {{:?}}", e),
                context::MockError::PredicateError(e) => panic!("{{:?}}", e.0),
                context::MockError::NoMatchingId => panic!("failed to find mock id"),
                context::MockError::AlreadyRegistered => unreachable!(),
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
        mock_id_expr = mock_id_expr,
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

/// Generate the convenience API (wrapper types + on_call helpers) for all
/// collected functions and methods.
/// Returns source code that can be parsed and injected as top-level items.
pub fn gen_convenience_api(api: &CrateApiModel) -> String {
    let mut source = String::new();

    // Generate wrappers for free functions
    for func in &api.root.functions {
        source.push_str(&gen_fn_wrappers(&api.crate_name, func));
        source.push('\n');
    }

    // Generate wrappers for impl methods (both inherent and trait impls)
    for imp in &api.root.impls {
        let struct_name = imp.self_type_name.as_str();
        let trackable = api.root.structs.iter()
            .find(|s| s.name.as_str() == struct_name)
            .map(|s| s.trackable)
            .unwrap_or(false);
        for method in &imp.methods {
            source.push_str(&gen_method_wrappers(&api.crate_name, struct_name, method, trackable, struct_name));
            source.push('\n');
        }
    }

    // Generate wrappers for functions in child modules (recursively)
    for child in &api.root.children {
        let mod_prefix = format!("{}_{}", api.crate_name, child.name.as_str());
        let mod_path = child.name.as_str().to_string();
        gen_module_convenience_wrappers(child, &mod_prefix, &mod_path, &mut source);
    }

    source
}

/// Recursively generate convenience API wrappers for a module's contents.
/// `prefix` is the accumulated mock_id prefix (e.g. "fns_a" or "fns_a_nested").
/// `mod_path` is the Rust module path (e.g. "a::nested") for qualifying type names
/// since convenience items are injected at the crate root.
fn gen_module_convenience_wrappers(module: &ModuleModel, prefix: &str, mod_path: &str, source: &mut String) {
    // Free functions in this module
    for func in &module.functions {
        source.push_str(&gen_fn_wrappers_with_prefix(prefix, func, module.name.as_str()));
        source.push('\n');
    }

    // Impl methods in this module
    for imp in &module.impls {
        let struct_name = imp.self_type_name.as_str();
        // Qualify the struct name with the module path so it resolves at the crate root
        let qualified_struct_name = format!("{}::{}", mod_path, struct_name);
        let trackable = module.structs.iter()
            .find(|s| s.name.as_str() == struct_name)
            .map(|s| s.trackable)
            .unwrap_or(false);
        for method in &imp.methods {
            source.push_str(&gen_method_wrappers(prefix, struct_name, method, trackable, &qualified_struct_name));
            source.push('\n');
        }
    }

    // Recurse into child modules
    for child in &module.children {
        let child_prefix = format!("{}_{}", prefix, child.name.as_str());
        let child_mod_path = format!("{}::{}", mod_path, child.name.as_str());
        gen_module_convenience_wrappers(child, &child_prefix, &child_mod_path, source);
    }
}

/// Generate wrapper newtypes and on_call for a free function.
fn gen_fn_wrappers(crate_name: &str, func: &FunctionModel) -> String {
    let fn_name = func.name.as_str();
    let fn_cap = capitalize(fn_name);
    let mock_id = format!("{}_{}", crate_name, fn_name);

    let input_types_str = func.params.iter()
        .map(|p| ty_to_string(&p.ty))
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = match &func.return_type {
        Some(ty) => ty_to_string(ty),
        None => "()".to_string(),
    };

    // Type tuple (with trailing comma for single-element)
    let type_tuple = if func.params.len() == 1 {
        format!("{},", input_types_str)
    } else {
        input_types_str.clone()
    };

    // Closure params (named, for the actual closure body)
    let closure_params = func.params.iter()
        .enumerate()
        .map(|(i, p)| format!("_{}: {}", i, ty_to_string(&p.ty)))
        .collect::<Vec<_>>()
        .join(", ");

    // Closure type params (types only, for Fn trait bound syntax)
    let closure_type_params = func.params.iter()
        .map(|p| ty_to_string(&p.ty))
        .collect::<Vec<_>>()
        .join(", ");

    // Predicate closure params (all by reference)
    let closure_params_ref = func.params.iter()
        .enumerate()
        .map(|(i, p)| format!("_{}: &{}", i, ty_to_string(&p.ty)))
        .collect::<Vec<_>>()
        .join(", ");

    // Predicate closure type params (types only, by reference)
    let closure_type_params_ref = func.params.iter()
        .map(|p| format!("&{}", ty_to_string(&p.ty)))
        .collect::<Vec<_>>()
        .join(", ");

    // Access from &tuple reference for predicate (pass references)
    let input_access_ref = func.params.iter()
        .enumerate()
        .map(|(i, _)| format!("&input.{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    // Closure args from destructured tuple
    let closure_args = func.params.iter()
        .enumerate()
        .map(|(i, _)| format!("_{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    // Destructure for return closure (value destructure)
    let destructure = if func.params.is_empty() {
        "".to_string()
    } else {
        func.params.iter()
            .enumerate()
            .map(|(i, _)| format!("_{}", i))
            .collect::<Vec<_>>()
            .join(", ")
    };

    // The full destructure pattern with trailing comma (for tuples)
    let destructure_pattern = if func.params.is_empty() {
        "()".to_string()
    } else {
        format!("({},)", destructure)
    };

    // Access from &tuple reference for predicate
    let input_access = func.params.iter()
        .enumerate()
        .map(|(i, _)| format!("input.{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
r#"pub struct Predicate{fn_cap}(pub context::Predicate);
pub struct Return{fn_cap}(pub context::ReturnValDoublePointer);

impl Return{fn_cap} {{
    pub fn from_fn(closure: impl Fn({closure_type_params}) -> {ret_type_str} + 'static) -> Self {{
        Self(context::ReturnValDoublePointer::from_fn::<({type_tuple}), {ret_type_str}>(
            Box::new(move |{destructure_pattern}| closure({closure_args}))
        ))
    }}
}}

impl Predicate{fn_cap} {{
    pub fn from_fn(closure: impl Fn({closure_type_params_ref}) -> context::errors::PredicateResult<()> + 'static) -> Self {{
        let mock_id = context::MockId::new("{mock_id}");
        let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(
            Box::new(move |input: &({type_tuple})| closure({input_access_ref}))
        );
        Self(context::Predicate::create_single::<({type_tuple})>(&mock_id, cond))
    }}
}}

pub fn on_call_{fn_name}(ret: impl Into<Return{fn_cap}>) {{
    let inner: Return{fn_cap} = ret.into();
    let mock_id = context::MockId::new("{mock_id}");
    match context::add_mock::<({type_tuple}), {ret_type_str}>(mock_id.clone(), None) {{
        Ok(()) => {{}},
        Err(context::MockError::AlreadyRegistered) => {{}},
        Err(e) => panic!("failed to add mock: {{:?}}", e),
    }}
    let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(Box::new(|_| Ok(())));
    context::add_expectation::<({type_tuple}), {ret_type_str}>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}}

pub fn expect_{fn_name}(condition: impl Fn({closure_type_params_ref}) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn({closure_type_params}) -> {ret_type_str} + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {{
    let mock_id = context::MockId::new("{mock_id}");
    match context::add_mock::<({type_tuple}), {ret_type_str}>(mock_id.clone(), None) {{
        Ok(()) => {{}},
        Err(context::MockError::AlreadyRegistered) => {{}},
        Err(e) => panic!("failed to add mock: {{:?}}", e),
    }}
    let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(
        Box::new(move |input: &({type_tuple})| condition({input_access_ref}))
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<({type_tuple}), {ret_type_str}>(
        Box::new(move |{destructure_pattern}| ret({closure_args}))
    );
    context::add_expectation::<({type_tuple}), {ret_type_str}>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}}

pub fn sequence_{fn_name}(seq_name: &str, index: usize, condition: impl Fn({closure_type_params_ref}) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn({closure_type_params}) -> {ret_type_str} + 'static) {{
    let mock_id = context::MockId::new("{mock_id}");
    match context::add_mock::<({type_tuple}), {ret_type_str}>(mock_id.clone(), None) {{
        Ok(()) => {{}},
        Err(context::MockError::AlreadyRegistered) => {{}},
        Err(e) => panic!("failed to add mock: {{:?}}", e),
    }}
    let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(
        Box::new(move |input: &({type_tuple})| condition({input_access_ref}))
    );
    context::add_expectation_to_sequence::<({type_tuple}), {ret_type_str}>(
        &mock_id, cond, Some(Box::new(move |{destructure_pattern}| ret({closure_args}))),
        seq_name, index, None,
    ).unwrap();
}}
"#,
        fn_cap = fn_cap,
        fn_name = fn_name,
        mock_id = mock_id,
        closure_type_params = closure_type_params,
        closure_type_params_ref = closure_type_params_ref,
        closure_args = closure_args,
        ret_type_str = ret_type_str,
        type_tuple = type_tuple,
        destructure_pattern = destructure_pattern,
        input_access_ref = input_access_ref,
    )
}

/// Generate wrapper newtypes and on_call for an impl method.
/// `struct_name` is the bare struct name (for mock IDs and wrapper type names).
/// `struct_type_path` is the fully qualified path for use in generated code (e.g., "a::nested::Inner").
fn gen_method_wrappers(crate_name: &str, struct_name: &str, method: &MethodSigModel, trackable: bool, struct_type_path: &str) -> String {
    let method_name = method.name.as_str();
    let suffix = format!("{}{}", capitalize(struct_name), capitalize(method_name));
    let mock_id = format!("{}_{}_{}", crate_name, struct_name, method_name);

    let input_types_str = method.params.iter()
        .map(|p| ty_to_string(&p.ty))
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = match &method.return_type {
        Some(ty) => ty_to_string(ty),
        None => "()".to_string(),
    };

    // Self type for the type tuple (use qualified path for code generation)
    let self_type = match method.receiver {
        ReceiverKind::Ref => format!("&{}", struct_type_path),
        ReceiverKind::RefMut => format!("&mut {}", struct_type_path),
        ReceiverKind::Owned => struct_type_path.to_string(),
        ReceiverKind::None => "".to_string(),
    };

    // Full type tuple
    let full_type_tuple = if self_type.is_empty() && input_types_str.is_empty() {
        "".to_string()
    } else if self_type.is_empty() {
        if method.params.len() == 1 { format!("{},", input_types_str) } else { input_types_str.clone() }
    } else if input_types_str.is_empty() {
        format!("{},", self_type)
    } else {
        format!("{}, {}", self_type, input_types_str)
    };

    // Closure params: self_ptr + method params
    let mut closure_params = Vec::new();
    if method.receiver != ReceiverKind::None {
        closure_params.push(format!("_self: {}", self_type));
    }
    for (i, p) in method.params.iter().enumerate() {
        closure_params.push(format!("_{}: {}", i, ty_to_string(&p.ty)));
    }
    let closure_params_str = closure_params.join(", ");

    // Closure type params (types only, for Fn trait bound)
    let mut closure_type_params = Vec::new();
    if method.receiver != ReceiverKind::None {
        closure_type_params.push(self_type.clone());
    }
    for p in method.params.iter() {
        closure_type_params.push(ty_to_string(&p.ty));
    }
    let closure_type_params_str = closure_type_params.join(", ");

    // Predicate closure params (all by reference)
    let mut closure_params_ref = Vec::new();
    if method.receiver != ReceiverKind::None {
        closure_params_ref.push(format!("_self: &{}", self_type));
    }
    for (i, p) in method.params.iter().enumerate() {
        closure_params_ref.push(format!("_{}: &{}", i, ty_to_string(&p.ty)));
    }
    let closure_params_ref_str = closure_params_ref.join(", ");

    // Predicate closure type params (types only, by reference)
    let mut closure_type_params_ref = Vec::new();
    if method.receiver != ReceiverKind::None {
        closure_type_params_ref.push(format!("&{}", self_type));
    }
    for p in method.params.iter() {
        closure_type_params_ref.push(format!("&{}", ty_to_string(&p.ty)));
    }
    let closure_type_params_ref_str = closure_type_params_ref.join(", ");

    // Access from &tuple reference for predicate (pass references)
    let total_fields = (if method.receiver != ReceiverKind::None { 1 } else { 0 }) + method.params.len();
    let input_access_ref = (0..total_fields)
        .map(|i| format!("&input.{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    // Closure args
    let mut closure_args = Vec::new();
    if method.receiver != ReceiverKind::None {
        closure_args.push("_self".to_string());
    }
    for (i, _) in method.params.iter().enumerate() {
        closure_args.push(format!("_{}", i));
    }
    let closure_args_str = closure_args.join(", ");

    // Destructure from input tuple (for return closure)
    let total_fields = (if method.receiver != ReceiverKind::None { 1 } else { 0 }) + method.params.len();
    let destructure = (0..total_fields)
        .map(|i| if i == 0 && method.receiver != ReceiverKind::None { "_self".to_string() } else { format!("_{}", i - if method.receiver != ReceiverKind::None { 1 } else { 0 }) })
        .collect::<Vec<_>>()
        .join(", ");

    let destructure_pattern = if total_fields == 0 {
        "()".to_string()
    } else {
        format!("({},)", destructure)
    };

    // Access from &tuple reference for predicate
    let input_access = (0..total_fields)
        .map(|i| format!("input.{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    // on_call: instance method for trackable, static for non-trackable
    let on_call_self_param = if trackable { "&self, " } else { "" };
    let on_call_mock_id_expr = if trackable {
        format!("format!(\"{{}}{{}}\", \"{}\", self.adt_mock_id.0)", mock_id)
    } else {
        format!("\"{}\"", mock_id)
    };

    format!(
r#"pub struct Predicate{suffix}(pub context::Predicate);
pub struct Return{suffix}(pub context::ReturnValDoublePointer);

impl Return{suffix} {{
    pub fn from_fn(closure: impl Fn({closure_type_params_str}) -> {ret_type_str} + 'static) -> Self {{
        Self(context::ReturnValDoublePointer::from_fn::<({full_type_tuple}), {ret_type_str}>(
            Box::new(move |{destructure_pattern}| closure({closure_args_str}))
        ))
    }}
}}

impl Predicate{suffix} {{
    pub fn from_fn(closure: impl Fn({closure_type_params_ref_str}) -> context::errors::PredicateResult<()> + 'static) -> Self {{
        let mock_id = context::MockId::new("{mock_id}");
        let cond = context::ConditionDoublePointer::from_fn::<({full_type_tuple})>(
            Box::new(move |input: &({full_type_tuple})| closure({input_access_ref}))
        );
        Self(context::Predicate::create_single::<({full_type_tuple})>(&mock_id, cond))
    }}
}}

impl {struct_type_path} {{
    pub fn on_call_{method_name}({on_call_self_param}ret: impl Into<Return{suffix}>) {{
        let inner: Return{suffix} = ret.into();
        let mock_id = context::MockId::new({on_call_mock_id_expr});
        match context::add_mock::<({full_type_tuple}), {ret_type_str}>(mock_id.clone(), None) {{
            Ok(()) => {{}},
            Err(context::MockError::AlreadyRegistered) => {{}},
            Err(e) => panic!("failed to add mock: {{:?}}", e),
        }}
        let cond = context::ConditionDoublePointer::from_fn::<({full_type_tuple})>(Box::new(|_| Ok(())));
        context::add_expectation::<({full_type_tuple}), {ret_type_str}>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }}
}}
"#,
        suffix = suffix,
        mock_id = mock_id,
        struct_type_path = struct_type_path,
        method_name = method_name,
        closure_type_params_str = closure_type_params_str,
        closure_type_params_ref_str = closure_type_params_ref_str,
        closure_args_str = closure_args_str,
        ret_type_str = ret_type_str,
        full_type_tuple = full_type_tuple,
        destructure_pattern = destructure_pattern,
        input_access_ref = input_access_ref,
        on_call_self_param = on_call_self_param,
        on_call_mock_id_expr = on_call_mock_id_expr,
    )
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Generate wrapper newtypes and on_call for a function inside a submodule.
/// The mock_id uses the full path (crate_mod_fn), and the on_call function is
/// placed inside a `mod` block so it's accessible as `fns::a::on_call_modules(...)`.
fn gen_fn_wrappers_with_prefix(mock_id_prefix: &str, func: &FunctionModel, mod_name: &str) -> String {
    let fn_name = func.name.as_str();
    let fn_cap = format!("{}_{}", capitalize(mod_name), capitalize(fn_name));
    let mock_id = format!("{}_{}", mock_id_prefix, fn_name);

    let input_types_str = func.params.iter()
        .map(|p| ty_to_string(&p.ty))
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = match &func.return_type {
        Some(ty) => ty_to_string(ty),
        None => "()".to_string(),
    };

    let type_tuple = if func.params.len() == 1 {
        format!("{},", input_types_str)
    } else {
        input_types_str.clone()
    };

    let closure_type_params = func.params.iter()
        .map(|p| ty_to_string(&p.ty))
        .collect::<Vec<_>>()
        .join(", ");

    let closure_args = func.params.iter()
        .enumerate()
        .map(|(i, _)| format!("_{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    let destructure_pattern = if func.params.is_empty() {
        "()".to_string()
    } else {
        let d = func.params.iter()
            .enumerate()
            .map(|(i, _)| format!("_{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        format!("({},)", d)
    };

    let closure_type_params_ref = func.params.iter()
        .map(|p| format!("&{}", ty_to_string(&p.ty)))
        .collect::<Vec<_>>()
        .join(", ");

    let input_access_ref = func.params.iter()
        .enumerate()
        .map(|(i, _)| format!("&input.{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
r#"pub struct Predicate{fn_cap}(pub context::Predicate);
pub struct Return{fn_cap}(pub context::ReturnValDoublePointer);

impl Return{fn_cap} {{
    pub fn from_fn(closure: impl Fn({closure_type_params}) -> {ret_type_str} + 'static) -> Self {{
        Self(context::ReturnValDoublePointer::from_fn::<({type_tuple}), {ret_type_str}>(
            Box::new(move |{destructure_pattern}| closure({closure_args}))
        ))
    }}
}}

impl Predicate{fn_cap} {{
    pub fn from_fn(closure: impl Fn({closure_type_params_ref}) -> context::errors::PredicateResult<()> + 'static) -> Self {{
        let mock_id = context::MockId::new("{mock_id}");
        let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(
            Box::new(move |input: &({type_tuple})| closure({input_access_ref}))
        );
        Self(context::Predicate::create_single::<({type_tuple})>(&mock_id, cond))
    }}
}}

pub fn on_call_{mod_name}_{fn_name}(ret: impl Into<Return{fn_cap}>) {{
    let inner: Return{fn_cap} = ret.into();
    let mock_id = context::MockId::new("{mock_id}");
    match context::add_mock::<({type_tuple}), {ret_type_str}>(mock_id.clone(), None) {{
        Ok(()) => {{}},
        Err(context::MockError::AlreadyRegistered) => {{}},
        Err(e) => panic!("failed to add mock: {{:?}}", e),
    }}
    let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(Box::new(|_| Ok(())));
    context::add_expectation::<({type_tuple}), {ret_type_str}>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}}
"#,
        fn_cap = fn_cap,
        fn_name = fn_name,
        mod_name = mod_name,
        mock_id = mock_id,
        closure_type_params = closure_type_params,
        closure_type_params_ref = closure_type_params_ref,
        closure_args = closure_args,
        ret_type_str = ret_type_str,
        type_tuple = type_tuple,
        destructure_pattern = destructure_pattern,
        input_access_ref = input_access_ref,
    )
}

/// Generate a constructor body that:
/// 1. Constructs the struct (pub fields matched from params, private fields as PhantomData)
/// 2. Calls add_mock for every mockable method
/// 3. Returns the instance
pub fn gen_constructor_body(
    crate_name: &str,
    struct_name: &str,
    constructor: &MethodSigModel,
    struct_model: &super::crate_api::StructModel,
    all_methods: &[MethodSigModel],
    all_public: bool,
) -> String {
    let ctor_name = constructor.name.as_str();

    let params_str = constructor.params.iter()
        .map(|p| format!("{}: {}", p.name.as_str(), ty_to_string(&p.ty)))
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = match &constructor.return_type {
        Some(ty) => ty_to_string(ty),
        None => "Self".to_string(),
    };

    // Build field initializers
    let field_inits: Vec<String> = struct_model.fields.iter().map(|f| {
        let field_name = f.name.as_str();
        if f.is_pub {
            // Check if a constructor param matches this field name
            let has_param = constructor.params.iter().any(|p| p.name.as_str() == field_name);
            if has_param {
                format!("            {}", field_name)
            } else {
                format!("            {}: Default::default()", field_name)
            }
        } else {
            // Private field → PhantomData
            format!("            {}: std::marker::PhantomData", field_name)
        }
    }).collect();

    // Add adt_mock_id for trackable structs
    let adt_mock_id_init = if !all_public {
        "            adt_mock_id: context::new_id(),\n"
    } else {
        ""
    };

    let field_inits_str = field_inits.join(",\n");

    // Generate add_mock calls for every mockable method (methods with a self receiver)
    let mock_registrations: Vec<String> = all_methods.iter()
        .filter(|m| m.receiver != super::crate_api::ReceiverKind::None)
        .map(|m| {
            let method_name = m.name.as_str();
            let mock_id_prefix = format!("{}_{}_{}", crate_name, struct_name, method_name);

            let self_type = match m.receiver {
                super::crate_api::ReceiverKind::Ref => format!("&{}", struct_name),
                super::crate_api::ReceiverKind::RefMut => format!("&mut {}", struct_name),
                super::crate_api::ReceiverKind::Owned => struct_name.to_string(),
                super::crate_api::ReceiverKind::None => unreachable!(),
            };

            let input_types = m.params.iter()
                .map(|p| ty_to_string(&p.ty))
                .collect::<Vec<_>>()
                .join(", ");

            let full_type_tuple = if input_types.is_empty() {
                format!("{},", self_type)
            } else {
                format!("{}, {}", self_type, input_types)
            };

            let ret_type = match &m.return_type {
                Some(ty) => ty_to_string(ty),
                None => "()".to_string(),
            };

            if all_public {
                format!(
                    r#"        let _ = context::add_mock::<({full_type_tuple}), {ret_type}>(context::MockId::new("{mock_id_prefix}"), None);"#,
                    full_type_tuple = full_type_tuple,
                    ret_type = ret_type,
                    mock_id_prefix = mock_id_prefix,
                )
            } else {
                format!(
                    r#"        context::add_mock::<({full_type_tuple}), {ret_type}>(context::MockId::new(format!("{{}}{{}}", "{mock_id_prefix}", slf.adt_mock_id.0)), None).unwrap();"#,
                    full_type_tuple = full_type_tuple,
                    ret_type = ret_type,
                    mock_id_prefix = mock_id_prefix,
                )
            }
        })
        .collect();

    let mock_registrations_str = mock_registrations.join("\n");

    format!(
        r#"pub fn {ctor_name}({params_str}) -> {ret_type_str} {{
        let slf = Self {{
{field_inits_str},
{adt_mock_id_init}        }};
{mock_registrations_str}
        slf
    }}"#,
        ctor_name = ctor_name,
        params_str = params_str,
        ret_type_str = ret_type_str,
        field_inits_str = field_inits_str,
        adt_mock_id_init = adt_mock_id_init,
        mock_registrations_str = mock_registrations_str,
    )
}

/// Check if a method is a constructor (no self receiver, returns Self or the struct type)
pub fn is_constructor(method: &MethodSigModel, struct_name: &str) -> bool {
    if method.receiver != super::crate_api::ReceiverKind::None {
        return false;
    }
    match &method.return_type {
        Some(ty) => {
            let ty_str = ty_to_string(ty);
            ty_str == "Self" || ty_str == struct_name
        }
        None => false,
    }
}
