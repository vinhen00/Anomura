//! Code generator for `mock_crate!` — generates Rust source strings from `CrateApiModel`
//! that implement the full mock infrastructure.
//!
//! The generated code is parsed back into AST items using `rustc_parse` and injected
//! into the target crate's AST within the same compiler session.

use crate::crate_api::*;
use crate::generics_model::{GenericParams, Receiver};

/// Generate mock dispatch body for a free function.
///
/// Returns a function source string with the mock dispatch body.
/// The original function is preserved as `{name}_original` by the caller.
pub fn gen_mock_fn_body(crate_name: &str, fn_name: &str, func: &FunctionData) -> String {
    let mock_id = format!("{}_{}", crate_name, fn_name);
    let fn_generics = func.generics.to_source();
    let where_clause = func.generics.where_clause_to_source();
    let qualifiers = func.qualifiers.to_source_prefix();

    let params_str = func
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name.as_str(), p.ty.to_source()))
        .collect::<Vec<_>>()
        .join(", ");

    let input_types_str = func
        .params
        .iter()
        .map(|p| p.ty.to_source())
        .collect::<Vec<_>>()
        .join(", ");

    let input_idents_str = func
        .params
        .iter()
        .map(|p| p.name.as_str().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = func.return_type.to_source();

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
        r#"pub {qualifiers}fn {fn_name}{fn_generics}({params_str}) -> {ret_type_str} {where_clause}{{
    let {mock_id}_mock_id = context::MockId::new_fn("{mock_id}");
    if !context::ctx_built() {{
        panic!("mock_crate: `{fn_name}` called during build phase — call finish_building_context() first");
    }}
    if context::contains_id(&{mock_id}_mock_id) {{
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
        panic!("mock_crate: no mock registered for `{fn_name}`");
    }}
}}"#,
        fn_name = fn_name,
        qualifiers = qualifiers,
        fn_generics = fn_generics,
        params_str = params_str,
        ret_type_str = ret_type_str,
        mock_id = mock_id,
        input_types_tuple = input_types_tuple,
        input_idents_tuple = input_idents_tuple,
        where_clause = if where_clause.is_empty() { String::new() } else { format!("{} ", where_clause) },
    )
}

/// Generate mock dispatch body for a method within an impl block.
pub fn gen_mock_method_body(
    crate_name: &str,
    struct_name: &str,
    method_name: &str,
    method: &MethodData,
    trackable: bool,
    is_trait_impl: bool,
    struct_type_path: &str,
) -> String {
    let adt_path = format!("{}_{}", crate_name, struct_name);
    let mock_id = format!("{}_{}", adt_path, method_name);

    let params_str = method
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name.as_str(), p.ty.to_source()))
        .collect::<Vec<_>>()
        .join(", ");

    let input_types_str = method
        .params
        .iter()
        .map(|p| p.ty.to_source())
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = method.return_type.to_source();

    let (receiver_str, self_type_prefix, self_value_prefix) = match &method.receiver {
        Receiver::Ref | Receiver::RefLifetime(_) => (
            format!("{}, ", method.receiver.to_source()),
            format!("&{}, ", struct_type_path),
            "self, ".to_string(),
        ),
        Receiver::RefMut | Receiver::RefMutLifetime(_) => (
            format!("{}, ", method.receiver.to_source()),
            format!("&mut {}, ", struct_type_path),
            "self, ".to_string(),
        ),
        Receiver::Owned => (
            "self, ".to_string(),
            format!("{}, ", struct_type_path),
            "self, ".to_string(),
        ),
        Receiver::Arbitrary(ty) => (
            format!("{}, ", method.receiver.to_source()),
            format!("{}, ", ty.to_source()),
            "self, ".to_string(),
        ),
        Receiver::None => ("".to_string(), "".to_string(), "".to_string()),
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
    let full_input_types = match (method.receiver.has_self(), method.params.len()) {
        (true, 0) => format!("{},", full_input_types), // just self type → (T,)
        (false, 1) => format!("{},", full_input_types), // single param → (T,)
        _ => full_input_types,
    };

    // Full value tuple for run_mock
    let input_idents_str = method
        .params
        .iter()
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
    let full_input_values_tuple = match (method.receiver.has_self(), method.params.len())
    {
        (true, 0) => format!("{},", full_input_values),
        (false, 1) => format!("{},", full_input_values),
        _ => full_input_values.clone(),
    };

    // Mock ID: instance-specific for trackable structs with self, static otherwise
    let mock_id_expr = if trackable && method.receiver.has_self() {
        format!(
            "context::MockId::new_adt_instance(\"{}\", \"{}\", self.adt_mock_id)",
            adt_path, method_name
        )
    } else {
        format!("context::MockId::new_adt_static(\"{}\", \"{}\")", adt_path, method_name)
    };

    let method_generics = method.generics.to_source();
    let method_where = method.generics.where_clause_to_source();
    let method_where_str = if method_where.is_empty() { String::new() } else { format!("{} ", method_where) };
    let qualifiers = method.qualifiers.to_source_prefix();
    let visibility = if is_trait_impl { "" } else { "pub " };

    format!(
        r#"{visibility}{qualifiers}fn {method_name}{method_generics}({receiver_str}{params_str}) -> {ret_type_str} {method_where_str}{{
    let {mock_id}_mock_id = {mock_id_expr};
    if !context::ctx_built() {{
        panic!("mock_crate: `{method_name}` called during build phase — call finish_building_context() first");
    }}
    if context::contains_id(&{mock_id}_mock_id) {{
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
        panic!("mock_crate: no mock registered for `{method_name}`");
    }}
}}"#,
        method_name = method_name,
        visibility = visibility,
        qualifiers = qualifiers,
        method_generics = method_generics,
        receiver_str = receiver_str,
        params_str = params_str,
        ret_type_str = ret_type_str,
        mock_id = mock_id,
        mock_id_expr = mock_id_expr,
        full_input_types = full_input_types,
        full_input_values_tuple = full_input_values_tuple,
        method_where_str = method_where_str,
    )
}

