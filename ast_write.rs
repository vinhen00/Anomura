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
        let fns_MockStruct_new_mock_id =
            context::MockId::new(stringify!(fns_MockStruct_new));
        if context::ctx_built_and_contains_id(&fns_MockStruct_new_mock_id) {
            match context::run_mock::<(),
                        Self>(fns_MockStruct_new_mock_id, ()) {
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
        } else { panic!("mock_crate: no mock context built for new"); }
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
        let fns_Foo_ret_owned_mock_id =
            context::MockId::new(stringify!(fns_Foo_ret_owned));
        if context::ctx_built_and_contains_id(&fns_Foo_ret_owned_mock_id) {
            match context::run_mock::<(), Foo>(fns_Foo_ret_owned_mock_id, ())
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
        } else { panic!("mock_crate: no mock context built for ret_owned"); }
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

