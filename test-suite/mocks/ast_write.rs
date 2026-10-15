pub fn ref_param(x: &u32) -> () {
    let fns_ref_param_mock_id = context::MockId::new_fn("fns_ref_param");
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
                    context::MockError::AlreadyRegistered => unreachable!(),
                },
        }
    } else { panic!("mock_crate: no mock context built for ref_param"); }
}

pub fn cons_param(x: Box<u32>) -> () {
    let fns_cons_param_mock_id = context::MockId::new_fn("fns_cons_param");
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
                    context::MockError::AlreadyRegistered => unreachable!(),
                },
        }
    } else { panic!("mock_crate: no mock context built for cons_param"); }
}

#[derive(Debug)]
pub struct ConsSelfStruct;

impl ConsSelfStruct {
    pub fn consume_self(self) -> () {
        let fns_ConsSelfStruct_consume_self_mock_id =
            context::MockId::new_adt_static("fns_ConsSelfStruct",
                "consume_self");
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
                        context::MockError::AlreadyRegistered => unreachable!(),
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
    let fns_foreign_mock_id = context::MockId::new_fn("fns_foreign");
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
                    context::MockError::AlreadyRegistered => unreachable!(),
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
            context::MockId::new_adt_static("fns_MockStruct", "new");
        if context::ctx_built_and_contains_id(&fns_MockStruct_new_mock_id) {
            let public: crate::PublicMockStruct =
                context::run_mock::<(),
                            crate::PublicMockStruct>(fns_MockStruct_new_mock_id,
                        ()).unwrap();
            let slf: Self = public.into();
            let _ =
                context::register_mock(&context::MockId::new_adt_instance("fns_MockStruct",
                            "get_value", slf.adt_mock_id));
            slf
        } else {
            let slf =
                Self {
                    pubfield: Default::default(),
                    privfield: std::marker::PhantomData,
                    adt_mock_id: context::new_id(),
                };
            let _ =
                context::register_mock(&context::MockId::new_adt_instance("fns_MockStruct",
                            "get_value", slf.adt_mock_id));
            slf
        }
    }
    pub fn foo() -> () {
        let fns_MockStruct_foo_mock_id =
            context::MockId::new_adt_static("fns_MockStruct", "foo");
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
                        context::MockError::AlreadyRegistered => unreachable!(),
                    },
            }
        } else { panic!("mock_crate: no mock context built for foo"); }
    }
    pub fn get_value(&self) -> u32 {
        let fns_MockStruct_get_value_mock_id =
            context::MockId::new_adt_instance("fns_MockStruct", "get_value",
                self.adt_mock_id);
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
                        context::MockError::AlreadyRegistered => unreachable!(),
                    },
            }
        } else { panic!("mock_crate: no mock context built for get_value"); }
    }
}

pub fn ret_call_w_args(x: i16) -> i16 {
    let fns_ret_call_w_args_mock_id =
        context::MockId::new_fn("fns_ret_call_w_args");
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
                    context::MockError::AlreadyRegistered => unreachable!(),
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
            context::MockId::new_adt_static("fns_Foo", "ret_ref");
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
                        context::MockError::AlreadyRegistered => unreachable!(),
                    },
            }
        } else { panic!("mock_crate: no mock context built for ret_ref"); }
    }
    pub fn ret_mut_ref(&mut self) -> &mut u32 {
        let fns_Foo_ret_mut_ref_mock_id =
            context::MockId::new_adt_static("fns_Foo", "ret_mut_ref");
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
                        context::MockError::AlreadyRegistered => unreachable!(),
                    },
            }
        } else {
            panic!("mock_crate: no mock context built for ret_mut_ref");
        }
    }
    pub fn ret_owned() -> Foo {
        let fns_Foo_ret_owned_mock_id =
            context::MockId::new_adt_static("fns_Foo", "ret_owned");
        if context::ctx_built_and_contains_id(&fns_Foo_ret_owned_mock_id) {
            let public: crate::PublicFoo =
                context::run_mock::<(),
                            crate::PublicFoo>(fns_Foo_ret_owned_mock_id, ()).unwrap();
            let slf: Self = public.into();
            let _ =
                context::register_mock(&context::MockId::new_adt_static("fns_Foo",
                            "ret_ref"));
            let _ =
                context::register_mock(&context::MockId::new_adt_static("fns_Foo",
                            "ret_mut_ref"));
            let _ =
                context::register_mock(&context::MockId::new_adt_static("fns_Foo",
                            "fallback"));
            slf
        } else {
            let slf = Self { x: Default::default() };
            let _ =
                context::register_mock(&context::MockId::new_adt_static("fns_Foo",
                            "ret_ref"));
            let _ =
                context::register_mock(&context::MockId::new_adt_static("fns_Foo",
                            "ret_mut_ref"));
            let _ =
                context::register_mock(&context::MockId::new_adt_static("fns_Foo",
                            "fallback"));
            slf
        }
    }
    pub fn static_method() -> () {
        let fns_Foo_static_method_mock_id =
            context::MockId::new_adt_static("fns_Foo", "static_method");
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
                        context::MockError::AlreadyRegistered => unreachable!(),
                    },
            }
        } else {
            panic!("mock_crate: no mock context built for static_method");
        }
    }
    pub fn fallback(&self) -> u32 {
        let fns_Foo_fallback_mock_id =
            context::MockId::new_adt_static("fns_Foo", "fallback");
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
                        context::MockError::AlreadyRegistered => unreachable!(),
                    },
            }
        } else { panic!("mock_crate: no mock context built for fallback"); }
    }
}

pub fn ret_param(x: &mut u32) -> () {
    let fns_ret_param_mock_id = context::MockId::new_fn("fns_ret_param");
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
                    context::MockError::AlreadyRegistered => unreachable!(),
                },
        }
    } else { panic!("mock_crate: no mock context built for ret_param"); }
}

