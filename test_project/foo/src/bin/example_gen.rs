use std::marker::PhantomData;

use context::{TimesModifier, new_expectations::SingleExpectation};

use crate::Mod::Example;

// in crate named krate
mod Mod {
    /*pub Struct Example {
        a : f32,
        b : pub f32
    }

    pub trait ExTrait {
        pub fn meth2(&mut self, text : String) -> bool {

        }
    }
    impl ExTrait for Example {
        ..
    }
    //original impl
    impl Example {
        fn meth1(&self, a: f32, b : f32) -> usize;
        fn new(a: f32, b: f32) -> Self;
    }
    impl From<(f32,f32)> for Example {
        ...
    }*/

    pub trait Mockable {
        fn add_on_call<Input, Return>(expr: impl Fn(Input) -> Return);

        fn create_expectation<Input>(expr: impl Fn(Input) -> context::Result<()>) -> Expectation;
        fn add_expectation<Input, Ret>(
            expr: Expectation,
            ret: Option<Ret>,
            time_modifier: Option<context::TimesModifier>,
        );
    }

    use std::marker::PhantomData;

    use context::{ConditionDoublePointer, Expectation};

    //should generate something like
    // Example is given an id field, all private fields are made into PhantomData
    pub struct Example {
        a: PhantomData<f32>,
        pub b: f32,
        adt_mock_id: context::AdtMockId,
    }

    /*impl Mockable for Example {
            /// can only be called once
            fn add_on_call<Return>(expr : Fn);

            fn create_expectation<Input, Ret>(expr : impl Fn(Input) -> Result<()>) -> Expectation<Input,Ret>;
            fn add_expectation<Input,Ret>(expr : Expectation<Input>, ret : Option<Ret>, time_modifier : Option<TimesModifier>);
    }*/

    impl Drop for Example {
        fn drop(&mut self) {
            // Each method has a known mock_id pattern and known type signature.
            // We must drop all ConditionDoublePointers and ReturnValDoublePointers
            // that were registered under this instance's mock IDs.

            let meth1_mock_id =
                context::MockId::new(format!("krate_Mod_Example_meth1{}", self.adt_mock_id.0));
            let meth2_mock_id = context::MockId::new(format!(
                "krate_Mod_Example_ExTrait_meth2{}",
                self.adt_mock_id.0
            ));

            // Helper: recursively walk a Predicate tree, dropping all ConditionDoublePointers
            // with the correct type for each mock_id.
            unsafe fn drop_predicate_meth1(pred: context::Predicate) {
                match pred.kind {
                    context::new_expectations::PredicateKind::Single(single) => {
                        unsafe { single.condition.id_drop::<(*const Example, f32, f32)>() };
                    }
                    context::new_expectations::PredicateKind::And(children)
                    | context::new_expectations::PredicateKind::Or(children)
                    | context::new_expectations::PredicateKind::Xor(children) => {
                        for child in children {
                            unsafe { drop_predicate_meth1(child) };
                        }
                    }
                    context::new_expectations::PredicateKind::Not(inner)
                    | context::new_expectations::PredicateKind::After { then: inner, .. } => {
                        unsafe { drop_predicate_meth1(*inner) };
                    }
                    context::new_expectations::PredicateKind::Times { inner, .. } => {
                        unsafe { drop_predicate_meth1(*inner) };
                    }
                }
            }

            unsafe fn drop_predicate_meth2(pred: context::Predicate) {
                match pred.kind {
                    context::new_expectations::PredicateKind::Single(single) => {
                        unsafe { single.condition.id_drop::<(*const Example, String)>() };
                    }
                    context::new_expectations::PredicateKind::And(children)
                    | context::new_expectations::PredicateKind::Or(children)
                    | context::new_expectations::PredicateKind::Xor(children) => {
                        for child in children {
                            unsafe { drop_predicate_meth2(child) };
                        }
                    }
                    context::new_expectations::PredicateKind::Not(inner)
                    | context::new_expectations::PredicateKind::After { then: inner, .. } => {
                        unsafe { drop_predicate_meth2(*inner) };
                    }
                    context::new_expectations::PredicateKind::Times { inner, .. } => {
                        unsafe { drop_predicate_meth2(*inner) };
                    }
                }
            }

            context::active_or_latest_checkpoint_mut(|cp| {
                // Drop meth1 expectations: Input = (*const Example, f32, f32), Return = usize
                if let Some(expectations) = cp.expectations.remove(&meth1_mock_id) {
                    for exp in expectations {
                        // Drop the return value closure
                        if let Some(ret) = exp.return_val {
                            unsafe {
                                ret.id_drop::<(*const Example, f32, f32), usize>();
                            }
                        }
                        // Drop the predicate tree (conditions)
                        if let Some(pred) = cp.arena.take(exp.predicate) {
                            unsafe {
                                drop_predicate_meth1(pred);
                            }
                        }
                    }
                }

                // Drop meth2 expectations: Input = (*const Example, String), Return = bool
                if let Some(expectations) = cp.expectations.remove(&meth2_mock_id) {
                    for exp in expectations {
                        if let Some(ret) = exp.return_val {
                            unsafe {
                                ret.id_drop::<(*const Example, String), bool>();
                            }
                        }
                        if let Some(pred) = cp.arena.take(exp.predicate) {
                            unsafe {
                                drop_predicate_meth2(pred);
                            }
                        }
                    }
                }
            });
        }
    }

