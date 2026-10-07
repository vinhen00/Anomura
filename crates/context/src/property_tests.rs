//! Property-based tests for the context crate using quickcheck.
//!
//! These tests exercise the core invariants of the mock evaluation system
//! with randomly generated inputs and cardinality parameters, catching
//! corner cases that hand-written examples might miss.
//!
//! # Test categories
//!
//! 1. **Pointer round-trips**: ConditionDoublePointer and ReturnValDoublePointer
//!    must faithfully preserve closures through type-erased storage.
//!
//! 2. **TimesModifier cardinality**: Flat and nested modifiers must accept
//!    exactly the right number of calls, complete at the right moment,
//!    and exhaust at the right moment.
//!
//! 3. **Combinator logic**: And/Or/Not/Xor must implement their boolean
//!    semantics correctly for arbitrary inputs.
//!
//! 4. **Sequence ordering**: Steps must advance in the declared order and
//!    reject out-of-order calls.
//!
//! 5. **Initial state correctness**: The completed/exhausted flags at
//!    construction time must be consistent with the modifier semantics
//!    (degenerate nesting produces immediate completion/exhaustion).

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use crate::{
        ConditionDoublePointer, ReturnValDoublePointer,
        errors::PredicateResult,
        mock::MockId,
        new_expectations::{Checkpoint, TimesModifier},
    };

    // ─── Helpers ────────────────────────────────────────────────────────

    /// Helper to create a ConditionDoublePointer from a typed closure.
    /// Identical to the one in unit_tests.rs — duplicated here so property_tests
    /// is self-contained.
    fn cond<Input: 'static>(
        f: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>,
    ) -> ConditionDoublePointer {
        ConditionDoublePointer::from_fn::<Input>(f)
    }

    /// Build a checkpoint with a single always-matching expectation wrapped
    /// in the given TimesModifier. Returns the checkpoint and mock_id.
    ///
    /// This is the workhorse helper for cardinality tests. It sets up:
    /// - A leaf predicate that accepts any u32 input
    /// - Wrapped with the given TimesModifier
    /// - Committed as an expectation with a return closure (identity + 1)
    fn setup_flat_expectation(modifier: TimesModifier) -> (Checkpoint, MockId) {
        let mock_id = MockId::new_fn("prop");
        let mut cp = Checkpoint::new();
        let pred = cp.create_single::<u32>(&mock_id, cond::<u32>(Box::new(|_| Ok(()))));
        let timed = cp.times_arena(pred, modifier);
        cp.expect::<u32, u32>(
            &mock_id,
            timed,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x + 1,
            ))),
        );
        (cp, mock_id)
    }

    /// Build a checkpoint with a nested modifier: outer(inner(leaf)).
    /// Both modifiers wrap an always-matching leaf predicate.
    fn setup_nested_expectation(
        outer: TimesModifier,
        inner: TimesModifier,
    ) -> (Checkpoint, MockId) {
        let mock_id = MockId::new_fn("prop_nested");
        let mut cp = Checkpoint::new();
        let pred = cp.create_single::<u32>(&mock_id, cond::<u32>(Box::new(|_| Ok(()))));
        let inner_timed = cp.times_arena(pred, inner);
        let outer_timed = cp.times_arena(inner_timed, outer);
        cp.expect::<u32, u32>(
            &mock_id,
            outer_timed,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x + 1,
            ))),
        );
        (cp, mock_id)
    }

    /// Count how many successful evaluations a checkpoint accepts before
    /// returning an error. Caps at `max` to avoid infinite loops for
    /// unlimited modifiers.
    fn count_accepted_calls(cp: &mut Checkpoint, mock_id: &MockId, max: u32) -> u32 {
        let mut count = 0u32;
        for i in 0..max {
            match unsafe { cp.evaluate::<u32, u32>(mock_id, i) } {
                Ok(_) => count += 1,
                Err(_) => break,
            }
        }
        count
    }

    /// Clamp n to a reasonable range for cardinality tests to avoid
    /// extremely long-running iterations. quickcheck generates arbitrary
    /// u32s, which can be enormous.
    fn clamp(n: u32) -> u32 {
        // Range 1..=50 keeps tests fast while still exploring interesting values
        (n % 50) + 1
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  1. POINTER ROUND-TRIP PROPERTIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Property: ConditionDoublePointer preserves predicate semantics for u32 inputs.
    ///
    /// What this measures:
    ///   The type-erased storage in ConditionDoublePointer must faithfully
    ///   reproduce the original closure's accept/reject decisions for any u32 input.
    ///   We test a threshold-based predicate (`input >= threshold`) and verify
    ///   the round-tripped closure agrees with direct evaluation for random inputs.
    ///
    /// Steps:
    ///   1. Create a predicate closure: accepts inputs >= threshold, rejects below.
    ///   2. Erase the type via `from_fn` → raw pointer storage.
    ///   3. Recover the typed closure via unsafe `into_fn`.
    ///   4. For a random `input`, compare the recovered closure's result against
    ///      the direct computation (input >= threshold).
    #[quickcheck]
    fn condition_pointer_round_trip_u32(threshold: u32, input: u32) -> bool {
        let expected = input >= threshold;
        let closure: Box<dyn Fn(&u32) -> PredicateResult<()> + 'static> = Box::new(move |a| {
            if *a >= threshold {
                Ok(())
            } else {
                Err("below threshold".into())
            }
        });
        let ptr = ConditionDoublePointer::from_fn(closure);
        let recovered = unsafe { ptr.into_fn::<u32>() };
        let actual = recovered(&input).is_ok();
        actual == expected
    }

    /// Property: ConditionDoublePointer preserves predicate semantics for String inputs.
    ///
    /// What this measures:
    ///   Same as above but with heap-allocated String inputs. This exercises the
    ///   raw-pointer machinery with a non-Copy type that requires proper reference
    ///   handling through the `Fn(&Input)` interface.
    ///
    /// Steps:
    ///   1. Create a predicate: accepts strings whose length >= threshold.
    ///   2. Round-trip through type-erased storage.
    ///   3. Verify the recovered closure agrees with direct length comparison.
    #[quickcheck]
    fn condition_pointer_round_trip_string(threshold: u8, input: String) -> bool {
        let threshold = threshold as usize;
        let expected = input.len() >= threshold;
        let closure: Box<dyn Fn(&String) -> PredicateResult<()> + 'static> =
            Box::new(move |s| {
                if s.len() >= threshold {
                    Ok(())
                } else {
                    Err("too short".into())
                }
            });
        let ptr = ConditionDoublePointer::from_fn(closure);
        let recovered = unsafe { ptr.into_fn::<String>() };
        let actual = recovered(&input).is_ok();
        actual == expected
    }

    /// Property: ReturnValDoublePointer preserves return value computation for u32.
    ///
    /// What this measures:
    ///   The type-erased return-value storage must faithfully reproduce the original
    ///   closure's computed output. We test an arithmetic closure (input * factor)
    ///   and verify the round-tripped closure produces identical results.
    ///
    /// Steps:
    ///   1. Create a return closure: multiplies input by a random factor.
    ///   2. Erase via `from_fn`, recover via unsafe `into_fn`.
    ///   3. Compare recovered output against direct computation.
    #[quickcheck]
    fn return_pointer_round_trip_u32(factor: u32, input: u32) -> bool {
        let expected = input.wrapping_mul(factor);
        let closure: Box<dyn Fn(u32) -> u32 + 'static> =
            Box::new(move |x| x.wrapping_mul(factor));
        let ptr = ReturnValDoublePointer::from_fn(closure);
        let recovered = unsafe { ptr.into_fn::<u32, u32>() };
        recovered(input) == expected
    }

    /// Property: ReturnValDoublePointer preserves return value computation for String → usize.
    ///
    /// What this measures:
    ///   Tests type-erased return-value storage with a cross-type closure (String → usize).
    ///   This verifies that the raw pointer cast correctly handles different input and
    ///   output types.
    ///
    /// Steps:
    ///   1. Create a return closure: returns the string's length.
    ///   2. Round-trip through type-erased storage.
    ///   3. Verify the recovered closure produces the correct length.
    #[quickcheck]
    fn return_pointer_round_trip_string_to_usize(input: String) -> bool {
        let expected = input.len();
        let closure: Box<dyn Fn(String) -> usize + 'static> = Box::new(|s| s.len());
        let ptr = ReturnValDoublePointer::from_fn(closure);
        let recovered = unsafe { ptr.into_fn::<String, usize>() };
        recovered(input.clone()) == expected
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  2. FLAT CARDINALITY PROPERTIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Property: Times(n) accepts exactly n calls, then exhausts.
    ///
    /// What this measures:
    ///   The fundamental contract of `TimesModifier::Times(n)` — it must accept
    ///   exactly n successful evaluations before rejecting further calls. The
    ///   predicate must not be complete before all n calls, and must be complete
    ///   (and exhausted) after exactly n calls.
    ///
    /// Steps:
    ///   1. Clamp n to [1, 50] for test performance.
    ///   2. Build a checkpoint with Times(n) wrapping an always-matching predicate.
    ///   3. Count accepted calls (up to n + 10 to detect over-acceptance).
    ///   4. Assert: exactly n calls accepted.
    ///   5. Assert: checkpoint is complete after n calls.
    #[quickcheck]
    fn times_n_accepts_exactly_n(n: u32) -> TestResult {
        let n = clamp(n);
        let (mut cp, mock_id) = setup_flat_expectation(TimesModifier::Times(n));

        // Before any calls: Times(n) is not complete (n > 0)
        if cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, n + 10);
        TestResult::from_bool(accepted == n && cp.is_complete())
    }

    /// Property: AtLeast(n) is not complete before n calls, complete after, and never exhausts.
    ///
    /// What this measures:
    ///   `AtLeast(n)` requires a minimum of n calls to become complete. After that,
    ///   it stays complete and continues accepting calls indefinitely. This is the
    ///   "floor with no ceiling" modifier.
    ///
    /// Steps:
    ///   1. Build a checkpoint with AtLeast(n).
    ///   2. Make (n-1) calls — must not be complete yet.
    ///   3. Make 1 more call (reaching n) — must now be complete.
    ///   4. Make 20 more calls — all must succeed (never exhausts).
    #[quickcheck]
    fn atleast_n_needs_n_calls_then_unlimited(n: u32) -> TestResult {
        let n = clamp(n);
        let (mut cp, mock_id) = setup_flat_expectation(TimesModifier::AtLeast(n));

        // Not complete at start
        if cp.is_complete() {
            return TestResult::failed();
        }

        // Make n-1 calls — still not complete
        for i in 0..(n - 1) {
            if unsafe { cp.evaluate::<u32, u32>(&mock_id, i) }.is_err() {
                return TestResult::failed();
            }
        }
        if cp.is_complete() {
            return TestResult::failed(); // shouldn't be complete yet
        }

        // The n-th call completes it
        if unsafe { cp.evaluate::<u32, u32>(&mock_id, n) }.is_err() {
            return TestResult::failed();
        }
        if !cp.is_complete() {
            return TestResult::failed(); // should be complete now
        }

        // More calls still accepted (AtLeast never exhausts)
        for i in 0..20 {
            if unsafe { cp.evaluate::<u32, u32>(&mock_id, n + 1 + i) }.is_err() {
                return TestResult::failed();
            }
        }
        TestResult::passed()
    }

    /// Property: AtMost(n) is immediately complete and accepts exactly n calls.
    ///
    /// What this measures:
    ///   `AtMost(n)` has a minimum of 0 — it's satisfied even with zero calls.
    ///   But it caps at n calls, after which it exhausts. This is the
    ///   "ceiling with no floor" modifier.
    ///
    /// Steps:
    ///   1. Build a checkpoint with AtMost(n).
    ///   2. Verify immediately complete (0 >= 0).
    ///   3. Count accepted calls — must be exactly n.
    ///   4. Verify the (n+1)-th call fails.
    #[quickcheck]
    fn atmost_n_immediately_complete_accepts_n(n: u32) -> TestResult {
        let n = clamp(n);
        let (mut cp, mock_id) = setup_flat_expectation(TimesModifier::AtMost(n));

        // AtMost is always immediately complete (0 <= n)
        if !cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, n + 10);
        TestResult::from_bool(accepted == n)
    }

    /// Any is immediately complete and never exhausts.
    ///
    /// What this measures:
    ///   `Any` has no minimum (complete at birth) and no maximum (never exhausts).
    ///   It's the fully unconstrained modifier. This test verifies it accepts an
    ///   arbitrarily large number of calls without failing.
    ///
    /// Steps:
    ///   1. Build a checkpoint with Any.
    ///   2. Verify immediately complete.
    ///   3. Make 100 calls — all must succeed.
    ///
    /// Note: no random parameters — Any's behavior is fixed. Uses #[test] not #[quickcheck].
    #[test]
    fn any_is_immediately_complete_and_unlimited() {
        let (mut cp, mock_id) = setup_flat_expectation(TimesModifier::Any);
        assert!(cp.is_complete(), "Any should be immediately complete");
        let accepted = count_accepted_calls(&mut cp, &mock_id, 100);
        assert_eq!(accepted, 100, "Any should accept unlimited calls");
    }

    /// Once is equivalent to Times(1).
    ///
    /// What this measures:
    ///   `Once` is documented as an alias for `Times(1)`. This test verifies
    ///   behavioral equivalence: both accept exactly 1 call, both start
    ///   incomplete, both complete and exhaust after the first call.
    ///
    /// Steps:
    ///   1. Build two checkpoints: one with Once, one with Times(1).
    ///   2. Verify both start incomplete.
    ///   3. Both accept exactly 1 call.
    ///   4. Both are complete after 1 call.
    ///   5. Both reject the 2nd call.
    ///
    /// Note: no random parameters — Once and Times(1) are fixed variants. Uses #[test].
    #[test]
    fn once_is_equivalent_to_times_1() {
        let (mut cp_once, mock_once) = setup_flat_expectation(TimesModifier::Once);
        let (mut cp_times1, mock_times1) = setup_flat_expectation(TimesModifier::Times(1));

        // Both start incomplete
        assert!(!cp_once.is_complete());
        assert!(!cp_times1.is_complete());

        // Both accept first call
        assert!(unsafe { cp_once.evaluate::<u32, u32>(&mock_once, 0) }.is_ok());
        assert!(unsafe { cp_times1.evaluate::<u32, u32>(&mock_times1, 0) }.is_ok());

        // Both complete
        assert!(cp_once.is_complete());
        assert!(cp_times1.is_complete());

        // Both reject second call
        assert!(unsafe { cp_once.evaluate::<u32, u32>(&mock_once, 1) }.is_err());
        assert!(unsafe { cp_times1.evaluate::<u32, u32>(&mock_times1, 1) }.is_err());
    }

    /// Never is immediately complete and immediately exhausted (zero calls accepted).
    ///
    /// What this measures:
    ///   `Never` means "must be called zero times". It starts complete (0 == 0) and
    ///   exhausted (no calls allowed). Any attempt to evaluate must fail.
    ///
    /// Steps:
    ///   1. Build a checkpoint with Never.
    ///   2. Verify immediately complete.
    ///   3. Verify the very first call is rejected.
    ///
    /// Note: no random parameters — Never's behavior is fixed. Uses #[test].
    #[test]
    fn never_accepts_zero_calls() {
        let (mut cp, mock_id) = setup_flat_expectation(TimesModifier::Never);
        assert!(cp.is_complete(), "Never should be immediately complete");
        let accepted = count_accepted_calls(&mut cp, &mock_id, 5);
        assert_eq!(accepted, 0, "Never should reject all calls");
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  3. NESTED CARDINALITY PROPERTIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Property: Times(n, Times(m, P)) accepts exactly n*m calls, same as Times(n*m, P).
    ///
    /// What this measures:
    ///   Nested Times modifiers must be multiplicative. The outer Times(n) counts
    ///   completions of the inner Times(m), and each inner completion requires m
    ///   calls. Total = n*m. This must be identical to a single flat Times(n*m).
    ///
    /// Steps:
    ///   1. Build a nested checkpoint: Times(n, Times(m, leaf)).
    ///   2. Build a flat checkpoint: Times(n*m, leaf).
    ///   3. Count accepted calls for both.
    ///   4. Assert: nested == flat == n*m.
    #[quickcheck]
    fn nested_times_is_multiplicative(n: u32, m: u32) -> TestResult {
        let n = clamp(n);
        let m = clamp(m);

        // Avoid overflow in n*m
        if (n as u64) * (m as u64) > 2500 {
            return TestResult::discard();
        }

        let (mut cp_nested, mock_nested) =
            setup_nested_expectation(TimesModifier::Times(n), TimesModifier::Times(m));
        let (mut cp_flat, mock_flat) =
            setup_flat_expectation(TimesModifier::Times(n * m));

        let nested_count = count_accepted_calls(&mut cp_nested, &mock_nested, n * m + 10);
        let flat_count = count_accepted_calls(&mut cp_flat, &mock_flat, n * m + 10);

        TestResult::from_bool(nested_count == n * m && flat_count == n * m)
    }

    /// Property: Times(n, AtLeast(m, P)) accepts exactly n*m calls.
    ///
    /// What this measures:
    ///   Inner AtLeast(m) requires m calls to complete, then the outer counts
    ///   one completion and resets it. After n completions (n*m calls), the
    ///   outer Times(n) exhausts. The inner's "never exhausts" property is
    ///   overridden by the outer's cap.
    ///
    /// Steps:
    ///   1. Build Times(n, AtLeast(m, leaf)).
    ///   2. Verify not complete at birth (inner requires real calls).
    ///   3. Count accepted calls — must be exactly n*m.
    ///   4. Verify complete after n*m calls.
    #[quickcheck]
    fn times_n_atleast_m_accepts_n_times_m(n: u32, m: u32) -> TestResult {
        let n = clamp(n);
        let m = clamp(m);

        if (n as u64) * (m as u64) > 2500 {
            return TestResult::discard();
        }

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::Times(n), TimesModifier::AtLeast(m));

        // Not complete at birth (inner AtLeast requires real calls)
        if cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, n * m + 10);
        TestResult::from_bool(accepted == n * m && cp.is_complete())
    }

    /// Property: AtLeast(n, Times(m, P)) requires n*m calls to complete, then unlimited.
    ///
    /// What this measures:
    ///   Inner Times(m) completes after m calls and exhausts. The outer AtLeast(n)
    ///   counts completions and resets the inner. After n completions (n*m calls),
    ///   the outer is satisfied. Since AtLeast never exhausts, further calls are
    ///   accepted indefinitely (inner resets each cycle).
    ///
    /// Steps:
    ///   1. Build AtLeast(n, Times(m, leaf)).
    ///   2. Verify not complete at birth.
    ///   3. Make n*m - 1 calls — not yet complete.
    ///   4. Make 1 more call — now complete.
    ///   5. Make 20 more calls — all accepted (unlimited).
    #[quickcheck]
    fn atleast_n_times_m_needs_nm_then_unlimited(n: u32, m: u32) -> TestResult {
        let n = clamp(n);
        let m = clamp(m);

        if (n as u64) * (m as u64) > 2500 {
            return TestResult::discard();
        }

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::AtLeast(n), TimesModifier::Times(m));

        if cp.is_complete() {
            return TestResult::failed();
        }

        // Make n*m - 1 calls, should still not be complete
        for i in 0..(n * m - 1) {
            if unsafe { cp.evaluate::<u32, u32>(&mock_id, i) }.is_err() {
                return TestResult::failed();
            }
        }
        if cp.is_complete() {
            return TestResult::failed();
        }

        // The n*m-th call should complete it
        if unsafe { cp.evaluate::<u32, u32>(&mock_id, 999) }.is_err() {
            return TestResult::failed();
        }
        if !cp.is_complete() {
            return TestResult::failed();
        }

        // More calls accepted (AtLeast never exhausts)
        for i in 0..20 {
            if unsafe { cp.evaluate::<u32, u32>(&mock_id, 1000 + i) }.is_err() {
                return TestResult::failed();
            }
        }
        TestResult::passed()
    }

    /// Property: AtMost(n, Times(m, P)) is immediately complete and accepts exactly n*m calls.
    ///
    /// What this measures:
    ///   AtMost starts complete (0 <= n). Inner Times(m) exhausts after m calls,
    ///   triggering an outer cycle. The outer AtMost(n) caps at n cycles = n*m total
    ///   calls. This is a "productive" nesting: calls are needed, but completion is
    ///   satisfied from birth.
    ///
    /// Steps:
    ///   1. Build AtMost(n, Times(m, leaf)).
    ///   2. Verify immediately complete.
    ///   3. Count accepted calls — must be exactly n*m.
    #[quickcheck]
    fn atmost_n_times_m_accepts_nm(n: u32, m: u32) -> TestResult {
        let n = clamp(n);
        let m = clamp(m);

        if (n as u64) * (m as u64) > 2500 {
            return TestResult::discard();
        }

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::AtMost(n), TimesModifier::Times(m));

        // AtMost starts complete
        if !cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, n * m + 10);
        TestResult::from_bool(accepted == n * m)
    }

    /// Property: AtMost(n, AtLeast(m, P)) is immediately complete and accepts exactly n*m calls.
    ///
    /// What this measures:
    ///   AtMost starts complete. Inner AtLeast(m) requires m calls to complete.
    ///   Outer counts completions, exhausts at n → exactly n*m calls. Despite
    ///   AtLeast's "never exhausts" property, the outer AtMost caps it.
    ///
    /// Steps:
    ///   1. Build AtMost(n, AtLeast(m, leaf)).
    ///   2. Verify immediately complete.
    ///   3. Count accepted calls — must be exactly n*m.
    #[quickcheck]
    fn atmost_n_atleast_m_accepts_nm(n: u32, m: u32) -> TestResult {
        let n = clamp(n);
        let m = clamp(m);

        if (n as u64) * (m as u64) > 2500 {
            return TestResult::discard();
        }

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::AtMost(n), TimesModifier::AtLeast(m));

        if !cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, n * m + 10);
        TestResult::from_bool(accepted == n * m)
    }

    /// Property: Times(n, AtMost(m, P)) with m > 0 is immediately complete+exhausted (degenerate).
    ///
    /// What this measures:
    ///   AtMost(m>0) starts completed, so the outer Times(n) phantom-cycles n times
    ///   at construction (each cycle resets the inner). After n cycles, outer is
    ///   completed + exhausted. No runtime calls accepted.
    ///
    ///   Edge case: m=0 → AtMost(0) starts exhausted → no cycling → Times(n>0) not completed.
    ///   But clamp ensures m >= 1, so this case doesn't arise here.
    ///
    /// Steps:
    ///   1. Build Times(n, AtMost(m, leaf)) with m >= 1.
    ///   2. Verify immediately complete.
    ///   3. Verify zero runtime calls accepted (exhausted).
    #[quickcheck]
    fn times_n_atmost_m_degenerate(n: u32, m: u32) -> TestResult {
        let n = clamp(n);
        let m = clamp(m); // clamp gives 1..=50

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::Times(n), TimesModifier::AtMost(m));

        // Always immediately complete+exhausted (inner resets each phantom cycle)
        if !cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, 10);
        TestResult::from_bool(accepted == 0)
    }

    /// Property: Times(n, Any(P)) is degenerate — immediately complete+exhausted (zero calls).
    ///
    /// What this measures:
    ///   Any(P) starts completed, so the outer Times(n) cycles n times at construction
    ///   → completed + exhausted. No runtime calls accepted.
    ///
    /// Steps:
    ///   1. Build Times(n, Any(leaf)).
    ///   2. Verify immediately complete.
    ///   3. Verify zero calls accepted.
    #[quickcheck]
    fn times_n_any_is_degenerate(n: u32) -> TestResult {
        let n = clamp(n);

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::Times(n), TimesModifier::Any);

        if !cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, 10);
        TestResult::from_bool(accepted == 0)
    }

    /// Property: Any(Times(m, P)) is immediately complete and accepts unlimited calls.
    ///
    /// What this measures:
    ///   Any has no minimum → immediately complete. Any never exhausts. Inner Times(m)
    ///   cycles: accepts m calls, exhausts, gets reset by outer, repeats. The result
    ///   is unlimited calls, with the inner resetting every m calls.
    ///
    /// Steps:
    ///   1. Build Any(Times(m, leaf)).
    ///   2. Verify immediately complete.
    ///   3. Make 100 calls — all must succeed.
    #[quickcheck]
    fn any_times_m_is_unlimited(m: u32) -> TestResult {
        let m = clamp(m);

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::Any, TimesModifier::Times(m));

        if !cp.is_complete() {
            return TestResult::failed();
        }

        // Should accept many calls without exhausting
        let accepted = count_accepted_calls(&mut cp, &mock_id, 100);
        TestResult::from_bool(accepted == 100)
    }

    /// Property: AtLeast(n, AtMost(m, P)) is degenerate — immediately complete, unlimited calls.
    /// Exception: m=0 → AtMost(0) starts exhausted, so no phantom cycling occurs and
    /// the outer AtLeast(n) never reaches its minimum (unless n=0 too, but clamp prevents that).
    ///
    /// What this measures:
    ///   AtMost(m>0) starts completed → outer AtLeast(n) phantom-cycles n times at
    ///   construction (each cycle resets the inner) → immediately complete. AtLeast
    ///   never exhausts → unlimited runtime calls.
    ///
    /// Steps:
    ///   1. Build AtLeast(n, AtMost(m, leaf)) with m >= 1 (discard m=0).
    ///   2. Verify immediately complete.
    ///   3. Make 100 calls — all must succeed.
    #[quickcheck]
    fn atleast_n_atmost_m_is_degenerate_unlimited(n: u32, m: u32) -> TestResult {
        let n = clamp(n);
        let m = clamp(m); // clamp gives 1..=50, so m >= 1 always

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::AtLeast(n), TimesModifier::AtMost(m));

        if !cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, 100);
        TestResult::from_bool(accepted == 100)
    }

    /// Property: Once(AtLeast(m, P)) = Times(1, AtLeast(m, P)) accepts exactly m calls.
    ///
    /// What this measures:
    ///   Inner AtLeast(m) requires m calls to complete. Outer Once counts 1 completion
    ///   and exhausts → exactly m calls. Verifies that Once properly limits even
    ///   "never-exhausting" inner modifiers.
    ///
    /// Steps:
    ///   1. Build Once(AtLeast(m, leaf)).
    ///   2. Verify not complete at birth.
    ///   3. Count accepted calls — must be exactly m.
    ///   4. Verify complete after m calls.
    #[quickcheck]
    fn once_atleast_m_accepts_m(m: u32) -> TestResult {
        let m = clamp(m);

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::Once, TimesModifier::AtLeast(m));

        if cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, m + 10);
        TestResult::from_bool(accepted == m && cp.is_complete())
    }

    /// Property: Never(X) is always immediately complete and accepts zero calls,
    /// regardless of what X is.
    ///
    /// What this measures:
    ///   Never means "zero completions required". It's satisfied at birth and
    ///   exhausted immediately. The inner modifier is irrelevant — it's never
    ///   reached. This tests all inner modifier variants.
    ///
    /// Steps:
    ///   1. Build Never(Times(m, leaf)) for random m.
    ///   2. Verify immediately complete.
    ///   3. Verify zero calls accepted.
    #[quickcheck]
    fn never_outer_always_zero(m: u32) -> TestResult {
        let m = clamp(m);

        let (mut cp, mock_id) =
            setup_nested_expectation(TimesModifier::Never, TimesModifier::Times(m));

        if !cp.is_complete() {
            return TestResult::failed();
        }

        let accepted = count_accepted_calls(&mut cp, &mock_id, 5);
        TestResult::from_bool(accepted == 0)
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  4. COMBINATOR PROPERTIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Property: And(P, Q) matches iff both P and Q match.
    ///
    /// What this measures:
    ///   The And combinator must implement logical conjunction. For two threshold
    ///   predicates (input >= a, input >= b), the And should accept iff input is
    ///   at or above both thresholds, i.e., input >= max(a, b).
    ///
    /// Steps:
    ///   1. Create two threshold predicates: input >= a, input >= b.
    ///   2. Combine with And.
    ///   3. For a random input, compare evaluate result against (input >= max(a, b)).
    #[quickcheck]
    fn and_matches_iff_both_match(a: u32, b: u32, input: u32) -> TestResult {
        // Clamp thresholds to reasonable range to avoid all-pass or all-fail
        let a = a % 200;
        let b = b % 200;
        let input = input % 300;

        let expected = input >= a && input >= b;

        let mock_id = MockId::new_fn("and_test");
        let mut cp = Checkpoint::new();

        let pred_a = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(move |x| {
                if *x >= a { Ok(()) } else { Err("below a".into()) }
            })),
        );
        let pred_b = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(move |x| {
                if *x >= b { Ok(()) } else { Err("below b".into()) }
            })),
        );

        let combined = cp.and(vec![pred_a, pred_b]);
        let combined_any = cp.times_arena(combined, TimesModifier::Any);
        cp.expect::<u32, u32>(
            &mock_id,
            combined_any,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );

        let actual = unsafe { cp.evaluate::<u32, u32>(&mock_id, input) }.is_ok();
        TestResult::from_bool(actual == expected)
    }

    /// Property: Or(P, Q) matches iff at least one of P or Q matches.
    ///
    /// What this measures:
    ///   The Or combinator must implement logical disjunction. For two threshold
    ///   predicates, the Or should accept iff input is at or above at least one
    ///   threshold, i.e., input >= min(a, b).
    ///
    /// Steps:
    ///   1. Create two threshold predicates: input >= a, input >= b.
    ///   2. Combine with Or.
    ///   3. For a random input, compare evaluate result against (input >= a || input >= b).
    #[quickcheck]
    fn or_matches_iff_either_matches(a: u32, b: u32, input: u32) -> TestResult {
        let a = a % 200;
        let b = b % 200;
        let input = input % 300;

        let expected = input >= a || input >= b;

        let mock_id = MockId::new_fn("or_test");
        let mut cp = Checkpoint::new();

        let pred_a = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(move |x| {
                if *x >= a { Ok(()) } else { Err("below a".into()) }
            })),
        );
        let pred_b = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(move |x| {
                if *x >= b { Ok(()) } else { Err("below b".into()) }
            })),
        );

        let combined = cp.or(vec![pred_a, pred_b]);
        let combined_any = cp.times_arena(combined, TimesModifier::Any);
        cp.expect::<u32, u32>(
            &mock_id,
            combined_any,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );

        let actual = unsafe { cp.evaluate::<u32, u32>(&mock_id, input) }.is_ok();
        TestResult::from_bool(actual == expected)
    }

    /// Property: Not(P) matches iff P does NOT match.
    ///
    /// What this measures:
    ///   The Not combinator must implement logical negation. For a threshold
    ///   predicate (input >= threshold), Not should accept iff input < threshold.
    ///
    /// Steps:
    ///   1. Create a threshold predicate.
    ///   2. Wrap with Not.
    ///   3. For a random input, compare against !(input >= threshold).
    #[quickcheck]
    fn not_inverts_predicate(threshold: u32, input: u32) -> TestResult {
        let threshold = threshold % 200;
        let input = input % 300;

        let expected = !(input >= threshold);

        let mock_id = MockId::new_fn("not_test");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(move |x| {
                if *x >= threshold { Ok(()) } else { Err("below".into()) }
            })),
        );

        let negated = cp.not(pred);
        let negated_any = cp.times_arena(negated, TimesModifier::Any);
        cp.expect::<u32, u32>(
            &mock_id,
            negated_any,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );

        let actual = unsafe { cp.evaluate::<u32, u32>(&mock_id, input) }.is_ok();
        TestResult::from_bool(actual == expected)
    }

    /// Property: Xor(P, Q) matches iff exactly one of P or Q matches.
    ///
    /// What this measures:
    ///   The Xor combinator must implement exclusive disjunction. For two
    ///   threshold predicates, Xor should accept iff exactly one threshold is met.
    ///
    /// Steps:
    ///   1. Create two threshold predicates.
    ///   2. Combine with Xor.
    ///   3. For a random input, compare against ((input >= a) XOR (input >= b)).
    #[quickcheck]
    fn xor_matches_iff_exactly_one(a: u32, b: u32, input: u32) -> TestResult {
        let a = a % 200;
        let b = b % 200;
        let input = input % 300;

        let p_matches = input >= a;
        let q_matches = input >= b;
        let expected = p_matches ^ q_matches;

        let mock_id = MockId::new_fn("xor_test");
        let mut cp = Checkpoint::new();

        let pred_a = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(move |x| {
                if *x >= a { Ok(()) } else { Err("below a".into()) }
            })),
        );
        let pred_b = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(move |x| {
                if *x >= b { Ok(()) } else { Err("below b".into()) }
            })),
        );

        let combined = cp.xor(vec![pred_a, pred_b]);
        let combined_any = cp.times_arena(combined, TimesModifier::Any);
        cp.expect::<u32, u32>(
            &mock_id,
            combined_any,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );

        let actual = unsafe { cp.evaluate::<u32, u32>(&mock_id, input) }.is_ok();
        TestResult::from_bool(actual == expected)
    }

    /// Property: Not(Not(P)) is semantically equivalent to P.
    ///
    /// What this measures:
    ///   Double negation must cancel out. For any predicate P and any input,
    ///   Not(Not(P)) must produce the same accept/reject result as P alone.
    ///   This tests that the Not combinator correctly propagates through
    ///   nested structures.
    ///
    /// Steps:
    ///   1. Build two checkpoints: one with P, one with Not(Not(P)).
    ///   2. For a random input, evaluate both.
    ///   3. Assert: same result.
    #[quickcheck]
    fn double_negation_is_identity(threshold: u32, input: u32) -> TestResult {
        let threshold = threshold % 200;
        let input = input % 300;

        // Plain P
        let mock_plain = MockId::new_fn("plain");
        let mut cp_plain = Checkpoint::new();
        let pred_plain = cp_plain.create_single::<u32>(
            &mock_plain,
            cond::<u32>(Box::new(move |x| {
                if *x >= threshold { Ok(()) } else { Err("below".into()) }
            })),
        );
        let pred_plain_any = cp_plain.times_arena(pred_plain, TimesModifier::Any);
        cp_plain.expect::<u32, u32>(
            &mock_plain,
            pred_plain_any,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );

        // Not(Not(P))
        let mock_nn = MockId::new_fn("double_not");
        let mut cp_nn = Checkpoint::new();
        let pred_inner = cp_nn.create_single::<u32>(
            &mock_nn,
            cond::<u32>(Box::new(move |x| {
                if *x >= threshold { Ok(()) } else { Err("below".into()) }
            })),
        );
        let not_once = cp_nn.not(pred_inner);
        let not_not = cp_nn.not(not_once);
        let nn_any = cp_nn.times_arena(not_not, TimesModifier::Any);
        cp_nn.expect::<u32, u32>(
            &mock_nn,
            nn_any,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );

        let result_plain = unsafe { cp_plain.evaluate::<u32, u32>(&mock_plain, input) }.is_ok();
        let result_nn = unsafe { cp_nn.evaluate::<u32, u32>(&mock_nn, input) }.is_ok();
        TestResult::from_bool(result_plain == result_nn)
    }

    /// And with an empty predicate list starts completed (vacuous truth).
    /// Or with an empty predicate list starts NOT completed (vacuous falsity).
    ///
    /// What this measures:
    ///   The degenerate case of zero-operand combinators. And([]) = true (all zero
    ///   predicates are satisfied), Or([]) = false (none of zero predicates are
    ///   satisfied). These are the mathematical identities for conjunction and
    ///   disjunction over empty sets.
    ///
    ///   We commit the predicates directly (no Times wrapper) and check the
    ///   expectation's initial completed state, which is read from the predicate's
    ///   state at commit time.
    ///
    /// Steps:
    ///   1. Create And([]) and Or([]) predicates.
    ///   2. Commit them as expectations (no cardinality wrapper).
    ///   3. And([]) starts completed → checkpoint complete.
    ///   4. Or([]) starts NOT completed → checkpoint not complete.
    #[test]
    fn empty_and_is_vacuously_true_empty_or_is_vacuously_false() {
        let mock_id = MockId::new_fn("empty");

        // And([]) → completed=true (vacuous truth). Commit directly as expectation.
        let mut cp_and = Checkpoint::new();
        let and_pred = cp_and.and(vec![]);
        cp_and.expect::<u32, u32>(
            &mock_id,
            and_pred,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(|x: u32| x))),
        );
        assert!(cp_and.is_complete(), "And([]) should be vacuously true → complete");

        // Or([]) → completed=false (vacuous falsity). Commit directly as expectation.
        let mut cp_or = Checkpoint::new();
        let or_pred = cp_or.or(vec![]);
        cp_or.expect::<u32, u32>(
            &mock_id,
            or_pred,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(|x: u32| x))),
        );
        assert!(!cp_or.is_complete(), "Or([]) should be vacuously false → not complete");
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  5. SEQUENCE PROPERTIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Property: A sequence of length n requires calls in exact order; wrong-order calls fail.
    ///
    /// What this measures:
    ///   Sequences enforce strict temporal ordering of mock calls. Step i must be
    ///   called before step i+1. Attempting to call step j when step i < j is
    ///   expected must fail. This tests with random sequence lengths.
    ///
    /// Steps:
    ///   1. Create a sequence of n steps, each for a different mock_id ("step_0", "step_1", ...).
    ///   2. Finalize and activate the sequence.
    ///   3. Call steps in order — all must succeed.
    ///   4. Verify is_complete at the end.
    #[quickcheck]
    fn sequence_accepts_correct_order(n: u32) -> TestResult {
        // Clamp to [2, 10] — need at least 2 steps to test ordering
        let n = (n % 9) + 2;

        let mut cp = Checkpoint::new();

        let mock_ids: Vec<MockId> = (0..n)
            .map(|i| MockId::new_fn(&format!("step_{i}")))
            .collect();

        let seq = cp.create_sequence(n as usize, TimesModifier::Once);

        for (i, mock_id) in mock_ids.iter().enumerate() {
            let pred = cp.create_single::<u32>(mock_id, cond::<u32>(Box::new(|_| Ok(()))));
            cp.set_sequence_step::<u32, u32>(
                seq,
                i,
                mock_id,
                pred,
                Some(Box::new(move |x: u32| x + i as u32)),
            )
            .unwrap();
        }

        cp.finalize_sequences();
        cp.activate_sequence(seq).unwrap();

        // Call in correct order
        for (i, mock_id) in mock_ids.iter().enumerate() {
            let result = unsafe { cp.evaluate::<u32, u32>(mock_id, 10) };
            match result {
                Ok(Some(val)) => {
                    if val != 10 + i as u32 {
                        return TestResult::failed();
                    }
                }
                _ => return TestResult::failed(),
            }
        }

        TestResult::from_bool(cp.is_complete())
    }

    /// Calling step 1 when step 0 is expected fails.
    ///
    /// What this measures:
    ///   The sequence state machine must reject out-of-order calls. Specifically,
    ///   calling any step other than the currently expected one must produce an error.
    ///
    /// Steps:
    ///   1. Create a 2-step sequence.
    ///   2. Attempt to call step 1 first (expected: step 0).
    ///   3. Verify the call fails.
    ///
    /// Note: no random parameters — wrong-order rejection is a fixed invariant. Uses #[test].
    #[test]
    fn sequence_rejects_wrong_order() {
        let mock_a = MockId::new_fn("first");
        let mock_b = MockId::new_fn("second");
        let mut cp = Checkpoint::new();

        let pred_a = cp.create_single::<u32>(&mock_a, cond::<u32>(Box::new(|_| Ok(()))));
        let pred_b = cp.create_single::<u32>(&mock_b, cond::<u32>(Box::new(|_| Ok(()))));

        let seq = cp.create_sequence(2, TimesModifier::Once);
        cp.set_sequence_step::<u32, u32>(seq, 0, &mock_a, pred_a, Some(Box::new(|x| x)))
            .unwrap();
        cp.set_sequence_step::<u32, u32>(seq, 1, &mock_b, pred_b, Some(Box::new(|x| x)))
            .unwrap();

        cp.finalize_sequences();
        cp.activate_sequence(seq).unwrap();

        // Call mock_b first — should fail because sequence expects mock_a at step 0
        let result = unsafe { cp.evaluate::<u32, u32>(&mock_b, 5) };
        assert!(result.is_err(), "out-of-order sequence call should fail");
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  6. EVALUATE RETURN VALUE PROPERTIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Property: evaluate returns the value computed by the mock's return closure.
    ///
    /// What this measures:
    ///   The end-to-end pipeline: predicate matches → return closure executes →
    ///   caller receives the computed return value. We use an arithmetic closure
    ///   (input + offset) and verify the returned value matches direct computation.
    ///
    /// Steps:
    ///   1. Build a checkpoint with a return closure: |x| x + offset.
    ///   2. Evaluate with a random input.
    ///   3. Assert returned value == input + offset.
    #[quickcheck]
    fn evaluate_returns_computed_value(offset: u32, input: u32) -> TestResult {
        // Avoid overflow
        if (input as u64) + (offset as u64) > u32::MAX as u64 {
            return TestResult::discard();
        }

        let mock_id = MockId::new_fn("ret_val");
        let mut cp = Checkpoint::new();
        let pred = cp.create_single::<u32>(&mock_id, cond::<u32>(Box::new(|_| Ok(()))));
        let timed = cp.times_arena(pred, TimesModifier::Any);
        cp.expect::<u32, u32>(
            &mock_id,
            timed,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                move |x: u32| x + offset,
            ))),
        );

        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, input) };
        match result {
            Ok(Some(val)) => TestResult::from_bool(val == input + offset),
            _ => TestResult::failed(),
        }
    }

    /// Evaluate with a non-matching predicate returns Err.
    ///
    /// What this measures:
    ///   When the condition closure rejects the input, the evaluate call must
    ///   return an error — the mock should not fire. This confirms that predicate
    ///   evaluation is actually gating the return path, not just accepting everything.
    ///
    /// Steps:
    ///   1. Build a checkpoint where the condition rejects ALL inputs.
    ///   2. Evaluate with an arbitrary input.
    ///   3. Assert: result is Err.
    ///
    /// Note: the predicate rejects unconditionally, so the input value is irrelevant.
    #[test]
    fn evaluate_with_non_matching_predicate_fails() {
        let mock_id = MockId::new_fn("reject_all");
        let mut cp = Checkpoint::new();
        let pred = cp.create_single::<u32>(
            &mock_id,
            cond::<u32>(Box::new(|_| Err("always reject".into()))),
        );
        let timed = cp.times_arena(pred, TimesModifier::Any);
        cp.expect::<u32, u32>(
            &mock_id,
            timed,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );

        let result = unsafe { cp.evaluate::<u32, u32>(&mock_id, 42) };
        assert!(result.is_err());
    }

    /// Evaluate for an unknown mock_id returns an error (or None).
    ///
    /// What this measures:
    ///   If no expectations are registered for a given mock_id, evaluate must not
    ///   return a value. This prevents silent mock mismatches where a call goes
    ///   to the wrong mock.
    ///
    /// Steps:
    ///   1. Build a checkpoint with mock_id "registered".
    ///   2. Evaluate with mock_id "unknown".
    ///   3. Assert: result is not Ok(Some(_)).
    ///
    /// Note: the input value is irrelevant since the mock_id doesn't match any
    /// registered expectation.
    #[test]
    fn evaluate_unknown_mock_returns_none() {
        let registered = MockId::new_fn("registered");
        let unknown = MockId::new_fn("unknown");
        let mut cp = Checkpoint::new();

        let pred = cp.create_single::<u32>(&registered, cond::<u32>(Box::new(|_| Ok(()))));
        let timed = cp.times_arena(pred, TimesModifier::Any);
        cp.expect::<u32, u32>(
            &registered,
            timed,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                |x: u32| x,
            ))),
        );

        let result = unsafe { cp.evaluate::<u32, u32>(&unknown, 0) };
        assert!(
            !matches!(result, Ok(Some(_))),
            "unknown mock_id should not return a value"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  7. CHECKPOINT COMPLETION PROPERTIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Property: A checkpoint with n Times(1) expectations needs exactly n calls to complete.
    ///
    /// What this measures:
    ///   is_complete must track ALL expectations, not just one. With n independent
    ///   Once expectations (for different mock IDs), the checkpoint isn't complete
    ///   until all n have been satisfied.
    ///
    /// Steps:
    ///   1. Create n mock IDs with Once expectations (one per mock).
    ///   2. Satisfy them one at a time.
    ///   3. After each call, check: is_complete == (all n satisfied).
    ///   4. After all n calls, is_complete must be true.
    #[quickcheck]
    fn checkpoint_needs_all_expectations_satisfied(n: u32) -> TestResult {
        let n = (n % 10) + 1; // 1..=10

        let mut cp = Checkpoint::new();
        let mock_ids: Vec<MockId> = (0..n)
            .map(|i| MockId::new_fn(&format!("mock_{i}")))
            .collect();

        for mock_id in &mock_ids {
            let pred = cp.create_single::<u32>(mock_id, cond::<u32>(Box::new(|_| Ok(()))));
            let timed = cp.times_arena(pred, TimesModifier::Once);
            cp.expect::<u32, u32>(
                mock_id,
                timed,
                Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(
                    |x: u32| x,
                ))),
            );
        }

        // Before any calls: not complete
        if cp.is_complete() {
            return TestResult::failed();
        }

        // Satisfy one at a time
        for (i, mock_id) in mock_ids.iter().enumerate() {
            let _ = unsafe { cp.evaluate::<u32, u32>(mock_id, 0) };

            // Complete only after ALL are satisfied
            let should_be_complete = i == (n as usize - 1);
            if cp.is_complete() != should_be_complete {
                return TestResult::failed();
            }
        }

        TestResult::passed()
    }

    /// A checkpoint with only Any/AtMost/Never expectations is immediately complete.
    ///
    /// What this measures:
    ///   Modifiers with min=0 (Any, AtMost, Never) are satisfied at construction.
    ///   A checkpoint containing only such expectations should report is_complete()
    ///   = true without any calls.
    ///
    /// Steps:
    ///   1. Create a checkpoint with one Any and one AtMost(5) expectation.
    ///   2. Verify is_complete() is true before any evaluations.
    ///
    /// Note: AtMost(n) is always immediately complete regardless of n, so
    /// randomizing n adds no value.
    #[test]
    fn checkpoint_with_zero_minimum_modifiers_is_immediately_complete() {
        let mock_any = MockId::new_fn("any");
        let mock_atmost = MockId::new_fn("atmost");
        let mut cp = Checkpoint::new();

        let pred1 = cp.create_single::<u32>(&mock_any, cond::<u32>(Box::new(|_| Ok(()))));
        let timed1 = cp.times_arena(pred1, TimesModifier::Any);
        cp.expect::<u32, u32>(
            &mock_any,
            timed1,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(|x: u32| x))),
        );

        let pred2 = cp.create_single::<u32>(&mock_atmost, cond::<u32>(Box::new(|_| Ok(()))));
        let timed2 = cp.times_arena(pred2, TimesModifier::AtMost(5));
        cp.expect::<u32, u32>(
            &mock_atmost,
            timed2,
            Some(ReturnValDoublePointer::from_fn::<u32, u32>(Box::new(|x: u32| x))),
        );

        assert!(cp.is_complete());
    }
}