pub mod a {
    pub fn modules() -> u32 {
        let fns_a_modules_mock_id = context::MockId::new_fn("fns_a_modules");
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
                        context::MockError::AlreadyRegistered => unreachable!(),
                    },
            }
        } else { panic!("mock_crate: no mock context built for modules"); }
    }
    pub mod nested {
        pub struct Inner {
            pub value: i32,
        }
        impl Inner {
            pub fn new(value: i32) -> Self {
                let fns_a_nested_Inner_new_mock_id =
                    context::MockId::new_adt_static("fns_a_nested_Inner",
                        "new");
                if context::ctx_built_and_contains_id(&fns_a_nested_Inner_new_mock_id)
                    {
                    let public: crate::PublicANestedInner =
                        context::run_mock::<(i32,),
                                    crate::PublicANestedInner>(fns_a_nested_Inner_new_mock_id,
                                (value,)).unwrap();
                    let slf: Self = public.into();
                    let _ =
                        context::register_mock(&context::MockId::new_adt_static("fns_a_nested_Inner",
                                    "double"));
                    let _ =
                        context::register_mock(&context::MockId::new_adt_static("fns_a_nested_Inner",
                                    "tripple"));
                    slf
                } else {
                    let slf = Self { value };
                    let _ =
                        context::register_mock(&context::MockId::new_adt_static("fns_a_nested_Inner",
                                    "double"));
                    let _ =
                        context::register_mock(&context::MockId::new_adt_static("fns_a_nested_Inner",
                                    "tripple"));
                    slf
                }
            }
            pub fn double(&self) -> i32 {
                let fns_a_nested_Inner_double_mock_id =
                    context::MockId::new_adt_static("fns_a_nested_Inner",
                        "double");
                if context::ctx_built_and_contains_id(&fns_a_nested_Inner_double_mock_id)
                    {
                    match context::run_mock::<(&Inner,),
                                i32>(fns_a_nested_Inner_double_mock_id, (self,)) {
                        Ok(res) => res,
                        Err(e) =>
                            match e {
                                context::MockError::Other(e) =>
                                    panic!("unexpected Error: {:?}", e),
                                context::MockError::PredicateError(e) =>
                                    panic!("{:?}", e.0),
                                context::MockError::NoMatchingId =>
                                    panic!("failed to find mock id"),
                                context::MockError::AlreadyRegistered => unreachable!(),
                            },
                    }
                } else {
                    panic!("mock_crate: no mock context built for double");
                }
            }
            pub fn tripple(&self) -> i32 {
                let fns_a_nested_Inner_tripple_mock_id =
                    context::MockId::new_adt_static("fns_a_nested_Inner",
                        "tripple");
                if context::ctx_built_and_contains_id(&fns_a_nested_Inner_tripple_mock_id)
                    {
                    match context::run_mock::<(&Inner,),
                                i32>(fns_a_nested_Inner_tripple_mock_id, (self,)) {
                        Ok(res) => res,
                        Err(e) =>
                            match e {
                                context::MockError::Other(e) =>
                                    panic!("unexpected Error: {:?}", e),
                                context::MockError::PredicateError(e) =>
                                    panic!("{:?}", e.0),
                                context::MockError::NoMatchingId =>
                                    panic!("failed to find mock id"),
                                context::MockError::AlreadyRegistered => unreachable!(),
                            },
                    }
                } else {
                    panic!("mock_crate: no mock context built for tripple");
                }
            }
        }
        pub fn deep_fn() -> &'static str {
            let fns_a_nested_deep_fn_mock_id =
                context::MockId::new_fn("fns_a_nested_deep_fn");
            if context::ctx_built_and_contains_id(&fns_a_nested_deep_fn_mock_id)
                {
                match context::run_mock::<(),
                            &'static str>(fns_a_nested_deep_fn_mock_id, ()) {
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
                            context::MockError::AlreadyRegistered => unreachable!(),
                        },
                }
            } else {
                panic!("mock_crate: no mock context built for deep_fn");
            }
        }
    }
}

/// by default i don't panic. i don't do anything c:
pub fn return_panic() -> () {
    let fns_return_panic_mock_id =
        context::MockId::new_fn("fns_return_panic");
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
                    context::MockError::AlreadyRegistered => unreachable!(),
                },
        }
    } else { panic!("mock_crate: no mock context built for return_panic"); }
}

#[derive(Debug)]
pub enum Pattern { Okay, NotOkay, }

pub struct ClosureWrapper(pub Box<dyn Fn(u32) -> u32>);

impl std::fmt::Debug for ClosureWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fns_ClosureWrapper_fmt_mock_id =
            context::MockId::new_adt_static("fns_ClosureWrapper", "fmt");
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
                        context::MockError::AlreadyRegistered => unreachable!(),
                    },
            }
        } else { panic!("mock_crate: no mock context built for fmt"); }
    }
}

pub fn closure_param(f: ClosureWrapper) -> u32 {
    let fns_closure_param_mock_id =
        context::MockId::new_fn("fns_closure_param");
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
                    context::MockError::AlreadyRegistered => unreachable!(),
                },
        }
    } else { panic!("mock_crate: no mock context built for closure_param"); }
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
            context::MockId::new_adt_static("fns_Foo", "compute");
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
                        context::MockError::AlreadyRegistered => unreachable!(),
                    },
            }
        } else { panic!("mock_crate: no mock context built for compute"); }
    }
}

pub struct PredicateRef_param(pub context::Predicate);

pub struct ReturnRef_param(pub context::ReturnValDoublePointer);

impl ReturnRef_param {
    pub fn from_fn(closure: impl Fn(&u32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&u32,),
                    ()>(Box::new(move |(_0,)| closure(_0))))
    }
}

impl PredicateRef_param {
    pub fn from_fn(closure:
            impl Fn(&&u32) -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_fn("fns_ref_param");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&u32,)>(Box::new(move
                        |input: &(&u32,)| closure(&input.0)));
        Self(context::Predicate::create_single::<(&u32,)>(&mock_id, cond))
    }
}

pub fn on_call_ref_param(condition: impl Fn(&&u32) -> bool + 'static,
    ret: impl Into<ReturnRef_param>) {
    let inner: ReturnRef_param = ret.into();
    let mock_id = context::MockId::new_fn("fns_ref_param");
    let cond =
        context::ConditionDoublePointer::from_fn::<(&u32,)>(Box::new(move
                    |input: &(&u32,)|
                    {
                        if condition(&input.0) {
                            Ok(())
                        } else {
                            Err("on_call condition failed for ref_param".into())
                        }
                    }));
    context::add_on_call::<(&u32,)>(&mock_id, cond, inner.0, None).unwrap();
}