    pub struct PredicateExampleMeth1(context::Predicate);
    pub struct ExpectationExampleMeth1(context::Expectation);
    pub struct ReturnExampleMeth1(context::ReturnValDoublePointer);
    //trait impls
    pub struct PredicateExampleImplExTraitMeth2(context::Predicate);
    pub struct ExpectationExampleImplExTraitMeth2(context::Expectation);
    pub struct ReturnExampleImplExTraitMeth2(context::ReturnValDoublePointer);

    impl ReturnExampleMeth1 {
        pub fn from_fn(closure: impl Fn(*const Example, f32, f32) -> usize + 'static) -> Self {
            Self(context::ReturnValDoublePointer::from_fn::<
                (*const Example, f32, f32),
                usize,
            >(Box::new(move |(a, b, c)| closure(a, b, c))))
        }
    }

    impl ReturnExampleImplExTraitMeth2 {
        pub fn from_fn(closure: impl Fn(*const Example, String) -> bool + 'static) -> Self {
            Self(context::ReturnValDoublePointer::from_fn::<
                (*const Example, String),
                bool,
            >(Box::new(move |(a, b)| closure(a, b))))
        }
    }

    impl PredicateExampleMeth1 {
        pub fn from_fn(
            closure: impl Fn(*const Example, f32, f32) -> context::errors::PredicateResult<()> + 'static,
        ) -> Self {
            let mock_id = context::MockId::new("krate_Mod_Example_meth1");
            let cond = ConditionDoublePointer::from_fn::<(*const Example, f32, f32)>(Box::new(
                move |input: &(*const Example, f32, f32)| closure(input.0, input.1, input.2),
            ));
            Self(context::Predicate::create_single::<(
                *const Example,
                f32,
                f32,
            )>(&mock_id, cond))
        }
    }

    impl PredicateExampleImplExTraitMeth2 {
        pub fn from_fn(
            closure: impl Fn(*const Example, String) -> context::errors::PredicateResult<()> + 'static,
        ) -> Self {
            let mock_id = context::MockId::new("krate_Mod_Example_ExTrait_meth2");
            let cond = ConditionDoublePointer::from_fn::<(*const Example, String)>(Box::new(
                move |input: &(*const Example, String)| closure(input.0, input.1.clone()),
            ));
            Self(context::Predicate::create_single::<(*const Example, String)>(&mock_id, cond))
        }
    }

    impl Example {
        pub fn meth1(&self, a: f32, b: f32) -> usize {
            std::eprintln!("INFO: Mocked version of method meth1 was used",);
            let mock_id =
                context::MockId::new(format!("krate_Mod_Example_meth1{}", self.adt_mock_id.0));
            if context::ctx_built_and_contains_id(&mock_id) {
                match context::run_mock::<(*const Self, f32, f32), usize>(
                    mock_id,
                    (self as *const Self, a, b),
                ) {
                    Ok(res) => res,
                    Err(e) => match e {
                        context::MockError::Other(e) => panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) => panic!("{:?}", e.0),
                        context::MockError::NoMatchingId => panic!("failed to find mock id"),
                    },
                }
            } else {
                panic!("no id found in context matching krate_Mod_Example_meth1")
            }
        }

        pub fn on_call_meth1(ret: impl Into<ReturnExampleMeth1>) {
            //ret
            let inner: ReturnExampleMeth1 = ret.into();
            let cond =
                ConditionDoublePointer::from_fn::<(*const Example, f32, f32)>(Box::new(|_| Ok(())));
            context::add_expectation::<(*const Example, f32, f32), usize>(
                &context::MockId::new("krate_Mod_Example_meth1"),
                cond,
                Some(inner.0),
                None,
                context::TimesModifier::Any,
            )
            .unwrap();
        }
        pub fn create_predicinate_meth1(
            &self,
            condition: impl Fn((&Example, f32, f32)) -> bool + 'static,
            on_failure: Option<String>,
        ) -> PredicateExampleMeth1 {
            let mock_id =
                context::MockId::new(format!("krate_Mod_Example_meth1{}", self.adt_mock_id.0));
            let cond: context::ConditionDoublePointer =
                context::ConditionDoublePointer::from_fn::<(*const Example, f32, f32)>(Box::new(
                    move |input: &(*const Example, f32, f32)| {
                        // SAFETY: the pointer is valid for the duration of the mock call —
                        // it originates from an active &self in meth1.
                        let self_ref = unsafe { &*input.0 };
                        if condition((self_ref, input.1, input.2)) {
                            Ok(())
                        } else {
                            Err(on_failure
                                .clone()
                                .unwrap_or("failed to uphold condition for meth1".into())
                                .into())
                        }
                    },
                ));

            let single =
                context::Predicate::create_single::<(*const Example, f32, f32)>(&mock_id, cond);
            PredicateExampleMeth1(single)
        }
        pub fn meth1_times(
            checkpoint: Option<impl Into<context::CheckpointName>>,
            condition: impl Into<PredicateExampleMeth1>,
            tmod: context::TimesModifier,
        ) -> PredicateExampleMeth1 {
            let pred: PredicateExampleMeth1 = condition.into();

            let result = std::cell::Cell::new(None);
            let do_times = |cp: &mut context::Checkpoint| {
                result.set(Some(PredicateExampleMeth1(cp.times(pred.0, tmod))));
            };

            if let Some(name) = checkpoint {
                let name: context::CheckpointName = name.into();
                context::checkpoint_by_name_mut(&name.0, do_times)
                    .expect("failed to resolve checkpoint by name");
            } else {
                context::latest_checkpoint_mut(do_times);
            }

            result.into_inner().expect("checkpoint closure did not run")
        }

