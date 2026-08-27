pub fn ref_param(x: &u32) -> () {
    let fns_ref_param_mock_id =
        context::MockId::new(stringify!(fns_ref_param));
    if context::ctx_built_and_contains_id(&fns_ref_param_mock_id) {
        match context::run_mock::<(&u32,), ()>(fns_ref_param_mock_id, (x,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for ref_param"); }
}

pub fn cons_param(x: Box<u32>) -> () {
    let fns_cons_param_mock_id =
        context::MockId::new(stringify!(fns_cons_param));
    if context::ctx_built_and_contains_id(&fns_cons_param_mock_id) {
        match context::run_mock::<(Box<u32>,),
                    ()>(fns_cons_param_mock_id, (x,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for cons_param"); }
}

#[derive(Debug)]
pub struct ConsSelfStruct;

impl ConsSelfStruct {
    pub fn consume_self(self) -> () {
        let fns_ConsSelfStruct_consume_self_mock_id =
            context::MockId::new(stringify!(fns_ConsSelfStruct_consume_self));
        if context::ctx_built_and_contains_id(&fns_ConsSelfStruct_consume_self_mock_id)
            {
            match context::run_mock::<(ConsSelfStruct,),
                        ()>(fns_ConsSelfStruct_consume_self_mock_id, (self,)) {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId =>
                            panic!("failed to find mock id"),
                    },
            }
        } else {
            panic!("mock_crate: no mock context built for consume_self");
        }
    }
}

mod ffi {
    unsafe extern "Rust" {
        pub fn foreign();
    }
}

pub fn foreign() -> () {
    let fns_foreign_mock_id = context::MockId::new(stringify!(fns_foreign));
    if context::ctx_built_and_contains_id(&fns_foreign_mock_id) {
        match context::run_mock::<(), ()>(fns_foreign_mock_id, ()) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for foreign"); }
}

#[derive(Debug)]
pub struct MockStruct {
    pub pubfield: u32,
    privfield: std::marker::PhantomData<u32>,
    pub adt_mock_id: context::AdtMockId,
}

impl MockStruct {
    pub fn new() -> Self {
        let slf =
            Self {
                pubfield: Default::default(),
                privfield: std::marker::PhantomData,
                adt_mock_id: context::new_id(),
            };
        context::add_mock::<(&MockStruct,),
                    u32>(context::MockId::new(format!("{}{}",
                        "fns_MockStruct_get_value", slf.adt_mock_id.0)),
                None).unwrap();
        slf
    }
    pub fn foo() -> () {
        let fns_MockStruct_foo_mock_id =
            context::MockId::new(stringify!(fns_MockStruct_foo));
        if context::ctx_built_and_contains_id(&fns_MockStruct_foo_mock_id) {
            match context::run_mock::<(), ()>(fns_MockStruct_foo_mock_id, ())
                {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId =>
                            panic!("failed to find mock id"),
                    },
            }
        } else { panic!("mock_crate: no mock context built for foo"); }
    }
    pub fn get_value(&self) -> u32 {
        let fns_MockStruct_get_value_mock_id =
            context::MockId::new(stringify!(fns_MockStruct_get_value));
        if context::ctx_built_and_contains_id(&fns_MockStruct_get_value_mock_id)
            {
            match context::run_mock::<(&MockStruct,),
                        u32>(fns_MockStruct_get_value_mock_id, (self,)) {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId =>
                            panic!("failed to find mock id"),
                    },
            }
        } else { panic!("mock_crate: no mock context built for get_value"); }
    }
}

pub fn ret_call_w_args(x: i16) -> i16 {
    let fns_ret_call_w_args_mock_id =
        context::MockId::new(stringify!(fns_ret_call_w_args));
    if context::ctx_built_and_contains_id(&fns_ret_call_w_args_mock_id) {
        match context::run_mock::<(i16,),
                    i16>(fns_ret_call_w_args_mock_id, (x,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else {
        panic!("mock_crate: no mock context built for ret_call_w_args");
    }
}

#[derive(PartialEq, Debug)]
pub struct Foo {
    pub x: u32,
}

impl Foo {
    pub fn ret_ref(&self) -> &u32 {
        let fns_Foo_ret_ref_mock_id =
            context::MockId::new(stringify!(fns_Foo_ret_ref));
        if context::ctx_built_and_contains_id(&fns_Foo_ret_ref_mock_id) {
            match context::run_mock::<(&Foo,),
                        &u32>(fns_Foo_ret_ref_mock_id, (self,)) {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId =>
                            panic!("failed to find mock id"),
                    },
            }
        } else { panic!("mock_crate: no mock context built for ret_ref"); }
    }
    pub fn ret_mut_ref(&mut self) -> &mut u32 {
        let fns_Foo_ret_mut_ref_mock_id =
            context::MockId::new(stringify!(fns_Foo_ret_mut_ref));
        if context::ctx_built_and_contains_id(&fns_Foo_ret_mut_ref_mock_id) {
            match context::run_mock::<(&mut Foo,),
                        &mut u32>(fns_Foo_ret_mut_ref_mock_id, (self,)) {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId =>
                            panic!("failed to find mock id"),
                    },
            }
        } else {
            panic!("mock_crate: no mock context built for ret_mut_ref");
        }
    }
    pub fn ret_owned() -> Foo {
        let slf = Self { x: Default::default() };
        let _ =
            context::add_mock::<(&Foo,),
                    &u32>(context::MockId::new("fns_Foo_ret_ref"), None);
        let _ =
            context::add_mock::<(&mut Foo,),
                    &mut u32>(context::MockId::new("fns_Foo_ret_mut_ref"),
                None);
        let _ =
            context::add_mock::<(&Foo,),
                    u32>(context::MockId::new("fns_Foo_fallback"), None);
        slf
    }
    pub fn static_method() -> () {
        let fns_Foo_static_method_mock_id =
            context::MockId::new(stringify!(fns_Foo_static_method));
        if context::ctx_built_and_contains_id(&fns_Foo_static_method_mock_id)
            {
            match context::run_mock::<(),
                        ()>(fns_Foo_static_method_mock_id, ()) {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId =>
                            panic!("failed to find mock id"),
                    },
            }
        } else {
            panic!("mock_crate: no mock context built for static_method");
        }
    }
    pub fn fallback(&self) -> u32 {
        let fns_Foo_fallback_mock_id =
            context::MockId::new(stringify!(fns_Foo_fallback));
        if context::ctx_built_and_contains_id(&fns_Foo_fallback_mock_id) {
            match context::run_mock::<(&Foo,),
                        u32>(fns_Foo_fallback_mock_id, (self,)) {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId =>
                            panic!("failed to find mock id"),
                    },
            }
        } else { panic!("mock_crate: no mock context built for fallback"); }
    }
}

pub fn ret_param(x: &mut u32) -> () {
    let fns_ret_param_mock_id =
        context::MockId::new(stringify!(fns_ret_param));
    if context::ctx_built_and_contains_id(&fns_ret_param_mock_id) {
        match context::run_mock::<(&mut u32,),
                    ()>(fns_ret_param_mock_id, (x,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for ret_param"); }
}

pub mod a {
    pub fn modules() -> u32 {
        let fns_a_modules_mock_id =
            context::MockId::new(stringify!(fns_a_modules));
        if context::ctx_built_and_contains_id(&fns_a_modules_mock_id) {
            match context::run_mock::<(), u32>(fns_a_modules_mock_id, ()) {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId => {
                            panic!("failed to find mock id");
                        }
                    },
            }
        } else { panic!("mock_crate: no mock context built for modules"); }
    }
}

pub fn return_const() -> i16 {
    let fns_return_const_mock_id =
        context::MockId::new(stringify!(fns_return_const));
    if context::ctx_built_and_contains_id(&fns_return_const_mock_id) {
        match context::run_mock::<(), i16>(fns_return_const_mock_id, ()) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for return_const"); }
}

/// by default i don't panic. i don't do anything c:
pub fn return_panic() -> () {
    let fns_return_panic_mock_id =
        context::MockId::new(stringify!(fns_return_panic));
    if context::ctx_built_and_contains_id(&fns_return_panic_mock_id) {
        match context::run_mock::<(), ()>(fns_return_panic_mock_id, ()) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for return_panic"); }
}

pub fn foo(a: i8, b: i8, c: i8, d: i8, e: i8, f: i8, g: i8, h: i8, i: i8,
    j: i8, k: i8, l: i8, m: i8, n: i8, o: i8, p: i8) -> () {
    let fns_foo_mock_id = context::MockId::new(stringify!(fns_foo));
    if context::ctx_built_and_contains_id(&fns_foo_mock_id) {
        match context::run_mock::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8,
                    i8, i8, i8, i8, i8),
                    ()>(fns_foo_mock_id,
                (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for foo"); }
}

pub fn times_once() -> () {
    let fns_times_once_mock_id =
        context::MockId::new(stringify!(fns_times_once));
    if context::ctx_built_and_contains_id(&fns_times_once_mock_id) {
        match context::run_mock::<(), ()>(fns_times_once_mock_id, ()) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for times_once"); }
}

pub fn times_any() -> () {
    let fns_times_any_mock_id =
        context::MockId::new(stringify!(fns_times_any));
    if context::ctx_built_and_contains_id(&fns_times_any_mock_id) {
        match context::run_mock::<(), ()>(fns_times_any_mock_id, ()) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for times_any"); }
}

pub fn match_const(key: u32) -> () {
    let fns_match_const_mock_id =
        context::MockId::new(stringify!(fns_match_const));
    if context::ctx_built_and_contains_id(&fns_match_const_mock_id) {
        match context::run_mock::<(u32,), ()>(fns_match_const_mock_id, (key,))
            {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for match_const"); }
}

pub fn match_operator(key: u32) -> () {
    let fns_match_operator_mock_id =
        context::MockId::new(stringify!(fns_match_operator));
    if context::ctx_built_and_contains_id(&fns_match_operator_mock_id) {
        match context::run_mock::<(u32,),
                    ()>(fns_match_operator_mock_id, (key,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for match_operator"); }
}

#[derive(Debug)]
pub enum Pattern { Okay, NotOkay, }

pub fn match_patter(pattern: Pattern) -> () {
    let fns_match_patter_mock_id =
        context::MockId::new(stringify!(fns_match_patter));
    if context::ctx_built_and_contains_id(&fns_match_patter_mock_id) {
        match context::run_mock::<(Pattern,),
                    ()>(fns_match_patter_mock_id, (pattern,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for match_patter"); }
}

pub fn match_range(key: u32) -> () {
    let fns_match_range_mock_id =
        context::MockId::new(stringify!(fns_match_range));
    if context::ctx_built_and_contains_id(&fns_match_range_mock_id) {
        match context::run_mock::<(u32,), ()>(fns_match_range_mock_id, (key,))
            {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for match_range"); }
}

pub fn match_wildcard(key: u32) -> () {
    let fns_match_wildcard_mock_id =
        context::MockId::new(stringify!(fns_match_wildcard));
    if context::ctx_built_and_contains_id(&fns_match_wildcard_mock_id) {
        match context::run_mock::<(u32,),
                    ()>(fns_match_wildcard_mock_id, (key,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for match_wildcard"); }
}

pub fn match_function(key: u32) -> () {
    let fns_match_function_mock_id =
        context::MockId::new(stringify!(fns_match_function));
    if context::ctx_built_and_contains_id(&fns_match_function_mock_id) {
        match context::run_mock::<(u32,),
                    ()>(fns_match_function_mock_id, (key,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for match_function"); }
}

pub struct ClosureWrapper(pub Box<dyn Fn(u32) -> u32>);

impl std::fmt::Debug for ClosureWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fns_ClosureWrapper_fmt_mock_id =
            context::MockId::new(stringify!(fns_ClosureWrapper_fmt));
        if context::ctx_built_and_contains_id(&fns_ClosureWrapper_fmt_mock_id)
            {
            match context::run_mock::<(&ClosureWrapper,
                        &mut std::fmt::Formatter<'_>),
                        std::fmt::Result>(fns_ClosureWrapper_fmt_mock_id, (self, f))
                {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId =>
                            panic!("failed to find mock id"),
                    },
            }
        } else { panic!("mock_crate: no mock context built for fmt"); }
    }
}

pub fn closure_param(f: ClosureWrapper) -> u32 {
    let fns_closure_param_mock_id =
        context::MockId::new(stringify!(fns_closure_param));
    if context::ctx_built_and_contains_id(&fns_closure_param_mock_id) {
        match context::run_mock::<(ClosureWrapper,),
                    u32>(fns_closure_param_mock_id, (f,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else { panic!("mock_crate: no mock context built for closure_param"); }
}

pub fn match_combination(key: i32) -> () {
    let fns_match_combination_mock_id =
        context::MockId::new(stringify!(fns_match_combination));
    if context::ctx_built_and_contains_id(&fns_match_combination_mock_id) {
        match context::run_mock::<(i32,),
                    ()>(fns_match_combination_mock_id, (key,)) {
            Ok(res) => res,
            Err(e) =>
                match e {
                    context::MockError::Other(e) =>
                        panic!("unexpected Error: {:?}", e),
                    context::MockError::PredicateError(e) =>
                        panic!("{:?}", e.0),
                    context::MockError::NoMatchingId => {
                        panic!("failed to find mock id");
                    }
                },
        }
    } else {
        panic!("mock_crate: no mock context built for match_combination");
    }
}

trait InternalBehavior {}

impl InternalBehavior for Foo {}

pub trait Computable {
    fn compute(&self)
    -> u32;
}

impl Computable for Foo {
    fn compute(&self) -> u32 {
        let fns_Foo_compute_mock_id =
            context::MockId::new(stringify!(fns_Foo_compute));
        if context::ctx_built_and_contains_id(&fns_Foo_compute_mock_id) {
            match context::run_mock::<(&Foo,),
                        u32>(fns_Foo_compute_mock_id, (self,)) {
                Ok(res) => res,
                Err(e) =>
                    match e {
                        context::MockError::Other(e) =>
                            panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) =>
                            panic!("{:?}", e.0),
                        context::MockError::NoMatchingId =>
                            panic!("failed to find mock id"),
                    },
            }
        } else { panic!("mock_crate: no mock context built for compute"); }
    }
}

// ─── Generated convenience API ───

pub struct PredicateRef_param(pub context::Predicate);
pub struct ReturnRef_param(pub context::ReturnValDoublePointer);

impl ReturnRef_param {
    pub fn from_fn(closure: impl Fn(&u32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&u32,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateRef_param {
    pub fn from_fn(closure: impl Fn(&&u32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_ref_param");
        let cond = context::ConditionDoublePointer::from_fn::<(&u32,)>(
            Box::new(move |input: &(&u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&u32,)>(&mock_id, cond))
    }
}

pub fn on_call_ref_param(ret: impl Into<ReturnRef_param>) {
    let inner: ReturnRef_param = ret.into();
    let mock_id = context::MockId::new("fns_ref_param");
    match context::add_mock::<(&u32,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(&u32,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(&u32,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_ref_param(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_ref_param");
    match context::add_mock::<(&u32,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_ref_param_at(seq_name: &str, index: usize, ret: impl Fn(&u32) -> () + 'static) {
    let mock_id = context::MockId::new("fns_ref_param");
    let cond = context::ConditionDoublePointer::from_fn::<(&u32,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(&u32,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateCons_param(pub context::Predicate);
pub struct ReturnCons_param(pub context::ReturnValDoublePointer);

impl ReturnCons_param {
    pub fn from_fn(closure: impl Fn(Box<u32>) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(Box<u32>,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateCons_param {
    pub fn from_fn(closure: impl Fn(&Box<u32>) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_cons_param");
        let cond = context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(
            Box::new(move |input: &(Box<u32>,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(Box<u32>,)>(&mock_id, cond))
    }
}

pub fn on_call_cons_param(ret: impl Into<ReturnCons_param>) {
    let inner: ReturnCons_param = ret.into();
    let mock_id = context::MockId::new("fns_cons_param");
    match context::add_mock::<(Box<u32>,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(Box<u32>,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_cons_param(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_cons_param");
    match context::add_mock::<(Box<u32>,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_cons_param_at(seq_name: &str, index: usize, ret: impl Fn(Box<u32>) -> () + 'static) {
    let mock_id = context::MockId::new("fns_cons_param");
    let cond = context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(Box<u32>,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateForeign(pub context::Predicate);
pub struct ReturnForeign(pub context::ReturnValDoublePointer);

impl ReturnForeign {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), ()>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateForeign {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_foreign");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_foreign(ret: impl Into<ReturnForeign>) {
    let inner: ReturnForeign = ret.into();
    let mock_id = context::MockId::new("fns_foreign");
    match context::add_mock::<(), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation::<(), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_foreign(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_foreign");
    match context::add_mock::<(), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_foreign_at(seq_name: &str, index: usize, ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new("fns_foreign");
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(), ()>(
        &mock_id, cond, Some(Box::new(move |()| ret())),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateRet_call_w_args(pub context::Predicate);
pub struct ReturnRet_call_w_args(pub context::ReturnValDoublePointer);

impl ReturnRet_call_w_args {
    pub fn from_fn(closure: impl Fn(i16) -> i16 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(i16,), i16>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateRet_call_w_args {
    pub fn from_fn(closure: impl Fn(&i16) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_ret_call_w_args");
        let cond = context::ConditionDoublePointer::from_fn::<(i16,)>(
            Box::new(move |input: &(i16,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(i16,)>(&mock_id, cond))
    }
}

pub fn on_call_ret_call_w_args(ret: impl Into<ReturnRet_call_w_args>) {
    let inner: ReturnRet_call_w_args = ret.into();
    let mock_id = context::MockId::new("fns_ret_call_w_args");
    match context::add_mock::<(i16,), i16>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(i16,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(i16,), i16>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_ret_call_w_args(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_ret_call_w_args");
    match context::add_mock::<(i16,), i16>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_ret_call_w_args_at(seq_name: &str, index: usize, ret: impl Fn(i16) -> i16 + 'static) {
    let mock_id = context::MockId::new("fns_ret_call_w_args");
    let cond = context::ConditionDoublePointer::from_fn::<(i16,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(i16,), i16>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateRet_param(pub context::Predicate);
pub struct ReturnRet_param(pub context::ReturnValDoublePointer);

impl ReturnRet_param {
    pub fn from_fn(closure: impl Fn(&mut u32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&mut u32,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateRet_param {
    pub fn from_fn(closure: impl Fn(&&mut u32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_ret_param");
        let cond = context::ConditionDoublePointer::from_fn::<(&mut u32,)>(
            Box::new(move |input: &(&mut u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&mut u32,)>(&mock_id, cond))
    }
}

pub fn on_call_ret_param(ret: impl Into<ReturnRet_param>) {
    let inner: ReturnRet_param = ret.into();
    let mock_id = context::MockId::new("fns_ret_param");
    match context::add_mock::<(&mut u32,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(&mut u32,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(&mut u32,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_ret_param(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_ret_param");
    match context::add_mock::<(&mut u32,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_ret_param_at(seq_name: &str, index: usize, ret: impl Fn(&mut u32) -> () + 'static) {
    let mock_id = context::MockId::new("fns_ret_param");
    let cond = context::ConditionDoublePointer::from_fn::<(&mut u32,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(&mut u32,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateReturn_const(pub context::Predicate);
pub struct ReturnReturn_const(pub context::ReturnValDoublePointer);

impl ReturnReturn_const {
    pub fn from_fn(closure: impl Fn() -> i16 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), i16>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateReturn_const {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_return_const");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_return_const(ret: impl Into<ReturnReturn_const>) {
    let inner: ReturnReturn_const = ret.into();
    let mock_id = context::MockId::new("fns_return_const");
    match context::add_mock::<(), i16>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation::<(), i16>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_return_const(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_return_const");
    match context::add_mock::<(), i16>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_return_const_at(seq_name: &str, index: usize, ret: impl Fn() -> i16 + 'static) {
    let mock_id = context::MockId::new("fns_return_const");
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(), i16>(
        &mock_id, cond, Some(Box::new(move |()| ret())),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateReturn_panic(pub context::Predicate);
pub struct ReturnReturn_panic(pub context::ReturnValDoublePointer);

impl ReturnReturn_panic {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), ()>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateReturn_panic {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_return_panic");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_return_panic(ret: impl Into<ReturnReturn_panic>) {
    let inner: ReturnReturn_panic = ret.into();
    let mock_id = context::MockId::new("fns_return_panic");
    match context::add_mock::<(), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation::<(), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_return_panic(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_return_panic");
    match context::add_mock::<(), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_return_panic_at(seq_name: &str, index: usize, ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new("fns_return_panic");
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(), ()>(
        &mock_id, cond, Some(Box::new(move |()| ret())),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateFoo(pub context::Predicate);
pub struct ReturnFoo(pub context::ReturnValDoublePointer);

impl ReturnFoo {
    pub fn from_fn(closure: impl Fn(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8), ()>(
            Box::new(move |(_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15,)| closure(_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15))
        ))
    }
}

impl PredicateFoo {
    pub fn from_fn(closure: impl Fn(&i8, &i8, &i8, &i8, &i8, &i8, &i8, &i8, &i8, &i8, &i8, &i8, &i8, &i8, &i8, &i8) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_foo");
        let cond = context::ConditionDoublePointer::from_fn::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8)>(
            Box::new(move |input: &(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8)| closure(&input.0, &input.1, &input.2, &input.3, &input.4, &input.5, &input.6, &input.7, &input.8, &input.9, &input.10, &input.11, &input.12, &input.13, &input.14, &input.15))
        );
        Self(context::Predicate::create_single::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8)>(&mock_id, cond))
    }
}

pub fn on_call_foo(ret: impl Into<ReturnFoo>) {
    let inner: ReturnFoo = ret.into();
    let mock_id = context::MockId::new("fns_foo");
    match context::add_mock::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8)>(Box::new(|_| Ok(())));
    context::add_expectation::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_foo(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_foo");
    match context::add_mock::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_foo_at(seq_name: &str, index: usize, ret: impl Fn(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8) -> () + 'static) {
    let mock_id = context::MockId::new("fns_foo");
    let cond = context::ConditionDoublePointer::from_fn::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8), ()>(
        &mock_id, cond, Some(Box::new(move |(_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15,)| ret(_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateTimes_once(pub context::Predicate);
pub struct ReturnTimes_once(pub context::ReturnValDoublePointer);

impl ReturnTimes_once {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), ()>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateTimes_once {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_times_once");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_times_once(ret: impl Into<ReturnTimes_once>) {
    let inner: ReturnTimes_once = ret.into();
    let mock_id = context::MockId::new("fns_times_once");
    match context::add_mock::<(), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation::<(), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_times_once(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_times_once");
    match context::add_mock::<(), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_times_once_at(seq_name: &str, index: usize, ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new("fns_times_once");
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(), ()>(
        &mock_id, cond, Some(Box::new(move |()| ret())),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateTimes_any(pub context::Predicate);
pub struct ReturnTimes_any(pub context::ReturnValDoublePointer);

impl ReturnTimes_any {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), ()>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateTimes_any {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_times_any");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_times_any(ret: impl Into<ReturnTimes_any>) {
    let inner: ReturnTimes_any = ret.into();
    let mock_id = context::MockId::new("fns_times_any");
    match context::add_mock::<(), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation::<(), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_times_any(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_times_any");
    match context::add_mock::<(), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_times_any_at(seq_name: &str, index: usize, ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new("fns_times_any");
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(), ()>(
        &mock_id, cond, Some(Box::new(move |()| ret())),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMatch_const(pub context::Predicate);
pub struct ReturnMatch_const(pub context::ReturnValDoublePointer);

impl ReturnMatch_const {
    pub fn from_fn(closure: impl Fn(u32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(u32,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateMatch_const {
    pub fn from_fn(closure: impl Fn(&u32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_match_const");
        let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(
            Box::new(move |input: &(u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(u32,)>(&mock_id, cond))
    }
}

pub fn on_call_match_const(ret: impl Into<ReturnMatch_const>) {
    let inner: ReturnMatch_const = ret.into();
    let mock_id = context::MockId::new("fns_match_const");
    match context::add_mock::<(u32,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(u32,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_match_const(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_match_const");
    match context::add_mock::<(u32,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_match_const_at(seq_name: &str, index: usize, ret: impl Fn(u32) -> () + 'static) {
    let mock_id = context::MockId::new("fns_match_const");
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(u32,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMatch_operator(pub context::Predicate);
pub struct ReturnMatch_operator(pub context::ReturnValDoublePointer);

impl ReturnMatch_operator {
    pub fn from_fn(closure: impl Fn(u32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(u32,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateMatch_operator {
    pub fn from_fn(closure: impl Fn(&u32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_match_operator");
        let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(
            Box::new(move |input: &(u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(u32,)>(&mock_id, cond))
    }
}

pub fn on_call_match_operator(ret: impl Into<ReturnMatch_operator>) {
    let inner: ReturnMatch_operator = ret.into();
    let mock_id = context::MockId::new("fns_match_operator");
    match context::add_mock::<(u32,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(u32,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_match_operator(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_match_operator");
    match context::add_mock::<(u32,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_match_operator_at(seq_name: &str, index: usize, ret: impl Fn(u32) -> () + 'static) {
    let mock_id = context::MockId::new("fns_match_operator");
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(u32,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMatch_patter(pub context::Predicate);
pub struct ReturnMatch_patter(pub context::ReturnValDoublePointer);

impl ReturnMatch_patter {
    pub fn from_fn(closure: impl Fn(Pattern) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(Pattern,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateMatch_patter {
    pub fn from_fn(closure: impl Fn(&Pattern) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_match_patter");
        let cond = context::ConditionDoublePointer::from_fn::<(Pattern,)>(
            Box::new(move |input: &(Pattern,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(Pattern,)>(&mock_id, cond))
    }
}

pub fn on_call_match_patter(ret: impl Into<ReturnMatch_patter>) {
    let inner: ReturnMatch_patter = ret.into();
    let mock_id = context::MockId::new("fns_match_patter");
    match context::add_mock::<(Pattern,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(Pattern,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(Pattern,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_match_patter(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_match_patter");
    match context::add_mock::<(Pattern,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_match_patter_at(seq_name: &str, index: usize, ret: impl Fn(Pattern) -> () + 'static) {
    let mock_id = context::MockId::new("fns_match_patter");
    let cond = context::ConditionDoublePointer::from_fn::<(Pattern,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(Pattern,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMatch_range(pub context::Predicate);
pub struct ReturnMatch_range(pub context::ReturnValDoublePointer);

impl ReturnMatch_range {
    pub fn from_fn(closure: impl Fn(u32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(u32,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateMatch_range {
    pub fn from_fn(closure: impl Fn(&u32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_match_range");
        let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(
            Box::new(move |input: &(u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(u32,)>(&mock_id, cond))
    }
}

pub fn on_call_match_range(ret: impl Into<ReturnMatch_range>) {
    let inner: ReturnMatch_range = ret.into();
    let mock_id = context::MockId::new("fns_match_range");
    match context::add_mock::<(u32,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(u32,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_match_range(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_match_range");
    match context::add_mock::<(u32,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_match_range_at(seq_name: &str, index: usize, ret: impl Fn(u32) -> () + 'static) {
    let mock_id = context::MockId::new("fns_match_range");
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(u32,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMatch_wildcard(pub context::Predicate);
pub struct ReturnMatch_wildcard(pub context::ReturnValDoublePointer);

impl ReturnMatch_wildcard {
    pub fn from_fn(closure: impl Fn(u32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(u32,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateMatch_wildcard {
    pub fn from_fn(closure: impl Fn(&u32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_match_wildcard");
        let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(
            Box::new(move |input: &(u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(u32,)>(&mock_id, cond))
    }
}

pub fn on_call_match_wildcard(ret: impl Into<ReturnMatch_wildcard>) {
    let inner: ReturnMatch_wildcard = ret.into();
    let mock_id = context::MockId::new("fns_match_wildcard");
    match context::add_mock::<(u32,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(u32,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_match_wildcard(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_match_wildcard");
    match context::add_mock::<(u32,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_match_wildcard_at(seq_name: &str, index: usize, ret: impl Fn(u32) -> () + 'static) {
    let mock_id = context::MockId::new("fns_match_wildcard");
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(u32,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMatch_function(pub context::Predicate);
pub struct ReturnMatch_function(pub context::ReturnValDoublePointer);

impl ReturnMatch_function {
    pub fn from_fn(closure: impl Fn(u32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(u32,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateMatch_function {
    pub fn from_fn(closure: impl Fn(&u32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_match_function");
        let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(
            Box::new(move |input: &(u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(u32,)>(&mock_id, cond))
    }
}

pub fn on_call_match_function(ret: impl Into<ReturnMatch_function>) {
    let inner: ReturnMatch_function = ret.into();
    let mock_id = context::MockId::new("fns_match_function");
    match context::add_mock::<(u32,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(u32,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_match_function(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_match_function");
    match context::add_mock::<(u32,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_match_function_at(seq_name: &str, index: usize, ret: impl Fn(u32) -> () + 'static) {
    let mock_id = context::MockId::new("fns_match_function");
    let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(u32,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateClosure_param(pub context::Predicate);
pub struct ReturnClosure_param(pub context::ReturnValDoublePointer);

impl ReturnClosure_param {
    pub fn from_fn(closure: impl Fn(ClosureWrapper) -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(ClosureWrapper,), u32>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateClosure_param {
    pub fn from_fn(closure: impl Fn(&ClosureWrapper) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_closure_param");
        let cond = context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(
            Box::new(move |input: &(ClosureWrapper,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(ClosureWrapper,)>(&mock_id, cond))
    }
}

pub fn on_call_closure_param(ret: impl Into<ReturnClosure_param>) {
    let inner: ReturnClosure_param = ret.into();
    let mock_id = context::MockId::new("fns_closure_param");
    match context::add_mock::<(ClosureWrapper,), u32>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(ClosureWrapper,), u32>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_closure_param(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_closure_param");
    match context::add_mock::<(ClosureWrapper,), u32>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_closure_param_at(seq_name: &str, index: usize, ret: impl Fn(ClosureWrapper) -> u32 + 'static) {
    let mock_id = context::MockId::new("fns_closure_param");
    let cond = context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(ClosureWrapper,), u32>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMatch_combination(pub context::Predicate);
pub struct ReturnMatch_combination(pub context::ReturnValDoublePointer);

impl ReturnMatch_combination {
    pub fn from_fn(closure: impl Fn(i32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(i32,), ()>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateMatch_combination {
    pub fn from_fn(closure: impl Fn(&i32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_match_combination");
        let cond = context::ConditionDoublePointer::from_fn::<(i32,)>(
            Box::new(move |input: &(i32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(i32,)>(&mock_id, cond))
    }
}

pub fn on_call_match_combination(ret: impl Into<ReturnMatch_combination>) {
    let inner: ReturnMatch_combination = ret.into();
    let mock_id = context::MockId::new("fns_match_combination");
    match context::add_mock::<(i32,), ()>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<(i32,)>(Box::new(|_| Ok(())));
    context::add_expectation::<(i32,), ()>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

pub fn sequence_match_combination(name: &str, size: usize, modifier: context::TimesModifier) {
    let mock_id = context::MockId::new("fns_match_combination");
    match context::add_mock::<(i32,), ()>(mock_id, None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    context::new_sequence(name, size, modifier, None).unwrap();
}

pub fn expect_match_combination_at(seq_name: &str, index: usize, ret: impl Fn(i32) -> () + 'static) {
    let mock_id = context::MockId::new("fns_match_combination");
    let cond = context::ConditionDoublePointer::from_fn::<(i32,)>(Box::new(|_| Ok(())));
    context::add_expectation_to_sequence::<(i32,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateConsSelfStructConsume_self(pub context::Predicate);
pub struct ReturnConsSelfStructConsume_self(pub context::ReturnValDoublePointer);

impl ReturnConsSelfStructConsume_self {
    pub fn from_fn(closure: impl Fn(ConsSelfStruct) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(ConsSelfStruct,), ()>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateConsSelfStructConsume_self {
    pub fn from_fn(closure: impl Fn(&ConsSelfStruct) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_ConsSelfStruct_consume_self");
        let cond = context::ConditionDoublePointer::from_fn::<(ConsSelfStruct,)>(
            Box::new(move |input: &(ConsSelfStruct,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(ConsSelfStruct,)>(&mock_id, cond))
    }
}

impl ConsSelfStruct {
    pub fn on_call_consume_self(ret: impl Into<ReturnConsSelfStructConsume_self>) {
        let inner: ReturnConsSelfStructConsume_self = ret.into();
        let mock_id = context::MockId::new("fns_ConsSelfStruct_consume_self");
        match context::add_mock::<(ConsSelfStruct,), ()>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<(ConsSelfStruct,)>(Box::new(|_| Ok(())));
        context::add_expectation::<(ConsSelfStruct,), ()>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateMockStructNew(pub context::Predicate);
pub struct ReturnMockStructNew(pub context::ReturnValDoublePointer);

impl ReturnMockStructNew {
    pub fn from_fn(closure: impl Fn() -> Self + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), Self>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateMockStructNew {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_MockStruct_new");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_new(ret: impl Into<ReturnMockStructNew>) {
        let inner: ReturnMockStructNew = ret.into();
        let mock_id = context::MockId::new("fns_MockStruct_new");
        match context::add_mock::<(), Self>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
        context::add_expectation::<(), Self>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateMockStructFoo(pub context::Predicate);
pub struct ReturnMockStructFoo(pub context::ReturnValDoublePointer);

impl ReturnMockStructFoo {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), ()>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateMockStructFoo {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_MockStruct_foo");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_foo(ret: impl Into<ReturnMockStructFoo>) {
        let inner: ReturnMockStructFoo = ret.into();
        let mock_id = context::MockId::new("fns_MockStruct_foo");
        match context::add_mock::<(), ()>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
        context::add_expectation::<(), ()>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateMockStructGet_value(pub context::Predicate);
pub struct ReturnMockStructGet_value(pub context::ReturnValDoublePointer);

impl ReturnMockStructGet_value {
    pub fn from_fn(closure: impl Fn(&MockStruct) -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&MockStruct,), u32>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateMockStructGet_value {
    pub fn from_fn(closure: impl Fn(&&MockStruct) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_MockStruct_get_value");
        let cond = context::ConditionDoublePointer::from_fn::<(&MockStruct,)>(
            Box::new(move |input: &(&MockStruct,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&MockStruct,)>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_get_value(ret: impl Into<ReturnMockStructGet_value>) {
        let inner: ReturnMockStructGet_value = ret.into();
        let mock_id = context::MockId::new("fns_MockStruct_get_value");
        match context::add_mock::<(&MockStruct,), u32>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<(&MockStruct,)>(Box::new(|_| Ok(())));
        context::add_expectation::<(&MockStruct,), u32>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateFooRet_ref(pub context::Predicate);
pub struct ReturnFooRet_ref(pub context::ReturnValDoublePointer);

impl ReturnFooRet_ref {
    pub fn from_fn(closure: impl Fn(&Foo) -> &u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo,), &u32>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateFooRet_ref {
    pub fn from_fn(closure: impl Fn(&&Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_Foo_ret_ref");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_ret_ref(ret: impl Into<ReturnFooRet_ref>) {
        let inner: ReturnFooRet_ref = ret.into();
        let mock_id = context::MockId::new("fns_Foo_ret_ref");
        match context::add_mock::<(&Foo,), &u32>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(|_| Ok(())));
        context::add_expectation::<(&Foo,), &u32>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateFooRet_mut_ref(pub context::Predicate);
pub struct ReturnFooRet_mut_ref(pub context::ReturnValDoublePointer);

impl ReturnFooRet_mut_ref {
    pub fn from_fn(closure: impl Fn(&mut Foo) -> &mut u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&mut Foo,), &mut u32>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateFooRet_mut_ref {
    pub fn from_fn(closure: impl Fn(&&mut Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_Foo_ret_mut_ref");
        let cond = context::ConditionDoublePointer::from_fn::<(&mut Foo,)>(
            Box::new(move |input: &(&mut Foo,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&mut Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_ret_mut_ref(ret: impl Into<ReturnFooRet_mut_ref>) {
        let inner: ReturnFooRet_mut_ref = ret.into();
        let mock_id = context::MockId::new("fns_Foo_ret_mut_ref");
        match context::add_mock::<(&mut Foo,), &mut u32>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<(&mut Foo,)>(Box::new(|_| Ok(())));
        context::add_expectation::<(&mut Foo,), &mut u32>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateFooRet_owned(pub context::Predicate);
pub struct ReturnFooRet_owned(pub context::ReturnValDoublePointer);

impl ReturnFooRet_owned {
    pub fn from_fn(closure: impl Fn() -> Foo + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), Foo>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateFooRet_owned {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_Foo_ret_owned");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_ret_owned(ret: impl Into<ReturnFooRet_owned>) {
        let inner: ReturnFooRet_owned = ret.into();
        let mock_id = context::MockId::new("fns_Foo_ret_owned");
        match context::add_mock::<(), Foo>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
        context::add_expectation::<(), Foo>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateFooStatic_method(pub context::Predicate);
pub struct ReturnFooStatic_method(pub context::ReturnValDoublePointer);

impl ReturnFooStatic_method {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), ()>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateFooStatic_method {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_Foo_static_method");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_static_method(ret: impl Into<ReturnFooStatic_method>) {
        let inner: ReturnFooStatic_method = ret.into();
        let mock_id = context::MockId::new("fns_Foo_static_method");
        match context::add_mock::<(), ()>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
        context::add_expectation::<(), ()>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateFooFallback(pub context::Predicate);
pub struct ReturnFooFallback(pub context::ReturnValDoublePointer);

impl ReturnFooFallback {
    pub fn from_fn(closure: impl Fn(&Foo) -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo,), u32>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateFooFallback {
    pub fn from_fn(closure: impl Fn(&&Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_Foo_fallback");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_fallback(ret: impl Into<ReturnFooFallback>) {
        let inner: ReturnFooFallback = ret.into();
        let mock_id = context::MockId::new("fns_Foo_fallback");
        match context::add_mock::<(&Foo,), u32>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(|_| Ok(())));
        context::add_expectation::<(&Foo,), u32>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateClosureWrapperFmt(pub context::Predicate);
pub struct ReturnClosureWrapperFmt(pub context::ReturnValDoublePointer);

impl ReturnClosureWrapperFmt {
    pub fn from_fn(closure: impl Fn(&ClosureWrapper, &mut std::fmt::Formatter<'_>) -> std::fmt::Result + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>), std::fmt::Result>(
            Box::new(move |(_self, _0,)| closure(_self, _0))
        ))
    }
}

impl PredicateClosureWrapperFmt {
    pub fn from_fn(closure: impl Fn(&&ClosureWrapper, &&mut std::fmt::Formatter<'_>) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_ClosureWrapper_fmt");
        let cond = context::ConditionDoublePointer::from_fn::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>)>(
            Box::new(move |input: &(&ClosureWrapper, &mut std::fmt::Formatter<'_>)| closure(&input.0, &input.1))
        );
        Self(context::Predicate::create_single::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>)>(&mock_id, cond))
    }
}

impl ClosureWrapper {
    pub fn on_call_fmt(ret: impl Into<ReturnClosureWrapperFmt>) {
        let inner: ReturnClosureWrapperFmt = ret.into();
        let mock_id = context::MockId::new("fns_ClosureWrapper_fmt");
        match context::add_mock::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>), std::fmt::Result>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>)>(Box::new(|_| Ok(())));
        context::add_expectation::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>), std::fmt::Result>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateFooSecret(pub context::Predicate);
pub struct ReturnFooSecret(pub context::ReturnValDoublePointer);

impl ReturnFooSecret {
    pub fn from_fn(closure: impl Fn(&Foo) -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo,), u32>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateFooSecret {
    pub fn from_fn(closure: impl Fn(&&Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_Foo_secret");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_secret(ret: impl Into<ReturnFooSecret>) {
        let inner: ReturnFooSecret = ret.into();
        let mock_id = context::MockId::new("fns_Foo_secret");
        match context::add_mock::<(&Foo,), u32>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(|_| Ok(())));
        context::add_expectation::<(&Foo,), u32>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateFooCompute(pub context::Predicate);
pub struct ReturnFooCompute(pub context::ReturnValDoublePointer);

impl ReturnFooCompute {
    pub fn from_fn(closure: impl Fn(&Foo) -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo,), u32>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateFooCompute {
    pub fn from_fn(closure: impl Fn(&&Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_Foo_compute");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_compute(ret: impl Into<ReturnFooCompute>) {
        let inner: ReturnFooCompute = ret.into();
        let mock_id = context::MockId::new("fns_Foo_compute");
        match context::add_mock::<(&Foo,), u32>(mock_id.clone(), None) {
            Ok(()) => {},
            Err(e) if e.to_string().contains("registered twice") => {},
            Err(e) => panic!("failed to add mock: {:?}", e),
        }
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(|_| Ok(())));
        context::add_expectation::<(&Foo,), u32>(
            &mock_id,
            cond,
            Some(inner.0),
            None,
            context::TimesModifier::Any,
        ).unwrap();
    }
}

pub struct PredicateA_Modules(pub context::Predicate);
pub struct ReturnA_Modules(pub context::ReturnValDoublePointer);

impl ReturnA_Modules {
    pub fn from_fn(closure: impl Fn() -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), u32>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateA_Modules {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new("fns_a_modules");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_a_modules(ret: impl Into<ReturnA_Modules>) {
    let inner: ReturnA_Modules = ret.into();
    let mock_id = context::MockId::new("fns_a_modules");
    match context::add_mock::<(), u32>(mock_id.clone(), None) {
        Ok(()) => {},
        Err(e) if e.to_string().contains("registered twice") => {},
        Err(e) => panic!("failed to add mock: {:?}", e),
    }
    let cond = context::ConditionDoublePointer::from_fn::<()>(Box::new(|_| Ok(())));
    context::add_expectation::<(), u32>(
        &mock_id,
        cond,
        Some(inner.0),
        None,
        context::TimesModifier::Any,
    ).unwrap();
}