pub fn expect_ref_param(condition:
        impl Fn(&&u32) -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn(&u32) -> () + 'static, modifier: context::TimesModifier,
    checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_ref_param");
    let cond =
        context::ConditionDoublePointer::from_fn::<(&u32,)>(Box::new(move
                    |input: &(&u32,)| condition(&input.0)));
    let ret_ptr =
        context::ReturnValDoublePointer::from_fn::<(&u32,),
                ()>(Box::new(move |(_0,)| ret(_0)));
    context::add_expectation::<(&u32,),
                ()>(&mock_id, cond, Some(ret_ptr),
            checkpoint.map(|s| context::CheckpointName(s.to_string())),
            modifier).unwrap();
}

pub fn sequence_ref_param(seq_name: &str, index: usize,
    condition:
        impl Fn(&&u32) -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn(&u32) -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_ref_param");
    let _ = context::register_mock(&mock_id);
    let cond =
        context::ConditionDoublePointer::from_fn::<(&u32,)>(Box::new(move
                    |input: &(&u32,)| condition(&input.0)));
    context::add_expectation_to_sequence::<(&u32,),
                ()>(&mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
            seq_name, index, None).unwrap();
}

pub struct PredicateCons_param(pub context::Predicate);

pub struct ReturnCons_param(pub context::ReturnValDoublePointer);

impl ReturnCons_param {
    pub fn from_fn(closure: impl Fn(Box<u32>) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(Box<u32>,),
                    ()>(Box::new(move |(_0,)| closure(_0))))
    }
}

impl PredicateCons_param {
    pub fn from_fn(closure:
            impl Fn(&Box<u32>) -> context::errors::PredicateResult<()> +
            'static) -> Self {
        let mock_id = context::MockId::new_fn("fns_cons_param");
        let cond =
            context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(Box::new(move
                        |input: &(Box<u32>,)| closure(&input.0)));
        Self(context::Predicate::create_single::<(Box<u32>,)>(&mock_id, cond))
    }
}

pub fn on_call_cons_param(condition: impl Fn(&Box<u32>) -> bool + 'static,
    ret: impl Into<ReturnCons_param>) {
    let inner: ReturnCons_param = ret.into();
    let mock_id = context::MockId::new_fn("fns_cons_param");
    let cond =
        context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(Box::new(move
                    |input: &(Box<u32>,)|
                    {
                        if condition(&input.0) {
                            Ok(())
                        } else {
                            Err("on_call condition failed for cons_param".into())
                        }
                    }));
    context::add_on_call::<(Box<u32>,)>(&mock_id, cond, inner.0,
            None).unwrap();
}

pub fn expect_cons_param(condition:
        impl Fn(&Box<u32>) -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn(Box<u32>) -> () + 'static, modifier: context::TimesModifier,
    checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_cons_param");
    let cond =
        context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(Box::new(move
                    |input: &(Box<u32>,)| condition(&input.0)));
    let ret_ptr =
        context::ReturnValDoublePointer::from_fn::<(Box<u32>,),
                ()>(Box::new(move |(_0,)| ret(_0)));
    context::add_expectation::<(Box<u32>,),
                ()>(&mock_id, cond, Some(ret_ptr),
            checkpoint.map(|s| context::CheckpointName(s.to_string())),
            modifier).unwrap();
}

pub fn sequence_cons_param(seq_name: &str, index: usize,
    condition:
        impl Fn(&Box<u32>) -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn(Box<u32>) -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_cons_param");
    let _ = context::register_mock(&mock_id);
    let cond =
        context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(Box::new(move
                    |input: &(Box<u32>,)| condition(&input.0)));
    context::add_expectation_to_sequence::<(Box<u32>,),
                ()>(&mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
            seq_name, index, None).unwrap();
}

pub struct PredicateForeign(pub context::Predicate);

pub struct ReturnForeign(pub context::ReturnValDoublePointer);

impl ReturnForeign {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(),
                    ()>(Box::new(move |()| closure())))
    }
}

impl PredicateForeign {
    pub fn from_fn(closure:
            impl Fn() -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_fn("fns_foreign");
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()| closure()));
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_foreign(condition: impl Fn() -> bool + 'static,
    ret: impl Into<ReturnForeign>) {
    let inner: ReturnForeign = ret.into();
    let mock_id = context::MockId::new_fn("fns_foreign");
    let cond =
        context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                    |input: &()|
                    {
                        if condition() {
                            Ok(())
                        } else {
                            Err("on_call condition failed for foreign".into())
                        }
                    }));
    context::add_on_call::<()>(&mock_id, cond, inner.0, None).unwrap();
}

pub fn expect_foreign(condition:
        impl Fn() -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn() -> () + 'static, modifier: context::TimesModifier,
    checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_foreign");
    let cond =
        context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                    |input: &()| condition()));
    let ret_ptr =
        context::ReturnValDoublePointer::from_fn::<(),
                ()>(Box::new(move |()| ret()));
    context::add_expectation::<(),
                ()>(&mock_id, cond, Some(ret_ptr),
            checkpoint.map(|s| context::CheckpointName(s.to_string())),
            modifier).unwrap();
}

pub fn sequence_foreign(seq_name: &str, index: usize,
    condition: impl Fn() -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_foreign");
    let _ = context::register_mock(&mock_id);
    let cond =
        context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                    |input: &()| condition()));
    context::add_expectation_to_sequence::<(),
                ()>(&mock_id, cond, Some(Box::new(move |()| ret())), seq_name,
            index, None).unwrap();
}

pub struct PredicateRet_call_w_args(pub context::Predicate);

pub struct ReturnRet_call_w_args(pub context::ReturnValDoublePointer);

impl ReturnRet_call_w_args {
    pub fn from_fn(closure: impl Fn(i16) -> i16 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(i16,),
                    i16>(Box::new(move |(_0,)| closure(_0))))
    }
}

impl PredicateRet_call_w_args {
    pub fn from_fn(closure:
            impl Fn(&i16) -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_fn("fns_ret_call_w_args");
        let cond =
            context::ConditionDoublePointer::from_fn::<(i16,)>(Box::new(move
                        |input: &(i16,)| closure(&input.0)));
        Self(context::Predicate::create_single::<(i16,)>(&mock_id, cond))
    }
}

pub fn on_call_ret_call_w_args(condition: impl Fn(&i16) -> bool + 'static,
    ret: impl Into<ReturnRet_call_w_args>) {
    let inner: ReturnRet_call_w_args = ret.into();
    let mock_id = context::MockId::new_fn("fns_ret_call_w_args");
    let cond =
        context::ConditionDoublePointer::from_fn::<(i16,)>(Box::new(move
                    |input: &(i16,)|
                    {
                        if condition(&input.0) {
                            Ok(())
                        } else {
                            Err("on_call condition failed for ret_call_w_args".into())
                        }
                    }));
    context::add_on_call::<(i16,)>(&mock_id, cond, inner.0, None).unwrap();
}