/// Generate wrapper newtypes and on_call for a free function.
fn gen_fn_wrappers(crate_name: &str, fn_name: &str, func: &FunctionData) -> String {
    let fn_cap = capitalize(fn_name);
    let mock_id = format!("{}_{}", crate_name, fn_name);
    let fn_generics = func.generics.to_source();
    let fn_where = func.generics.where_clause_to_source();
    let fn_where_str = if fn_where.is_empty() { String::new() } else { format!(" {fn_where}") };
    let is_generic = !func.generics.is_empty();

    let input_types_str = func
        .params
        .iter()
        .map(|p| p.ty.to_source())
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = func.return_type.to_source();

    // Type tuple (with trailing comma for single-element)
    let type_tuple = if func.params.len() == 1 {
        format!("{},", input_types_str)
    } else {
        input_types_str.clone()
    };

    // Closure params (named, for the actual closure body)
    let closure_params = func
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| format!("_{}: {}", i, p.ty.to_source()))
        .collect::<Vec<_>>()
        .join(", ");

    // Closure type params (types only, for Fn trait bound syntax)
    let closure_type_params = func
        .params
        .iter()
        .map(|p| p.ty.to_source())
        .collect::<Vec<_>>()
        .join(", ");

    // Predicate closure params (all by reference)
    let closure_params_ref = func
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| format!("_{}: &{}", i, p.ty.to_source()))
        .collect::<Vec<_>>()
        .join(", ");

    // Predicate closure type params (types only, by reference)
    let closure_type_params_ref = func
        .params
        .iter()
        .map(|p| format!("&{}", p.ty.to_source()))
        .collect::<Vec<_>>()
        .join(", ");

    // Access from &tuple reference for predicate (pass references)
    let input_access_ref = func
        .params
        .iter()
        .enumerate()
        .map(|(i, _)| format!("&input.{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    // Closure args from destructured tuple
    let closure_args = func
        .params
        .iter()
        .enumerate()
        .map(|(i, _)| format!("_{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    // Destructure for return closure (value destructure)
    let destructure = if func.params.is_empty() {
        "".to_string()
    } else {
        func.params
            .iter()
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
    let input_access = func
        .params
        .iter()
        .enumerate()
        .map(|(i, _)| format!("input.{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    // Predicate/Return wrapper structs — only for non-generic functions
    let wrapper_structs = if is_generic {
        String::new()
    } else {
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
        let mock_id = context::MockId::new_fn("{mock_id}");
        let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(
            Box::new(move |input: &({type_tuple})| closure({input_access_ref}))
        );
        Self(context::Predicate::create_single::<({type_tuple})>(&mock_id, cond))
    }}
}}
"#,
            fn_cap = fn_cap,
            mock_id = mock_id,
            closure_type_params = closure_type_params,
            closure_type_params_ref = closure_type_params_ref,
            closure_args = closure_args,
            ret_type_str = ret_type_str,
            type_tuple = type_tuple,
            destructure_pattern = destructure_pattern,
            input_access_ref = input_access_ref,
        )
    };

    format!(
        r#"{wrapper_structs}
pub fn on_call_{fn_name}{fn_generics}(condition: impl Fn({closure_type_params_ref}) -> bool + 'static, ret: impl Fn({closure_type_params}) -> {ret_type_str} + 'static){fn_where_str} {{
    let mock_id = context::MockId::new_fn("{mock_id}");
    let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(
        Box::new(move |input: &({type_tuple})| {{
            if condition({input_access_ref}) {{
                Ok(())
            }} else {{
                Err("on_call condition failed for {fn_name}".into())
            }}
        }})
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<({type_tuple}), {ret_type_str}>(
        Box::new(move |{destructure_pattern}| ret({closure_args}))
    );
    context::add_on_call::<({type_tuple})>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}}

pub fn expect_{fn_name}{fn_generics}(condition: impl Fn({closure_type_params_ref}) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn({closure_type_params}) -> {ret_type_str} + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>){fn_where_str} {{
    let mock_id = context::MockId::new_fn("{mock_id}");
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

pub fn sequence_{fn_name}{fn_generics}(seq_name: &str, index: usize, condition: impl Fn({closure_type_params_ref}) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn({closure_type_params}) -> {ret_type_str} + 'static){fn_where_str} {{
    let mock_id = context::MockId::new_fn("{mock_id}");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(
        Box::new(move |input: &({type_tuple})| condition({input_access_ref}))
    );
    context::add_expectation_to_sequence::<({type_tuple}), {ret_type_str}>(
        &mock_id, cond, Some(Box::new(move |{destructure_pattern}| ret({closure_args}))),
        seq_name, index, None,
    ).unwrap();
}}
"#,
        wrapper_structs = wrapper_structs,
        fn_name = fn_name,
        fn_generics = fn_generics,
        fn_where_str = fn_where_str,
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
fn gen_method_wrappers(
    path: &ModPath,
    struct_name: &str,
    method_name: &str,
    method: &MethodData,
    trackable: bool,
    struct_type_path: &str,
    ret_type_override: Option<&str>,
    impl_generics: &GenericParams,
) -> String {
    let suffix = format!("{}{}", capitalize(struct_name), capitalize(method_name));
    let object_mock_id = path.mock_prefix(struct_name);

    let input_types_str = method
        .params
        .iter()
        .map(|p| p.ty.to_source())
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = if let Some(override_ty) = ret_type_override {
        override_ty.to_string()
    } else {
        method.return_type.to_source()
    };

    // Self type for the type tuple (use qualified path for code generation)
    let self_type = match &method.receiver {
        Receiver::Ref | Receiver::RefLifetime(_) => format!("&{}", struct_type_path),
        Receiver::RefMut | Receiver::RefMutLifetime(_) => format!("&mut {}", struct_type_path),
        Receiver::Owned => struct_type_path.to_string(),
        Receiver::Arbitrary(ty) => ty.to_source(),
        Receiver::None => "".to_string(),
    };

    // Full type tuple
    let full_type_tuple = if self_type.is_empty() && input_types_str.is_empty() {
        "".to_string()
    } else if self_type.is_empty() {
        if method.params.len() == 1 {
            format!("{},", input_types_str)
        } else {
            input_types_str.clone()
        }
    } else if input_types_str.is_empty() {
        format!("{},", self_type)
    } else {
        format!("{}, {}", self_type, input_types_str)
    };

    // Closure params: self_ptr + method params
    let mut closure_params = Vec::new();
    if method.receiver.has_self() {
        closure_params.push(format!("_self: {}", self_type));
    }
    for (i, p) in method.params.iter().enumerate() {
        closure_params.push(format!("_{}: {}", i, p.ty.to_source()));
    }
    let closure_params_str = closure_params.join(", ");

    // Closure type params (types only, for Fn trait bound)
    let mut closure_type_params = Vec::new();
    if method.receiver.has_self() {
        closure_type_params.push(self_type.clone());
    }
    for p in method.params.iter() {
        closure_type_params.push(p.ty.to_source());
    }
    let closure_type_params_str = closure_type_params.join(", ");

    // Predicate closure params (all by reference)
    // For self types that are already references (&T, &mut T), don't add another &.
    // For &mut self, we reborrow as &T for condition closures (conditions shouldn't mutate).
    let self_already_ref = method.receiver.is_ref() || method.receiver.is_ref_mut();
    let self_is_mut_ref = method.receiver.is_ref_mut();
    // The user-facing condition type for self: &T for both &self and &mut self
    let condition_self_type = if self_is_mut_ref {
        format!("&{}", struct_type_path) // &mut Foo → &Foo for condition
    } else {
        self_type.clone() // &Foo stays &Foo, Foo stays Foo
    };
    let mut closure_params_ref = Vec::new();
    if method.receiver.has_self() {
        if self_already_ref {
            closure_params_ref.push(format!("_self: {}", condition_self_type));
        } else {
            closure_params_ref.push(format!("_self: &{}", self_type));
        }
    }
    for (i, p) in method.params.iter().enumerate() {
        closure_params_ref.push(format!("_{}: &{}", i, p.ty.to_source()));
    }
    let closure_params_ref_str = closure_params_ref.join(", ");

    // Predicate closure type params (types only, by reference)
    let mut closure_type_params_ref = Vec::new();
    if method.receiver.has_self() {
        if self_already_ref {
            closure_type_params_ref.push(condition_self_type.clone());
        } else {
            closure_type_params_ref.push(format!("&{}", self_type));
        }
    }
    for p in method.params.iter() {
        closure_type_params_ref.push(format!("&{}", p.ty.to_source()));
    }
    let closure_type_params_ref_str = closure_type_params_ref.join(", ");

    // Access from &tuple reference for predicate (pass references)
    // For &self: input.0 is &T, pass it directly → &T
    // For &mut self: input.0 is &mut T, reborrow as &*input.0 → &T
    // For owned self: input.0 is T, pass &input.0 → &T
    // For params: always &input.N
    let total_fields = (if method.receiver.has_self() {
        1
    } else {
        0
    }) + method.params.len();
    let self_field_count = if method.receiver.has_self() { 1 } else { 0 };
    let input_access_ref = (0..total_fields)
        .map(|i| {
            if i == 0 && self_is_mut_ref {
                format!("&*input.{}", i) // reborrow &mut T as &T
            } else if i == 0 && self_already_ref {
                format!("input.{}", i) // &T passed directly
            } else {
                format!("&input.{}", i)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    // Closure args
    let mut closure_args = Vec::new();
    if method.receiver.has_self() {
        closure_args.push("_self".to_string());
    }
    for (i, _) in method.params.iter().enumerate() {
        closure_args.push(format!("_{}", i));
    }
    let closure_args_str = closure_args.join(", ");

    // Destructure from input tuple (for return closure)
    let total_fields = (if method.receiver.has_self() {
        1
    } else {
        0
    }) + method.params.len();
    let destructure = (0..total_fields)
        .map(|i| {
            if i == 0 && method.receiver.has_self() {
                "_self".to_string()
            } else {
                format!(
                    "_{}",
                    i - if method.receiver.has_self() {
                        1
                    } else {
                        0
                    }
                )
            }
        })
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

    // on_call: instance method for trackable structs with a self receiver,
    // static for non-trackable or receiverless methods (constructors, static fns).
    let has_receiver = method.receiver.has_self();
    let instance_scoped = trackable && has_receiver;
    let on_call_self_param = if instance_scoped { "&self, " } else { "" };
    let on_call_mock_id_expr = if instance_scoped {
        format!(
            "context::MockId::new_adt_instance(\"{}\", \"{}\", self.adt_mock_id)",
            object_mock_id, method_name
        )
    } else {
        format!("context::MockId::new_adt_static(\"{}\", \"{}\")", object_mock_id, method_name)
    };
    let on_call_object_mock_id_expr = if instance_scoped {
        format!(
            "context::AdtId::new(\"{}\", self.adt_mock_id)",
            object_mock_id
        )
    } else {
        format!("context::AdtId::new(\"{}\", context::AdtIdNumber::default())", object_mock_id)
    };

    let impl_gen_source = impl_generics.to_source();
    let impl_where = impl_generics.where_clause_to_source();
    let is_generic = !impl_generics.is_empty() || !method.generics.is_empty();

    let method_gen_source = method.generics.to_source();
    let method_where = method.generics.where_clause_to_source();
    let method_where_str = if method_where.is_empty() { String::new() } else { format!(" {method_where}") };

    // For generic types, Predicate*/Return* wrapper structs can't carry the type params
    // at root level. Only generate them for non-generic types.
    let wrapper_structs = if is_generic {
        String::new()
    } else {
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
        let mock_id = context::MockId::new_adt_static("{object_mock_id}", "{method_name}");
        let cond = context::ConditionDoublePointer::from_fn::<({full_type_tuple})>(
            Box::new(move |input: &({full_type_tuple})| closure({input_access_ref}))
        );
        Self(context::Predicate::create_single::<({full_type_tuple})>(&mock_id, cond))
    }}
}}
"#,
            suffix = suffix,
            object_mock_id = object_mock_id,
            method_name = method_name,
            closure_type_params_str = closure_type_params_str,
            closure_type_params_ref_str = closure_type_params_ref_str,
            closure_args_str = closure_args_str,
            ret_type_str = ret_type_str,
            full_type_tuple = full_type_tuple,
            destructure_pattern = destructure_pattern,
            input_access_ref = input_access_ref,
        )
    };

    let impl_where_str = if impl_where.is_empty() { String::new() } else { format!(" {}", impl_where) };

    format!(
        r#"{wrapper_structs}
impl{impl_gen_source} {struct_type_path}{impl_where_str} {{
    pub fn on_call_{method_name}{method_gen_source}({on_call_self_param}condition: impl Fn({closure_type_params_ref_str}) -> bool + 'static, ret: impl Fn({closure_type_params_str}) -> {ret_type_str} + 'static){method_where_str} {{
        let mock_id = {on_call_mock_id_expr};
        let object_mock_id = {on_call_object_mock_id_expr};
        let cond = context::ConditionDoublePointer::from_fn::<({full_type_tuple})>(
            Box::new(move |input: &({full_type_tuple})| {{
                if condition({input_access_ref}) {{
                    Ok(())
                }} else {{
                    Err("on_call condition failed for {method_name}".into())
                }}
            }})
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<({full_type_tuple}), {ret_type_str}>(
            Box::new(move |{destructure_pattern}| ret({closure_args_str}))
        );
        context::add_method_on_call::<({full_type_tuple})>(
            &object_mock_id,
            "{method_name}",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }}
}}
"#,
        wrapper_structs = wrapper_structs,
        impl_gen_source = impl_gen_source,
        struct_type_path = struct_type_path,
        impl_where_str = impl_where_str,
        method_name = method_name,
        method_gen_source = method_gen_source,
        method_where_str = method_where_str,
        closure_type_params_str = closure_type_params_str,
        closure_type_params_ref_str = closure_type_params_ref_str,
        closure_args_str = closure_args_str,
        ret_type_str = ret_type_str,
        full_type_tuple = full_type_tuple,
        destructure_pattern = destructure_pattern,
        input_access_ref = input_access_ref,
        on_call_self_param = on_call_self_param,
        on_call_mock_id_expr = on_call_mock_id_expr,
        on_call_object_mock_id_expr = on_call_object_mock_id_expr,
    )
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Build the Public* struct name from a ModPath and struct name.
/// For root-level `MockStruct` (no segments): `PublicMockStruct`
/// For `a::nested::Inner` (segments=["a","nested"]): `PublicANestedInner`
fn public_struct_name(path: &ModPath, struct_name: &str) -> String {
    let mod_path = path.mod_path();
    let mut name = "Public".to_string();
    if !mod_path.is_empty() {
        for segment in mod_path.split("::") {
            name.push_str(&capitalize(segment));
        }
    }
    name.push_str(&capitalize(struct_name));
    name
}

/// Generate wrapper newtypes and on_call for a function inside a submodule.
/// The mock_id uses the full path (crate_mod_fn), and the on_call function is
/// placed inside a `mod` block so it's accessible as `fns::a::on_call_modules(...)`.
fn gen_fn_wrappers_with_prefix(path: &ModPath, fn_name: &str, func: &FunctionData, mod_name: &str) -> String {
    let fn_cap = format!("{}_{}", capitalize(mod_name), capitalize(fn_name));
    let mock_id = format!("{}_{}", path.mock_prefix(""), fn_name);

    let input_types_str = func
        .params
        .iter()
        .map(|p| p.ty.to_source())
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = func.return_type.to_source();

    let type_tuple = if func.params.len() == 1 {
        format!("{},", input_types_str)
    } else {
        input_types_str.clone()
    };

    let closure_type_params = func
        .params
        .iter()
        .map(|p| p.ty.to_source())
        .collect::<Vec<_>>()
        .join(", ");

    let closure_args = func
        .params
        .iter()
        .enumerate()
        .map(|(i, _)| format!("_{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    let destructure_pattern = if func.params.is_empty() {
        "()".to_string()
    } else {
        let d = func
            .params
            .iter()
            .enumerate()
            .map(|(i, _)| format!("_{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        format!("({},)", d)
    };

    let closure_type_params_ref = func
        .params
        .iter()
        .map(|p| format!("&{}", p.ty.to_source()))
        .collect::<Vec<_>>()
        .join(", ");

    let input_access_ref = func
        .params
        .iter()
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
        let mock_id = context::MockId::new_fn("{mock_id}");
        let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(
            Box::new(move |input: &({type_tuple})| closure({input_access_ref}))
        );
        Self(context::Predicate::create_single::<({type_tuple})>(&mock_id, cond))
    }}
}}

pub fn on_call_{mod_name}_{fn_name}(condition: impl Fn({closure_type_params_ref}) -> bool + 'static, ret: impl Fn({closure_type_params}) -> {ret_type_str} + 'static) {{
    let mock_id = context::MockId::new_fn("{mock_id}");
    let cond = context::ConditionDoublePointer::from_fn::<({type_tuple})>(
        Box::new(move |input: &({type_tuple})| {{
            if condition({input_access_ref}) {{
                Ok(())
            }} else {{
                Err("on_call condition failed for {mod_name}_{fn_name}".into())
            }}
        }})
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<({type_tuple}), {ret_type_str}>(
        Box::new(move |{destructure_pattern}| ret({closure_args}))
    );
    context::add_on_call::<({type_tuple})>(
        &mock_id,
        cond,
        ret_ptr,
        None,
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
    path: &ModPath,
    struct_name: &str,
    ctor_name: &str,
    constructor: &MethodData,
    all_method_ids: &[ItemId],
    items: &ItemStore,
    all_public: bool,
    struct_data: &StructData,
) -> String {

    let params_str = constructor
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name.as_str(), p.ty.to_source()))
        .collect::<Vec<_>>()
        .join(", ");

    let ret_type_str = constructor.return_type.to_source();

    // Build field initializers for the build-phase fallback (direct construction)
    let field_inits: Vec<String> = struct_data
        .fields
        .iter()
        .map(|&fid| {
            let fi = items.get(fid);
            let fd = match &fi.kind { ItemKind::Field(fd) => fd, _ => return String::new() };
            let field_name = &fi.name;
            if fi.is_pub() {
                let has_param = constructor.params.iter().any(|p| p.name == *field_name);
                if has_param {
                    format!("            {}", field_name)
                } else {
                    format!("            {}: Default::default()", field_name)
                }
            } else {
                format!("            {}: std::marker::PhantomData", field_name)
            }
        })
        .collect();

    // Add adt_mock_id for trackable structs
    let adt_mock_id_init = if !all_public {
        "            adt_mock_id: context::new_id(),\n"
    } else {
        ""
    };

    let field_inits_str = field_inits.join(",\n");

    // Generate register_method calls for every mockable method (methods with a self receiver)
    let object_mock_id_prefix = path.mock_prefix(struct_name);
    let mock_registrations: Vec<String> = all_method_ids.iter()
        .filter_map(|&mid| {
            let mi = items.get(mid);
            let md = match &mi.kind { ItemKind::Method(md) => md, _ => return None };
            if md.receiver.is_none() { return None; }
            let method_name = &mi.name;

            if all_public {
                Some(format!(
                    r#"        let _ = context::register_mock(&context::MockId::new_adt_static("{object_mock_id_prefix}", "{method_name}"));"#,
                    object_mock_id_prefix = object_mock_id_prefix,
                    method_name = method_name,
                ))
            } else {
                Some(format!(
                    r#"        let _ = context::register_mock(&context::MockId::new_adt_instance("{object_mock_id_prefix}", "{method_name}", slf.adt_mock_id));"#,
                    object_mock_id_prefix = object_mock_id_prefix,
                    method_name = method_name,
                ))
            }
        })
        .collect();

    let mock_registrations_str = mock_registrations.join("\n");

    let struct_generics = struct_data.generics.to_source();
    let public_struct_name = format!("crate::{}{}", public_struct_name(path, struct_name), struct_generics);

    // Constructor mock_id
    let ctor_mock_id = format!("{}_{}", path.mock_prefix(struct_name), ctor_name);

    // Input type tuple for the constructor (params only, no self)
    let ctor_input_types = constructor
        .params
        .iter()
        .map(|p| p.ty.to_source())
        .collect::<Vec<_>>()
        .join(", ");
    let ctor_input_types_tuple = if constructor.params.len() == 1 {
        format!("{},", ctor_input_types)
    } else {
        ctor_input_types.clone()
    };

    // Input values for run_mock
    let ctor_input_values = constructor
        .params
        .iter()
        .map(|p| p.name.as_str().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let ctor_input_values_tuple = if constructor.params.len() == 1 {
        format!("{},", ctor_input_values)
    } else {
        ctor_input_values.clone()
    };

    // Mock ID expression: instance-specific for trackable, static for all-public
    let ctor_adt_path = path.mock_prefix(struct_name);
    let ctor_mock_id_check = if all_public {
        format!("context::MockId::new_adt_static(\"{}\", \"{}\")", ctor_adt_path, ctor_name)
    } else {
        // For trackable structs, on_call_new is instance-specific — but at constructor dispatch
        // time we don't have an instance yet. Use the static mock_id for the constructor.
        format!("context::MockId::new_adt_static(\"{}\", \"{}\")", ctor_adt_path, ctor_name)
    };

    // Determine whether the fallback branches can construct the struct directly.
    // If the struct derives Default we can fill unmatched public fields with Default::default().
    // Otherwise we emit a panic — there is no safe way to conjure field values.
    // We distinguish two fallback cases:
    //   - Build phase (!ctx_built): constructor called before finish_building_context()
    //   - Post-build (ctx_built but no mock): constructor called after context finalized
    let default_fallback = format!(
        r#"        let slf = Self {{
{field_inits_str},
{adt_mock_id_init}            }};
{mock_registrations_str}
            slf"#,
        field_inits_str = field_inits_str,
        adt_mock_id_init = adt_mock_id_init,
        mock_registrations_str = mock_registrations_str,
    );

    let full_name = if path.segments.is_empty() {
        format!("{}::{}", path.crate_name, struct_name)
    } else {
        format!("{}::{}::{}", path.crate_name, path.mod_path(), struct_name)
    };

    let build_phase_branch = if struct_data.has_default() {
        default_fallback.clone()
    } else {
        format!(
            r#"        panic!("mock_crate: `{full_name}::{ctor_name}` called during build phase, \
but `{full_name}` does not derive Default — register an on_call for this constructor before calling it")"#,
            ctor_name = ctor_name,
            full_name = full_name,
        )
    };

    let no_mock_branch = if struct_data.has_default() {
        default_fallback
    } else {
        format!(
            r#"        panic!("mock_crate: no mock registered for constructor `{ctor_name}` of `{full_name}` — \
struct does not derive Default, so a mock_struct! invocation with on_call for this constructor is required")"#,
            ctor_name = ctor_name,
            full_name = full_name,
        )
    };

    format!(
        r#"pub fn {ctor_name}({params_str}) -> {ret_type_str} {{
        let {ctor_mock_id}_mock_id = {ctor_mock_id_check};
        if !context::ctx_built() {{
{build_phase_branch}
        }} else if context::contains_id(&{ctor_mock_id}_mock_id) {{
            let public: {public_struct_name} = context::run_mock::<({ctor_input_types_tuple}), {public_struct_name}>({ctor_mock_id}_mock_id, ({ctor_input_values_tuple})).unwrap();
            let slf: Self = public.into();
{mock_registrations_str}
            slf
        }} else {{
{no_mock_branch}
        }}
    }}"#,
        ctor_name = ctor_name,
        params_str = params_str,
        ret_type_str = ret_type_str,
        ctor_mock_id = ctor_mock_id,
        ctor_mock_id_check = ctor_mock_id_check,
        public_struct_name = public_struct_name,
        ctor_input_types_tuple = ctor_input_types_tuple,
        ctor_input_values_tuple = ctor_input_values_tuple,
        mock_registrations_str = mock_registrations_str,
        build_phase_branch = build_phase_branch,
        no_mock_branch = no_mock_branch,
    )
}

/// Check if a method is a constructor (no self receiver, returns Self or the struct type)
pub fn is_constructor(method_name: &str, method: &MethodData, struct_name: &str) -> bool {
    if method.receiver.has_self() {
        return false;
    }
    if method.return_type.is_unit() {
        return false;
    }
    let ty_str = method.return_type.to_source();
    ty_str == "Self" || ty_str == struct_name
}

/// Generate a `Public{...}` struct containing only the public fields of the original,
/// plus an `Into<{StructName}>` impl that fills in private fields (PhantomData) and adt_mock_id.
///
/// `public_name` is the name for the generated struct (e.g., "PublicMockStruct" or "PublicANestedInner").
/// `struct_type_path` is the qualified path for the target struct in generated code.
pub fn gen_public_struct(
    public_name: &str,
    struct_type_path: &str,
    struct_data: &StructData,
    items: &ItemStore,
) -> String {
    let generics = struct_data.generics.to_source();
    let where_clause = struct_data.generics.where_clause_to_source();
    let where_str = if where_clause.is_empty() { String::new() } else { format!(" {where_clause}") };

    let pub_fields: Vec<String> = struct_data.fields.iter().filter_map(|&fid| {
        let fi = items.get(fid);
        if !fi.is_pub() { return None; }
        let fd = match &fi.kind { ItemKind::Field(fd) => fd, _ => return None };
        Some(format!("    pub {}: {},", fi.name, fd.ty.to_source()))
    }).collect();
    let pub_fields_str = pub_fields.join("\n");

    let field_mappings: Vec<String> = struct_data.fields.iter().map(|&fid| {
        let fi = items.get(fid);
        if fi.is_pub() {
            format!("            {}: self.{},", fi.name, fi.name)
        } else {
            format!("            {}: std::marker::PhantomData,", fi.name)
        }
    }).collect();

    let adt_mock_id_field = if struct_data.trackable {
        "            adt_mock_id: context::new_id(),\n"
    } else {
        ""
    };

    let field_mappings_str = field_mappings.join("\n");

    format!(
        r#"pub struct {public_name}{generics}{where_str} {{
{pub_fields_str}
}}

impl{generics} Into<{struct_type_path}> for {public_name}{generics}{where_str} {{
    fn into(self) -> {struct_type_path} {{
        {struct_type_path} {{
{field_mappings_str}
{adt_mock_id_field}        }}
    }}
}}
"#,
        public_name = public_name,
        generics = generics,
        where_str = where_str,
        struct_type_path = struct_type_path,
        pub_fields_str = pub_fields_str,
        field_mappings_str = field_mappings_str,
        adt_mock_id_field = adt_mock_id_field,
    )
}

/// Generate the entire crate body from the analysis model.
///
/// Generates all mockable items (definitions + mock dispatch + convenience API)
/// in a single pass. Convenience wrappers for all items (including submodule items)
/// are placed at the crate root level.
pub fn gen_crate_from_model(api: &CrateApiModel) -> String {
    let root_path = ModPath::root(&api.crate_name);
    let mut root_wrappers = String::new();
    let module_source = gen_module_items(api.root, &api.items, &root_path, &api.crate_name, &mut root_wrappers);

    let mut source = module_source;
    source.push_str(&root_wrappers);
    source
}

/// Generate all items for a module (recursive).
fn gen_module_items(
    module_id: ItemId,
    items: &ItemStore,
    path: &ModPath,
    crate_name: &str,
    root_wrappers: &mut String,
) -> String {
    let module_item = items.get(module_id);
    let ItemKind::Module(ref module_data) = module_item.kind else {
        return String::new();
    };
    let children = module_data.children.clone();
    let module_name = module_item.name.clone();

    let mut source = String::new();

    for &child_id in &children {
        let child = items.get(child_id);
        match &child.kind {
            // 1. Struct definitions
            ItemKind::Struct(s) => {
                source.push_str(&gen_struct_def(&child.name, s, items));
                source.push_str("\n\n");
            }
            // 2. Enum definitions
            ItemKind::Enum(e) => {
                source.push_str(&gen_enum_def(&child.name, e, items));
                source.push_str("\n\n");
            }
            // 3. Trait definitions
            ItemKind::Trait(t) => {
                source.push_str(&gen_trait_def(&child.name, t, items));
                source.push_str("\n\n");
            }
            // 4. Free functions with mock dispatch
            ItemKind::Function(func) => {
                source.push_str(&gen_mock_fn_body(&path.mock_prefix(""), &child.name, func));
                source.push_str("\n\n");
                if path.segments.is_empty() {
                    root_wrappers.push_str(&gen_fn_wrappers(crate_name, &child.name, func));
                } else {
                    root_wrappers.push_str(&gen_fn_wrappers_with_prefix(path, &child.name, func, &module_name));
                }
                root_wrappers.push_str("\n\n");
            }
            // 5. Impl blocks with mock dispatch + convenience wrappers
            ItemKind::Impl(imp) => {
                let struct_name = &imp.self_type_name;
                // Find the struct model among siblings
                let struct_item = children.iter().find_map(|&sid| {
                    let si = items.get(sid);
                    if si.name == *struct_name { if let ItemKind::Struct(_) = &si.kind { return Some(sid); } }
                    None
                });
                let struct_data = struct_item.and_then(|sid| {
                    if let ItemKind::Struct(ref s) = items.get(sid).kind { Some(s) } else { None }
                });
                let trackable = struct_data.map(|s| s.trackable).unwrap_or(false);

                let self_type_source = imp.self_type.to_source();
                let struct_type_path = if path.segments.is_empty() {
                    self_type_source
                } else {
                    format!("{}::{}", path.mod_path(), self_type_source)
                };

                source.push_str(&gen_impl_block(imp, items, struct_data, path));
                source.push_str("\n\n");

                let has_constructors = imp.methods.iter().any(|&mid| {
                    let m = items.get(mid);
                    if let ItemKind::Method(ref md) = m.kind { is_constructor(&m.name, md, struct_name) } else { false }
                });
                if has_constructors {
                    if let Some(sd) = struct_data {
                        let pub_name = public_struct_name(path, struct_name);
                        root_wrappers.push_str(&gen_public_struct(&pub_name, &struct_type_path, sd, items));
                        root_wrappers.push_str("\n\n");
                    }
                }

                for &method_id in &imp.methods {
                    let method_item = items.get(method_id);
                    let ItemKind::Method(ref method_data) = method_item.kind else { continue };
                    let ret_override = if is_constructor(&method_item.name, method_data, struct_name) {
                        let gen_source = imp.generics.to_source();
                        Some(format!("{}{}", public_struct_name(path, struct_name), gen_source))
                    } else {
                        None
                    };
                    root_wrappers.push_str(&gen_method_wrappers(
                        path, struct_name, &method_item.name, method_data, trackable,
                        &struct_type_path, ret_override.as_deref(), &imp.generics,
                    ));
                    root_wrappers.push_str("\n\n");
                }
            }
            // 6. Child modules (recursive)
            ItemKind::Module(_) => {
                let child_path = path.child(&child.name);
                let child_items = gen_module_items(child_id, items, &child_path, crate_name, root_wrappers);
                source.push_str(&format!("pub mod {} {{\n{}}}\n\n", child.name, child_items));
            }
            _ => {}
        }
    }

    source
}

/// Generate a struct definition.
///
/// Handles three kinds:
/// - **Named structs**: `pub struct Foo { fields... }`
///   - Private fields → `PhantomData<OriginalType>`
///   - Trackable structs get `adt_mock_id: context::AdtMockId`
/// - **Tuple structs**: `pub struct Foo(pub Type1, Type2);`
///   - Fields have numeric names ("0", "1", ...)
/// - **Unit structs**: `pub struct Foo;`
fn gen_struct_def(name: &str, s: &StructData, items: &ItemStore) -> String {
    let generics = s.generics.to_source();
    let where_clause = s.generics.where_clause_to_source();

    if s.kind == StructKind::Unit || s.fields.is_empty() {
        return format!("pub struct {}{}{};", name, generics,
            if where_clause.is_empty() { String::new() } else { format!(" {}", where_clause) });
    }

    if s.kind == StructKind::Tuple {
        let mut field_types: Vec<String> = s.fields.iter().map(|&fid| {
            let fi = items.get(fid);
            let ty = match &fi.kind { ItemKind::Field(fd) => fd.ty.to_source(), _ => "()".to_string() };
            if fi.is_pub() {
                format!("pub {ty}")
            } else {
                format!("std::marker::PhantomData<{ty}>")
            }
        }).collect();
        if s.trackable {
            field_types.push("pub context::AdtMockId".to_string());
        }
        return if where_clause.is_empty() {
            format!("pub struct {}{}({});", name, generics, field_types.join(", "))
        } else {
            format!("pub struct {}{}({}) {};", name, generics, field_types.join(", "), where_clause)
        };
    }

    // Plain (named) struct
    let mut fields = Vec::new();
    for &fid in &s.fields {
        let fi = items.get(fid);
        let fd = match &fi.kind { ItemKind::Field(fd) => fd, _ => continue };
        if fi.is_pub() {
            fields.push(format!("    pub {}: {},", fi.name, fd.ty.to_source()));
        } else {
            fields.push(format!("    {}: std::marker::PhantomData<{}>,", fi.name, fd.ty.to_source()));
        }
    }
    if s.trackable {
        fields.push("    pub adt_mock_id: context::AdtMockId,".to_string());
    }
    let fields_str = fields.join("\n");
    if where_clause.is_empty() {
        format!("pub struct {}{} {{\n{}\n}}", name, generics, fields_str)
    } else {
        format!("pub struct {}{} {} {{\n{}\n}}", name, generics, where_clause, fields_str)
    }
}

/// Generate an enum definition (passed through from model).
fn gen_enum_def(name: &str, e: &EnumData, items: &ItemStore) -> String {
    let generics = e.generics.to_source();
    let where_clause = e.generics.where_clause_to_source();
    let variants: Vec<String> = e.variants.iter().map(|&vid| {
        let vi = items.get(vid);
        let vd = match &vi.kind { ItemKind::Variant(vd) => vd, _ => return format!("    {},", vi.name) };
        match vd.kind {
            StructKind::Unit => format!("    {},", vi.name),
            StructKind::Tuple => {
                let field_types: Vec<String> = vd.fields.iter().map(|&fid| {
                    match &items.get(fid).kind { ItemKind::Field(fd) => fd.ty.to_source(), _ => "()".to_string() }
                }).collect();
                format!("    {}({}),", vi.name, field_types.join(", "))
            }
            StructKind::Plain => {
                let field_defs: Vec<String> = vd.fields.iter().map(|&fid| {
                    let fi = items.get(fid);
                    let ty = match &fi.kind { ItemKind::Field(fd) => fd.ty.to_source(), _ => "()".to_string() };
                    format!("{}: {}", fi.name, ty)
                }).collect();
                format!("    {} {{ {} }},", vi.name, field_defs.join(", "))
            }
        }
    }).collect();
    if where_clause.is_empty() {
        format!("pub enum {}{} {{\n{}\n}}", name, generics, variants.join("\n"))
    } else {
        format!("pub enum {}{} {} {{\n{}\n}}", name, generics, where_clause, variants.join("\n"))
    }
}

/// Generate a trait definition with method signatures (no default bodies).
fn gen_trait_def(name: &str, t: &TraitData, items: &ItemStore) -> String {
    let generics = t.generics.to_source();
    let where_clause = t.generics.where_clause_to_source();
    let supertraits = if t.supertraits.is_empty() {
        String::new()
    } else {
        let supers: Vec<String> = t.supertraits.iter().map(|s| s.to_source()).collect();
        format!(": {}", supers.join(" + "))
    };
    let methods: Vec<String> = t.methods.iter().map(|&mid| {
        let mi = items.get(mid);
        let md = match &mi.kind { ItemKind::Method(md) => md, _ => return String::new() };
        let name = &mi.name;
        let qualifiers = md.qualifiers.to_source_prefix();
        let method_generics = md.generics.to_source();
        let receiver_str = md.receiver.to_source();
        let params_str = md.params.iter().map(|p| format!("{}: {}", p.name, p.ty.to_source())).collect::<Vec<_>>().join(", ");
        let full_params = if receiver_str.is_empty() { params_str }
            else if params_str.is_empty() { receiver_str.to_string() }
            else { format!("{receiver_str}, {params_str}") };
        let ret_str = if md.return_type.is_unit() { String::new() } else { format!(" -> {}", md.return_type.to_source()) };
        let method_where = md.generics.where_clause_to_source();
        let where_str = if method_where.is_empty() { String::new() } else { format!(" {method_where}") };
        format!("    {qualifiers}fn {name}{method_generics}({full_params}){ret_str}{where_str};\n")
    }).collect();
    let assoc_types: Vec<String> = t.associated_types.iter().map(|&aid| {
        let ai = items.get(aid);
        let ad = match &ai.kind { ItemKind::AssociatedType(ad) => ad, _ => return String::new() };
        let at_generics = ad.generics.to_source();
        let at_where = ad.generics.where_clause_to_source();
        let at_where_str = if at_where.is_empty() { String::new() } else { format!(" {at_where}") };
        let bounds = if ad.bounds.is_empty() { String::new() } else {
            let b: Vec<String> = ad.bounds.iter().map(|b| b.to_source()).collect();
            format!(": {}", b.join(" + "))
        };
        if let Some(ref default) = ad.ty {
            format!("    type {}{}{} = {}{at_where_str};\n", ai.name, at_generics, bounds, default.to_source())
        } else {
            format!("    type {}{}{}{at_where_str};\n", ai.name, at_generics, bounds)
        }
    }).collect();
    if where_clause.is_empty() {
        format!("pub trait {}{}{} {{\n{}{}}}", name, generics, supertraits, assoc_types.join(""), methods.join(""))
    } else {
        format!("pub trait {}{}{} {} {{\n{}{}}}", name, generics, supertraits, where_clause, assoc_types.join(""), methods.join(""))
    }
}

/// Generate an impl block with mock dispatch bodies for all methods.
fn gen_impl_block(imp: &ImplData, items: &ItemStore, struct_data: Option<&StructData>, path: &ModPath) -> String {
    let struct_name = &imp.self_type_name;
    let trackable = struct_data.map(|s| s.trackable).unwrap_or(false);
    let all_public = !trackable;
    let is_trait_impl = imp.trait_name.is_some();

    let impl_generics = imp.generics.to_source();
    let where_clause = imp.generics.where_clause_to_source();
    let self_type_with_generics = imp.self_type.to_source();

    let impl_header = if let Some(ref trait_ref) = imp.trait_ref {
        format!("impl{} {} for {}", impl_generics, trait_ref.to_source(), self_type_with_generics)
    } else {
        format!("impl{} {}", impl_generics, self_type_with_generics)
    };

    let mut methods_source = String::new();

    // Associated type bindings
    for &aid in &imp.associated_types {
        let ai = items.get(aid);
        if let ItemKind::AssociatedType(ref ad) = ai.kind {
            if let Some(ref ty) = ad.ty {
                let at_generics = ad.generics.to_source();
                let at_where = ad.generics.where_clause_to_source();
                let at_where_str = if at_where.is_empty() { String::new() } else { format!(" {at_where}") };
                methods_source.push_str(&format!("    type {}{} = {}{at_where_str};\n", ai.name, at_generics, ty.to_source()));
            }
        }
    }

    for &method_id in &imp.methods {
        let mi = items.get(method_id);
        let md = match &mi.kind { ItemKind::Method(md) => md, _ => continue };
        let method_source = if is_constructor(&mi.name, md, struct_name) {
            if let Some(sd) = struct_data {
                gen_constructor_body(path, struct_name, &mi.name, md, &imp.methods, items, all_public, sd)
            } else {
                gen_mock_method_body(&path.mock_prefix(""), struct_name, &mi.name, md, trackable, is_trait_impl, &self_type_with_generics)
            }
        } else {
            gen_mock_method_body(&path.mock_prefix(""), struct_name, &mi.name, md, trackable, is_trait_impl, &self_type_with_generics)
        };
        methods_source.push_str("    ");
        methods_source.push_str(&method_source);
        methods_source.push('\n');
    }

    if where_clause.is_empty() {
        format!("{} {{\n{}}}\n", impl_header, methods_source)
    } else {
        format!("{} {} {{\n{}}}\n", impl_header, where_clause, methods_source)
    }
}