        pub fn expect_meth1(
            &self,
            checkpoint: Option<impl Into<context::CheckpointName>>,
            condition: impl Into<PredicateExampleMeth1>,
            ret: impl Into<ReturnExampleMeth1>,
            tmod: Option<context::TimesModifier>,
        ) {
            let mock_id =
                context::MockId::new(format!("krate_Mod_Example_meth1{}", self.adt_mock_id.0));

            // Convert condition into our predicate wrapper
            let mut pred: PredicateExampleMeth1 = condition.into();
            let ret_val: ReturnExampleMeth1 = ret.into();

            // Patch the predicate's mock_id to be instance-specific
            if let context::new_expectations::PredicateKind::Single(ref mut single) = pred.0.kind {
                single.mock_id = mock_id.clone();
            }

            let do_expect = |cp: &mut context::Checkpoint| {
                // Insert the owned predicate into the checkpoint's arena
                let pred_idx = cp.arena.insert(pred.0);

                // Optionally wrap with a times modifier
                let final_pred_idx = if let Some(tmod) = tmod {
                    cp.times_arena(pred_idx, tmod)
                } else {
                    pred_idx
                };

                // Commit the expectation with the return value
                cp.expect::<(*const Example, f32, f32), usize>(
                    &mock_id,
                    final_pred_idx,
                    Some(ret_val.0),
                );
            };

            if let Some(name) = checkpoint {
                let name: context::CheckpointName = name.into();
                context::checkpoint_by_name_mut(&name.0, do_expect)
                    .expect("failed to resolve checkpoint by name");
            } else {
                context::latest_checkpoint_mut(do_expect);
            }
        }

        // ─── Trait impl: ExTrait::meth2(&mut self, text: String) -> bool ────

        pub fn meth2(&mut self, text: String) -> bool {
            std::eprintln!("INFO: Mocked version of method meth2 was used",);
            let mock_id = context::MockId::new(format!(
                "krate_Mod_Example_ExTrait_meth2{}",
                self.adt_mock_id.0
            ));
            if context::ctx_built_and_contains_id(&mock_id) {
                match context::run_mock::<(*const Self, String), bool>(
                    mock_id,
                    (self as *const Self, text),
                ) {
                    Ok(res) => res,
                    Err(e) => match e {
                        context::MockError::Other(e) => panic!("unexpected Error: {:?}", e),
                        context::MockError::PredicateError(e) => panic!("{:?}", e.0),
                        context::MockError::NoMatchingId => panic!("failed to find mock id"),
                    },
                }
            } else {
                panic!("no id found in context matching krate_Mod_Example_ExTrait_meth2")
            }
        }

        pub fn on_call_meth2(ret: impl Into<ReturnExampleImplExTraitMeth2>) {
            let inner: ReturnExampleImplExTraitMeth2 = ret.into();
            let cond =
                ConditionDoublePointer::from_fn::<(*const Example, String)>(Box::new(|_| Ok(())));
            context::add_expectation::<(*const Example, String), bool>(
                &context::MockId::new("krate_Mod_Example_ExTrait_meth2"),
                cond,
                Some(inner.0),
                None,
                context::TimesModifier::Any,
            )
            .unwrap();
        }

        pub fn create_predicate_meth2(
            &self,
            condition: impl Fn(&Example, &str) -> bool + 'static,
            on_failure: Option<String>,
        ) -> PredicateExampleImplExTraitMeth2 {
            let mock_id = context::MockId::new(format!(
                "krate_Mod_Example_ExTrait_meth2{}",
                self.adt_mock_id.0
            ));
            let cond: context::ConditionDoublePointer =
                context::ConditionDoublePointer::from_fn::<(*const Example, String)>(Box::new(
                    move |input: &(*const Example, String)| {
                        // SAFETY: the pointer is valid for the duration of the mock call —
                        // it originates from an active &mut self in meth2.
                        let self_ref = unsafe { &*input.0 };
                        if condition(self_ref, &input.1) {
                            Ok(())
                        } else {
                            Err(on_failure
                                .clone()
                                .unwrap_or("failed to uphold condition for meth2".into())
                                .into())
                        }
                    },
                ));

            let single =
                context::Predicate::create_single::<(*const Example, String)>(&mock_id, cond);
            PredicateExampleImplExTraitMeth2(single)
        }

        pub fn meth2_times(
            checkpoint: Option<impl Into<context::CheckpointName>>,
            condition: impl Into<PredicateExampleImplExTraitMeth2>,
            tmod: context::TimesModifier,
        ) -> PredicateExampleImplExTraitMeth2 {
            let pred: PredicateExampleImplExTraitMeth2 = condition.into();

            let result = std::cell::Cell::new(None);
            let do_times = |cp: &mut context::Checkpoint| {
                result.set(Some(PredicateExampleImplExTraitMeth2(
                    cp.times(pred.0, tmod),
                )));
            };

            if let Some(name) = checkpoint {
                let name: context::CheckpointName = name.into();
                context::checkpoint_by_name_mut(&name.0, do_times)
                    .expect("failed to resolve checkpoint by name");
            } else {
                context::latest_checkpoint_mut(do_times);
            }

            result.into_inner().expect("checkpoint closure did not run")
        }