pub fn expect_ret_call_w_args(condition:
        impl Fn(&i16) -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn(i16) -> i16 + 'static, modifier: context::TimesModifier,
    checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_ret_call_w_args");
    let cond =
        context::ConditionDoublePointer::from_fn::<(i16,)>(Box::new(move
                    |input: &(i16,)| condition(&input.0)));
    let ret_ptr =
        context::ReturnValDoublePointer::from_fn::<(i16,),
                i16>(Box::new(move |(_0,)| ret(_0)));
    context::add_expectation::<(i16,),
                i16>(&mock_id, cond, Some(ret_ptr),
            checkpoint.map(|s| context::CheckpointName(s.to_string())),
            modifier).unwrap();
}

pub fn sequence_ret_call_w_args(seq_name: &str, index: usize,
    condition:
        impl Fn(&i16) -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn(i16) -> i16 + 'static) {
    let mock_id = context::MockId::new_fn("fns_ret_call_w_args");
    let _ = context::register_mock(&mock_id);
    let cond =
        context::ConditionDoublePointer::from_fn::<(i16,)>(Box::new(move
                    |input: &(i16,)| condition(&input.0)));
    context::add_expectation_to_sequence::<(i16,),
                i16>(&mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
            seq_name, index, None).unwrap();
}

pub struct PredicateRet_param(pub context::Predicate);

pub struct ReturnRet_param(pub context::ReturnValDoublePointer);

impl ReturnRet_param {
    pub fn from_fn(closure: impl Fn(&mut u32) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&mut u32,),
                    ()>(Box::new(move |(_0,)| closure(_0))))
    }
}

impl PredicateRet_param {
    pub fn from_fn(closure:
            impl Fn(&&mut u32) -> context::errors::PredicateResult<()> +
            'static) -> Self {
        let mock_id = context::MockId::new_fn("fns_ret_param");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&mut u32,)>(Box::new(move
                        |input: &(&mut u32,)| closure(&input.0)));
        Self(context::Predicate::create_single::<(&mut u32,)>(&mock_id, cond))
    }
}

pub fn on_call_ret_param(condition: impl Fn(&&mut u32) -> bool + 'static,
    ret: impl Into<ReturnRet_param>) {
    let inner: ReturnRet_param = ret.into();
    let mock_id = context::MockId::new_fn("fns_ret_param");
    let cond =
        context::ConditionDoublePointer::from_fn::<(&mut u32,)>(Box::new(move
                    |input: &(&mut u32,)|
                    {
                        if condition(&input.0) {
                            Ok(())
                        } else {
                            Err("on_call condition failed for ret_param".into())
                        }
                    }));
    context::add_on_call::<(&mut u32,)>(&mock_id, cond, inner.0,
            None).unwrap();
}

pub fn expect_ret_param(condition:
        impl Fn(&&mut u32) -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn(&mut u32) -> () + 'static, modifier: context::TimesModifier,
    checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_ret_param");
    let cond =
        context::ConditionDoublePointer::from_fn::<(&mut u32,)>(Box::new(move
                    |input: &(&mut u32,)| condition(&input.0)));
    let ret_ptr =
        context::ReturnValDoublePointer::from_fn::<(&mut u32,),
                ()>(Box::new(move |(_0,)| ret(_0)));
    context::add_expectation::<(&mut u32,),
                ()>(&mock_id, cond, Some(ret_ptr),
            checkpoint.map(|s| context::CheckpointName(s.to_string())),
            modifier).unwrap();
}

pub fn sequence_ret_param(seq_name: &str, index: usize,
    condition:
        impl Fn(&&mut u32) -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn(&mut u32) -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_ret_param");
    let _ = context::register_mock(&mock_id);
    let cond =
        context::ConditionDoublePointer::from_fn::<(&mut u32,)>(Box::new(move
                    |input: &(&mut u32,)| condition(&input.0)));
    context::add_expectation_to_sequence::<(&mut u32,),
                ()>(&mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
            seq_name, index, None).unwrap();
}

pub struct PredicateReturn_panic(pub context::Predicate);

pub struct ReturnReturn_panic(pub context::ReturnValDoublePointer);

impl ReturnReturn_panic {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(),
                    ()>(Box::new(move |()| closure())))
    }
}

impl PredicateReturn_panic {
    pub fn from_fn(closure:
            impl Fn() -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_fn("fns_return_panic");
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()| closure()));
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_return_panic(condition: impl Fn() -> bool + 'static,
    ret: impl Into<ReturnReturn_panic>) {
    let inner: ReturnReturn_panic = ret.into();
    let mock_id = context::MockId::new_fn("fns_return_panic");
    let cond =
        context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                    |input: &()|
                    {
                        if condition() {
                            Ok(())
                        } else {
                            Err("on_call condition failed for return_panic".into())
                        }
                    }));
    context::add_on_call::<()>(&mock_id, cond, inner.0, None).unwrap();
}

pub fn expect_return_panic(condition:
        impl Fn() -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn() -> () + 'static, modifier: context::TimesModifier,
    checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_return_panic");
    let cond =
        context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                    |input: &()| condition()));
    let ret_ptr =
        context::ReturnValDoublePointer::from_fn::<(),
                ()>(Box::new(move |()| ret()));
    context::add_expectation::<(),
                ()>(&mock_id, cond, Some(ret_ptr),
            checkpoint.map(|s| context::CheckpointName(s.to_string())),
            modifier).unwrap();
}

pub fn sequence_return_panic(seq_name: &str, index: usize,
    condition: impl Fn() -> context::errors::PredicateResult<()> + 'static,
    ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_return_panic");
    let _ = context::register_mock(&mock_id);
    let cond =
        context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                    |input: &()| condition()));
    context::add_expectation_to_sequence::<(),
                ()>(&mock_id, cond, Some(Box::new(move |()| ret())), seq_name,
            index, None).unwrap();
}

pub struct PredicateClosure_param(pub context::Predicate);

pub struct ReturnClosure_param(pub context::ReturnValDoublePointer);

impl ReturnClosure_param {
    pub fn from_fn(closure: impl Fn(ClosureWrapper) -> u32 + 'static)
        -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(ClosureWrapper,),
                    u32>(Box::new(move |(_0,)| closure(_0))))
    }
}

