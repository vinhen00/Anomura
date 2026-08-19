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
            //lookup context, find all the saved expectations, cast to closures using the right ids and drop them
            todo!()
        }
    }

    pub struct PredicateExampleMeth1(context::Predicate);
    pub struct ExpectationExampleMeth1(context::Expectation);
    pub struct ReturnExampleMeth1(context::ReturnValDoublePointer);
    //trait impls
    pub struct PredicateExampleImplExTraitMeth2(context::Predicate);
    pub struct ExpectationExampleImplExTraitMeth2(context::Expectation);
    pub struct ReturnExampleImplExTraitMeth2(context::ReturnValDoublePointer);

    impl Example {
        pub fn meth1(&self, a: f32, b: f32) -> usize {
            std::eprintln!("INFO: Mocked version of method meth1 was used",);
            let mock_id =
                context::MockId::new(format!("krate_Mod_Example_new{}", self.adt_mock_id.0));
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
                context::MockId::new(format!("krate_Mod_Example_new{}", self.adt_mock_id.0));
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
                context::MockId::new(format!("krate_Mod_Example_new{}", self.adt_mock_id.0));

            // Convert condition into our predicate wrapper
            let pred: PredicateExampleMeth1 = condition.into();
            let ret_val: ReturnExampleMeth1 = ret.into();

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
            let pred: PredicateExampleImplExTraitMeth2 = condition.into();
            let ret_val: ReturnExampleImplExTraitMeth2 = ret.into();

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

        //constructors don't
        pub fn new(a: f32, b: f32) -> Self {
            // Initialize the mock object — private fields become PhantomData,
            // public fields are passed through, adt_mock_id tracks the instance.
            let slf = Self {
                a: PhantomData,
                b,
                adt_mock_id: context::new_id(),
            };

            let mock_id =
                context::MockId::new(format!("krate_Mod_Example_new{}", slf.adt_mock_id.0));

            // Register this mock so the context knows about it
            context::add_mock::<(f32, f32), Self>(
                mock_id.clone(),
                Some(Box::new(move |(_a, _b)| Self {
                    a: PhantomData,
                    b: _b,
                    adt_mock_id: context::new_id(),
                })),
            )
            .unwrap();
            slf
        }
    }
}

pub fn main() {
    let mut ex = Example::new(1.0, 2.0);
    let mut ex2 = Example::new(2.0, 30.0);
    ex.expect_meth1(None, |a,b,c| {}, ret, tmod);
}