        pub fn expect_meth2(
            &self,
            checkpoint: Option<impl Into<context::CheckpointName>>,
            condition: impl Into<PredicateExampleImplExTraitMeth2>,
            ret: impl Into<ReturnExampleImplExTraitMeth2>,
            tmod: Option<context::TimesModifier>,
        ) {
            let mock_id = context::MockId::new(format!(
                "krate_Mod_Example_ExTrait_meth2{}",
                self.adt_mock_id.0
            ));

            // Convert condition into our predicate wrapper
            let mut pred: PredicateExampleImplExTraitMeth2 = condition.into();
            let ret_val: ReturnExampleImplExTraitMeth2 = ret.into();

            // Patch the predicate's mock_id to be instance-specific
            if let context::new_expectations::PredicateKind::Single(ref mut single) = pred.0.kind {
                single.mock_id = mock_id.clone();
            }

            let do_expect = |cp: &mut context::Checkpoint| {
                // Insert the owned predicate into the checkpoint's arena
                let pred_idx = cp.arena.insert(pred.0);

                // Optionally wrap with a times modifier
                let final_pred_idx = if let Some(tmod) = tmod {
                    cp.times_arena(pred_idx, tmod)
                } else {
                    pred_idx
                };

                // Commit the expectation with the return value
                cp.expect::<(*const Example, String), bool>(
                    &mock_id,
                    final_pred_idx,
                    Some(ret_val.0),
                );
            };

            if let Some(name) = checkpoint {
                let name: context::CheckpointName = name.into();
                context::checkpoint_by_name_mut(&name.0, do_expect)
                    .expect("failed to resolve checkpoint by name");
            } else {
                context::latest_checkpoint_mut(do_expect);
            }
        }

        // ─── Sequence helpers ────────────────────────────────────────────

        /// Add meth1 as a step in a named sequence.
        /// `sequence_name`: the name of the sequence (must already be created via `context::new_sequence`)
        /// `sequence_index`: the position within the sequence (0-based)
        /// `condition`: predicate closure for this step
        /// `ret`: return value closure for this step
        pub fn expect_meth1_in_sequence(
            &self,
            sequence_name: impl Into<context::SequenceName>,
            sequence_index: usize,
            condition: impl Fn(*const Example, f32, f32) -> context::errors::PredicateResult<()>
                + 'static,
            ret: impl Fn(*const Example, f32, f32) -> usize + 'static,
            checkpoint: Option<impl Into<context::CheckpointName>>,
        ) {
            let mock_id =
                context::MockId::new(format!("krate_Mod_Example_meth1{}", self.adt_mock_id.0));
            let cond = context::ConditionDoublePointer::from_fn::<(*const Example, f32, f32)>(
                Box::new(move |input: &(*const Example, f32, f32)| {
                    condition(input.0, input.1, input.2)
                }),
            );
            let ret_closure: Box<dyn Fn((*const Example, f32, f32)) -> usize> =
                Box::new(move |(a, b, c)| ret(a, b, c));

            context::add_expectation_to_sequence::<(*const Example, f32, f32), usize>(
                &mock_id,
                cond,
                Some(ret_closure),
                sequence_name,
                sequence_index,
                checkpoint.map(|c| c.into()),
            )
            .expect("failed to add meth1 to sequence");
        }

        /// Add meth2 (trait method) as a step in a named sequence.
        pub fn expect_meth2_in_sequence(
            &self,
            sequence_name: impl Into<context::SequenceName>,
            sequence_index: usize,
            condition: impl Fn(*const Example, String) -> context::errors::PredicateResult<()> + 'static,
            ret: impl Fn(*const Example, String) -> bool + 'static,
            checkpoint: Option<impl Into<context::CheckpointName>>,
        ) {
            let mock_id = context::MockId::new(format!(
                "krate_Mod_Example_ExTrait_meth2{}",
                self.adt_mock_id.0
            ));
            let cond = context::ConditionDoublePointer::from_fn::<(*const Example, String)>(
                Box::new(move |input: &(*const Example, String)| {
                    condition(input.0, input.1.clone())
                }),
            );
            let ret_closure: Box<dyn Fn((*const Example, String)) -> bool> =
                Box::new(move |(a, b)| ret(a, b));

            context::add_expectation_to_sequence::<(*const Example, String), bool>(
                &mock_id,
                cond,
                Some(ret_closure),
                sequence_name,
                sequence_index,
                checkpoint.map(|c| c.into()),
            )
            .expect("failed to add meth2 to sequence");
        }