impl PredicateClosure_param {
    pub fn from_fn(closure:
            impl Fn(&ClosureWrapper) -> context::errors::PredicateResult<()> +
            'static) -> Self {
        let mock_id = context::MockId::new_fn("fns_closure_param");
        let cond =
            context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(Box::new(move
                        |input: &(ClosureWrapper,)| closure(&input.0)));
        Self(context::Predicate::create_single::<(ClosureWrapper,)>(&mock_id,
                cond))
    }
}

pub fn on_call_closure_param(condition:
        impl Fn(&ClosureWrapper) -> bool + 'static,
    ret: impl Into<ReturnClosure_param>) {
    let inner: ReturnClosure_param = ret.into();
    let mock_id = context::MockId::new_fn("fns_closure_param");
    let cond =
        context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(Box::new(move
                    |input: &(ClosureWrapper,)|
                    {
                        if condition(&input.0) {
                            Ok(())
                        } else {
                            Err("on_call condition failed for closure_param".into())
                        }
                    }));
    context::add_on_call::<(ClosureWrapper,)>(&mock_id, cond, inner.0,
            None).unwrap();
}

pub fn expect_closure_param(condition:
        impl Fn(&ClosureWrapper) -> context::errors::PredicateResult<()> +
        'static, ret: impl Fn(ClosureWrapper) -> u32 + 'static,
    modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_closure_param");
    let cond =
        context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(Box::new(move
                    |input: &(ClosureWrapper,)| condition(&input.0)));
    let ret_ptr =
        context::ReturnValDoublePointer::from_fn::<(ClosureWrapper,),
                u32>(Box::new(move |(_0,)| ret(_0)));
    context::add_expectation::<(ClosureWrapper,),
                u32>(&mock_id, cond, Some(ret_ptr),
            checkpoint.map(|s| context::CheckpointName(s.to_string())),
            modifier).unwrap();
}

pub fn sequence_closure_param(seq_name: &str, index: usize,
    condition:
        impl Fn(&ClosureWrapper) -> context::errors::PredicateResult<()> +
        'static, ret: impl Fn(ClosureWrapper) -> u32 + 'static) {
    let mock_id = context::MockId::new_fn("fns_closure_param");
    let _ = context::register_mock(&mock_id);
    let cond =
        context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(Box::new(move
                    |input: &(ClosureWrapper,)| condition(&input.0)));
    context::add_expectation_to_sequence::<(ClosureWrapper,),
                u32>(&mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
            seq_name, index, None).unwrap();
}

pub struct PredicateConsSelfStructConsume_self(pub context::Predicate);

pub struct ReturnConsSelfStructConsume_self(pub context::ReturnValDoublePointer);

impl ReturnConsSelfStructConsume_self {
    pub fn from_fn(closure: impl Fn(ConsSelfStruct) -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(ConsSelfStruct,),
                    ()>(Box::new(move |(_self,)| closure(_self))))
    }
}

impl PredicateConsSelfStructConsume_self {
    pub fn from_fn(closure:
            impl Fn(&ConsSelfStruct) -> context::errors::PredicateResult<()> +
            'static) -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_ConsSelfStruct",
                "consume_self");
        let cond =
            context::ConditionDoublePointer::from_fn::<(ConsSelfStruct,)>(Box::new(move
                        |input: &(ConsSelfStruct,)| closure(&input.0)));
        Self(context::Predicate::create_single::<(ConsSelfStruct,)>(&mock_id,
                cond))
    }
}

impl ConsSelfStruct {
    pub fn on_call_consume_self(condition:
            impl Fn(&ConsSelfStruct) -> bool + 'static,
        ret: impl Into<ReturnConsSelfStructConsume_self>) {
        let inner: ReturnConsSelfStructConsume_self = ret.into();
        let mock_id =
            context::MockId::new_adt_static("fns_ConsSelfStruct",
                "consume_self");
        let object_mock_id =
            context::AdtId::new("fns_ConsSelfStruct",
                context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(ConsSelfStruct,)>(Box::new(move
                        |input: &(ConsSelfStruct,)|
                        {
                            if condition(&input.0) {
                                Ok(())
                            } else {
                                Err("on_call condition failed for consume_self".into())
                            }
                        }));
        context::add_method_on_call::<(ConsSelfStruct,)>(&object_mock_id,
                "consume_self", &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PublicMockStruct {
    pub pubfield: u32,
}

impl Into<MockStruct> for PublicMockStruct {
    fn into(self) -> MockStruct {
        MockStruct {
            pubfield: self.pubfield,
            privfield: std::marker::PhantomData,
            adt_mock_id: context::new_id(),
        }
    }
}

pub struct PredicateMockStructNew(pub context::Predicate);

pub struct ReturnMockStructNew(pub context::ReturnValDoublePointer);

impl ReturnMockStructNew {
    pub fn from_fn(closure: impl Fn() -> PublicMockStruct + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(),
                    PublicMockStruct>(Box::new(move |()| closure())))
    }
}

impl PredicateMockStructNew {
    pub fn from_fn(closure:
            impl Fn() -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_MockStruct", "new");
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()| closure()));
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_new(&self, condition: impl Fn() -> bool + 'static,
        ret: impl Into<ReturnMockStructNew>) {
        let inner: ReturnMockStructNew = ret.into();
        let mock_id =
            context::MockId::new_adt_instance("fns_MockStruct", "new",
                self.adt_mock_id);
        let object_mock_id =
            context::AdtId::new("fns_MockStruct", self.adt_mock_id);
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()|
                        {
                            if condition() {
                                Ok(())
                            } else { Err("on_call condition failed for new".into()) }
                        }));
        context::add_method_on_call::<()>(&object_mock_id, "new", &mock_id,
                cond, inner.0, None).unwrap();
    }
}

pub struct PredicateMockStructFoo(pub context::Predicate);

pub struct ReturnMockStructFoo(pub context::ReturnValDoublePointer);

impl ReturnMockStructFoo {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(),
                    ()>(Box::new(move |()| closure())))
    }
}

