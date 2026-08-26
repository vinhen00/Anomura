mod closure_wrappers;
pub mod errors;
mod mock;
pub mod mockable;
pub mod new_expectations;
pub mod time_mod;

#[cfg(test)]
mod unit_tests;
pub use crate::closure_wrappers::{ConditionDoublePointer, ReturnValDoublePointer};
pub use crate::mock::MockId;
use crate::mock::{MockHead, StrictnessKind};
pub use crate::new_expectations::Expectation;
pub use crate::new_expectations::{
    Checkpoint, CheckpointIndex, CheckpointName, GlobalContext, Predicate, PredicateIndex,
    PredicateKind, SequenceIdx, SequenceName, TimesModifier,
};
pub use errors::{MockError, PredicateError, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;

// ─── Context State Machine ──────────────────────────────────────────────────

/// The context lives in one of two states:
/// - `Building`: accepting mock registrations, expectations, sequences, checkpoints
/// - `Active`: ready to evaluate mock calls
/// - `Processing`: transient state during finish()
pub enum CtxState {
    Building(BuildingContext),
    Active(GlobalContext),
    Processing,
}

/// State during the build phase. Holds a `GlobalContext` being constructed.
/// When `finish()` is called, sequences are finalized and the context becomes Active.
pub struct BuildingContext {
    ctx: GlobalContext,
    /// Default return values for mocks, keyed by MockId.
    default_returns: HashMap<MockId, ReturnValDoublePointer>,
    adt_instance_counter: AdtMockId,
}

impl BuildingContext {
    pub fn new() -> Self {
        // Start with one default (unnamed) checkpoint
        let mut ctx = GlobalContext::new();
        ctx.add_checkpoint(Checkpoint::new());
        Self {
            ctx,
            default_returns: HashMap::new(),
            adt_instance_counter: AdtMockId::default(),
        }
    }

    /// Finalize: convert all sequence builders to sequences, transition to Active.
    pub fn finish(mut self) -> GlobalContext {
        // Finalize sequences in all checkpoints
        for cp in self.ctx.checkpoints_mut() {
            let _warnings = cp.finalize_sequences();
            // TODO: surface warnings to user
        }
        self.ctx
    }
}

impl Default for BuildingContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CtxState {
    pub fn finish(&mut self) {
        let old_self = mem::replace(self, CtxState::Processing);
        *self = match old_self {
            CtxState::Building(builder) => CtxState::Active(builder.finish()),
            _ => panic!("finish was called on state that is not Building"),
        }
    }
}

// ─── Thread-Local Global Context ────────────────────────────────────────────

thread_local! {
    pub static GLOBAL_CONTEXT: RefCell<CtxState> =
        RefCell::new(CtxState::Building(BuildingContext::new()));
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// Finalize the build phase:
/// the context becomes ready for evaluation.
pub fn finish_building_context() {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| {
        ctx.finish();
    });
}

/// Tear down the context and reset to a fresh build state.
pub fn teardown() {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| {
        *ctx = CtxState::Building(BuildingContext::new());
    });
}

/// Register a mock with an optional default return value.
/// Must be called during the build phase.
pub fn add_mock<Input, ReturnVal>(
    mock_id: MockId,
    default_return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
) -> Result<()> {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Building(builder) => {
            if builder.ctx.mocks().contains_key(&mock_id) {
                return Err(format!("mock {:?} registered twice", mock_id).into());
            }
            let default_ret =
                default_return_val_closure.map(|c| ReturnValDoublePointer::from_fn(c));
            let head = MockHead {
                default_return_val: default_ret.clone(),
                strictness: StrictnessKind::default(),
            };
            builder.ctx.register_mock(mock_id.clone(), head);
            if let Some(ret) = default_ret {
                builder.default_returns.insert(mock_id, ret);
            }
            Ok(())
        }
        _ => panic!("add_mock called outside of build phase"),
    })
}

/// Returns true if the context is in Active state and contains the given mock id.
pub fn ctx_built_and_contains_id(id: &MockId) -> bool {
    GLOBAL_CONTEXT.with_borrow(|ctx| match ctx {
        CtxState::Active(global_context) => global_context.mocks().contains_key(id),
        _ => false,
    })
}
#[derive(Clone, Copy, Debug, Default)]
pub struct AdtMockId(pub u64);

impl std::fmt::Display for AdtMockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
pub fn new_id() -> AdtMockId {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Building(global_context) => {
            let r = global_context.adt_instance_counter;
            global_context.adt_instance_counter.0 += 1;
            r
        }
        _ => panic!("add_mock called outside of build phase"),
    })
}

// ─── Checkpoint operations ──────────────────────────────────────────────────

/// Create a new named checkpoint. Expectations added after this call
/// go into this checkpoint (the latest one).
pub fn new_checkpoint(name: impl Into<CheckpointName>) -> Result<()> {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Building(builder) => {
            builder.ctx.add_named_checkpoint(name, Checkpoint::new())?;
            Ok(())
        }
        _ => panic!("new_checkpoint called outside of build phase"),
    })
}