        //constructors don't
        pub fn new(a: f32, b: f32) -> Self {
            // Initialize the mock object — private fields become PhantomData,
            // public fields are passed through, adt_mock_id tracks the instance.
            let slf = Self {
                a: PhantomData,
                b,
                adt_mock_id: context::new_id(),
            };

            let meth1_mock_id =
                context::MockId::new(format!("krate_Mod_Example_meth1{}", slf.adt_mock_id.0));
            let meth2_mock_id = context::MockId::new(format!(
                "krate_Mod_Example_ExTrait_meth2{}",
                slf.adt_mock_id.0
            ));

            // Register mocks for each method so the context knows about them
            context::add_mock::<(*const Self, f32, f32), usize>(meth1_mock_id, None).unwrap();
            context::add_mock::<(*const Self, String), bool>(meth2_mock_id, None).unwrap();
            slf
        }
    }
}
use crate::Mod::*;
pub fn main() {
    let mut ex = Example::new(1.0, 2.0);
    let mut ex2 = Example::new(2.0, 30.0);
    let ret = ReturnExampleMeth1::from_fn(|a, b, c| 0);
    let pred = PredicateExampleMeth1::from_fn(|a, b, c| Ok(()));
    ex.expect_meth1(None::<String>, pred, ret, None);

    // ex: meth1 with a specific condition — return 42 when b > 5.0
    let ret = ReturnExampleMeth1::from_fn(|_self_ptr, b, _c| (b as usize) * 2);
    let pred = PredicateExampleMeth1::from_fn(|_self_ptr, b, _c| {
        if b > 5.0 {
            Ok(())
        } else {
            Err("expected b > 5.0".into())
        }
    });
    ex.expect_meth1(
        None::<String>,
        pred,
        ret,
        Some(context::TimesModifier::Times(3)),
    );

    // ex: meth2 (trait method) — accept any input, return true
    let ret = ReturnExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| true);
    let pred = PredicateExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| Ok(()));
    ex.expect_meth2(None::<String>, pred, ret, None);

    // ex: meth2 with condition — only match when text starts with "hello"
    let ret = ReturnExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| false);
    let pred = PredicateExampleImplExTraitMeth2::from_fn(|_self_ptr, text| {
        if text.starts_with("hello") {
            Ok(())
        } else {
            Err("expected text to start with 'hello'".into())
        }
    });
    ex.expect_meth2(
        None::<String>,
        pred,
        ret,
        Some(context::TimesModifier::Once),
    );

    // ex2: meth1 — always return 99
    let ret = ReturnExampleMeth1::from_fn(|_self_ptr, _a, _b| 99);
    let pred = PredicateExampleMeth1::from_fn(|_self_ptr, _a, _b| Ok(()));
    ex2.expect_meth1(None::<String>, pred, ret, None);

    // ex2: meth1 with condition — only when both a and b are positive
    let ret = ReturnExampleMeth1::from_fn(|_self_ptr, a, b| (a + b) as usize);
    let pred = PredicateExampleMeth1::from_fn(|_self_ptr, a, b| {
        if a > 0.0 && b > 0.0 {
            Ok(())
        } else {
            Err("expected both a and b to be positive".into())
        }
    });
    ex2.expect_meth1(
        None::<String>,
        pred,
        ret,
        Some(context::TimesModifier::AtLeast(2)),
    );

    // ex2: meth2 — return true when text is non-empty
    let ret = ReturnExampleImplExTraitMeth2::from_fn(|_self_ptr, text| !text.is_empty());
    let pred = PredicateExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| Ok(()));
    ex2.expect_meth2(None::<String>, pred, ret, None);

    // ─── Sequence Examples ──────────────────────────────────────────────

    // Example 1: Basic sequence — meth1 must be called before meth2 on the same instance
    // Create a sequence with 2 steps
    context::new_sequence("meth1_then_meth2", 2, context::TimesModifier::Once, None).unwrap();

    let mut ex3 = Example::new(10.0, 20.0);

    // Step 0: meth1 must be called first (any args accepted, returns 1)
    ex3.expect_meth1_in_sequence(
        "meth1_then_meth2",
        0,
        |_self_ptr, _a, _b| Ok(()),
        |_self_ptr, _a, _b| 1,
        None::<String>,
    );

    // Step 1: meth2 must be called second (any args accepted, returns true)
    ex3.expect_meth2_in_sequence(
        "meth1_then_meth2",
        1,
        |_self_ptr, _text| Ok(()),
        |_self_ptr, _text| true,
        None::<String>,
    );

    // Example 2: Sequence with conditions — enforce ordering with predicates
    context::new_sequence("conditional_ordering", 3, context::TimesModifier::Once, None).unwrap();

    let ex4 = Example::new(5.0, 5.0);

    // Step 0: meth1 called with positive a
    ex4.expect_meth1_in_sequence(
        "conditional_ordering",
        0,
        |_self_ptr, a, _b| {
            if a > 0.0 {
                Ok(())
            } else {
                Err("step 0: expected a > 0".into())
            }
        },
        |_self_ptr, a, b| (a + b) as usize,
        None::<String>,
    );

    // Step 1: meth1 called with b > 10
    ex4.expect_meth1_in_sequence(
        "conditional_ordering",
        1,
        |_self_ptr, _a, b| {
            if b > 10.0 {
                Ok(())
            } else {
                Err("step 1: expected b > 10".into())
            }
        },
        |_self_ptr, _a, b| b as usize,
        None::<String>,
    );

    // Step 2: meth1 called with both args zero (terminal step)
    ex4.expect_meth1_in_sequence(
        "conditional_ordering",
        2,
        |_self_ptr, a, b| {
            if a == 0.0 && b == 0.0 {
                Ok(())
            } else {
                Err("step 2: expected both args to be 0".into())
            }
        },
        |_self_ptr, _a, _b| 0,
        None::<String>,
    );

    // Example 3: Cross-instance sequence — steps span different mock instances
    context::new_sequence("cross_instance", 2, context::TimesModifier::Once, None).unwrap();

    let ex5 = Example::new(1.0, 1.0);
    let ex6 = Example::new(2.0, 2.0);

    // Step 0: ex5.meth1 must happen first
    ex5.expect_meth1_in_sequence(
        "cross_instance",
        0,
        |_self_ptr, _a, _b| Ok(()),
        |_self_ptr, _a, _b| 55,
        None::<String>,
    );

    // Step 1: ex6.meth1 must happen second
    ex6.expect_meth1_in_sequence(
        "cross_instance",
        1,
        |_self_ptr, _a, _b| Ok(()),
        |_self_ptr, _a, _b| 66,
        None::<String>,
    );

    // Example 4: Sequence with repetition — sequence can be traversed multiple times
    context::new_sequence(
        "repeatable_seq",
        2,
        context::TimesModifier::Times(3), // must complete the full sequence exactly 3 times
        None,
    )
    .unwrap();

    let ex7 = Example::new(0.0, 0.0);

    // Step 0: meth1 (called once per iteration)
    ex7.expect_meth1_in_sequence(
        "repeatable_seq",
        0,
        |_self_ptr, _a, _b| Ok(()),
        |_self_ptr, a, _b| a as usize,
        None::<String>,
    );

    // Step 1: meth1 again (called once per iteration)
    ex7.expect_meth1_in_sequence(
        "repeatable_seq",
        1,
        |_self_ptr, _a, _b| Ok(()),
        |_self_ptr, _a, b| b as usize,
        None::<String>,
    );
}