impl PredicateMockStructFoo {
    pub fn from_fn(closure:
            impl Fn() -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_MockStruct", "foo");
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()| closure()));
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_foo(&self, condition: impl Fn() -> bool + 'static,
        ret: impl Into<ReturnMockStructFoo>) {
        let inner: ReturnMockStructFoo = ret.into();
        let mock_id =
            context::MockId::new_adt_instance("fns_MockStruct", "foo",
                self.adt_mock_id);
        let object_mock_id =
            context::AdtId::new("fns_MockStruct", self.adt_mock_id);
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()|
                        {
                            if condition() {
                                Ok(())
                            } else { Err("on_call condition failed for foo".into()) }
                        }));
        context::add_method_on_call::<()>(&object_mock_id, "foo", &mock_id,
                cond, inner.0, None).unwrap();
    }
}

pub struct PredicateMockStructGet_value(pub context::Predicate);

pub struct ReturnMockStructGet_value(pub context::ReturnValDoublePointer);

impl ReturnMockStructGet_value {
    pub fn from_fn(closure: impl Fn(&MockStruct) -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&MockStruct,),
                    u32>(Box::new(move |(_self,)| closure(_self))))
    }
}

impl PredicateMockStructGet_value {
    pub fn from_fn(closure:
            impl Fn(&MockStruct) -> context::errors::PredicateResult<()> +
            'static) -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_MockStruct", "get_value");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&MockStruct,)>(Box::new(move
                        |input: &(&MockStruct,)| closure(input.0)));
        Self(context::Predicate::create_single::<(&MockStruct,)>(&mock_id,
                cond))
    }
}

impl MockStruct {
    pub fn on_call_get_value(&self,
        condition: impl Fn(&MockStruct) -> bool + 'static,
        ret: impl Into<ReturnMockStructGet_value>) {
        let inner: ReturnMockStructGet_value = ret.into();
        let mock_id =
            context::MockId::new_adt_instance("fns_MockStruct", "get_value",
                self.adt_mock_id);
        let object_mock_id =
            context::AdtId::new("fns_MockStruct", self.adt_mock_id);
        let cond =
            context::ConditionDoublePointer::from_fn::<(&MockStruct,)>(Box::new(move
                        |input: &(&MockStruct,)|
                        {
                            if condition(input.0) {
                                Ok(())
                            } else {
                                Err("on_call condition failed for get_value".into())
                            }
                        }));
        context::add_method_on_call::<(&MockStruct,)>(&object_mock_id,
                "get_value", &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PublicFoo {
    pub x: u32,
}

impl Into<Foo> for PublicFoo {
    fn into(self) -> Foo { Foo { x: self.x } }
}

pub struct PredicateFooRet_ref(pub context::Predicate);

pub struct ReturnFooRet_ref(pub context::ReturnValDoublePointer);

impl ReturnFooRet_ref {
    pub fn from_fn(closure: impl Fn(&Foo) -> &u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo,),
                    &u32>(Box::new(move |(_self,)| closure(_self))))
    }
}

impl PredicateFooRet_ref {
    pub fn from_fn(closure:
            impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_ref");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(move
                        |input: &(&Foo,)| closure(input.0)));
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_ret_ref(condition: impl Fn(&Foo) -> bool + 'static,
        ret: impl Into<ReturnFooRet_ref>) {
        let inner: ReturnFooRet_ref = ret.into();
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_ref");
        let object_mock_id =
            context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(move
                        |input: &(&Foo,)|
                        {
                            if condition(input.0) {
                                Ok(())
                            } else {
                                Err("on_call condition failed for ret_ref".into())
                            }
                        }));
        context::add_method_on_call::<(&Foo,)>(&object_mock_id, "ret_ref",
                &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateFooRet_mut_ref(pub context::Predicate);

pub struct ReturnFooRet_mut_ref(pub context::ReturnValDoublePointer);

impl ReturnFooRet_mut_ref {
    pub fn from_fn(closure: impl Fn(&mut Foo) -> &mut u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&mut Foo,),
                    &mut u32>(Box::new(move |(_self,)| closure(_self))))
    }
}

impl PredicateFooRet_mut_ref {
    pub fn from_fn(closure:
            impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_Foo", "ret_mut_ref");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&mut Foo,)>(Box::new(move
                        |input: &(&mut Foo,)| closure(&*input.0)));
        Self(context::Predicate::create_single::<(&mut Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_ret_mut_ref(condition: impl Fn(&Foo) -> bool + 'static,
        ret: impl Into<ReturnFooRet_mut_ref>) {
        let inner: ReturnFooRet_mut_ref = ret.into();
        let mock_id =
            context::MockId::new_adt_static("fns_Foo", "ret_mut_ref");
        let object_mock_id =
            context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(&mut Foo,)>(Box::new(move
                        |input: &(&mut Foo,)|
                        {
                            if condition(&*input.0) {
                                Ok(())
                            } else {
                                Err("on_call condition failed for ret_mut_ref".into())
                            }
                        }));
        context::add_method_on_call::<(&mut Foo,)>(&object_mock_id,
                "ret_mut_ref", &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateFooRet_owned(pub context::Predicate);

pub struct ReturnFooRet_owned(pub context::ReturnValDoublePointer);

impl ReturnFooRet_owned {
    pub fn from_fn(closure: impl Fn() -> PublicFoo + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(),
                    PublicFoo>(Box::new(move |()| closure())))
    }
}

impl PredicateFooRet_owned {
    pub fn from_fn(closure:
            impl Fn() -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_owned");
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()| closure()));
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_ret_owned(condition: impl Fn() -> bool + 'static,
        ret: impl Into<ReturnFooRet_owned>) {
        let inner: ReturnFooRet_owned = ret.into();
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_owned");
        let object_mock_id =
            context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()|
                        {
                            if condition() {
                                Ok(())
                            } else {
                                Err("on_call condition failed for ret_owned".into())
                            }
                        }));
        context::add_method_on_call::<()>(&object_mock_id, "ret_owned",
                &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateFooStatic_method(pub context::Predicate);

pub struct ReturnFooStatic_method(pub context::ReturnValDoublePointer);

impl ReturnFooStatic_method {
    pub fn from_fn(closure: impl Fn() -> () + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(),
                    ()>(Box::new(move |()| closure())))
    }
}

impl PredicateFooStatic_method {
    pub fn from_fn(closure:
            impl Fn() -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_Foo", "static_method");
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()| closure()));
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_static_method(condition: impl Fn() -> bool + 'static,
        ret: impl Into<ReturnFooStatic_method>) {
        let inner: ReturnFooStatic_method = ret.into();
        let mock_id =
            context::MockId::new_adt_static("fns_Foo", "static_method");
        let object_mock_id =
            context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()|
                        {
                            if condition() {
                                Ok(())
                            } else {
                                Err("on_call condition failed for static_method".into())
                            }
                        }));
        context::add_method_on_call::<()>(&object_mock_id, "static_method",
                &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateFooFallback(pub context::Predicate);

pub struct ReturnFooFallback(pub context::ReturnValDoublePointer);

impl ReturnFooFallback {
    pub fn from_fn(closure: impl Fn(&Foo) -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo,),
                    u32>(Box::new(move |(_self,)| closure(_self))))
    }
}