/// Check that all top-level predicates in the current checkpoint have been fulfilled.
/// If yes, advance to the next checkpoint.
/// If no, return an error with details of what's unsatisfied.
pub fn control_checkpoint() -> Result<()> {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Active(global_context) => {
            let cp = global_context
                .active_checkpoint()
                .ok_or_else(|| MockError::from("no active checkpoint"))?;

            if !cp.is_complete() {
                let unsatisfied_exp = cp.unsatisfied_expectations();
                let unsatisfied_seq = cp.unsatisfied_sequences();
                return Err(format!(
                    "checkpoint not complete: {} unsatisfied expectations, {} unsatisfied sequences",
                    unsatisfied_exp.len(),
                    unsatisfied_seq.len(),
                )
                .into());
            }

            global_context.advance_checkpoint();
            Ok(())
        }
        _ => panic!("control_checkpoint called outside of active phase"),
    })
}

// ─── Sequence operations ────────────────────────────────────────────────────

/// Create a named sequence with a declared length in the specified (or latest) checkpoint.
pub fn new_sequence(
    name: impl Into<String>,
    size: usize,
    modifier: TimesModifier,
    checkpoint_name: Option<CheckpointName>,
) -> Result<()> {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Building(builder) => {
            let cp = resolve_or_latest_checkpoint_mut(&mut builder.ctx, checkpoint_name.as_ref())?;
            cp.create_named_sequence(name.into(), size, modifier)?;
            Ok(())
        }
        _ => panic!("new_sequence called outside of build phase"),
    })
}

/// Add an expectation to a specific position in a named sequence.
pub fn add_expectation_to_sequence<Input, ReturnVal>(
    mock_id: &MockId,
    condition: ConditionDoublePointer,
    return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
    sequence_name: impl Into<SequenceName>,
    sequence_index: usize,
    checkpoint_name: Option<CheckpointName>,
) -> Result<()> {
    let sequence_name = sequence_name.into();
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Building(builder) => {
            let cp = resolve_or_latest_checkpoint_mut(&mut builder.ctx, checkpoint_name.as_ref())?;

            // Create the predicate in the arena
            let pred_idx = cp.create_single::<Input>(mock_id, condition);

            // Resolve the sequence by name
            let seq_idx = cp
                .resolve_sequence_name(&sequence_name)
                .ok_or_else(|| format!("sequence '{}' not found", sequence_name.0))?;

            // Set the step at the given index
            cp.set_sequence_step::<Input, ReturnVal>(
                seq_idx,
                sequence_index,
                mock_id,
                pred_idx,
                return_val_closure,
            )?;

            Ok(())
        }
        _ => panic!("add_expectation_to_sequence called outside of build phase"),
    })
}

/// Activate a named sequence in the active checkpoint.
/// Must be called after `finish_building_context`.
/// The sequence will hijack all mocks that appear in its steps.
pub fn activate_sequence(sequence_name: impl Into<SequenceName>) -> Result<()> {
    let sequence_name = sequence_name.into();
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Active(global_context) => {
            let cp = global_context
                .active_checkpoint_mut()
                .ok_or_else(|| MockError::from("no active checkpoint"))?;

            let seq_idx = cp
                .resolve_sequence_name(&sequence_name)
                .ok_or_else(|| format!("sequence '{}' not found", sequence_name.0))?;

            cp.activate_sequence(seq_idx)
        }
        _ => panic!("activate_sequence called outside of active phase"),
    })
}

// ─── Expectation operations ─────────────────────────────────────────────────

/// Add an expectation to the specified (or latest) checkpoint.
/// Cardinality is wrapped into the predicate tree via a Times node.
pub fn add_expectation<Input, ReturnVal>(
    mock_id: &MockId,
    condition: ConditionDoublePointer,
    return_val_closure: Option<ReturnValDoublePointer>,
    checkpoint_name: Option<CheckpointName>,
    modifier: TimesModifier,
) -> Result<()> {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Building(builder) => {
            let cp = resolve_or_latest_checkpoint_mut(&mut builder.ctx, checkpoint_name.as_ref())?;

            // Create predicate in arena
            let pred_idx = cp.create_single::<Input>(mock_id, condition);

            // Wrap with cardinality in the predicate tree
            let timed_pred = cp.times_arena(pred_idx, modifier);

            // Commit it as an expectation (cardinality lives in the tree)
            cp.expect::<Input, ReturnVal>(mock_id, timed_pred, return_val_closure);

            Ok(())
        }
        _ => panic!("add_expectation called outside of build phase"),
    })
}

// ─── Mock execution ─────────────────────────────────────────────────────────

/// Execute a mock call. Evaluates the active checkpoint's expectations/sequences.
///
/// # Safety
/// This is unsafe because it relies on the caller ensuring that `Input` and `ReturnVal`
/// match the types used when registering expectations.
pub fn run_mock<Input, ReturnVal>(mock_id: MockId, input: Input) -> Result<ReturnVal> {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Active(global_context) => {
            // Check that mock is registered
            if !global_context.mocks().contains_key(&mock_id) {
                return Err(MockError::NoMatchingId);
            }

            let cp = global_context
                .active_checkpoint_mut()
                .ok_or_else(|| MockError::from("no active checkpoint"))?;

            // Safety: caller must ensure types match
            let result = unsafe { cp.evaluate::<Input, ReturnVal>(&mock_id, input) };

            match result {
                Ok(Some(ret)) => Ok(ret),
                Ok(None) => {
                    // No return value from expectation — try default
                    // Note: we can't easily access the default here without consuming input.
                    // For now, this is an error. The expectation should always provide a return.
                    Err(
                        "expectation matched but no return value provided and no default available"
                            .into(),
                    )
                }
                Err(e) => Err(e),
            }
        }
        _ => panic!("run_mock called outside of active phase"),
    })
}