#[cfg(test)]
mod tests {
    use super::Mod::*;
    use context;

    /// Helper to set up a fresh context for each test.
    fn setup() {
        context::teardown();
    }

    #[test]
    fn test_meth1_unconditional_returns_constant() {
        setup();
        let ex = Example::new(1.0, 2.0);

        let ret = ReturnExampleMeth1::from_fn(|_self_ptr, _a, _b| 42);
        let pred = PredicateExampleMeth1::from_fn(|_self_ptr, _a, _b| Ok(()));
        ex.expect_meth1(None::<String>, pred, ret, Some(context::TimesModifier::Any));

        context::finish_building_context();

        assert_eq!(ex.meth1(3.0, 4.0), 42);
        assert_eq!(ex.meth1(0.0, 0.0), 42);
    }

    #[test]
    fn test_meth1_returns_computed_value() {
        setup();
        let ex = Example::new(5.0, 10.0);

        let ret = ReturnExampleMeth1::from_fn(|_self_ptr, a, b| (a * b) as usize);
        let pred = PredicateExampleMeth1::from_fn(|_self_ptr, _a, _b| Ok(()));
        ex.expect_meth1(None::<String>, pred, ret, Some(context::TimesModifier::Any));

        context::finish_building_context();

        assert_eq!(ex.meth1(3.0, 7.0), 21);
        assert_eq!(ex.meth1(2.0, 5.0), 10);
    }

    #[test]
    fn test_meth1_with_condition_positive_inputs() {
        setup();
        let ex = Example::new(1.0, 1.0);

        // Expectation: only matches when both a and b are positive
        let ret = ReturnExampleMeth1::from_fn(|_self_ptr, a, b| (a + b) as usize);
        let pred = PredicateExampleMeth1::from_fn(|_self_ptr, a, b| {
            if a > 0.0 && b > 0.0 {
                Ok(())
            } else {
                Err("both inputs must be positive".into())
            }
        });
        ex.expect_meth1(None::<String>, pred, ret, Some(context::TimesModifier::Any));

        context::finish_building_context();

        assert_eq!(ex.meth1(2.0, 3.0), 5);
        assert_eq!(ex.meth1(10.0, 0.5), 10);
    }

    #[test]
    #[should_panic(expected = "both inputs must be positive")]
    fn test_meth1_condition_fails_on_negative() {
        setup();
        let ex = Example::new(1.0, 1.0);

        let ret = ReturnExampleMeth1::from_fn(|_self_ptr, _a, _b| 0);
        let pred = PredicateExampleMeth1::from_fn(|_self_ptr, a, b| {
            if a > 0.0 && b > 0.0 {
                Ok(())
            } else {
                Err("both inputs must be positive".into())
            }
        });
        ex.expect_meth1(None::<String>, pred, ret, None);

        context::finish_building_context();

        // This should panic because -1.0 is not positive
        ex.meth1(-1.0, 5.0);
    }