impl PredicateFooFallback {
    pub fn from_fn(closure:
            impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "fallback");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(move
                        |input: &(&Foo,)| closure(input.0)));
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_fallback(condition: impl Fn(&Foo) -> bool + 'static,
        ret: impl Into<ReturnFooFallback>) {
        let inner: ReturnFooFallback = ret.into();
        let mock_id = context::MockId::new_adt_static("fns_Foo", "fallback");
        let object_mock_id =
            context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(move
                        |input: &(&Foo,)|
                        {
                            if condition(input.0) {
                                Ok(())
                            } else {
                                Err("on_call condition failed for fallback".into())
                            }
                        }));
        context::add_method_on_call::<(&Foo,)>(&object_mock_id, "fallback",
                &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateClosureWrapperFmt(pub context::Predicate);

pub struct ReturnClosureWrapperFmt(pub context::ReturnValDoublePointer);

impl ReturnClosureWrapperFmt {
    pub fn from_fn(closure:
            impl Fn(&ClosureWrapper, &mut std::fmt::Formatter<'_>)
            -> std::fmt::Result + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&ClosureWrapper,
                    &mut std::fmt::Formatter<'_>),
                    std::fmt::Result>(Box::new(move |(_self, _0)|
                        closure(_self, _0))))
    }
}

impl PredicateClosureWrapperFmt {
    pub fn from_fn(closure:
            impl Fn(&ClosureWrapper, &&mut std::fmt::Formatter<'_>)
            -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_ClosureWrapper", "fmt");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&ClosureWrapper,
                    &mut std::fmt::Formatter<'_>)>(Box::new(move
                        |input: &(&ClosureWrapper, &mut std::fmt::Formatter<'_>)|
                        closure(input.0, &input.1)));
        Self(context::Predicate::create_single::<(&ClosureWrapper,
                    &mut std::fmt::Formatter<'_>)>(&mock_id, cond))
    }
}

impl ClosureWrapper {
    pub fn on_call_fmt(condition:
            impl Fn(&ClosureWrapper, &&mut std::fmt::Formatter<'_>) -> bool +
            'static, ret: impl Into<ReturnClosureWrapperFmt>) {
        let inner: ReturnClosureWrapperFmt = ret.into();
        let mock_id =
            context::MockId::new_adt_static("fns_ClosureWrapper", "fmt");
        let object_mock_id =
            context::AdtId::new("fns_ClosureWrapper",
                context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(&ClosureWrapper,
                    &mut std::fmt::Formatter<'_>)>(Box::new(move
                        |input: &(&ClosureWrapper, &mut std::fmt::Formatter<'_>)|
                        {
                            if condition(input.0, &input.1) {
                                Ok(())
                            } else { Err("on_call condition failed for fmt".into()) }
                        }));
        context::add_method_on_call::<(&ClosureWrapper,
                    &mut std::fmt::Formatter<'_>)>(&object_mock_id, "fmt",
                &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateFooSecret(pub context::Predicate);

pub struct ReturnFooSecret(pub context::ReturnValDoublePointer);

impl ReturnFooSecret {
    pub fn from_fn(closure: impl Fn(&Foo) -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo,),
                    u32>(Box::new(move |(_self,)| closure(_self))))
    }
}

impl PredicateFooSecret {
    pub fn from_fn(closure:
            impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "secret");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(move
                        |input: &(&Foo,)| closure(input.0)));
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_secret(condition: impl Fn(&Foo) -> bool + 'static,
        ret: impl Into<ReturnFooSecret>) {
        let inner: ReturnFooSecret = ret.into();
        let mock_id = context::MockId::new_adt_static("fns_Foo", "secret");
        let object_mock_id =
            context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(move
                        |input: &(&Foo,)|
                        {
                            if condition(input.0) {
                                Ok(())
                            } else { Err("on_call condition failed for secret".into()) }
                        }));
        context::add_method_on_call::<(&Foo,)>(&object_mock_id, "secret",
                &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateFooCompute(pub context::Predicate);

pub struct ReturnFooCompute(pub context::ReturnValDoublePointer);

impl ReturnFooCompute {
    pub fn from_fn(closure: impl Fn(&Foo) -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo,),
                    u32>(Box::new(move |(_self,)| closure(_self))))
    }
}

impl PredicateFooCompute {
    pub fn from_fn(closure:
            impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "compute");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(move
                        |input: &(&Foo,)| closure(input.0)));
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_compute(condition: impl Fn(&Foo) -> bool + 'static,
        ret: impl Into<ReturnFooCompute>) {
        let inner: ReturnFooCompute = ret.into();
        let mock_id = context::MockId::new_adt_static("fns_Foo", "compute");
        let object_mock_id =
            context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(&Foo,)>(Box::new(move
                        |input: &(&Foo,)|
                        {
                            if condition(input.0) {
                                Ok(())
                            } else {
                                Err("on_call condition failed for compute".into())
                            }
                        }));
        context::add_method_on_call::<(&Foo,)>(&object_mock_id, "compute",
                &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateA_Modules(pub context::Predicate);

pub struct ReturnA_Modules(pub context::ReturnValDoublePointer);

impl ReturnA_Modules {
    pub fn from_fn(closure: impl Fn() -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(),
                    u32>(Box::new(move |()| closure())))
    }
}

impl PredicateA_Modules {
    pub fn from_fn(closure:
            impl Fn() -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_fn("fns_a_modules");
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()| closure()));
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_a_modules(condition: impl Fn() -> bool + 'static,
    ret: impl Into<ReturnA_Modules>) {
    let inner: ReturnA_Modules = ret.into();
    let mock_id = context::MockId::new_fn("fns_a_modules");
    let cond =
        context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                    |input: &()|
                    {
                        if condition() {
                            Ok(())
                        } else {
                            Err("on_call condition failed for a_modules".into())
                        }
                    }));
    context::add_on_call::<()>(&mock_id, cond, inner.0, None).unwrap();
}

pub struct PredicateNested_Deep_fn(pub context::Predicate);

pub struct ReturnNested_Deep_fn(pub context::ReturnValDoublePointer);

impl ReturnNested_Deep_fn {
    pub fn from_fn(closure: impl Fn() -> &'static str + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(),
                    &'static str>(Box::new(move |()| closure())))
    }
}

impl PredicateNested_Deep_fn {
    pub fn from_fn(closure:
            impl Fn() -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id = context::MockId::new_fn("fns_a_nested_deep_fn");
        let cond =
            context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                        |input: &()| closure()));
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_nested_deep_fn(condition: impl Fn() -> bool + 'static,
    ret: impl Into<ReturnNested_Deep_fn>) {
    let inner: ReturnNested_Deep_fn = ret.into();
    let mock_id = context::MockId::new_fn("fns_a_nested_deep_fn");
    let cond =
        context::ConditionDoublePointer::from_fn::<()>(Box::new(move
                    |input: &()|
                    {
                        if condition() {
                            Ok(())
                        } else {
                            Err("on_call condition failed for nested_deep_fn".into())
                        }
                    }));
    context::add_on_call::<()>(&mock_id, cond, inner.0, None).unwrap();
}

pub struct PublicANestedInner {
    pub value: i32,
}

impl Into<a::nested::Inner> for PublicANestedInner {
    fn into(self) -> a::nested::Inner {
        a::nested::Inner { value: self.value }
    }
}

pub struct PredicateInnerNew(pub context::Predicate);

pub struct ReturnInnerNew(pub context::ReturnValDoublePointer);

impl ReturnInnerNew {
    pub fn from_fn(closure: impl Fn(i32) -> PublicANestedInner + 'static)
        -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(i32,),
                    PublicANestedInner>(Box::new(move |(_0,)| closure(_0))))
    }
}

impl PredicateInnerNew {
    pub fn from_fn(closure:
            impl Fn(&i32) -> context::errors::PredicateResult<()> + 'static)
        -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_a_nested_Inner", "new");
        let cond =
            context::ConditionDoublePointer::from_fn::<(i32,)>(Box::new(move
                        |input: &(i32,)| closure(&input.0)));
        Self(context::Predicate::create_single::<(i32,)>(&mock_id, cond))
    }
}

impl a::nested::Inner {
    pub fn on_call_new(condition: impl Fn(&i32) -> bool + 'static,
        ret: impl Into<ReturnInnerNew>) {
        let inner: ReturnInnerNew = ret.into();
        let mock_id =
            context::MockId::new_adt_static("fns_a_nested_Inner", "new");
        let object_mock_id =
            context::AdtId::new("fns_a_nested_Inner",
                context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(i32,)>(Box::new(move
                        |input: &(i32,)|
                        {
                            if condition(&input.0) {
                                Ok(())
                            } else { Err("on_call condition failed for new".into()) }
                        }));
        context::add_method_on_call::<(i32,)>(&object_mock_id, "new",
                &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateInnerDouble(pub context::Predicate);

pub struct ReturnInnerDouble(pub context::ReturnValDoublePointer);

impl ReturnInnerDouble {
    pub fn from_fn(closure: impl Fn(&a::nested::Inner) -> i32 + 'static)
        -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&a::nested::Inner,),
                    i32>(Box::new(move |(_self,)| closure(_self))))
    }
}

impl PredicateInnerDouble {
    pub fn from_fn(closure:
            impl Fn(&a::nested::Inner)
            -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_a_nested_Inner", "double");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&a::nested::Inner,)>(Box::new(move
                        |input: &(&a::nested::Inner,)| closure(input.0)));
        Self(context::Predicate::create_single::<(&a::nested::Inner,)>(&mock_id,
                cond))
    }
}

impl a::nested::Inner {
    pub fn on_call_double(condition:
            impl Fn(&a::nested::Inner) -> bool + 'static,
        ret: impl Into<ReturnInnerDouble>) {
        let inner: ReturnInnerDouble = ret.into();
        let mock_id =
            context::MockId::new_adt_static("fns_a_nested_Inner", "double");
        let object_mock_id =
            context::AdtId::new("fns_a_nested_Inner",
                context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(&a::nested::Inner,)>(Box::new(move
                        |input: &(&a::nested::Inner,)|
                        {
                            if condition(input.0) {
                                Ok(())
                            } else { Err("on_call condition failed for double".into()) }
                        }));
        context::add_method_on_call::<(&a::nested::Inner,)>(&object_mock_id,
                "double", &mock_id, cond, inner.0, None).unwrap();
    }
}

pub struct PredicateInnerTripple(pub context::Predicate);

pub struct ReturnInnerTripple(pub context::ReturnValDoublePointer);

impl ReturnInnerTripple {
    pub fn from_fn(closure: impl Fn(&a::nested::Inner) -> i32 + 'static)
        -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&a::nested::Inner,),
                    i32>(Box::new(move |(_self,)| closure(_self))))
    }
}

impl PredicateInnerTripple {
    pub fn from_fn(closure:
            impl Fn(&a::nested::Inner)
            -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id =
            context::MockId::new_adt_static("fns_a_nested_Inner", "tripple");
        let cond =
            context::ConditionDoublePointer::from_fn::<(&a::nested::Inner,)>(Box::new(move
                        |input: &(&a::nested::Inner,)| closure(input.0)));
        Self(context::Predicate::create_single::<(&a::nested::Inner,)>(&mock_id,
                cond))
    }
}

impl a::nested::Inner {
    pub fn on_call_tripple(condition:
            impl Fn(&a::nested::Inner) -> bool + 'static,
        ret: impl Into<ReturnInnerTripple>) {
        let inner: ReturnInnerTripple = ret.into();
        let mock_id =
            context::MockId::new_adt_static("fns_a_nested_Inner", "tripple");
        let object_mock_id =
            context::AdtId::new("fns_a_nested_Inner",
                context::AdtIdNumber::default());
        let cond =
            context::ConditionDoublePointer::from_fn::<(&a::nested::Inner,)>(Box::new(move
                        |input: &(&a::nested::Inner,)|
                        {
                            if condition(input.0) {
                                Ok(())
                            } else {
                                Err("on_call condition failed for tripple".into())
                            }
                        }));
        context::add_method_on_call::<(&a::nested::Inner,)>(&object_mock_id,
                "tripple", &mock_id, cond, inner.0, None).unwrap();
    }
}

