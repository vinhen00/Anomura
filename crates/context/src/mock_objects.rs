//! MockObject and MockFunctionObject — the core mock dispatch structures.
//!
//! A `MockFunctionObject` holds expectations (ordered, with exhaustion/completeness)
//! and on_calls (fallback, no exhaustion tracking). Evaluation checks expectations first,
//! then falls back to on_calls.
//!
//! A `MockObject` groups `MockFunctionObject`s for all methods of an ADT instance.
//! It is keyed by the struct path + adt_mock_id.
//!
//! Standalone (free) functions are represented as bare `MockFunctionObject`s
//! stored directly in the checkpoint.

use crate::closure_wrappers::ReturnValDoublePointer;
use crate::mock::{AdtId, FnId};
use crate::new_expectations::Expectation;
use crate::{AdtIdNumber, ConditionDoublePointer, MockId, SequenceIdx, SequenceName};
use std::collections::HashMap;

// ─── OnCall ─────────────────────────────────────────────────────────────────

/// A fallback return-value binding with an optional condition.
/// Unlike expectations, on_calls are NOT checked for exhaustion or completeness.
/// They simply provide a return value when their condition matches.
pub struct OnCall {
    /// The condition predicate. If it matches the input, this on_call fires.
    pub condition: ConditionDoublePointer,
    /// The mock_id this on_call is associated with.
    pub mock_id: MockId,
    /// Return value closure, invoked when the condition matches.
    pub return_val: ReturnValDoublePointer,
}

impl OnCall {
    /// Evaluate this on_call against the given input.
    /// Returns `Ok(true)` if the condition matches, `Ok(false)` otherwise.
    ///
    /// # Safety
    /// The caller must ensure `Input` matches the type used when constructing the condition.
    pub unsafe fn matches<Input>(&self, input: &Input) -> bool {
        let condition = unsafe { self.condition.into_fn::<Input>() };
        condition(input).is_ok()
    }

    /// Extract the return value by invoking the return closure with the input.
    ///
    /// # Safety
    /// The caller must ensure `Input` and `ReturnVal` match the types used at construction.
    pub unsafe fn call<Input, ReturnVal>(&self, input: Input) -> ReturnVal {
        let ret_fn = unsafe { self.return_val.into_fn::<Input, ReturnVal>() };
        ret_fn(input)
    }
}

// ─── MockFunctionObject ─────────────────────────────────────────────────────

/// Holds all expectations and on_calls for a single mocked function or method.
///
/// Evaluation order:
/// 1. Walk expectations in order — find the first non-exhausted one whose predicate matches.
/// 2. If no expectation matches, walk on_calls in order — find the first whose condition matches.
/// 3. If nothing matches, return an error.
pub struct MockFunctionObject {
    /// The mock_id for this function (e.g. "fns_foo" or "fns_Foo_fallback").
    pub mock_id: FnId,
    /// Ordered expectations with exhaustion/completeness tracking.
    pub expectations: Vec<Expectation>,
    pub sequence: HashMap<SequenceName, (SequenceIdx, bool)>,
    /// Fallback on_calls — no exhaustion tracking.
    pub on_calls: Vec<OnCall>,
}

impl MockFunctionObject {
    pub fn new(mock_id: FnId) -> Self {
        Self {
            mock_id,
            expectations: Vec::new(),
            on_calls: Vec::new(),
            sequence: HashMap::new(),
        }
    }

    pub fn add_expectation(&mut self, expectation: Expectation) {
        self.expectations.push(expectation);
    }

    pub fn add_on_call(&mut self, on_call: OnCall) {
        self.on_calls.push(on_call);
    }

    /// Check if all expectations are complete (satisfied their cardinality requirements).
    pub fn is_complete(&self) -> bool {
        self.expectations.iter().all(|e| e.completed)
    }

    /// Return indices of expectations that are not yet completed.
    pub fn unsatisfied_expectations(&self) -> Vec<usize> {
        self.expectations
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.completed)
            .map(|(i, _)| i)
            .collect()
    }
}

// ─── MockObject ─────────────────────────────────────────────────────────────

/// Groups all instances and method of a Adt Object
/// Keyed by the struct path as String
pub struct MockObject {
    /// The base mock_id for this object (e.g. "fns_Foo" or "fns_MockStruct42").
    pub mock_id: AdtId,
    /// not instance specific
    pub static_methods: HashMap<FnId, MockFunctionObject>,
    pub instance_methods: HashMap<(AdtIdNumber, FnId), MockFunctionObject>,
    /// One MockFunctionObject per method, keyed by method name.
    pub methods: HashMap<String, MockFunctionObject>,
}

impl MockObject {
    pub fn new(mock_id: AdtId) -> Self {
        Self {
            mock_id,
            methods: HashMap::new(),
            static_methods: HashMap::new(),
            instance_methods: HashMap::new(),
        }
    }

    /// Get or create a MockFunctionObject for a method.
    pub fn get_or_create_method(
        &mut self,
        method_name: &str,
        method_mock_id: FnId,
    ) -> &mut MockFunctionObject {
        self.methods
            .entry(method_name.to_string())
            .or_insert_with(|| MockFunctionObject::new(method_mock_id))
    }

    /// Get a method's MockFunctionObject by name.
    pub fn get_method(&self, method_name: &str) -> Option<&MockFunctionObject> {
        self.methods.get(method_name)
    }

    /// Get a mutable reference to a method's MockFunctionObject by name.
    pub fn get_method_mut(&mut self, method_name: &str) -> Option<&mut MockFunctionObject> {
        self.methods.get_mut(method_name)
    }

    /// Check if all methods' expectations are complete.
    pub fn is_complete(&self) -> bool {
        self.methods.values().all(|m| m.is_complete())
    }
}
