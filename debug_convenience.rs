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
        let mock_id = context::MockId::new_fn("fns_ref_param");
        let cond = context::ConditionDoublePointer::from_fn::<(&u32,)>(
            Box::new(move |input: &(&u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&u32,)>(&mock_id, cond))
    }
}

pub fn on_call_ref_param(condition: impl Fn(&&u32) -> bool + 'static, ret: impl Fn(&u32) -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_ref_param");
    let cond = context::ConditionDoublePointer::from_fn::<(&u32,)>(
        Box::new(move |input: &(&u32,)| {
            if condition(&input.0) {
                Ok(())
            } else {
                Err("on_call condition failed for ref_param".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&u32,), ()>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_on_call::<(&u32,)>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_ref_param(condition: impl Fn(&&u32) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(&u32) -> () + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_ref_param");
    let cond = context::ConditionDoublePointer::from_fn::<(&u32,)>(
        Box::new(move |input: &(&u32,)| condition(&input.0))
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&u32,), ()>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_expectation::<(&u32,), ()>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_ref_param(seq_name: &str, index: usize, condition: impl Fn(&&u32) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(&u32) -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_ref_param");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<(&u32,)>(
        Box::new(move |input: &(&u32,)| condition(&input.0))
    );
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
        let mock_id = context::MockId::new_fn("fns_cons_param");
        let cond = context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(
            Box::new(move |input: &(Box<u32>,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(Box<u32>,)>(&mock_id, cond))
    }
}

pub fn on_call_cons_param(condition: impl Fn(&Box<u32>) -> bool + 'static, ret: impl Fn(Box<u32>) -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_cons_param");
    let cond = context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(
        Box::new(move |input: &(Box<u32>,)| {
            if condition(&input.0) {
                Ok(())
            } else {
                Err("on_call condition failed for cons_param".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(Box<u32>,), ()>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_on_call::<(Box<u32>,)>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_cons_param(condition: impl Fn(&Box<u32>) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(Box<u32>) -> () + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_cons_param");
    let cond = context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(
        Box::new(move |input: &(Box<u32>,)| condition(&input.0))
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(Box<u32>,), ()>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_expectation::<(Box<u32>,), ()>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_cons_param(seq_name: &str, index: usize, condition: impl Fn(&Box<u32>) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(Box<u32>) -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_cons_param");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<(Box<u32>,)>(
        Box::new(move |input: &(Box<u32>,)| condition(&input.0))
    );
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
        let mock_id = context::MockId::new_fn("fns_foreign");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_foreign(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_foreign");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| {
            if condition() {
                Ok(())
            } else {
                Err("on_call condition failed for foreign".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), ()>(
        Box::new(move |()| ret())
    );
    context::add_on_call::<()>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_foreign(condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> () + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_foreign");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), ()>(
        Box::new(move |()| ret())
    );
    context::add_expectation::<(), ()>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_foreign(seq_name: &str, index: usize, condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_foreign");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
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
        let mock_id = context::MockId::new_fn("fns_ret_call_w_args");
        let cond = context::ConditionDoublePointer::from_fn::<(i16,)>(
            Box::new(move |input: &(i16,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(i16,)>(&mock_id, cond))
    }
}

pub fn on_call_ret_call_w_args(condition: impl Fn(&i16) -> bool + 'static, ret: impl Fn(i16) -> i16 + 'static) {
    let mock_id = context::MockId::new_fn("fns_ret_call_w_args");
    let cond = context::ConditionDoublePointer::from_fn::<(i16,)>(
        Box::new(move |input: &(i16,)| {
            if condition(&input.0) {
                Ok(())
            } else {
                Err("on_call condition failed for ret_call_w_args".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(i16,), i16>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_on_call::<(i16,)>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_ret_call_w_args(condition: impl Fn(&i16) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(i16) -> i16 + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_ret_call_w_args");
    let cond = context::ConditionDoublePointer::from_fn::<(i16,)>(
        Box::new(move |input: &(i16,)| condition(&input.0))
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(i16,), i16>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_expectation::<(i16,), i16>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_ret_call_w_args(seq_name: &str, index: usize, condition: impl Fn(&i16) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(i16) -> i16 + 'static) {
    let mock_id = context::MockId::new_fn("fns_ret_call_w_args");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<(i16,)>(
        Box::new(move |input: &(i16,)| condition(&input.0))
    );
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
        let mock_id = context::MockId::new_fn("fns_ret_param");
        let cond = context::ConditionDoublePointer::from_fn::<(&mut u32,)>(
            Box::new(move |input: &(&mut u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(&mut u32,)>(&mock_id, cond))
    }
}

pub fn on_call_ret_param(condition: impl Fn(&&mut u32) -> bool + 'static, ret: impl Fn(&mut u32) -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_ret_param");
    let cond = context::ConditionDoublePointer::from_fn::<(&mut u32,)>(
        Box::new(move |input: &(&mut u32,)| {
            if condition(&input.0) {
                Ok(())
            } else {
                Err("on_call condition failed for ret_param".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&mut u32,), ()>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_on_call::<(&mut u32,)>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_ret_param(condition: impl Fn(&&mut u32) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(&mut u32) -> () + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_ret_param");
    let cond = context::ConditionDoublePointer::from_fn::<(&mut u32,)>(
        Box::new(move |input: &(&mut u32,)| condition(&input.0))
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&mut u32,), ()>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_expectation::<(&mut u32,), ()>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_ret_param(seq_name: &str, index: usize, condition: impl Fn(&&mut u32) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(&mut u32) -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_ret_param");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<(&mut u32,)>(
        Box::new(move |input: &(&mut u32,)| condition(&input.0))
    );
    context::add_expectation_to_sequence::<(&mut u32,), ()>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
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
        let mock_id = context::MockId::new_fn("fns_return_panic");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_return_panic(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_return_panic");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| {
            if condition() {
                Ok(())
            } else {
                Err("on_call condition failed for return_panic".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), ()>(
        Box::new(move |()| ret())
    );
    context::add_on_call::<()>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_return_panic(condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> () + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_return_panic");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), ()>(
        Box::new(move |()| ret())
    );
    context::add_expectation::<(), ()>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_return_panic(seq_name: &str, index: usize, condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> () + 'static) {
    let mock_id = context::MockId::new_fn("fns_return_panic");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
    context::add_expectation_to_sequence::<(), ()>(
        &mock_id, cond, Some(Box::new(move |()| ret())),
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
        let mock_id = context::MockId::new_fn("fns_closure_param");
        let cond = context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(
            Box::new(move |input: &(ClosureWrapper,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(ClosureWrapper,)>(&mock_id, cond))
    }
}

pub fn on_call_closure_param(condition: impl Fn(&ClosureWrapper) -> bool + 'static, ret: impl Fn(ClosureWrapper) -> u32 + 'static) {
    let mock_id = context::MockId::new_fn("fns_closure_param");
    let cond = context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(
        Box::new(move |input: &(ClosureWrapper,)| {
            if condition(&input.0) {
                Ok(())
            } else {
                Err("on_call condition failed for closure_param".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(ClosureWrapper,), u32>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_on_call::<(ClosureWrapper,)>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_closure_param(condition: impl Fn(&ClosureWrapper) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(ClosureWrapper) -> u32 + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_closure_param");
    let cond = context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(
        Box::new(move |input: &(ClosureWrapper,)| condition(&input.0))
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(ClosureWrapper,), u32>(
        Box::new(move |(_0,)| ret(_0))
    );
    context::add_expectation::<(ClosureWrapper,), u32>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_closure_param(seq_name: &str, index: usize, condition: impl Fn(&ClosureWrapper) -> context::errors::PredicateResult<()> + 'static, ret: impl Fn(ClosureWrapper) -> u32 + 'static) {
    let mock_id = context::MockId::new_fn("fns_closure_param");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<(ClosureWrapper,)>(
        Box::new(move |input: &(ClosureWrapper,)| condition(&input.0))
    );
    context::add_expectation_to_sequence::<(ClosureWrapper,), u32>(
        &mock_id, cond, Some(Box::new(move |(_0,)| ret(_0))),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMacro_generated_add(pub context::Predicate);
pub struct ReturnMacro_generated_add(pub context::ReturnValDoublePointer);

impl ReturnMacro_generated_add {
    pub fn from_fn(closure: impl Fn() -> u32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), u32>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateMacro_generated_add {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_fn("fns_macro_generated_add");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_macro_generated_add(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> u32 + 'static) {
    let mock_id = context::MockId::new_fn("fns_macro_generated_add");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| {
            if condition() {
                Ok(())
            } else {
                Err("on_call condition failed for macro_generated_add".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), u32>(
        Box::new(move |()| ret())
    );
    context::add_on_call::<()>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_macro_generated_add(condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> u32 + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_macro_generated_add");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), u32>(
        Box::new(move |()| ret())
    );
    context::add_expectation::<(), u32>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_macro_generated_add(seq_name: &str, index: usize, condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> u32 + 'static) {
    let mock_id = context::MockId::new_fn("fns_macro_generated_add");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
    context::add_expectation_to_sequence::<(), u32>(
        &mock_id, cond, Some(Box::new(move |()| ret())),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMacro_generated_greeting(pub context::Predicate);
pub struct ReturnMacro_generated_greeting(pub context::ReturnValDoublePointer);

impl ReturnMacro_generated_greeting {
    pub fn from_fn(closure: impl Fn() -> &'static str + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), &'static str>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateMacro_generated_greeting {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_fn("fns_macro_generated_greeting");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_macro_generated_greeting(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> &'static str + 'static) {
    let mock_id = context::MockId::new_fn("fns_macro_generated_greeting");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| {
            if condition() {
                Ok(())
            } else {
                Err("on_call condition failed for macro_generated_greeting".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), &'static str>(
        Box::new(move |()| ret())
    );
    context::add_on_call::<()>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_macro_generated_greeting(condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> &'static str + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_macro_generated_greeting");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), &'static str>(
        Box::new(move |()| ret())
    );
    context::add_expectation::<(), &'static str>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_macro_generated_greeting(seq_name: &str, index: usize, condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> &'static str + 'static) {
    let mock_id = context::MockId::new_fn("fns_macro_generated_greeting");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
    context::add_expectation_to_sequence::<(), &'static str>(
        &mock_id, cond, Some(Box::new(move |()| ret())),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateMain(pub context::Predicate);
pub struct ReturnMain(pub context::ReturnValDoublePointer);

impl ReturnMain {
    pub fn from_fn(closure: impl Fn() -> test::ExitCode + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), test::ExitCode>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateMain {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_fn("fns_main");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_main(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> test::ExitCode + 'static) {
    let mock_id = context::MockId::new_fn("fns_main");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| {
            if condition() {
                Ok(())
            } else {
                Err("on_call condition failed for main".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), test::ExitCode>(
        Box::new(move |()| ret())
    );
    context::add_on_call::<()>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub fn expect_main(condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> test::ExitCode + 'static, modifier: context::TimesModifier, checkpoint: Option<&str>) {
    let mock_id = context::MockId::new_fn("fns_main");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), test::ExitCode>(
        Box::new(move |()| ret())
    );
    context::add_expectation::<(), test::ExitCode>(
        &mock_id, cond, Some(ret_ptr),
        checkpoint.map(|s| context::CheckpointName(s.to_string())),
        modifier,
    ).unwrap();
}

pub fn sequence_main(seq_name: &str, index: usize, condition: impl Fn() -> context::errors::PredicateResult<()> + 'static, ret: impl Fn() -> test::ExitCode + 'static) {
    let mock_id = context::MockId::new_fn("fns_main");
    let _ = context::register_mock(&mock_id);
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| condition())
    );
    context::add_expectation_to_sequence::<(), test::ExitCode>(
        &mock_id, cond, Some(Box::new(move |()| ret())),
        seq_name, index, None,
    ).unwrap();
}

pub struct PredicateConsSelfStructFmt(pub context::Predicate);
pub struct ReturnConsSelfStructFmt(pub context::ReturnValDoublePointer);

impl ReturnConsSelfStructFmt {
    pub fn from_fn(closure: impl Fn(&ConsSelfStruct, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&ConsSelfStruct, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| closure(_self, _0))
        ))
    }
}

impl PredicateConsSelfStructFmt {
    pub fn from_fn(closure: impl Fn(&ConsSelfStruct, &&mut ::core::fmt::Formatter) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_ConsSelfStruct", "fmt");
        let cond = context::ConditionDoublePointer::from_fn::<(&ConsSelfStruct, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&ConsSelfStruct, &mut ::core::fmt::Formatter)| closure(input.0, &input.1))
        );
        Self(context::Predicate::create_single::<(&ConsSelfStruct, &mut ::core::fmt::Formatter)>(&mock_id, cond))
    }
}

impl ConsSelfStruct {
    pub fn on_call_fmt(condition: impl Fn(&ConsSelfStruct, &&mut ::core::fmt::Formatter) -> bool + 'static, ret: impl Fn(&ConsSelfStruct, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_ConsSelfStruct", "fmt");
        let object_mock_id = context::AdtId::new("fns_ConsSelfStruct", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&ConsSelfStruct, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&ConsSelfStruct, &mut ::core::fmt::Formatter)| {
                if condition(input.0, &input.1) {
                    Ok(())
                } else {
                    Err("on_call condition failed for fmt".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&ConsSelfStruct, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| ret(_self, _0))
        );
        context::add_method_on_call::<(&ConsSelfStruct, &mut ::core::fmt::Formatter)>(
            &object_mock_id,
            "fmt",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
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
        let mock_id = context::MockId::new_adt_static("fns_ConsSelfStruct", "consume_self");
        let cond = context::ConditionDoublePointer::from_fn::<(ConsSelfStruct,)>(
            Box::new(move |input: &(ConsSelfStruct,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(ConsSelfStruct,)>(&mock_id, cond))
    }
}

impl ConsSelfStruct {
    pub fn on_call_consume_self(condition: impl Fn(&ConsSelfStruct) -> bool + 'static, ret: impl Fn(ConsSelfStruct) -> () + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_ConsSelfStruct", "consume_self");
        let object_mock_id = context::AdtId::new("fns_ConsSelfStruct", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(ConsSelfStruct,)>(
            Box::new(move |input: &(ConsSelfStruct,)| {
                if condition(&input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for consume_self".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(ConsSelfStruct,), ()>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(ConsSelfStruct,)>(
            &object_mock_id,
            "consume_self",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PredicateMockStructFmt(pub context::Predicate);
pub struct ReturnMockStructFmt(pub context::ReturnValDoublePointer);

impl ReturnMockStructFmt {
    pub fn from_fn(closure: impl Fn(&MockStruct, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&MockStruct, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| closure(_self, _0))
        ))
    }
}

impl PredicateMockStructFmt {
    pub fn from_fn(closure: impl Fn(&MockStruct, &&mut ::core::fmt::Formatter) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_MockStruct", "fmt");
        let cond = context::ConditionDoublePointer::from_fn::<(&MockStruct, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&MockStruct, &mut ::core::fmt::Formatter)| closure(input.0, &input.1))
        );
        Self(context::Predicate::create_single::<(&MockStruct, &mut ::core::fmt::Formatter)>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_fmt(&self, condition: impl Fn(&MockStruct, &&mut ::core::fmt::Formatter) -> bool + 'static, ret: impl Fn(&MockStruct, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) {
        let mock_id = context::MockId::new_adt_instance("fns_MockStruct", "fmt", self.adt_mock_id);
        let object_mock_id = context::AdtId::new("fns_MockStruct", self.adt_mock_id);
        let cond = context::ConditionDoublePointer::from_fn::<(&MockStruct, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&MockStruct, &mut ::core::fmt::Formatter)| {
                if condition(input.0, &input.1) {
                    Ok(())
                } else {
                    Err("on_call condition failed for fmt".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&MockStruct, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| ret(_self, _0))
        );
        context::add_method_on_call::<(&MockStruct, &mut ::core::fmt::Formatter)>(
            &object_mock_id,
            "fmt",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
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

pub struct PredicateMockStructDefault(pub context::Predicate);
pub struct ReturnMockStructDefault(pub context::ReturnValDoublePointer);

impl ReturnMockStructDefault {
    pub fn from_fn(closure: impl Fn() -> PublicMockStruct + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), PublicMockStruct>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateMockStructDefault {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_MockStruct", "default");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_default(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> PublicMockStruct + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_MockStruct", "default");
        let object_mock_id = context::AdtId::new("fns_MockStruct", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| {
                if condition() {
                    Ok(())
                } else {
                    Err("on_call condition failed for default".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), PublicMockStruct>(
            Box::new(move |()| ret())
        );
        context::add_method_on_call::<()>(
            &object_mock_id,
            "default",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
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
    pub fn from_fn(closure: impl Fn(u32) -> PublicMockStruct + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(u32,), PublicMockStruct>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateMockStructNew {
    pub fn from_fn(closure: impl Fn(&u32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_MockStruct", "new");
        let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(
            Box::new(move |input: &(u32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(u32,)>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_new(condition: impl Fn(&u32) -> bool + 'static, ret: impl Fn(u32) -> PublicMockStruct + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_MockStruct", "new");
        let object_mock_id = context::AdtId::new("fns_MockStruct", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(u32,)>(
            Box::new(move |input: &(u32,)| {
                if condition(&input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for new".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(u32,), PublicMockStruct>(
            Box::new(move |(_0,)| ret(_0))
        );
        context::add_method_on_call::<(u32,)>(
            &object_mock_id,
            "new",
            &mock_id,
            cond,
            ret_ptr,
            None,
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
        let mock_id = context::MockId::new_adt_static("fns_MockStruct", "foo");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_foo(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> () + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_MockStruct", "foo");
        let object_mock_id = context::AdtId::new("fns_MockStruct", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| {
                if condition() {
                    Ok(())
                } else {
                    Err("on_call condition failed for foo".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), ()>(
            Box::new(move |()| ret())
        );
        context::add_method_on_call::<()>(
            &object_mock_id,
            "foo",
            &mock_id,
            cond,
            ret_ptr,
            None,
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
    pub fn from_fn(closure: impl Fn(&MockStruct) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_MockStruct", "get_value");
        let cond = context::ConditionDoublePointer::from_fn::<(&MockStruct,)>(
            Box::new(move |input: &(&MockStruct,)| closure(input.0))
        );
        Self(context::Predicate::create_single::<(&MockStruct,)>(&mock_id, cond))
    }
}

impl MockStruct {
    pub fn on_call_get_value(&self, condition: impl Fn(&MockStruct) -> bool + 'static, ret: impl Fn(&MockStruct) -> u32 + 'static) {
        let mock_id = context::MockId::new_adt_instance("fns_MockStruct", "get_value", self.adt_mock_id);
        let object_mock_id = context::AdtId::new("fns_MockStruct", self.adt_mock_id);
        let cond = context::ConditionDoublePointer::from_fn::<(&MockStruct,)>(
            Box::new(move |input: &(&MockStruct,)| {
                if condition(input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for get_value".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&MockStruct,), u32>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(&MockStruct,)>(
            &object_mock_id,
            "get_value",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PredicateFooEq(pub context::Predicate);
pub struct ReturnFooEq(pub context::ReturnValDoublePointer);

impl ReturnFooEq {
    pub fn from_fn(closure: impl Fn(&Foo, &Self) -> bool + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo, &Self), bool>(
            Box::new(move |(_self, _0,)| closure(_self, _0))
        ))
    }
}

impl PredicateFooEq {
    pub fn from_fn(closure: impl Fn(&Foo, &&Self) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "eq");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo, &Self)>(
            Box::new(move |input: &(&Foo, &Self)| closure(input.0, &input.1))
        );
        Self(context::Predicate::create_single::<(&Foo, &Self)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_eq(condition: impl Fn(&Foo, &&Self) -> bool + 'static, ret: impl Fn(&Foo, &Self) -> bool + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "eq");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo, &Self)>(
            Box::new(move |input: &(&Foo, &Self)| {
                if condition(input.0, &input.1) {
                    Ok(())
                } else {
                    Err("on_call condition failed for eq".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&Foo, &Self), bool>(
            Box::new(move |(_self, _0,)| ret(_self, _0))
        );
        context::add_method_on_call::<(&Foo, &Self)>(
            &object_mock_id,
            "eq",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PredicateFooFmt(pub context::Predicate);
pub struct ReturnFooFmt(pub context::ReturnValDoublePointer);

impl ReturnFooFmt {
    pub fn from_fn(closure: impl Fn(&Foo, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Foo, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| closure(_self, _0))
        ))
    }
}

impl PredicateFooFmt {
    pub fn from_fn(closure: impl Fn(&Foo, &&mut ::core::fmt::Formatter) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "fmt");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&Foo, &mut ::core::fmt::Formatter)| closure(input.0, &input.1))
        );
        Self(context::Predicate::create_single::<(&Foo, &mut ::core::fmt::Formatter)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_fmt(condition: impl Fn(&Foo, &&mut ::core::fmt::Formatter) -> bool + 'static, ret: impl Fn(&Foo, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "fmt");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&Foo, &mut ::core::fmt::Formatter)| {
                if condition(input.0, &input.1) {
                    Ok(())
                } else {
                    Err("on_call condition failed for fmt".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&Foo, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| ret(_self, _0))
        );
        context::add_method_on_call::<(&Foo, &mut ::core::fmt::Formatter)>(
            &object_mock_id,
            "fmt",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PublicFoo {
    pub x: u32,
}

impl Into<Foo> for PublicFoo {
    fn into(self) -> Foo {
        Foo {
            x: self.x,
        }
    }
}

pub struct PredicateFooDefault(pub context::Predicate);
pub struct ReturnFooDefault(pub context::ReturnValDoublePointer);

impl ReturnFooDefault {
    pub fn from_fn(closure: impl Fn() -> PublicFoo + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), PublicFoo>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateFooDefault {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "default");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_default(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> PublicFoo + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "default");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| {
                if condition() {
                    Ok(())
                } else {
                    Err("on_call condition failed for default".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), PublicFoo>(
            Box::new(move |()| ret())
        );
        context::add_method_on_call::<()>(
            &object_mock_id,
            "default",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PublicFoo {
    pub x: u32,
}

impl Into<Foo> for PublicFoo {
    fn into(self) -> Foo {
        Foo {
            x: self.x,
        }
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
    pub fn from_fn(closure: impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_ref");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| closure(input.0))
        );
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_ret_ref(condition: impl Fn(&Foo) -> bool + 'static, ret: impl Fn(&Foo) -> &u32 + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_ref");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| {
                if condition(input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for ret_ref".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&Foo,), &u32>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(&Foo,)>(
            &object_mock_id,
            "ret_ref",
            &mock_id,
            cond,
            ret_ptr,
            None,
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
    pub fn from_fn(closure: impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_mut_ref");
        let cond = context::ConditionDoublePointer::from_fn::<(&mut Foo,)>(
            Box::new(move |input: &(&mut Foo,)| closure(&*input.0))
        );
        Self(context::Predicate::create_single::<(&mut Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_ret_mut_ref(condition: impl Fn(&Foo) -> bool + 'static, ret: impl Fn(&mut Foo) -> &mut u32 + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_mut_ref");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&mut Foo,)>(
            Box::new(move |input: &(&mut Foo,)| {
                if condition(&*input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for ret_mut_ref".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&mut Foo,), &mut u32>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(&mut Foo,)>(
            &object_mock_id,
            "ret_mut_ref",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PredicateFooRet_owned(pub context::Predicate);
pub struct ReturnFooRet_owned(pub context::ReturnValDoublePointer);

impl ReturnFooRet_owned {
    pub fn from_fn(closure: impl Fn() -> PublicFoo + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), PublicFoo>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateFooRet_owned {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_owned");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_ret_owned(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> PublicFoo + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "ret_owned");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| {
                if condition() {
                    Ok(())
                } else {
                    Err("on_call condition failed for ret_owned".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), PublicFoo>(
            Box::new(move |()| ret())
        );
        context::add_method_on_call::<()>(
            &object_mock_id,
            "ret_owned",
            &mock_id,
            cond,
            ret_ptr,
            None,
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
        let mock_id = context::MockId::new_adt_static("fns_Foo", "static_method");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_static_method(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> () + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "static_method");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| {
                if condition() {
                    Ok(())
                } else {
                    Err("on_call condition failed for static_method".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), ()>(
            Box::new(move |()| ret())
        );
        context::add_method_on_call::<()>(
            &object_mock_id,
            "static_method",
            &mock_id,
            cond,
            ret_ptr,
            None,
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
    pub fn from_fn(closure: impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "fallback");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| closure(input.0))
        );
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_fallback(condition: impl Fn(&Foo) -> bool + 'static, ret: impl Fn(&Foo) -> u32 + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "fallback");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| {
                if condition(input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for fallback".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&Foo,), u32>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(&Foo,)>(
            &object_mock_id,
            "fallback",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PredicateNoDefaultStructFmt(pub context::Predicate);
pub struct ReturnNoDefaultStructFmt(pub context::ReturnValDoublePointer);

impl ReturnNoDefaultStructFmt {
    pub fn from_fn(closure: impl Fn(&NoDefaultStruct, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&NoDefaultStruct, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| closure(_self, _0))
        ))
    }
}

impl PredicateNoDefaultStructFmt {
    pub fn from_fn(closure: impl Fn(&NoDefaultStruct, &&mut ::core::fmt::Formatter) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_NoDefaultStruct", "fmt");
        let cond = context::ConditionDoublePointer::from_fn::<(&NoDefaultStruct, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&NoDefaultStruct, &mut ::core::fmt::Formatter)| closure(input.0, &input.1))
        );
        Self(context::Predicate::create_single::<(&NoDefaultStruct, &mut ::core::fmt::Formatter)>(&mock_id, cond))
    }
}

impl NoDefaultStruct {
    pub fn on_call_fmt(condition: impl Fn(&NoDefaultStruct, &&mut ::core::fmt::Formatter) -> bool + 'static, ret: impl Fn(&NoDefaultStruct, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_NoDefaultStruct", "fmt");
        let object_mock_id = context::AdtId::new("fns_NoDefaultStruct", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&NoDefaultStruct, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&NoDefaultStruct, &mut ::core::fmt::Formatter)| {
                if condition(input.0, &input.1) {
                    Ok(())
                } else {
                    Err("on_call condition failed for fmt".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&NoDefaultStruct, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| ret(_self, _0))
        );
        context::add_method_on_call::<(&NoDefaultStruct, &mut ::core::fmt::Formatter)>(
            &object_mock_id,
            "fmt",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PublicNoDefaultStruct {
    pub value: String,
}

impl Into<NoDefaultStruct> for PublicNoDefaultStruct {
    fn into(self) -> NoDefaultStruct {
        NoDefaultStruct {
            value: self.value,
        }
    }
}

pub struct PredicateNoDefaultStructNew(pub context::Predicate);
pub struct ReturnNoDefaultStructNew(pub context::ReturnValDoublePointer);

impl ReturnNoDefaultStructNew {
    pub fn from_fn(closure: impl Fn(String) -> PublicNoDefaultStruct + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(String,), PublicNoDefaultStruct>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateNoDefaultStructNew {
    pub fn from_fn(closure: impl Fn(&String) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_NoDefaultStruct", "new");
        let cond = context::ConditionDoublePointer::from_fn::<(String,)>(
            Box::new(move |input: &(String,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(String,)>(&mock_id, cond))
    }
}

impl NoDefaultStruct {
    pub fn on_call_new(condition: impl Fn(&String) -> bool + 'static, ret: impl Fn(String) -> PublicNoDefaultStruct + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_NoDefaultStruct", "new");
        let object_mock_id = context::AdtId::new("fns_NoDefaultStruct", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(String,)>(
            Box::new(move |input: &(String,)| {
                if condition(&input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for new".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(String,), PublicNoDefaultStruct>(
            Box::new(move |(_0,)| ret(_0))
        );
        context::add_method_on_call::<(String,)>(
            &object_mock_id,
            "new",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PredicateNoDefaultStructGet_value(pub context::Predicate);
pub struct ReturnNoDefaultStructGet_value(pub context::ReturnValDoublePointer);

impl ReturnNoDefaultStructGet_value {
    pub fn from_fn(closure: impl Fn(&NoDefaultStruct) -> &str + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&NoDefaultStruct,), &str>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateNoDefaultStructGet_value {
    pub fn from_fn(closure: impl Fn(&NoDefaultStruct) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_NoDefaultStruct", "get_value");
        let cond = context::ConditionDoublePointer::from_fn::<(&NoDefaultStruct,)>(
            Box::new(move |input: &(&NoDefaultStruct,)| closure(input.0))
        );
        Self(context::Predicate::create_single::<(&NoDefaultStruct,)>(&mock_id, cond))
    }
}

impl NoDefaultStruct {
    pub fn on_call_get_value(condition: impl Fn(&NoDefaultStruct) -> bool + 'static, ret: impl Fn(&NoDefaultStruct) -> &str + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_NoDefaultStruct", "get_value");
        let object_mock_id = context::AdtId::new("fns_NoDefaultStruct", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&NoDefaultStruct,)>(
            Box::new(move |input: &(&NoDefaultStruct,)| {
                if condition(input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for get_value".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&NoDefaultStruct,), &str>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(&NoDefaultStruct,)>(
            &object_mock_id,
            "get_value",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PredicatePatternFmt(pub context::Predicate);
pub struct ReturnPatternFmt(pub context::ReturnValDoublePointer);

impl ReturnPatternFmt {
    pub fn from_fn(closure: impl Fn(&Pattern, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&Pattern, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| closure(_self, _0))
        ))
    }
}

impl PredicatePatternFmt {
    pub fn from_fn(closure: impl Fn(&Pattern, &&mut ::core::fmt::Formatter) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Pattern", "fmt");
        let cond = context::ConditionDoublePointer::from_fn::<(&Pattern, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&Pattern, &mut ::core::fmt::Formatter)| closure(input.0, &input.1))
        );
        Self(context::Predicate::create_single::<(&Pattern, &mut ::core::fmt::Formatter)>(&mock_id, cond))
    }
}

impl Pattern {
    pub fn on_call_fmt(condition: impl Fn(&Pattern, &&mut ::core::fmt::Formatter) -> bool + 'static, ret: impl Fn(&Pattern, &mut ::core::fmt::Formatter) -> ::core::fmt::Result + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Pattern", "fmt");
        let object_mock_id = context::AdtId::new("fns_Pattern", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&Pattern, &mut ::core::fmt::Formatter)>(
            Box::new(move |input: &(&Pattern, &mut ::core::fmt::Formatter)| {
                if condition(input.0, &input.1) {
                    Ok(())
                } else {
                    Err("on_call condition failed for fmt".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&Pattern, &mut ::core::fmt::Formatter), ::core::fmt::Result>(
            Box::new(move |(_self, _0,)| ret(_self, _0))
        );
        context::add_method_on_call::<(&Pattern, &mut ::core::fmt::Formatter)>(
            &object_mock_id,
            "fmt",
            &mock_id,
            cond,
            ret_ptr,
            None,
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
    pub fn from_fn(closure: impl Fn(&ClosureWrapper, &&mut std::fmt::Formatter<'_>) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_ClosureWrapper", "fmt");
        let cond = context::ConditionDoublePointer::from_fn::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>)>(
            Box::new(move |input: &(&ClosureWrapper, &mut std::fmt::Formatter<'_>)| closure(input.0, &input.1))
        );
        Self(context::Predicate::create_single::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>)>(&mock_id, cond))
    }
}

impl ClosureWrapper {
    pub fn on_call_fmt(condition: impl Fn(&ClosureWrapper, &&mut std::fmt::Formatter<'_>) -> bool + 'static, ret: impl Fn(&ClosureWrapper, &mut std::fmt::Formatter<'_>) -> std::fmt::Result + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_ClosureWrapper", "fmt");
        let object_mock_id = context::AdtId::new("fns_ClosureWrapper", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>)>(
            Box::new(move |input: &(&ClosureWrapper, &mut std::fmt::Formatter<'_>)| {
                if condition(input.0, &input.1) {
                    Ok(())
                } else {
                    Err("on_call condition failed for fmt".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>), std::fmt::Result>(
            Box::new(move |(_self, _0,)| ret(_self, _0))
        );
        context::add_method_on_call::<(&ClosureWrapper, &mut std::fmt::Formatter<'_>)>(
            &object_mock_id,
            "fmt",
            &mock_id,
            cond,
            ret_ptr,
            None,
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
    pub fn from_fn(closure: impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "secret");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| closure(input.0))
        );
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_secret(condition: impl Fn(&Foo) -> bool + 'static, ret: impl Fn(&Foo) -> u32 + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "secret");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| {
                if condition(input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for secret".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&Foo,), u32>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(&Foo,)>(
            &object_mock_id,
            "secret",
            &mock_id,
            cond,
            ret_ptr,
            None,
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
    pub fn from_fn(closure: impl Fn(&Foo) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "compute");
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| closure(input.0))
        );
        Self(context::Predicate::create_single::<(&Foo,)>(&mock_id, cond))
    }
}

impl Foo {
    pub fn on_call_compute(condition: impl Fn(&Foo) -> bool + 'static, ret: impl Fn(&Foo) -> u32 + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_Foo", "compute");
        let object_mock_id = context::AdtId::new("fns_Foo", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&Foo,)>(
            Box::new(move |input: &(&Foo,)| {
                if condition(input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for compute".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&Foo,), u32>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(&Foo,)>(
            &object_mock_id,
            "compute",
            &mock_id,
            cond,
            ret_ptr,
            None,
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
        let mock_id = context::MockId::new_fn("fns_a_modules");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_a_modules(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> u32 + 'static) {
    let mock_id = context::MockId::new_fn("fns_a_modules");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| {
            if condition() {
                Ok(())
            } else {
                Err("on_call condition failed for a_modules".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), u32>(
        Box::new(move |()| ret())
    );
    context::add_on_call::<()>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub struct PredicateNested_Deep_fn(pub context::Predicate);
pub struct ReturnNested_Deep_fn(pub context::ReturnValDoublePointer);

impl ReturnNested_Deep_fn {
    pub fn from_fn(closure: impl Fn() -> &'static str + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(), &'static str>(
            Box::new(move |()| closure())
        ))
    }
}

impl PredicateNested_Deep_fn {
    pub fn from_fn(closure: impl Fn() -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_fn("fns_a_nested_deep_fn");
        let cond = context::ConditionDoublePointer::from_fn::<()>(
            Box::new(move |input: &()| closure())
        );
        Self(context::Predicate::create_single::<()>(&mock_id, cond))
    }
}

pub fn on_call_nested_deep_fn(condition: impl Fn() -> bool + 'static, ret: impl Fn() -> &'static str + 'static) {
    let mock_id = context::MockId::new_fn("fns_a_nested_deep_fn");
    let cond = context::ConditionDoublePointer::from_fn::<()>(
        Box::new(move |input: &()| {
            if condition() {
                Ok(())
            } else {
                Err("on_call condition failed for nested_deep_fn".into())
            }
        })
    );
    let ret_ptr = context::ReturnValDoublePointer::from_fn::<(), &'static str>(
        Box::new(move |()| ret())
    );
    context::add_on_call::<()>(
        &mock_id,
        cond,
        ret_ptr,
        None,
    ).unwrap();
}

pub struct PublicANestedInner {
    pub value: i32,
}

impl Into<a::nested::Inner> for PublicANestedInner {
    fn into(self) -> a::nested::Inner {
        a::nested::Inner {
            value: self.value,
        }
    }
}

pub struct PredicateInnerNew(pub context::Predicate);
pub struct ReturnInnerNew(pub context::ReturnValDoublePointer);

impl ReturnInnerNew {
    pub fn from_fn(closure: impl Fn(i32) -> PublicANestedInner + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(i32,), PublicANestedInner>(
            Box::new(move |(_0,)| closure(_0))
        ))
    }
}

impl PredicateInnerNew {
    pub fn from_fn(closure: impl Fn(&i32) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_a_nested_Inner", "new");
        let cond = context::ConditionDoublePointer::from_fn::<(i32,)>(
            Box::new(move |input: &(i32,)| closure(&input.0))
        );
        Self(context::Predicate::create_single::<(i32,)>(&mock_id, cond))
    }
}

impl a::nested::Inner {
    pub fn on_call_new(condition: impl Fn(&i32) -> bool + 'static, ret: impl Fn(i32) -> PublicANestedInner + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_a_nested_Inner", "new");
        let object_mock_id = context::AdtId::new("fns_a_nested_Inner", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(i32,)>(
            Box::new(move |input: &(i32,)| {
                if condition(&input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for new".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(i32,), PublicANestedInner>(
            Box::new(move |(_0,)| ret(_0))
        );
        context::add_method_on_call::<(i32,)>(
            &object_mock_id,
            "new",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PredicateInnerDouble(pub context::Predicate);
pub struct ReturnInnerDouble(pub context::ReturnValDoublePointer);

impl ReturnInnerDouble {
    pub fn from_fn(closure: impl Fn(&a::nested::Inner) -> i32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&a::nested::Inner,), i32>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateInnerDouble {
    pub fn from_fn(closure: impl Fn(&a::nested::Inner) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_a_nested_Inner", "double");
        let cond = context::ConditionDoublePointer::from_fn::<(&a::nested::Inner,)>(
            Box::new(move |input: &(&a::nested::Inner,)| closure(input.0))
        );
        Self(context::Predicate::create_single::<(&a::nested::Inner,)>(&mock_id, cond))
    }
}

impl a::nested::Inner {
    pub fn on_call_double(condition: impl Fn(&a::nested::Inner) -> bool + 'static, ret: impl Fn(&a::nested::Inner) -> i32 + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_a_nested_Inner", "double");
        let object_mock_id = context::AdtId::new("fns_a_nested_Inner", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&a::nested::Inner,)>(
            Box::new(move |input: &(&a::nested::Inner,)| {
                if condition(input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for double".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&a::nested::Inner,), i32>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(&a::nested::Inner,)>(
            &object_mock_id,
            "double",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

pub struct PredicateInnerTripple(pub context::Predicate);
pub struct ReturnInnerTripple(pub context::ReturnValDoublePointer);

impl ReturnInnerTripple {
    pub fn from_fn(closure: impl Fn(&a::nested::Inner) -> i32 + 'static) -> Self {
        Self(context::ReturnValDoublePointer::from_fn::<(&a::nested::Inner,), i32>(
            Box::new(move |(_self,)| closure(_self))
        ))
    }
}

impl PredicateInnerTripple {
    pub fn from_fn(closure: impl Fn(&a::nested::Inner) -> context::errors::PredicateResult<()> + 'static) -> Self {
        let mock_id = context::MockId::new_adt_static("fns_a_nested_Inner", "tripple");
        let cond = context::ConditionDoublePointer::from_fn::<(&a::nested::Inner,)>(
            Box::new(move |input: &(&a::nested::Inner,)| closure(input.0))
        );
        Self(context::Predicate::create_single::<(&a::nested::Inner,)>(&mock_id, cond))
    }
}

impl a::nested::Inner {
    pub fn on_call_tripple(condition: impl Fn(&a::nested::Inner) -> bool + 'static, ret: impl Fn(&a::nested::Inner) -> i32 + 'static) {
        let mock_id = context::MockId::new_adt_static("fns_a_nested_Inner", "tripple");
        let object_mock_id = context::AdtId::new("fns_a_nested_Inner", context::AdtIdNumber::default());
        let cond = context::ConditionDoublePointer::from_fn::<(&a::nested::Inner,)>(
            Box::new(move |input: &(&a::nested::Inner,)| {
                if condition(input.0) {
                    Ok(())
                } else {
                    Err("on_call condition failed for tripple".into())
                }
            })
        );
        let ret_ptr = context::ReturnValDoublePointer::from_fn::<(&a::nested::Inner,), i32>(
            Box::new(move |(_self,)| ret(_self))
        );
        context::add_method_on_call::<(&a::nested::Inner,)>(
            &object_mock_id,
            "tripple",
            &mock_id,
            cond,
            ret_ptr,
            None,
        ).unwrap();
    }
}