    #[test]
    fn test_meth2_returns_true_for_any_input() {
        setup();
        let mut ex = Example::new(1.0, 2.0);

        let ret = ReturnExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| true);
        let pred = PredicateExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| Ok(()));
        ex.expect_meth2(None::<String>, pred, ret, Some(context::TimesModifier::Any));

        context::finish_building_context();

        assert_eq!(ex.meth2("anything".into()), true);
        assert_eq!(ex.meth2("".into()), true);
    }

    #[test]
    fn test_meth2_condition_checks_string_content() {
        setup();
        let mut ex = Example::new(1.0, 2.0);

        // Return length > 3 check
        let ret = ReturnExampleImplExTraitMeth2::from_fn(|_self_ptr, text| text.len() > 3);
        let pred = PredicateExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| Ok(()));
        ex.expect_meth2(None::<String>, pred, ret, Some(context::TimesModifier::Any));

        context::finish_building_context();

        assert_eq!(ex.meth2("hello world".into()), true);
        assert_eq!(ex.meth2("hi".into()), false);
    }

    #[test]
    #[should_panic(expected = "must start with 'greet:'")]
    fn test_meth2_condition_rejects_wrong_prefix() {
        setup();
        let mut ex = Example::new(1.0, 2.0);

        let ret = ReturnExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| true);
        let pred = PredicateExampleImplExTraitMeth2::from_fn(|_self_ptr, text| {
            if text.starts_with("greet:") {
                Ok(())
            } else {
                Err("must start with 'greet:'".into())
            }
        });
        ex.expect_meth2(None::<String>, pred, ret, None);

        context::finish_building_context();

        // Should panic — doesn't start with "greet:"
        ex.meth2("goodbye".into());
    }

    #[test]
    fn test_multiple_instances_independent_expectations() {
        setup();
        let ex1 = Example::new(1.0, 2.0);
        let ex2 = Example::new(3.0, 4.0);

        // ex1 returns a constant
        let ret = ReturnExampleMeth1::from_fn(|_self_ptr, _a, _b| 100);
        let pred = PredicateExampleMeth1::from_fn(|_self_ptr, _a, _b| Ok(()));
        ex1.expect_meth1(None::<String>, pred, ret, None);

        // ex2 returns sum
        let ret = ReturnExampleMeth1::from_fn(|_self_ptr, a, b| (a + b) as usize);
        let pred = PredicateExampleMeth1::from_fn(|_self_ptr, _a, _b| Ok(()));
        ex2.expect_meth1(None::<String>, pred, ret, None);

        context::finish_building_context();

        assert_eq!(ex1.meth1(5.0, 5.0), 100);
        assert_eq!(ex2.meth1(5.0, 5.0), 10);
    }

    #[test]
    fn test_meth1_with_times_modifier() {
        setup();
        let ex = Example::new(1.0, 2.0);

        let ret = ReturnExampleMeth1::from_fn(|_self_ptr, _a, _b| 7);
        let pred = PredicateExampleMeth1::from_fn(|_self_ptr, _a, _b| Ok(()));
        ex.expect_meth1(
            None::<String>,
            pred,
            ret,
            Some(context::TimesModifier::AtLeast(1)),
        );

        context::finish_building_context();

        // Call multiple times — should succeed since AtLeast(1)
        assert_eq!(ex.meth1(1.0, 1.0), 7);
        assert_eq!(ex.meth1(2.0, 2.0), 7);
        assert_eq!(ex.meth1(3.0, 3.0), 7);
    }

    #[test]
    fn test_meth2_captures_environment() {
        setup();
        let mut ex = Example::new(1.0, 2.0);

        let secret = String::from("password123");
        let ret = ReturnExampleImplExTraitMeth2::from_fn(move |_self_ptr, text| text == secret);
        let pred = PredicateExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| Ok(()));
        ex.expect_meth2(None::<String>, pred, ret, Some(context::TimesModifier::Any));

        context::finish_building_context();

        assert_eq!(ex.meth2("password123".into()), true);
        assert_eq!(ex.meth2("wrong".into()), false);
    }

    #[test]
    fn test_drop_cleans_up_without_panic() {
        setup();
        {
            let ex = Example::new(1.0, 2.0);

            let ret = ReturnExampleMeth1::from_fn(|_self_ptr, _a, _b| 0);
            let pred = PredicateExampleMeth1::from_fn(|_self_ptr, _a, _b| Ok(()));
            ex.expect_meth1(None::<String>, pred, ret, None);

            let ret = ReturnExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| true);
            let pred = PredicateExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| Ok(()));
            ex.expect_meth2(None::<String>, pred, ret, None);

            context::finish_building_context();
            // ex drops here — should clean up all closures without panicking
        }
        // If we get here, drop succeeded
    }

    // ─── Sequence tests ─────────────────────────────────────────────────

    #[test]
    fn test_sequence_basic_ordering() {
        setup();
        let mut ex = Example::new(1.0, 2.0);

        // Create a 2-step sequence: meth1 must be called before meth2
        context::new_sequence("basic_order", 2, context::TimesModifier::Once, None).unwrap();

        ex.expect_meth1_in_sequence(
            "basic_order",
            0,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, _a, _b| 10,
            None::<String>,
        );

        ex.expect_meth2_in_sequence(
            "basic_order",
            1,
            |_self_ptr, _text| Ok(()),
            |_self_ptr, _text| true,
            None::<String>,
        );

        context::finish_building_context();
        context::activate_sequence("basic_order").unwrap();

        // Call in correct order
        assert_eq!(ex.meth1(1.0, 2.0), 10);
        assert_eq!(ex.meth2("hello".into()), true);
    }

    #[test]
    #[should_panic]
    fn test_sequence_wrong_order_panics() {
        setup();
        let mut ex = Example::new(1.0, 2.0);

        context::new_sequence("wrong_order", 2, context::TimesModifier::Once, None).unwrap();

        ex.expect_meth1_in_sequence(
            "wrong_order",
            0,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, _a, _b| 10,
            None::<String>,
        );

        ex.expect_meth2_in_sequence(
            "wrong_order",
            1,
            |_self_ptr, _text| Ok(()),
            |_self_ptr, _text| true,
            None::<String>,
        );

        context::finish_building_context();
        context::activate_sequence("wrong_order").unwrap();

        // Call in WRONG order — meth2 before meth1, should panic
        ex.meth2("hello".into());
    }

    #[test]
    fn test_sequence_with_conditions() {
        setup();
        let ex = Example::new(5.0, 5.0);

        context::new_sequence("cond_seq", 2, context::TimesModifier::Once, None).unwrap();

        // Step 0: a must be > 0
        ex.expect_meth1_in_sequence(
            "cond_seq",
            0,
            |_self_ptr, a, _b| {
                if a > 0.0 {
                    Ok(())
                } else {
                    Err("expected a > 0".into())
                }
            },
            |_self_ptr, a, b| (a + b) as usize,
            None::<String>,
        );

        // Step 1: b must be > 10
        ex.expect_meth1_in_sequence(
            "cond_seq",
            1,
            |_self_ptr, _a, b| {
                if b > 10.0 {
                    Ok(())
                } else {
                    Err("expected b > 10".into())
                }
            },
            |_self_ptr, _a, b| b as usize,
            None::<String>,
        );

        context::finish_building_context();
        context::activate_sequence("cond_seq").unwrap();

        // Step 0: a=5.0 > 0 ✓
        assert_eq!(ex.meth1(5.0, 3.0), 8);
        // Step 1: b=15.0 > 10 ✓
        assert_eq!(ex.meth1(1.0, 15.0), 15);
    }

    #[test]
    fn test_sequence_cross_instance() {
        setup();
        let ex1 = Example::new(1.0, 1.0);
        let ex2 = Example::new(2.0, 2.0);

        // Sequence spanning two different instances
        context::new_sequence("cross", 2, context::TimesModifier::Once, None).unwrap();

        // Step 0: ex1.meth1
        ex1.expect_meth1_in_sequence(
            "cross",
            0,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, _a, _b| 11,
            None::<String>,
        );

        // Step 1: ex2.meth1
        ex2.expect_meth1_in_sequence(
            "cross",
            1,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, _a, _b| 22,
            None::<String>,
        );

        context::finish_building_context();
        context::activate_sequence("cross").unwrap();

        // Must call ex1 first, then ex2
        assert_eq!(ex1.meth1(0.0, 0.0), 11);
        assert_eq!(ex2.meth1(0.0, 0.0), 22);
    }

    #[test]
    #[should_panic]
    fn test_sequence_cross_instance_wrong_order_panics() {
        setup();
        let ex1 = Example::new(1.0, 1.0);
        let ex2 = Example::new(2.0, 2.0);

        context::new_sequence("cross_wrong", 2, context::TimesModifier::Once, None).unwrap();

        ex1.expect_meth1_in_sequence(
            "cross_wrong",
            0,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, _a, _b| 11,
            None::<String>,
        );

        ex2.expect_meth1_in_sequence(
            "cross_wrong",
            1,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, _a, _b| 22,
            None::<String>,
        );

        context::finish_building_context();
        context::activate_sequence("cross_wrong").unwrap();

        // Wrong order: ex2 before ex1
        ex2.meth1(0.0, 0.0);
    }

    #[test]
    fn test_sequence_repeatable() {
        setup();
        let ex = Example::new(0.0, 0.0);

        // Sequence that must be traversed exactly 3 times
        context::new_sequence("repeat", 2, context::TimesModifier::Times(3), None).unwrap();

        ex.expect_meth1_in_sequence(
            "repeat",
            0,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, a, _b| a as usize,
            None::<String>,
        );

        ex.expect_meth1_in_sequence(
            "repeat",
            1,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, _a, b| b as usize,
            None::<String>,
        );

        context::finish_building_context();
        context::activate_sequence("repeat").unwrap();

        // Iteration 1
        assert_eq!(ex.meth1(1.0, 0.0), 1); // step 0
        assert_eq!(ex.meth1(0.0, 2.0), 2); // step 1
        // Iteration 2
        assert_eq!(ex.meth1(3.0, 0.0), 3); // step 0
        assert_eq!(ex.meth1(0.0, 4.0), 4); // step 1
        // Iteration 3
        assert_eq!(ex.meth1(5.0, 0.0), 5); // step 0
        assert_eq!(ex.meth1(0.0, 6.0), 6); // step 1
    }

    #[test]
    fn test_sequence_mixed_methods() {
        setup();
        let mut ex = Example::new(1.0, 2.0);

        // Sequence mixing meth1 and meth2 calls in a specific pattern
        context::new_sequence("mixed", 4, context::TimesModifier::Once, None).unwrap();

        // meth1 → meth2 → meth1 → meth2
        ex.expect_meth1_in_sequence(
            "mixed",
            0,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, _a, _b| 1,
            None::<String>,
        );
        ex.expect_meth2_in_sequence(
            "mixed",
            1,
            |_self_ptr, _text| Ok(()),
            |_self_ptr, _text| true,
            None::<String>,
        );
        ex.expect_meth1_in_sequence(
            "mixed",
            2,
            |_self_ptr, _a, _b| Ok(()),
            |_self_ptr, _a, _b| 2,
            None::<String>,
        );
        ex.expect_meth2_in_sequence(
            "mixed",
            3,
            |_self_ptr, _text| Ok(()),
            |_self_ptr, _text| false,
            None::<String>,
        );

        context::finish_building_context();
        context::activate_sequence("mixed").unwrap();

        assert_eq!(ex.meth1(0.0, 0.0), 1);
        assert_eq!(ex.meth2("a".into()), true);
        assert_eq!(ex.meth1(0.0, 0.0), 2);
        assert_eq!(ex.meth2("b".into()), false);
    }
}