// ─── Checkpoint queries ─────────────────────────────────────────────────────

/// Returns a reference to the latest (most recently added) checkpoint.
/// Works in both build and active phases.
///
/// Panics if no checkpoints exist (should never happen as one is created by default).
pub fn latest_checkpoint(f: impl FnOnce(&Checkpoint)) {
    GLOBAL_CONTEXT.with_borrow(|ctx| {
        let global = match ctx {
            CtxState::Building(builder) => &builder.ctx,
            CtxState::Active(global_context) => global_context,
            CtxState::Processing => panic!("latest_checkpoint called during processing"),
        };
        let cp = global.latest_checkpoint().expect("no checkpoints exist");
        f(cp);
    });
}

/// Returns a reference to a checkpoint looked up by name.
/// Works in both build and active phases.
///
/// Returns an error if no checkpoint with the given name exists.
pub fn checkpoint_by_name(name: &str, f: impl FnOnce(&Checkpoint)) -> Result<()> {
    GLOBAL_CONTEXT.with_borrow(|ctx| {
        let global = match ctx {
            CtxState::Building(builder) => &builder.ctx,
            CtxState::Active(global_context) => global_context,
            CtxState::Processing => panic!("checkpoint_by_name called during processing"),
        };
        let idx = global
            .resolve_checkpoint(name)
            .ok_or_else(|| MockError::from(format!("checkpoint '{}' not found", name)))?;
        let cp = global
            .get_checkpoint(idx)
            .ok_or_else(|| MockError::from(format!("checkpoint '{}' index invalid", name)))?;
        f(cp);
        Ok(())
    })
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Resolve a checkpoint by name, or return the latest (last) checkpoint.
fn resolve_or_latest_checkpoint_mut<'a>(
    ctx: &'a mut GlobalContext,
    name: Option<&CheckpointName>,
) -> Result<&'a mut Checkpoint> {
    match name {
        Some(cp_name) => {
            let idx = ctx
                .resolve_checkpoint(&cp_name.0)
                .ok_or_else(|| format!("checkpoint '{}' not found", cp_name.0))?;
            ctx.get_checkpoint_mut(idx)
                .ok_or_else(|| format!("checkpoint '{}' index invalid", cp_name.0).into())
        }
        None => ctx
            .latest_checkpoint_mut()
            .ok_or_else(|| "no checkpoints exist".into()),
    }
}

/// Mutable access to the latest checkpoint via a closure.
/// Works only in the build phase.
///
/// Panics if no checkpoints exist or if called outside the build phase.
pub fn latest_checkpoint_mut(f: impl FnOnce(&mut Checkpoint)) {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Building(builder) => {
            let cp = builder
                .ctx
                .latest_checkpoint_mut()
                .expect("no checkpoints exist");
            f(cp);
        }
        _ => panic!("latest_checkpoint_mut called outside of build phase"),
    });
}

/// Mutable access to the active checkpoint regardless of context phase.
/// In build phase, uses the latest checkpoint.
/// In active phase, uses the currently active checkpoint.
///
/// Intended for cleanup operations (e.g. Drop impls) that may run in either phase.
pub fn active_or_latest_checkpoint_mut(f: impl FnOnce(&mut Checkpoint)) {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Building(builder) => {
            let cp = builder
                .ctx
                .latest_checkpoint_mut()
                .expect("no checkpoints exist");
            f(cp);
        }
        CtxState::Active(global_context) => {
            let cp = global_context
                .active_checkpoint_mut()
                .expect("no active checkpoint");
            f(cp);
        }
        CtxState::Processing => panic!("cannot access checkpoint during processing"),
    });
}

/// Mutable access to a checkpoint looked up by name via a closure.
/// Works only in the build phase.
///
/// Panics if called outside the build phase.
/// Returns an error if no checkpoint with the given name exists.
pub fn checkpoint_by_name_mut(name: &str, f: impl FnOnce(&mut Checkpoint)) -> Result<()> {
    GLOBAL_CONTEXT.with_borrow_mut(|ctx| match ctx {
        CtxState::Building(builder) => {
            let idx = builder
                .ctx
                .resolve_checkpoint(name)
                .ok_or_else(|| MockError::from(format!("checkpoint '{}' not found", name)))?;
            let cp = builder
                .ctx
                .get_checkpoint_mut(idx)
                .ok_or_else(|| MockError::from(format!("checkpoint '{}' index invalid", name)))?;
            f(cp);
            Ok(())
        }
        _ => panic!("checkpoint_by_name_mut called outside of build phase"),
    })
}
