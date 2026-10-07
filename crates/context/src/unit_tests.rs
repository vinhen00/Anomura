#[cfg(test)]
mod tests {
    use crate::{
        ConditionDoublePointer, ReturnValDoublePointer,
        errors::PredicateResult,
        mock::MockId,
        new_expectations::{Checkpoint, TimesModifier},
    };

    /// Helper to create a `ConditionDoublePointer` from a typed closure.
    ///
    /// This wraps the closure into a type-erased raw pointer representation
    /// (`ConditionDoublePointer`) that can be stored in the predicate arena
    /// alongside predicates of different input types. The type is recovered
    /// via unsafe cast at evaluation time.
    fn cond<Input: 'static>(
        f: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>,
    ) -> ConditionDoublePointer {
        ConditionDoublePointer::from_fn::<Input>(f)
    }

    /// Validates that `ConditionDoublePointer` correctly round-trips a typed
    /// condition closure through type-erased storage and back.
    ///
    /// This is a foundational safety test. `ConditionDoublePointer` uses raw
    /// pointer casts to store closures of arbitrary input types in a
    /// homogeneous arena. If the erase→recover round-trip is broken, every
    /// predicate evaluation in the system would produce undefined behavior.
    /// This test confirms that a `Fn(&u32) -> PredicateResult<()>` survives
    /// the `from_fn` → `into_fn` cycle with correct semantics.
    #[test]
    fn pointers1() {
        // Create a condition closure that accepts u32 references: passes if > 2, fails otherwise.
        let a: Box<dyn Fn(&u32) -> PredicateResult<()> + 'static> =
            Box::new(|a| if *a > 2 { Ok(()) } else { Err("error".into()) });

        // Erase the type into a ConditionDoublePointer (raw pointer storage).
        let double_ptr = ConditionDoublePointer::from_fn(a);

        // Recover the original typed closure via unsafe cast.
        // Safety: we know the original type was Fn(&u32) -> PredicateResult<()>.
        let casted = unsafe { double_ptr.into_fn::<u32>() };

        // Verify the recovered closure preserves the original condition logic.
        assert!(casted(&3).is_ok()); // 3 > 2, should pass
        assert!(casted(&2).is_err()); // 2 == 2, not > 2, should fail
    }

    /// Validates that `ReturnValDoublePointer` correctly round-trips a typed
    /// return-value closure through type-erased storage and back.
    ///
    /// Analogous to `pointers1` but for the return-value path.
    /// `ReturnValDoublePointer` erases `Fn(Input) -> Output` closures so that
    /// expectations with different return types can coexist in the same
    /// checkpoint. If this round-trip is broken, mock evaluations would
    /// return garbage or segfault. This test uses a struct with a String
    /// field to exercise heap-allocated return values, not just Copy types.
    #[test]
    fn pointers2() {
        // A non-Copy struct to verify heap-allocated return values survive the round-trip.
        struct TestStruct {
            pub string: String,
        }

        // Create a return-value closure: takes () input, produces a TestStruct.
        let a: Box<dyn Fn(()) -> TestStruct + 'static> = Box::new(|()| TestStruct {
            string: String::from("hello pointers2"),
        });

        // Erase the type into a ReturnValDoublePointer.
        let double_ptr = ReturnValDoublePointer::from_fn(a);

        // Recover the original typed closure.
        // Safety: we know the original type was Fn(()) -> TestStruct.
        let casted = unsafe { double_ptr.into_fn::<(), TestStruct>() };

        // Verify the recovered closure produces the correct String value.
        assert_eq!(casted(()).string, "hello pointers2");
        // Negative check: the closure doesn't produce an incorrect value.
        assert_ne!(casted(()).string, "goodbye pointer2");
    }

    /// Validates the complete lifecycle of a single mock expectation:
    /// creation, evaluation with matching input, return value execution,
    /// and exhaustion after the cardinality limit is reached.
    ///
    /// This is the most fundamental integration test for the checkpoint system.
    /// It exercises the full pipeline that every mock call traverses:
    ///   1. `create_single` — registers a leaf predicate with a condition closure
    ///   2. `times_arena` — wraps the predicate with a Once cardinality modifier
    ///   3. `expect` — commits the predicate + return value as a top-level expectation
    ///   4. `evaluate` — finds the matching expectation, runs the return closure
    ///   5. Second `evaluate` — confirms Once exhaustion rejects further calls
    ///
    /// If any of these steps fail, no mock in the system would work correctly.
    #[test]
    fn single_expectation_matches() {
        struct Foo(u32);

        // Create a mock identifier for a function named "foo".
        let mock_id = MockId::new_fn("foo");
        let mut cp = Checkpoint::new();

        // Create a leaf predicate: the condition closure accepts only input == 7.
        // The predicate is stored in the checkpoint's arena and returns a PredicateIndex handle.
        let pred = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(
                |a| if *a == 7 { Ok(()) } else { Err("not 7".into()) },
            )),
        );

        // Wrap the leaf predicate with a Once cardinality modifier.
        // Once is an alias for Times(1): the predicate can match exactly one time
        // before it becomes exhausted and rejects further evaluations.
        let pred_once = cp.times_arena(pred, TimesModifier::Once);

        // Commit the expectation: when input matches pred_once, invoke the return closure.
        // The return closure takes the input by value (u32) and produces Foo(input * 10).
        // After this call, the expectation is registered under mock_id in the checkpoint.
        cp.expect::<u32, Foo>(
            &mock_id,
            pred_once,
            Some(ReturnValDoublePointer::from_fn::<u32, Foo>(Box::new(
                |a: u32| Foo(a * 10),
            ))),
        );

        // First evaluation: input 7 matches the condition (7 == 7).
        // The checkpoint finds the matching expectation, runs the return closure,
        // and marks the Once modifier as completed + exhausted (count=1, cap=1).
        // Safety: Input type u32 must match what was used in create_single and expect.
        let result: Option<Foo> = unsafe { cp.evaluate::<u32, Foo>(&mock_id, 7).unwrap() };
        assert_eq!(result.unwrap().0, 70); // 7 * 10 = 70

        // Second evaluation: input 7 still matches the condition closure itself,
        // but the Once modifier is exhausted (count=1 >= cap=1), so the predicate
        // rejects the call. With no other expectations registered, evaluate returns Err.
        let result = unsafe { cp.evaluate::<u32, Foo>(&mock_id, 7) };
        assert!(result.is_err());
    }

    /// Validates that a checkpoint can hold multiple expectations for the same
    /// mock ID, and that evaluation selects the correct expectation based on
    /// which predicate's condition matches the input.
    ///
    /// This tests the expectation dispatch mechanism: when `evaluate` is called,
    /// the checkpoint iterates through all non-exhausted expectations for the
    /// given mock ID in registration order and returns the first match. Without
    /// correct multi-expectation dispatch, users couldn't define different
    /// behaviors for different inputs to the same mocked function.
    #[test]
    fn multiple_expectations_in_order() {
        struct Foo(u32);
        let mock_id = MockId::new_fn("foo");
        let mut cp = Checkpoint::new();

        // First expectation: matches input == 7, returns Foo(100).
        // Registered first, so it will be checked first during evaluation.
        let pred1 = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(
                |a| if *a == 7 { Ok(()) } else { Err("not 7".into()) },
            )),
        );
        let pred1_once = cp.times_arena(pred1, TimesModifier::Once);
        cp.expect::<u32, Foo>(
            &mock_id,
            pred1_once,
            Some(ReturnValDoublePointer::from_fn::<u32, Foo>(Box::new(
                |_: u32| Foo(100),
            ))),
        );

        // Second expectation: matches input == 42, returns Foo(200).
        // Registered second, checked after pred1 during evaluation.
        let pred2 = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(|a| {
                if *a == 42 {
                    Ok(())
                } else {
                    Err("not 42".into())
                }
            })),
        );
        let pred2_once = cp.times_arena(pred2, TimesModifier::Once);
        cp.expect::<u32, Foo>(
            &mock_id,
            pred2_once,
            Some(ReturnValDoublePointer::from_fn::<u32, Foo>(Box::new(
                |_: u32| Foo(200),
            ))),
        );

        // Evaluate with input 7: pred1 matches (7 == 7), pred2 is not checked.
        // Returns Foo(100) from pred1's return closure.
        let result: Foo = unsafe { cp.evaluate::<u32, Foo>(&mock_id, 7).unwrap().unwrap() };
        assert_eq!(result.0, 100);

        // Evaluate with input 42: pred1 is now exhausted (Once, already used),
        // so the checkpoint skips it and tries pred2, which matches (42 == 42).
        // Returns Foo(200) from pred2's return closure.
        let result: Foo = unsafe { cp.evaluate::<u32, Foo>(&mock_id, 42).unwrap().unwrap() };
        assert_eq!(result.0, 200);
    }

    /// Validates that a single checkpoint can hold expectations for multiple
    /// independent mock IDs with different input and output types.
    ///
    /// This tests the checkpoint's type-erased storage: `ConditionDoublePointer`
    /// and `ReturnValDoublePointer` allow expectations with different type
    /// signatures (u32 → Foo, String → Bar) to coexist in the same arena.
    /// Mock IDs act as the dispatch key, routing each `evaluate` call to the
    /// correct set of expectations. If type erasure or ID-based dispatch is
    /// broken, mocks for one function could interfere with mocks for another.
    #[test]
    fn multiple_mocks() {
        struct Foo(u32);
        struct Bar(String);

        // Two independent mock IDs for two different functions.
        let mock_foo = MockId::new_fn("foo");
        let mock_bar = MockId::new_fn("bar");
        let mut cp = Checkpoint::new();

        // Foo expectation: takes u32 input, matches input == 7, returns Foo(42).
        let pred_foo = cp.create_single::<u32>(
            &mock_foo,
            cond::<u32>(Box::new(
                |a| if *a == 7 { Ok(()) } else { Err("not 7".into()) },
            )),
        );
        let pred_foo_once = cp.times_arena(pred_foo, TimesModifier::Once);
        cp.expect::<u32, Foo>(
            &mock_foo,
            pred_foo_once,
            Some(ReturnValDoublePointer::from_fn::<u32, Foo>(Box::new(
                |_: u32| Foo(42),
            ))),
        );

        // Bar expectation: takes String input, matches input == "hello", returns Bar("goodbye").
        // Completely different types from the Foo expectation, stored in the same checkpoint.
        let pred_bar = cp.create_single::<String>(
            &mock_bar,
            cond::<String>(Box::new(|a| {
                if a == "hello" {
                    Ok(())
                } else {
                    Err("not hello".into())
                }
            })),
        );
        let pred_bar_once = cp.times_arena(pred_bar, TimesModifier::Once);
        cp.expect::<String, Bar>(
            &mock_bar,
            pred_bar_once,
            Some(ReturnValDoublePointer::from_fn::<String, Bar>(Box::new(
                |_: String| Bar("goodbye".into()),
            ))),
        );

        // Evaluate foo: dispatches to mock_foo's expectations, uses u32 → Foo path.
        let foo_result: Foo = unsafe { cp.evaluate::<u32, Foo>(&mock_foo, 7).unwrap().unwrap() };
        assert_eq!(foo_result.0, 42);

        // Evaluate bar: dispatches to mock_bar's expectations, uses String → Bar path.
        // The type-erased storage correctly recovers the String input and Bar output types.
        let bar_result: Bar = unsafe {
            cp.evaluate::<String, Bar>(&mock_bar, "hello".to_string())
                .unwrap()
                .unwrap()
        };
        assert_eq!(bar_result.0, "goodbye");
    }

    /// Validates that `TimesModifier::Any` allows unlimited repeated calls
    /// without exhaustion.
    ///
    /// `Any` has no minimum requirement (starts complete) and no upper bound
    /// (never exhausts). This is the "don't care about call count" modifier,
    /// used when the user wants to mock a function that may be called any
    /// number of times. If `Any` incorrectly exhausted, it would break the
    /// common pattern of stubbing utility functions that are called
    /// throughout a test without a specific cardinality requirement.
    #[test]
    fn times_any_allows_repeated_calls() {
        let mock_id = MockId::new_fn("counter");
        let mut cp = Checkpoint::new();

        // Create a leaf predicate with an always-matching condition (accepts any u32).
        let pred = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(|_| Ok(()))),
        );

        // Wrap with Any: no minimum, no maximum. The predicate never exhausts.
        let pred_any = cp.times_arena(pred, TimesModifier::Any);
        cp.expect::<u32, u32>(
            &mock_id,
            pred_any,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x + 1,
            ))),
        );

        // Call multiple times — each call should succeed and return input + 1.
        // Any never exhausts, so this loop can run indefinitely.
        for i in 0..2 {
            let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_id, i).unwrap().unwrap() };
            assert_eq!(result, i + 1);
        }
    }

    /// Validates the `And` logical combinator: a predicate that requires ALL
    /// child conditions to pass.
    ///
    /// `cp.and(vec![a, b])` creates a composite predicate that evaluates both
    /// children against the same input and succeeds only if both return Ok.
    /// This is essential for expressing compound conditions like "input is
    /// greater than 5 AND less than 10" in a single expectation. If And
    /// short-circuited incorrectly or accepted partial matches, mock
    /// conditions would be too permissive.
    #[test]
    fn and_combinator() {
        let mock_id = MockId::new_fn("test");
        let mut cp = Checkpoint::new();

        // First child condition: input must be > 5.
        let gt5 = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(
                |a| if *a > 5 { Ok(()) } else { Err("> 5".into()) },
            )),
        );
        // Second child condition: input must be < 10.
        let lt10 = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(
                |a| if *a < 10 { Ok(()) } else { Err("< 10".into()) },
            )),
        );

        // Combine with And: both conditions must pass simultaneously.
        // The resulting predicate only matches inputs in the open interval (5, 10).
        let combined = cp.and(vec![gt5, lt10]);

        // Wrap with Any so we can test multiple inputs without exhaustion.
        let combined_any = cp.times_arena(combined, TimesModifier::Any);
        cp.expect::<u32, bool>(
            &mock_id,
            combined_any,
            Some(ReturnValDoublePointer::from_fn::<u32, bool>(Box::new(
                |_: u32| true,
            ))),
        );

        // Input 7: passes both (7 > 5 ✓, 7 < 10 ✓). And succeeds.
        let result = unsafe { cp.evaluate::<u32, bool>(&mock_id, 7) };
        assert!(result.is_ok());

        // Input 3: fails gt5 (3 > 5 ✗). And fails because not all children pass.
        let result = unsafe { cp.evaluate::<u32, bool>(&mock_id, 3) };
        assert!(result.is_err());

        // Input 15: fails lt10 (15 < 10 ✗). And fails because not all children pass.
        let result = unsafe { cp.evaluate::<u32, bool>(&mock_id, 15) };
        assert!(result.is_err());
    }

    /// Validates the `Or` logical combinator: a predicate that requires at
    /// least one child condition to pass.
    ///
    /// `cp.or(vec![a, b])` creates a composite predicate that succeeds if
    /// any child returns Ok. This enables "match input 1 OR input 2" style
    /// expectations. If Or failed to short-circuit or required all children
    /// to pass, users couldn't express disjunctive conditions on mock inputs.
    #[test]
    fn or_combinator() {
        let mock_id = MockId::new_fn("test");
        let mut cp = Checkpoint::new();

        // First child: matches input == 1.
        let eq1 = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(
                |a| if *a == 1 { Ok(()) } else { Err("!= 1".into()) },
            )),
        );
        // Second child: matches input == 2.
        let eq2 = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(
                |a| if *a == 2 { Ok(()) } else { Err("!= 2".into()) },
            )),
        );

        // Combine with Or: at least one child must pass.
        // The resulting predicate matches inputs that are either 1 or 2.
        let combined = cp.or(vec![eq1, eq2]);
        let combined_any = cp.times_arena(combined, TimesModifier::Any);
        cp.expect::<u32, bool>(
            &mock_id,
            combined_any,
            Some(ReturnValDoublePointer::from_fn::<u32, bool>(Box::new(
                |_: u32| true,
            ))),
        );

        // Input 1 or 2 should pass — at least one child matches for each.
        assert!(unsafe {
            cp.evaluate::<u32, bool>(&mock_id, 1).is_ok()
                || cp.evaluate::<u32, bool>(&mock_id, 2).is_ok()
        });

        // Input 3 should fail — neither eq1 (3 ≠ 1) nor eq2 (3 ≠ 2) matches.
        assert!(unsafe { cp.evaluate::<u32, bool>(&mock_id, 3) }.is_err());
    }

    /// Validates the named predicate registry: predicates can be stored by
    /// name and retrieved later, with duplicate names rejected.
    ///
    /// Named predicates enable a user-facing API where predicates are defined
    /// once and referenced by name in multiple expectations or sequences.
    /// `name_predicate` registers a predicate under a string key;
    /// `resolve_predicate` retrieves it. Duplicate registration must fail
    /// to prevent silent overwrites that would change mock behavior.
    /// If naming/resolution is broken, the macro layer's named predicate
    /// syntax would silently produce wrong behavior.
    #[test]
    fn named_predicates() {
        let mock_id = MockId::new_fn("test");
        let mut cp = Checkpoint::new();

        // Create a predicate that matches input == 99.
        let pred = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(|a| {
                if *a == 99 {
                    Ok(())
                } else {
                    Err("not 99".into())
                }
            })),
        );

        // Register it under the name "my_pred".
        cp.name_predicate("my_pred", pred).unwrap();

        // Resolve by name — should return the same PredicateIndex handle.
        let resolved = cp.resolve_predicate("my_pred");
        assert_eq!(resolved, Some(pred));

        // Attempt to register a different predicate under the same name.
        // This must fail to prevent silent overwrites.
        let pred2 = cp.create_single::<u32>(&mock_id, cond::<u32>(Box::new(|_| Ok(()))));
        assert!(cp.name_predicate("my_pred", pred2).is_err());
    }

    /// Validates the basic sequence mechanism: an ordered series of mock calls
    /// that must occur in a specific order across different mock IDs.
    ///
    /// Sequences are how users express temporal ordering constraints like
    /// "function A must be called before function B". The sequence builder
    /// allocates slots, each slot is filled with a (mock_id, predicate,
    /// return_fn) triple, then finalized and activated. During evaluation,
    /// the sequence enforces that calls arrive in slot order.
    ///
    /// This test creates a 2-step sequence (a → b), verifies correct-order
    /// calls succeed with the right return values, and confirms `is_complete`
    /// is true after the sequence finishes its single iteration (Once).
    #[test]
    fn sequence_basic() {
        let mock_a = MockId::new_fn("a");
        let mock_b = MockId::new_fn("b");
        let mut cp = Checkpoint::new();

        // Create always-matching predicates for each mock.
        let pred_a = cp.create_single::<u32>(&mock_a, cond::<u32>(Box::new(|_| Ok(()))));
        let pred_b = cp.create_single::<u32>(&mock_b, cond::<u32>(Box::new(|_| Ok(()))));

        // Create a 2-step sequence with Once cardinality (execute the full a→b order exactly once).
        let seq = cp.create_sequence(2, TimesModifier::Once);

        // Fill slot 0 with mock_a: returns input + 1.
        cp.set_sequence_step::<u32, u32>(seq, 0, &mock_a, pred_a, Some(Box::new(|x| x + 1)))
            .unwrap();
        // Fill slot 1 with mock_b: returns input + 2.
        cp.set_sequence_step::<u32, u32>(seq, 1, &mock_b, pred_b, Some(Box::new(|x| x + 2)))
            .unwrap();

        // Finalize: validates all slots are filled, returns any warnings (e.g., empty slots).
        let warnings = cp.finalize_sequences();
        assert!(warnings.is_empty());

        // Activate: makes the sequence live so it participates in evaluation.
        cp.activate_sequence(seq).unwrap();

        // Step 1: the sequence expects mock_a first. Calling mock_a with input 10
        // matches slot 0 and advances the sequence cursor to slot 1.
        let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_a, 10).unwrap().unwrap() };
        assert_eq!(result, 11); // 10 + 1

        // Step 2: the sequence now expects mock_b. Calling mock_b with input 10
        // matches slot 1 and completes the sequence.
        let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_b, 10).unwrap().unwrap() };
        assert_eq!(result, 12); // 10 + 2

        // The Once-wrapped sequence has completed its single iteration.
        // All expectations are satisfied → checkpoint is complete.
        assert!(cp.is_complete());
    }

    /// Validates that calling mocks out of sequence order produces an error.
    ///
    /// This is the core enforcement test for sequences. If out-of-order calls
    /// were silently accepted, sequences would provide no ordering guarantees,
    /// defeating their entire purpose. The test creates a sequence expecting
    /// a → b, but calls b first, which must fail.
    #[test]
    fn sequence_wrong_order_fails() {
        let mock_a = MockId::new_fn("a");
        let mock_b = MockId::new_fn("b");
        let mut cp = Checkpoint::new();

        let pred_a = cp.create_single::<u32>(&mock_a, cond::<u32>(Box::new(|_| Ok(()))));
        let pred_b = cp.create_single::<u32>(&mock_b, cond::<u32>(Box::new(|_| Ok(()))));

        // Sequence: slot 0 = mock_a, slot 1 = mock_b. Order is a then b.
        let seq = cp.create_sequence(2, TimesModifier::Once);
        cp.set_sequence_step::<u32, u32>(seq, 0, &mock_a, pred_a, Some(Box::new(|x| x)))
            .unwrap();
        cp.set_sequence_step::<u32, u32>(seq, 1, &mock_b, pred_b, Some(Box::new(|x| x)))
            .unwrap();

        cp.finalize_sequences();
        cp.activate_sequence(seq).unwrap();

        // Call mock_b first — the sequence cursor is at slot 0 which expects mock_a.
        // mock_b doesn't match the current slot, so evaluation must fail.
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_b, 5) };
        assert!(result.is_err());
    }

    /// Validates that attempting to fill an already-occupied sequence slot
    /// produces an error.
    ///
    /// Each slot in a sequence builder can only be assigned once. Double-filling
    /// would silently overwrite a previously configured step, leading to
    /// confusing behavior where the first `set_sequence_step` call is lost.
    /// This test ensures the builder detects and rejects the collision.
    #[test]
    fn sequence_builder_slot_collision() {
        let mock_a = MockId::new_fn("a");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_a, cond::<u32>(Box::new(|_| Ok(()))));

        // Create a 3-slot sequence.
        let seq = cp.create_sequence(3, TimesModifier::Once);

        // Fill slot 0 successfully.
        cp.set_sequence_step::<u32, u32>(seq, 0, &mock_a, pred, Some(Box::new(|x| x)))
            .unwrap();

        // Attempt to fill slot 0 again — must be rejected as a collision.
        let result = cp.set_sequence_step::<u32, u32>(seq, 0, &mock_a, pred, Some(Box::new(|x| x)));
        assert!(result.is_err());
    }

    /// Validates `is_complete()`: a checkpoint with unsatisfied expectations
    /// reports incomplete, and becomes complete once all minimum requirements
    /// are met.
    ///
    /// `is_complete()` is the mechanism that detects "expected this function
    /// to be called, but it wasn't". It's typically checked in test teardown
    /// to ensure all expectations were fulfilled. If `is_complete` returned
    /// true before expectations were satisfied, unfulfilled mocks would be
    /// silently ignored, hiding real test failures.
    #[test]
    fn checkpoint_completion() {
        let mock_id = MockId::new_fn("foo");
        let mut cp = Checkpoint::new();

        // Create a Once expectation: must be called exactly once.
        let pred = cp.create_single::<u32>(&mock_id, cond::<u32>(Box::new(|_| Ok(()))));
        let pred_once = cp.times_arena(pred, TimesModifier::Once);
        cp.expect::<u32, u32>(
            &mock_id,
            pred_once,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );

        // Before any calls: Once requires 1 completion but has 0.
        // The checkpoint must report NOT complete.
        assert!(!cp.is_complete());

        // Satisfy the expectation with one call.
        // After this, Once has count=1 which meets the requirement.
        let _ = unsafe { cp.evaluate::<u32, u32>(&mock_id, 1) };

        // After the call: Once is satisfied (count=1 >= 1).
        // The checkpoint must report complete.
        assert!(cp.is_complete());
    }

    /// Validates the thread-local global API: the high-level interface that
    /// the generated macro code calls at runtime.
    ///
    /// The proc macros (`mock_fn!`, `mock_method!`, etc.) generate code that
    /// calls `register_mock`, `add_expectation`, `finish_building_context`,
    /// and `run_mock` — not the low-level `Checkpoint` methods directly.
    /// This test exercises the full global API flow to ensure the thread-local
    /// state management, mock registration, expectation building, context
    /// finalization, mock evaluation, and completion checking all work
    /// together end-to-end.
    ///
    /// The `teardown()` calls at start and end ensure clean thread-local
    /// state, preventing interference from or to other tests.
    #[test]
    fn global_api_flow() {
        use crate::new_expectations::TimesModifier;
        use crate::{
            add_expectation, register_mock, control_checkpoint, finish_building_context,
            run_mock, teardown,
        };

        // Ensure clean thread-local state from any prior tests.
        teardown();

        struct Foo(u32);
        let mock_id = MockId::new_fn("global_test");

        // Register the mock ID in the global context (analogous to mock_fn! expansion).
        register_mock(&mock_id).unwrap();

        // Add an expectation via the global API (analogous to the expect() DSL in mock_fn!).
        // Matches input == 5, returns Foo(input * 2), called Once.
        add_expectation::<u32, Foo>(
            &mock_id,
            cond::<u32>(Box::new(
                |a| if *a == 5 { Ok(()) } else { Err("not 5".into()) },
            )),
            Some(ReturnValDoublePointer::from_fn::<u32, Foo>(Box::new(
                |x: u32| Foo(x * 2),
            ))),
            None,
            TimesModifier::Once,
        )
        .unwrap();

        // Finalize the context (analogous to context::finish_building_context() in test code).
        // After this, the context switches from build mode to evaluation mode.
        finish_building_context();

        // Evaluate the mock via the global API (this is what the substituted function body calls).
        let result: Foo = run_mock::<u32, Foo>(mock_id.clone(), 5).unwrap();
        assert_eq!(result.0, 10); // 5 * 2 = 10

        // Check that all expectations are satisfied via the global completion check.
        // control_checkpoint() returns Ok(()) if is_complete() is true.
        assert!(control_checkpoint().is_ok());

        // Clean up thread-local state for subsequent tests.
        teardown();
    }

    /// Validates the `after` ordering constraint: a predicate that is gated
    /// on another expectation being completed first.
    ///
    /// `cp.after((mock_id, expectation_index), inner_pred)` creates a predicate
    /// that will only evaluate `inner_pred` after the referenced expectation
    /// (identified by mock_id and its index in the expectation list) has been
    /// completed. Before the dependency is satisfied, the `after` predicate
    /// always fails regardless of whether `inner_pred` would match.
    ///
    /// This is the lower-level primitive that sequences are built on. It enables
    /// arbitrary dependency graphs between expectations. If `after` didn't
    /// properly gate on completion, temporal ordering constraints would be
    /// unenforceable.
    ///
    /// Test flow:
    /// 1. Register a "dependency" expectation (matches input == 1, Once)
    /// 2. Register a "guarded" expectation gated on the dependency (matches any input, Once)
    /// 3. Try to trigger guarded before dependency → must fail
    /// 4. Satisfy the dependency
    /// 5. Try to trigger guarded after dependency → must succeed
    #[test]
    fn after_blocks_until_dependency_completed() {
        let mock_id = MockId::new_fn("test");
        let mut cp = Checkpoint::new();

        // === Dependency expectation (index 0) ===
        // Matches only input == 1, can fire once. This is what the guarded predicate waits for.
        let dep = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(
                |a| if *a == 1 { Ok(()) } else { Err("not 1".into()) },
            )),
        );
        let dep_once = cp.times_arena(dep, TimesModifier::Once);
        cp.expect::<u32, u32>(
            &mock_id,
            dep_once,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );
        // This is the first expectation registered for mock_id "test", so its index is 0.

        // === Guarded expectation (index 1) ===
        // The inner predicate matches any input, but is wrapped with `after` which gates
        // on (mock_id, 0) — the dependency expectation above.
        let guarded_inner = cp.create_single::<u32>(&mock_id, cond::<u32>(Box::new(|_| Ok(()))));

        // `after` creates a new predicate that checks: "has expectation 0 for mock_id
        // been completed?" If yes, delegates to guarded_inner. If no, returns Err.
        let guarded = cp.after((mock_id.clone(), 0), guarded_inner);
        let guarded_once = cp.times_arena(guarded, TimesModifier::Once);
        cp.expect::<u32, u32>(
            &mock_id,
            guarded_once,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x * 10,
            ))),
        );

        // === Phase 1: Before dependency is satisfied ===
        // Input 99 doesn't match the dependency (needs 1), and the guarded predicate's
        // `after` gate is closed (dependency not yet completed). Neither expectation fires.
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 99) };
        assert!(
            result.is_err(),
            "guarded should not fire before dependency is completed"
        );

        // === Phase 2: Satisfy the dependency ===
        // Input 1 matches the dependency expectation (1 == 1). The dependency's Once
        // modifier is now completed (count=1 >= 1) and exhausted.
        let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_id, 1).unwrap().unwrap() };
        assert_eq!(result, 1);

        // === Phase 3: After dependency is satisfied ===
        // Now the `after` gate is open: the dependency at (mock_id, 0) is completed.
        // Input 5 passes the guarded_inner predicate (matches any), so the guarded
        // expectation fires with return value 5 * 10 = 50.
        let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_id, 5).unwrap().unwrap() };
        assert_eq!(
            result, 50,
            "guarded should fire after dependency is completed"
        );
    }

    // ─── Cardinality nesting tests ─────────────────────────────────────────
    //
    // All cardinality nesting combinations (Times×Times, Times×AtLeast,
    // Times×AtMost, AtLeast×Times, AtLeast×AtMost, AtMost×Times,
    // AtMost×AtLeast, Once×AtLeast, Never×Times, Times×Any, Any×Times)
    // are covered by property-based tests in property_tests.rs with random
    // n,m values via quickcheck. See that module for the full matrix.
}
