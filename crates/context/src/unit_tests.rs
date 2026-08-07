#[cfg(test)]
mod tests {
    use crate::{
        ConditionDoublePointer, ReturnValDoublePointer,
        errors::PredicateResult,
        mock::MockId,
        new_expectations::{Checkpoint, TimesModifier},
    };

    #[test]
    fn pointers1() {
        let a: Box<dyn Fn(&u32) -> PredicateResult<()> + 'static> =
            Box::new(|a| if *a > 2 { Ok(()) } else { Err("error".into()) });
        let double_ptr = ConditionDoublePointer::from_fn(a);
        let casted = unsafe { double_ptr.into_fn::<u32>() };
        assert!(casted(&3).is_ok());
        assert!(casted(&2).is_err());
    }

    #[test]
    fn pointers2() {
        struct TestStruct {
            pub string: String,
        }

        let a: Box<dyn Fn(()) -> TestStruct + 'static> = Box::new(|()| TestStruct {
            string: String::from("hello pointers2"),
        });
        let double_ptr = ReturnValDoublePointer::from_fn(a);
        let casted = unsafe { double_ptr.into_fn::<(), TestStruct>() };
        assert_eq!(casted(()).string, "hello pointers2");
        assert_ne!(casted(()).string, "goodbye pointer2");
    }

    #[test]
    fn single_expectation_matches() {
        struct Foo(u32);
        let mock_id = MockId::new("foo");
        let mut cp = Checkpoint::new();

        // Create a condition: input must equal 7
        let pred = cp.create_single::<u32>(
            &mock_id,
            Box::new(|a| if *a == 7 { Ok(()) } else { Err("not 7".into()) }),
        );
        // Wrap with Times(1) for once-only semantics
        let pred_once = cp.times(pred, TimesModifier::Once);

        // Commit with a return value
        cp.expect::<u32, Foo>(&mock_id, pred_once, Some(Box::new(|a: u32| Foo(a * 10))));

        // Evaluate — should match
        let result: Option<Foo> = unsafe { cp.evaluate::<u32, Foo>(&mock_id, 7).unwrap() };
        assert_eq!(result.unwrap().0, 70);

        // Evaluate again — should fail (Once exhausted)
        let result = unsafe { cp.evaluate::<u32, Foo>(&mock_id, 7) };
        assert!(result.is_err());
    }

    #[test]
    fn multiple_expectations_in_order() {
        struct Foo(u32);
        let mock_id = MockId::new("foo");
        let mut cp = Checkpoint::new();

        // First expectation: input == 7
        let pred1 = cp.create_single::<u32>(
            &mock_id,
            Box::new(|a| if *a == 7 { Ok(()) } else { Err("not 7".into()) }),
        );
        let pred1_once = cp.times(pred1, TimesModifier::Once);
        cp.expect::<u32, Foo>(&mock_id, pred1_once, Some(Box::new(|_: u32| Foo(100))));

        // Second expectation: input == 42
        let pred2 = cp.create_single::<u32>(
            &mock_id,
            Box::new(|a| {
                if *a == 42 {
                    Ok(())
                } else {
                    Err("not 42".into())
                }
            }),
        );
        let pred2_once = cp.times(pred2, TimesModifier::Once);
        cp.expect::<u32, Foo>(&mock_id, pred2_once, Some(Box::new(|_: u32| Foo(200))));

        // First call: 7 matches pred1
        let result: Foo = unsafe { cp.evaluate::<u32, Foo>(&mock_id, 7).unwrap().unwrap() };
        assert_eq!(result.0, 100);

        // Second call: 42 matches pred2
        let result: Foo = unsafe { cp.evaluate::<u32, Foo>(&mock_id, 42).unwrap().unwrap() };
        assert_eq!(result.0, 200);
    }

    #[test]
    fn multiple_mocks() {
        struct Foo(u32);
        struct Bar(String);

        let mock_foo = MockId::new("foo");
        let mock_bar = MockId::new("bar");
        let mut cp = Checkpoint::new();

        // Foo expectation: input == 7
        let pred_foo = cp.create_single::<u32>(
            &mock_foo,
            Box::new(|a| if *a == 7 { Ok(()) } else { Err("not 7".into()) }),
        );
        let pred_foo_once = cp.times(pred_foo, TimesModifier::Once);
        cp.expect::<u32, Foo>(&mock_foo, pred_foo_once, Some(Box::new(|_: u32| Foo(42))));

        // Bar expectation: input == "hello"
        let pred_bar = cp.create_single::<String>(
            &mock_bar,
            Box::new(|a| {
                if a == "hello" {
                    Ok(())
                } else {
                    Err("not hello".into())
                }
            }),
        );
        let pred_bar_once = cp.times(pred_bar, TimesModifier::Once);
        cp.expect::<String, Bar>(
            &mock_bar,
            pred_bar_once,
            Some(Box::new(|_: String| Bar("goodbye".into()))),
        );

        // Run foo
        let foo_result: Foo = unsafe { cp.evaluate::<u32, Foo>(&mock_foo, 7).unwrap().unwrap() };
        assert_eq!(foo_result.0, 42);

        // Run bar
        let bar_result: Bar = unsafe {
            cp.evaluate::<String, Bar>(&mock_bar, "hello".to_string())
                .unwrap()
                .unwrap()
        };
        assert_eq!(bar_result.0, "goodbye");
    }

    #[test]
    fn times_any_allows_repeated_calls() {
        let mock_id = MockId::new("counter");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(
            &mock_id,
            Box::new(|_| Ok(())), // always matches
        );
        // Wrap with Times(Any) for unlimited calls
        let pred_any = cp.times(pred, TimesModifier::Any);
        cp.expect::<u32, u32>(&mock_id, pred_any, Some(Box::new(|x: u32| x + 1)));

        // Can call many times
        for i in 0..2 {
            let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_id, i).unwrap().unwrap() };
            assert_eq!(result, i + 1);
        }
    }

    #[test]
    fn and_combinator() {
        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        // Condition: > 5
        let gt5 = cp.create_single::<u32>(
            &mock_id,
            Box::new(|a| if *a > 5 { Ok(()) } else { Err("> 5".into()) }),
        );
        // Condition: < 10
        let lt10 = cp.create_single::<u32>(
            &mock_id,
            Box::new(|a| if *a < 10 { Ok(()) } else { Err("< 10".into()) }),
        );

        // AND: must be > 5 AND < 10
        let combined = cp.and(vec![gt5, lt10]);
        let combined_any = cp.times(combined, TimesModifier::Any);
        cp.expect::<u32, bool>(&mock_id, combined_any, Some(Box::new(|_: u32| true)));

        // 7 passes both
        let result = unsafe { cp.evaluate::<u32, bool>(&mock_id, 7) };
        assert!(result.is_ok());

        // 3 fails (not > 5)
        let result = unsafe { cp.evaluate::<u32, bool>(&mock_id, 3) };
        assert!(result.is_err());

        // 15 fails (not < 10)
        let result = unsafe { cp.evaluate::<u32, bool>(&mock_id, 15) };
        assert!(result.is_err());
    }

    #[test]
    fn or_combinator() {
        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        // Condition: == 1
        let eq1 = cp.create_single::<u32>(
            &mock_id,
            Box::new(|a| if *a == 1 { Ok(()) } else { Err("!= 1".into()) }),
        );
        // Condition: == 2
        let eq2 = cp.create_single::<u32>(
            &mock_id,
            Box::new(|a| if *a == 2 { Ok(()) } else { Err("!= 2".into()) }),
        );

        let combined = cp.or(vec![eq1, eq2]);
        let combined_any = cp.times(combined, TimesModifier::Any);
        cp.expect::<u32, bool>(&mock_id, combined_any, Some(Box::new(|_: u32| true)));

        // 1 or 2 should pass
        assert!(unsafe {
            cp.evaluate::<u32, bool>(&mock_id, 1).is_ok()
                || cp.evaluate::<u32, bool>(&mock_id, 2).is_ok()
        });

        // 3 should fail
        assert!(unsafe { cp.evaluate::<u32, bool>(&mock_id, 3) }.is_err());
    }

    #[test]
    fn named_predicates() {
        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(
            &mock_id,
            Box::new(|a| {
                if *a == 99 {
                    Ok(())
                } else {
                    Err("not 99".into())
                }
            }),
        );
        cp.name_predicate("my_pred", pred).unwrap();

        // Resolve by name
        let resolved = cp.resolve_predicate("my_pred");
        assert_eq!(resolved, Some(pred));

        // Duplicate name should error
        let pred2 = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        assert!(cp.name_predicate("my_pred", pred2).is_err());
    }

    #[test]
    fn sequence_basic() {
        let mock_a = MockId::new("a");
        let mock_b = MockId::new("b");
        let mut cp = Checkpoint::new();

        // Create predicates
        let pred_a = cp.create_single::<u32>(&mock_a, Box::new(|_| Ok(())));
        let pred_b = cp.create_single::<u32>(&mock_b, Box::new(|_| Ok(())));

        // Create sequence: a then b
        let seq = cp.create_sequence(2, TimesModifier::Once);
        cp.set_sequence_step::<u32, u32>(seq, 0, &mock_a, pred_a, Some(Box::new(|x| x + 1)))
            .unwrap();
        cp.set_sequence_step::<u32, u32>(seq, 1, &mock_b, pred_b, Some(Box::new(|x| x + 2)))
            .unwrap();

        // Finalize and activate
        let warnings = cp.finalize_sequences();
        assert!(warnings.is_empty());
        cp.activate_sequence(seq).unwrap();

        // Step 1: must be mock_a
        let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_a, 10).unwrap().unwrap() };
        assert_eq!(result, 11);

        // Step 2: must be mock_b
        let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_b, 10).unwrap().unwrap() };
        assert_eq!(result, 12);

        // Sequence is done
        assert!(cp.is_complete()); // sequence completed one iteration
    }

    #[test]
    fn sequence_wrong_order_fails() {
        let mock_a = MockId::new("a");
        let mock_b = MockId::new("b");
        let mut cp = Checkpoint::new();

        let pred_a = cp.create_single::<u32>(&mock_a, Box::new(|_| Ok(())));
        let pred_b = cp.create_single::<u32>(&mock_b, Box::new(|_| Ok(())));

        let seq = cp.create_sequence(2, TimesModifier::Once);
        cp.set_sequence_step::<u32, u32>(seq, 0, &mock_a, pred_a, Some(Box::new(|x| x)))
            .unwrap();
        cp.set_sequence_step::<u32, u32>(seq, 1, &mock_b, pred_b, Some(Box::new(|x| x)))
            .unwrap();

        cp.finalize_sequences();
        cp.activate_sequence(seq).unwrap();

        // Call mock_b first — should fail because sequence expects mock_a first
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_b, 5) };
        assert!(result.is_err());
    }

    #[test]
    fn sequence_builder_slot_collision() {
        let mock_a = MockId::new("a");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_a, Box::new(|_| Ok(())));

        let seq = cp.create_sequence(3, TimesModifier::Once);
        // Fill slot 0
        cp.set_sequence_step::<u32, u32>(seq, 0, &mock_a, pred, Some(Box::new(|x| x)))
            .unwrap();
        // Try to fill slot 0 again — should error
        let result = cp.set_sequence_step::<u32, u32>(seq, 0, &mock_a, pred, Some(Box::new(|x| x)));
        assert!(result.is_err());
    }

    #[test]
    fn checkpoint_completion() {
        let mock_id = MockId::new("foo");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let pred_once = cp.times(pred, TimesModifier::Once);
        cp.expect::<u32, u32>(&mock_id, pred_once, Some(Box::new(|x: u32| x)));

        // Not complete yet
        assert!(!cp.is_complete());

        // Satisfy the expectation
        let _ = unsafe { cp.evaluate::<u32, u32>(&mock_id, 1) };

        // Now complete
        assert!(cp.is_complete());
    }

    #[test]
    fn global_api_flow() {
        // Test the thread-local API
        use crate::new_expectations::TimesModifier;
        use crate::{
            add_expectation, add_mock, control_checkpoint, finish_building_context, run_mock,
            teardown,
        };

        teardown(); // ensure clean state

        struct Foo(u32);
        let mock_id = MockId::new("global_test");

        add_mock::<u32, Foo>(mock_id.clone(), Some(Box::new(|_: u32| Foo(0)))).unwrap();

        add_expectation::<u32, Foo>(
            &mock_id,
            Box::new(|a| if *a == 5 { Ok(()) } else { Err("not 5".into()) }),
            Some(Box::new(|x: u32| Foo(x * 2))),
            None,
            TimesModifier::Once,
        )
        .unwrap();

        finish_building_context();

        let result: Foo = run_mock::<u32, Foo>(mock_id.clone(), 5).unwrap();
        assert_eq!(result.0, 10);

        // Checkpoint should be complete now
        assert!(control_checkpoint().is_ok());

        teardown(); // clean up
    }

    #[test]
    fn after_blocks_until_dependency_completed() {
        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        // Dependency expectation: only matches input == 1, once
        let dep = cp.create_single::<u32>(
            &mock_id,
            Box::new(|a| if *a == 1 { Ok(()) } else { Err("not 1".into()) }),
        );
        let dep_once = cp.times(dep, TimesModifier::Once);
        cp.expect::<u32, u32>(&mock_id, dep_once, Some(Box::new(|x: u32| x)));
        // dep expectation is at index 0 for mock_id "test"

        // Guarded predicate: matches any input, but only after dep expectation is completed
        let guarded_inner = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let guarded = cp.after((mock_id.clone(), 0), guarded_inner);
        let guarded_once = cp.times(guarded, TimesModifier::Once);
        cp.expect::<u32, u32>(&mock_id, guarded_once, Some(Box::new(|x: u32| x * 10)));

        // Try to trigger the guarded expectation before dep is satisfied — should fail
        // Input 99 won't match dep (needs 1), and guarded won't fire (dep not completed)
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 99) };
        assert!(
            result.is_err(),
            "guarded should not fire before dependency is completed"
        );

        // Satisfy the dependency
        let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_id, 1).unwrap().unwrap() };
        assert_eq!(result, 1);

        // Now the guarded expectation should fire
        let result: u32 = unsafe { cp.evaluate::<u32, u32>(&mock_id, 5).unwrap().unwrap() };
        assert_eq!(
            result, 50,
            "guarded should fire after dependency is completed"
        );
    }

    #[test]
    fn nested_times_is_multiplicative() {
        // Verify that Times(n, Times(m, a)) allows exactly n*m calls,
        // same as Times(n*m, a). Cardinality lives entirely in the predicate tree.
        let n = 3u32;
        let m = 2u32;

        // ─── Nested: Times(n, Times(m, a)) ───
        let mock_id = MockId::new("nested");
        let mut cp_nested = Checkpoint::new();

        let inner_pred = cp_nested.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let times_m = cp_nested.times(inner_pred, TimesModifier::Times(m));
        let times_n_m = cp_nested.times(times_m, TimesModifier::Times(n));

        cp_nested.expect::<u32, u32>(&mock_id, times_n_m, Some(Box::new(|x: u32| x + 1)));

        let mut nested_count = 0u32;
        for i in 0..20 {
            match unsafe { cp_nested.evaluate::<u32, u32>(&mock_id, i) } {
                Ok(_) => nested_count += 1,
                Err(_) => break,
            }
        }

        // ─── Flat: Times(n*m, a) ───
        let mock_id2 = MockId::new("flat");
        let mut cp_flat = Checkpoint::new();

        let flat_pred = cp_flat.create_single::<u32>(&mock_id2, Box::new(|_| Ok(())));
        let times_nm = cp_flat.times(flat_pred, TimesModifier::Times(n * m));

        cp_flat.expect::<u32, u32>(&mock_id2, times_nm, Some(Box::new(|x: u32| x + 1)));

        let mut flat_count = 0u32;
        for i in 0..20 {
            match unsafe { cp_flat.evaluate::<u32, u32>(&mock_id2, i) } {
                Ok(_) => flat_count += 1,
                Err(_) => break,
            }
        }

        assert_eq!(
            nested_count,
            n * m,
            "Times({n}, Times({m}, a)) should allow exactly {n}*{m} = {} calls",
            n * m
        );
        assert_eq!(
            flat_count,
            n * m,
            "Times({}, a) should allow exactly {} calls",
            n * m,
            n * m
        );
        assert_eq!(
            nested_count,
            flat_count,
            "nested Times({n}, Times({m}, a)) should equal flat Times({}, a)",
            n * m
        );
    }

    // ─── Cardinality nesting tests ─────────────────────────────────────────
    //
    // These tests verify the semantics of nested Times/AtLeast/AtMost modifiers.
    //
    // Core rule: a Times node exhausts only when BOTH its modifier cap is reached
    // AND its inner predicate is exhausted. This means:
    //   - If inner never exhausts (Any, AtLeast), outer never exhausts either.
    //   - If inner does exhaust (Once, Times, AtMost), outer exhausts at n × m.
    //
    // Nestings fall into two categories:
    //   1. Productive: inner doesn't start completed. Requires real calls.
    //      E.g. Times(n, Times(m, P)), AtLeast(n, Times(m, P)), AtMost(n, Times(m, P))
    //   2. Degenerate: inner starts completed (Any, AtMost have min=0).
    //      Outer cycles through phantom iterations at construction time.
    //      E.g. Times(n, Any(P)), AtLeast(n, AtMost(m, P))

    #[test]
    fn times_atleast_is_productive_and_bounded() {
        // Times(n, AtLeast(m, P)): inner completes after m calls, outer needs n
        // completions → requires n×m calls. Times(n) exhausts at n completions
        // → exactly n×m calls accepted.
        let n = 2u32;
        let m = 3u32;

        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let atleast_m = cp.times(pred, TimesModifier::AtLeast(m));
        let times_n_atleast = cp.times(atleast_m, TimesModifier::Times(n));

        cp.expect::<u32, u32>(&mock_id, times_n_atleast, Some(Box::new(|x: u32| x)));

        // Not complete at birth (inner requires real calls)
        assert!(
            !cp.is_complete(),
            "Times({n}, AtLeast({m}, P)) should NOT be complete at birth"
        );

        // After n*m calls, should be complete and exhausted
        for i in 0..(n * m) {
            let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, i) };
            assert!(result.is_ok(), "call {i} should succeed");
        }
        assert!(
            cp.is_complete(),
            "Times({n}, AtLeast({m}, P)) should be complete after {} calls",
            n * m
        );

        // Next call should fail (Times(n) exhausted)
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 99) };
        assert!(
            result.is_err(),
            "Times({n}, AtLeast({m}, P)) should exhaust after {} calls",
            n * m
        );
    }

    #[test]
    fn times_atmost_is_degenerate() {
        // Times(n, AtMost(m, P)): AtMost starts completed (0 ≤ m), so the outer
        // loops at construction. If n ≤ m, all n cycles succeed → completed +
        // exhausted at birth. If n > m, inner exhausts before outer finishes →
        // not completed + exhausted (failed).

        // Case 1: n ≤ m → succeeds at birth
        {
            let n = 3u32;
            let m = 4u32;

            let mock_id = MockId::new("test");
            let mut cp = Checkpoint::new();

            let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
            let atmost_m = cp.times(pred, TimesModifier::AtMost(m));
            let times_n_atmost = cp.times(atmost_m, TimesModifier::Times(n));

            cp.expect::<u32, u32>(&mock_id, times_n_atmost, Some(Box::new(|x: u32| x)));

            assert!(
                cp.is_complete(),
                "Times({n}, AtMost({m}, P)) with n≤m should be immediately complete"
            );
            let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 0) };
            assert!(
                result.is_err(),
                "Times({n}, AtMost({m}, P)) should be exhausted at birth"
            );
        }

        // Case 2: n > m → fails at birth
        {
            let n = 5u32;
            let m = 3u32;

            let mock_id = MockId::new("test");
            let mut cp = Checkpoint::new();

            let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
            let atmost_m = cp.times(pred, TimesModifier::AtMost(m));
            let times_n_atmost = cp.times(atmost_m, TimesModifier::Times(n));

            cp.expect::<u32, u32>(&mock_id, times_n_atmost, Some(Box::new(|x: u32| x)));

            assert!(
                !cp.is_complete(),
                "Times({n}, AtMost({m}, P)) with n>m should NOT be complete (failed)"
            );
            let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 0) };
            assert!(
                result.is_err(),
                "Times({n}, AtMost({m}, P)) with n>m should be exhausted"
            );
        }
    }

    #[test]
    fn atleast_times_is_productive_and_unlimited() {
        // AtLeast(n, Times(m, P)): inner completes after m calls. Outer needs n
        // completions = n×m calls to satisfy. AtLeast never exhausts → unlimited.
        let n = 2u32;
        let m = 3u32;

        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let times_m = cp.times(pred, TimesModifier::Times(m));
        let atleast_n_times = cp.times(times_m, TimesModifier::AtLeast(n));

        cp.expect::<u32, u32>(&mock_id, atleast_n_times, Some(Box::new(|x: u32| x)));

        // Not complete at birth
        assert!(
            !cp.is_complete(),
            "AtLeast({n}, Times({m}, P)) should NOT be complete at birth"
        );

        // After n*m calls, should be complete
        for i in 0..(n * m) {
            let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, i) };
            assert!(result.is_ok(), "call {i} should succeed");
        }
        assert!(
            cp.is_complete(),
            "AtLeast({n}, Times({m}, P)) should be complete after {} calls",
            n * m
        );

        // Should accept more (AtLeast never exhausts, inner Times resets each cycle)
        for i in 0..10 {
            let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, i) };
            assert!(
                result.is_ok(),
                "AtLeast({n}, Times({m}, P)) should accept unlimited calls, failed at extra call {i}"
            );
        }
    }

    #[test]
    fn atleast_atmost_is_degenerate() {
        // AtLeast(n, AtMost(m, P)): AtMost starts completed → outer cycles n
        // times instantly → completed at birth. AtLeast never exhausts, AtMost
        // inner doesn't exhaust until m → outer never exhausts. Degenerate.
        let n = 2u32;
        let m = 5u32;

        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let atmost_m = cp.times(pred, TimesModifier::AtMost(m));
        let atleast_n_atmost = cp.times(atmost_m, TimesModifier::AtLeast(n));

        cp.expect::<u32, u32>(&mock_id, atleast_n_atmost, Some(Box::new(|x: u32| x)));

        // Immediately complete
        assert!(
            cp.is_complete(),
            "AtLeast({n}, AtMost({m}, P)) should be immediately complete"
        );

        // AtLeast never exhausts → unlimited calls accepted
        for i in 0..20 {
            let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, i) };
            assert!(
                result.is_ok(),
                "AtLeast({n}, AtMost({m}, P)) should accept unlimited calls, failed at call {i}"
            );
        }
    }

    #[test]
    fn atmost_times_is_productive_and_bounded() {
        // AtMost(n, Times(m, P)): AtMost starts completed (min=0). Inner Times(m)
        // requires m calls to complete. Since Times(m) exhausts, outer exhausts
        // at n completions → exactly n×m calls accepted.
        let n = 3u32;
        let m = 2u32;

        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let times_m = cp.times(pred, TimesModifier::Times(m));
        let atmost_n_times = cp.times(times_m, TimesModifier::AtMost(n));

        cp.expect::<u32, u32>(&mock_id, atmost_n_times, Some(Box::new(|x: u32| x)));

        // Immediately complete (AtMost: 0 ≤ n)
        assert!(
            cp.is_complete(),
            "AtMost({n}, Times({m}, P)) should be immediately complete"
        );

        // Should accept exactly n*m calls
        for i in 0..(n * m) {
            let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, i) };
            assert!(result.is_ok(), "call {i} should succeed");
        }

        // Next call should fail
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 99) };
        assert!(
            result.is_err(),
            "AtMost({n}, Times({m}, P)) should exhaust after {} calls",
            n * m
        );
    }

    #[test]
    fn atmost_atleast_is_productive_and_bounded() {
        // AtMost(n, AtLeast(m, P)): inner AtLeast(m) requires m real calls to
        // complete. AtMost starts completed (min=0). Outer counts completions
        // and exhausts at n → exactly n×m calls accepted.
        let n = 2u32;
        let m = 3u32;

        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let atleast_m = cp.times(pred, TimesModifier::AtLeast(m));
        let atmost_n_atleast = cp.times(atleast_m, TimesModifier::AtMost(n));

        cp.expect::<u32, u32>(&mock_id, atmost_n_atleast, Some(Box::new(|x: u32| x)));

        // Immediately complete (AtMost min=0)
        assert!(
            cp.is_complete(),
            "AtMost({n}, AtLeast({m}, P)) should be immediately complete"
        );

        // Accepts exactly n*m calls
        for i in 0..(n * m) {
            let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, i) };
            assert!(
                result.is_ok(),
                "AtMost({n}, AtLeast({m}, P)) should accept call {i}"
            );
        }

        // Next call fails (AtMost exhausted)
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 99) };
        assert!(
            result.is_err(),
            "AtMost({n}, AtLeast({m}, P)) should exhaust after {} calls",
            n * m
        );
    }

    #[test]
    fn once_atleast_is_productive_and_bounded() {
        // Once(AtLeast(m, P)) = Times(1, AtLeast(m, P)): requires m calls to
        // complete. Once exhausts after 1 completion → exactly m calls.
        let m = 4u32;

        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let atleast_m = cp.times(pred, TimesModifier::AtLeast(m));
        let once_atleast = cp.times(atleast_m, TimesModifier::Once);

        cp.expect::<u32, u32>(&mock_id, once_atleast, Some(Box::new(|x: u32| x)));

        // Not complete yet
        assert!(!cp.is_complete());

        // After m calls, complete and exhausted
        for i in 0..m {
            let _ = unsafe { cp.evaluate::<u32, u32>(&mock_id, i) };
        }
        assert!(
            cp.is_complete(),
            "Once(AtLeast({m}, P)) should be complete after {m} calls"
        );

        // Next call fails (Once exhausted)
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 99) };
        assert!(
            result.is_err(),
            "Once(AtLeast({m}, P)) should exhaust after {m} calls"
        );
    }

    #[test]
    fn never_times_is_immediately_exhausted() {
        // Never(Times(m, P)): Never means "must be satisfied 0 times". It starts
        // completed (0 == 0) and exhausted (no calls allowed). The inner is irrelevant.
        let m = 3u32;

        let mock_id = MockId::new("test");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let times_m = cp.times(pred, TimesModifier::Times(m));
        let never_times = cp.times(times_m, TimesModifier::Never);

        cp.expect::<u32, u32>(&mock_id, never_times, Some(Box::new(|x: u32| x)));

        // Immediately complete and exhausted
        assert!(
            cp.is_complete(),
            "Never(Times({m}, P)) should be immediately complete"
        );

        // No calls accepted
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 1) };
        assert!(
            result.is_err(),
            "Never(Times({m}, P)) should reject all calls"
        );
    }

    #[test]
    fn times_any_exhausts_after_n() {
        // Times(n, Any(P)): Any(P) starts completed (no minimum), so the outer
        // loops n times at construction → completed + exhausted immediately.
        // No runtime calls are accepted.
        let n = 3u32;

        let mock_id = MockId::new("times_any");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let any_pred = cp.times(pred, TimesModifier::Any);
        let times_n_any = cp.times(any_pred, TimesModifier::Times(n));

        cp.expect::<u32, u32>(&mock_id, times_n_any, Some(Box::new(|x: u32| x)));

        // Immediately complete and exhausted
        assert!(
            cp.is_complete(),
            "Times({n}, Any(P)) should be immediately complete"
        );

        // No calls accepted (already exhausted)
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 0) };
        assert!(
            result.is_err(),
            "Times({n}, Any(P)) should be exhausted at birth, rejecting all calls"
        );
    }

    #[test]
    fn any_times_is_unlimited() {
        // Any(Times(n, P)): Any has no minimum → immediately complete. Any never
        // exhausts → unlimited calls. Inner Times(n) cycles: accepts n calls,
        // exhausts, gets reset by outer. Repeats indefinitely.
        let n = 3u32;

        let mock_id = MockId::new("any_times");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&mock_id, Box::new(|_| Ok(())));
        let times_n = cp.times(pred, TimesModifier::Times(n));
        let any_times = cp.times(times_n, TimesModifier::Any);

        cp.expect::<u32, u32>(&mock_id, any_times, Some(Box::new(|x: u32| x)));

        // Immediately complete (Any has no minimum)
        assert!(
            cp.is_complete(),
            "Any(Times({n}, P)) should be immediately complete"
        );

        // Accepts unlimited calls (Any never exhausts, inner Times resets each cycle)
        let call_count = 10u32;
        for i in 0..call_count {
            let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, i) };
            assert!(
                result.is_ok(),
                "Any(Times({n}, P)) should accept unlimited calls, failed at call {i}"
            );
        }
    }
}
