use std::collections::HashMap;

use crate::{
    ConditionDoublePointer, MockId, ReturnValDoublePointer,
    errors::{PredicateResult, Result},
    mock::MockHead,
};

// ─── Predicate Arena ────────────────────────────────────────────────────────

/// Typed index into the predicate arena. Cheap to copy and store in multiple places.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PredicateIndex(u32);

impl PredicateIndex {
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// Flat storage for all predicates within a checkpoint.
/// Slots may be `None` (tombstones) after predicates are consumed by combinators
/// via `take()`. Remaining predicates (those not consumed) are root-level or
/// serve as `After` dependency observation points.
pub struct PredicateArena {
    predicates: Vec<Option<Predicate>>,
}

impl PredicateArena {
    pub fn new() -> Self {
        Self {
            predicates: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            predicates: Vec::with_capacity(capacity),
        }
    }

    /// Inserts a predicate and returns its stable index.
    pub fn insert(&mut self, predicate: Predicate) -> PredicateIndex {
        let index = PredicateIndex(self.predicates.len() as u32);
        self.predicates.push(Some(predicate));
        index
    }

    /// Read-only access. Returns `None` if index is invalid or slot is a tombstone.
    pub fn get(&self, index: PredicateIndex) -> Option<&Predicate> {
        self.predicates.get(index.0 as usize)?.as_ref()
    }

    /// Mutable access. Returns `None` if index is invalid or slot is a tombstone.
    pub fn get_mut(&mut self, index: PredicateIndex) -> Option<&mut Predicate> {
        self.predicates.get_mut(index.0 as usize)?.as_mut()
    }

    /// Take ownership of a predicate, leaving a tombstone (`None`).
    /// Returns `None` if already taken or index is invalid.
    pub fn take(&mut self, index: PredicateIndex) -> Option<Predicate> {
        self.predicates.get_mut(index.0 as usize)?.take()
    }

    pub fn len(&self) -> usize {
        self.predicates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.predicates.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (PredicateIndex, &Predicate)> {
        self.predicates
            .iter()
            .enumerate()
            .filter_map(|(i, p)| p.as_ref().map(|pred| (PredicateIndex(i as u32), pred)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (PredicateIndex, &mut Predicate)> {
        self.predicates
            .iter_mut()
            .enumerate()
            .filter_map(|(i, p)| p.as_mut().map(|pred| (PredicateIndex(i as u32), pred)))
    }
}

// ─── Expectation / Predicate types ─────────────────────────────────────────

/// A leaf condition: checks input against a mock_id-specific closure.
/// Does NOT contain a return value — return values are attached at the root level only.
pub struct SingleExpectation {
    pub mock_id: MockId,
    pub condition: ConditionDoublePointer,
}

impl SingleExpectation {
    /// Safety: The caller must ensure that `Input` matches the type used
    /// when constructing the `ConditionDoublePointer`.
    pub unsafe fn check<Input>(&self, mock_id: &MockId, input: &Input) -> PredicateResult<()> {
        if self.mock_id != *mock_id {
            return Err("condition is not intended for this mock_id".into());
        }
        let condition = unsafe { self.condition.into_fn::<Input>() };
        condition(input)
    }
}

/// The structural/logical definition of a predicate.
/// Combinators and modifiers **own** their child predicates directly, forming a tree.
/// Only `After`'s dependency reference uses `(MockId, usize)` to observe an external
/// committed expectation's completion state.
///
/// Return values are NOT part of the predicate tree — they live exclusively
/// on `Expectation` (the committed top-level wrapper). This ensures that
/// predicates are pure conditions that can be freely composed.
pub enum PredicateKind {
    /// A leaf predicate that checks a single condition.
    Single(SingleExpectation),

    // ── N-ary logical combinators: own their children ────────────────────
    /// All children must be satisfied.
    And(Vec<Predicate>),
    /// At least one child must be satisfied (short-circuits on first match).
    Or(Vec<Predicate>),
    /// Exactly one child must be satisfied.
    Xor(Vec<Predicate>),
    /// Negation — the child predicate must NOT be satisfied.
    Not(Box<Predicate>),

    // ── Completion/ordering conditions ───────────────────────────────────
    /// This predicate is only active after the referenced expectation has completed.
    /// The dependency is identified by (MockId, expectation_index) — a reference to
    /// a committed top-level expectation, NOT an inner predicate node.
    /// The `then` predicate is owned.
    After {
        dependency: (MockId, usize),
        then: Box<Predicate>,
    },

    // ── Time/repetition modifiers: own the inner predicate ───────────────
    /// Wraps a predicate with a repetition/cardinality constraint.
    Times {
        inner: Box<Predicate>,
        modifier: TimesModifier,
    },
}

/// Specifies how many times a predicate must be satisfied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimesModifier {
    /// Exactly once.
    Once,
    /// Exactly N times.
    Times(u32),
    /// At least N times (inclusive).
    AtLeast(u32),
    /// At most N times (inclusive).
    AtMost(u32),
    /// Any number of times (zero or more).
    Any,
    /// Must never be satisfied (zero times).
    Never,
}

impl TimesModifier {
    /// Determine whether this modifier is exhausted given its accumulated count.
    ///
    /// Exhaustion is purely based on the modifier's own cap:
    /// - Once/Times(n)/AtMost(n): exhausted when count reaches the cap.
    /// - Never: unconditionally exhausted (no calls allowed).
    /// - Any/AtLeast: never exhausted (no upper bound).
    pub fn is_exhausted(&self, count: u32) -> bool {
        match self {
            TimesModifier::Once => count >= 1,
            TimesModifier::Times(n) => count >= *n,
            TimesModifier::AtMost(n) => count >= *n,
            TimesModifier::Never => true,
            TimesModifier::Any | TimesModifier::AtLeast(_) => false,
        }
    }
}

/// A predicate instance: structural kind + runtime state.
/// The kind defines *what* is checked; the state tracks *how it's progressing*.
pub struct Predicate {
    pub kind: PredicateKind,
    pub state: PredicateState,
}

/// Runtime state kept separate from structure so the same logical shape
/// can be reasoned about independently of execution progress.
pub struct PredicateState {
    /// Number of times this predicate has been successfully matched.
    pub call_count: u32,
    /// Whether this predicate has met its minimum requirement ("satisfied").
    /// A completed predicate may still accept further matches (e.g. AtLeast).
    pub completed: bool,
    /// Whether this predicate will not accept any more matches.
    /// An exhausted predicate gates further evaluation (returns false).
    pub exhausted: bool,
    /// For Or/Xor: index of the child that matched during the last eval_predicate call.
    /// Used by mark_matched to only recurse into the matched child.
    pub last_matched_child: Option<u32>,
}

impl PredicateState {
    pub fn new() -> Self {
        Self {
            call_count: 0,
            completed: false,
            exhausted: false,
            last_matched_child: None,
        }
    }

    /// Compute the correct initial state for a given predicate kind.
    /// Some modifiers (Any, AtMost, Never) and structural combinators (Not)
    /// have non-trivial initial completed/exhausted values.
    pub fn initial_for(kind: &mut PredicateKind) -> Self {
        let (completed, exhausted) = match kind {
            PredicateKind::Single(_) => (false, false),

            PredicateKind::And(children) => {
                // And is completed when ALL children are completed
                let completed = children.iter().all(|c| c.state.completed);
                let exhausted = children.iter().all(|c| c.state.exhausted);
                (completed, exhausted)
            }
            PredicateKind::Or(children) => {
                // Or is completed when ANY child is completed
                let completed = children.iter().any(|c| c.state.completed);
                let exhausted = children.iter().all(|c| c.state.exhausted);
                (completed, exhausted)
            }
            PredicateKind::Xor(children) => {
                // Xor is completed when exactly ONE child is completed
                let completed_count = children.iter().filter(|c| c.state.completed).count();
                let completed = completed_count == 1;
                let exhausted = children.iter().all(|c| c.state.exhausted);
                (completed, exhausted)
            }
            PredicateKind::Not(inner) => {
                // Not is completed when inner is NOT completed
                let completed = !inner.state.completed;
                let exhausted = inner.state.exhausted;
                (completed, exhausted)
            }

            PredicateKind::Times { inner, modifier } => {
                // Construction-time cycling: if the inner starts completed, we
                // fast-forward through cycles until either:
                //   (a) the outer's required count is met → completed + exhausted, or
                //   (b) the inner exhausts before that → failed (not completed + exhausted), or
                //   (c) the outer has no cap (Any/AtLeast) → completed, not exhausted.
                //
                // Examples:
                //   Times(3, Any(P)):       Any loops 3 times instantly → completed + exhausted.
                //   Times(3, AtMost(5, P)): AtMost loops 3 times (3 ≤ 5) → completed + exhausted.
                //   Times(5, AtMost(3, P)): AtMost exhausts at 3 < 5 → NOT completed + exhausted.
                //   AtLeast(2, Any(P)):     Any loops 2 times → completed, not exhausted.
                let mut call_count = 0u32;
                while inner.state.completed && !inner.state.exhausted {
                    call_count += 1;
                    // Did the outer reach its exhaustion cap?
                    if modifier.is_exhausted(call_count) {
                        break;
                    }
                    // For modifiers without an exhaustion cap (Any, AtLeast), stop
                    // once the completion threshold is reached — further cycling
                    // cannot change the outcome.
                    let completed_now = match &*modifier {
                        TimesModifier::AtLeast(n) => call_count >= *n,
                        TimesModifier::Any => true, // always completed
                        _ => false,                 // has a cap, handled above
                    };
                    if completed_now {
                        break;
                    }
                    // Advance the inner: count one cycle and recompute its state.
                    // This lets bounded inners (AtMost) track consumed capacity.
                    inner.state.call_count += 1;
                    let ic = inner.state.call_count;
                    if let PredicateKind::Times { modifier: ref m, .. } = inner.kind {
                        inner.state.completed = match m {
                            TimesModifier::Once => ic >= 1,
                            TimesModifier::Times(n) => ic >= *n,
                            TimesModifier::AtLeast(n) => ic >= *n,
                            TimesModifier::AtMost(_) => true,
                            TimesModifier::Any => true,
                            TimesModifier::Never => ic == 0,
                        };
                        inner.state.exhausted = m.is_exhausted(ic);
                    }
                }

                let completed = match &*modifier {
                    TimesModifier::Once => call_count >= 1,
                    TimesModifier::Times(n) => call_count >= *n,
                    TimesModifier::AtLeast(n) => call_count >= *n,
                    TimesModifier::AtMost(_) => true, // 0 <= any N, satisfied
                    TimesModifier::Any => true,       // no minimum
                    TimesModifier::Never => call_count == 0,
                };
                let exhausted = modifier.is_exhausted(call_count);
                return Self {
                    call_count,
                    completed,
                    exhausted,
                    last_matched_child: None,
                };
            }

            PredicateKind::After { then, .. } => {
                // After gates on dependency; initial state delegates to `then`
                // but the gate isn't open yet, so not completed.
                (false, false)
            }
        };

        Self {
            call_count: 0,
            completed,
            exhausted,
            last_matched_child: None,
        }
    }
}

impl Predicate {
    pub fn new(mut kind: PredicateKind) -> Self {
        let state = PredicateState::initial_for(&mut kind);
        Self { kind, state }
    }
}

// ─── Expectation ────────────────────────────────────────────────────────────

/// A committed top-level expectation: wraps a predicate tree with a return value.
/// This is the ONLY place return values exist.
///
/// Cardinality is NOT specified here — it must be part of the predicate tree
/// (via `Times` wrapping). The expectation delegates lifecycle queries to the
/// predicate's own state.
///
/// Created exclusively by `Checkpoint::expect()`.
pub struct Expectation {
    /// The predicate tree that must match for this expectation to fire.
    /// Controls its own cardinality and lifecycle internally via Times nodes.
    pub predicate: PredicateIndex,
    /// Optional return value closure, invoked when the predicate matches.
    /// Only expectations carry return values — inner predicates are pure conditions.
    pub return_val: Option<ReturnValDoublePointer>,
    /// Cached completion state — updated by mark_matched when the root predicate completes.
    /// Used by `After` dependencies to observe satisfaction without arena access.
    pub completed: bool,
    /// Cached exhaustion state — updated by mark_matched when the root predicate exhausts.
    pub exhausted: bool,
}

// ─── Named Identifiers ──────────────────────────────────────────────────────

/// A user-facing string identifier for a predicate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PredicateName(pub String);

impl<S: Into<String>> From<S> for PredicateName {
    fn from(s: S) -> Self {
        Self(s.into())
    }
}

/// A user-facing string identifier for a sequence.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceName(pub String);

impl<S: Into<String>> From<S> for SequenceName {
    fn from(s: S) -> Self {
        Self(s.into())
    }
}

/// A user-facing string identifier for a checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CheckpointName(pub String);

impl<S: Into<String>> From<S> for CheckpointName {
    fn from(s: S) -> Self {
        Self(s.into())
    }
}

/// Index into the GlobalContext's checkpoint list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CheckpointIndex(u32);

impl CheckpointIndex {
    pub fn raw(self) -> u32 {
        self.0
    }
}

// ─── Context structures ─────────────────────────────────────────────────────

pub struct GlobalContext {
    checkpoint_index: usize,
    checkpoints: Vec<Checkpoint>,
    /// Named checkpoint registry: look up checkpoints by user-assigned name.
    pub named_checkpoints: HashMap<CheckpointName, CheckpointIndex>,
    mocks: HashMap<MockId, MockHead>,
}

impl GlobalContext {
    pub fn new() -> Self {
        Self {
            checkpoint_index: 0,
            checkpoints: Vec::new(),
            named_checkpoints: HashMap::new(),
            mocks: HashMap::new(),
        }
    }

    /// Add a checkpoint and return its index.
    pub fn add_checkpoint(&mut self, checkpoint: Checkpoint) -> CheckpointIndex {
        let idx = CheckpointIndex(self.checkpoints.len() as u32);
        self.checkpoints.push(checkpoint);
        idx
    }

    /// Add a checkpoint with a name.
    pub fn add_named_checkpoint(
        &mut self,
        name: impl Into<CheckpointName>,
        checkpoint: Checkpoint,
    ) -> Result<CheckpointIndex> {
        let name = name.into();
        if self.named_checkpoints.contains_key(&name) {
            return Err(format!("checkpoint name '{}' is already defined", name.0).into());
        }
        let idx = self.add_checkpoint(checkpoint);
        self.named_checkpoints.insert(name, idx);
        Ok(idx)
    }

    /// Resolve a checkpoint by name.
    pub fn resolve_checkpoint(&self, name: &str) -> Option<CheckpointIndex> {
        self.named_checkpoints
            .get(&CheckpointName(name.to_owned()))
            .copied()
    }

    /// Get a reference to a checkpoint by index.
    pub fn get_checkpoint(&self, index: CheckpointIndex) -> Option<&Checkpoint> {
        self.checkpoints.get(index.0 as usize)
    }

    /// Get a mutable reference to a checkpoint by index.
    pub fn get_checkpoint_mut(&mut self, index: CheckpointIndex) -> Option<&mut Checkpoint> {
        self.checkpoints.get_mut(index.0 as usize)
    }

    /// Get the currently active checkpoint.
    pub fn active_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.get(self.checkpoint_index)
    }

    /// Get the currently active checkpoint mutably.
    pub fn active_checkpoint_mut(&mut self) -> Option<&mut Checkpoint> {
        self.checkpoints.get_mut(self.checkpoint_index)
    }

    /// Get the latest (last added) checkpoint mutably.
    pub fn latest_checkpoint_mut(&mut self) -> Option<&mut Checkpoint> {
        self.checkpoints.last_mut()
    }

    /// Get the latest (last added) checkpoint.
    pub fn latest_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }

    /// Advance to the next checkpoint. Returns false if already at the end.
    pub fn advance_checkpoint(&mut self) -> bool {
        if self.checkpoint_index + 1 < self.checkpoints.len() {
            self.checkpoint_index += 1;
            true
        } else {
            false
        }
    }

    /// Access the mocks registry.
    pub fn mocks(&self) -> &HashMap<MockId, MockHead> {
        &self.mocks
    }

    /// Register a mock in the global context.
    pub fn register_mock(&mut self, mock_id: MockId, head: MockHead) {
        self.mocks.insert(mock_id, head);
    }

    /// Mutable access to all checkpoints (for finalization).
    pub fn checkpoints_mut(&mut self) -> &mut Vec<Checkpoint> {
        &mut self.checkpoints
    }
}

pub struct Checkpoint {
    /// The arena holding all predicates for this checkpoint.
    /// Logical combinators reference other predicates by PredicateIndex into this arena.
    pub arena: PredicateArena,

    /// Named predicate registry: maps user-assigned names to arena indices.
    /// These are *not* directly evaluated at the top level — they exist so
    /// users can compose them into larger expressions before committing.
    ///
    /// Macro usage:
    /// ```ignore
    /// let a = condition_a;           // name a leaf
    /// let b = condition_b;
    /// let combined = And(a, b, c);   // name an N-ary composition
    /// expect(combined, once, return_val);
    /// ```
    pub named_predicates: HashMap<PredicateName, PredicateIndex>,

    /// Named sequence registry: maps user-assigned names to sequence indices.
    /// ```ignore
    /// let my_seq = sequence(once) { ... };
    /// ```
    pub named_sequences: HashMap<SequenceName, SequenceIdx>,

    /// Maps each mock to the expectations that are actively evaluated.
    /// Only expectations carry return values and cardinality modifiers.
    pub expectations: HashMap<MockId, Vec<Expectation>>,

    /// All sequences registered in this checkpoint (indexed by SequenceIdx).
    /// Populated by `finalize_sequences()` from the builders.
    pub sequences: Vec<Sequence>,

    /// Sequence builders — used during the build phase.
    /// Steps are assigned by index. Converted to `sequences` via `finalize_sequences()`.
    pub sequence_builders: Vec<SequenceBuilder>,

    /// Tracks which mocks are currently "hijacked" by an active sequence.
    /// When a mock appears in this map, `evaluate` must delegate to the
    /// sequence rather than checking normal expectations.
    pub hijacked_mocks: HashMap<MockId, SequenceIdx>,
}

impl Checkpoint {
    pub fn new() -> Self {
        Self {
            arena: PredicateArena::new(),
            named_predicates: HashMap::new(),
            named_sequences: HashMap::new(),
            expectations: HashMap::new(),
            sequences: Vec::new(),
            sequence_builders: Vec::new(),
            hijacked_mocks: HashMap::new(),
        }
    }

    // ─── Named predicate operations ─────────────────────────────────────

    /// Assign a name to any existing predicate index (leaf, combinator, or otherwise).
    /// This is the general-purpose "let" — it does NOT add the predicate to the
    /// top-level evaluation set, it merely makes it retrievable by name.
    pub fn name_predicate(
        &mut self,
        name: impl Into<PredicateName>,
        index: PredicateIndex,
    ) -> Result<PredicateIndex> {
        let name = name.into();
        if self.named_predicates.contains_key(&name) {
            return Err(format!("predicate name '{}' is already defined", name.0).into());
        }
        self.named_predicates.insert(name, index);
        Ok(index)
    }

    /// Look up a previously named predicate.
    pub fn resolve_predicate_name(&self, name: &PredicateName) -> Option<PredicateIndex> {
        self.named_predicates.get(name).copied()
    }

    /// Convenience: resolve predicate by str.
    pub fn resolve_predicate(&self, name: &str) -> Option<PredicateIndex> {
        self.named_predicates
            .get(&PredicateName(name.to_owned()))
            .copied()
    }

    // ─── Named sequence operations ──────────────────────────────────────

    /// Assign a name to a sequence so it can be referenced later.
    pub fn name_sequence(
        &mut self,
        name: impl Into<SequenceName>,
        index: SequenceIdx,
    ) -> Result<SequenceIdx> {
        let name = name.into();
        if self.named_sequences.contains_key(&name) {
            return Err(format!("sequence name '{}' is already defined", name.0).into());
        }
        self.named_sequences.insert(name, index);
        Ok(index)
    }

    /// Look up a previously named sequence.
    pub fn resolve_sequence_name(&self, name: &SequenceName) -> Option<SequenceIdx> {
        self.named_sequences.get(name).copied()
    }

    /// Convenience: resolve sequence by str.
    pub fn resolve_sequence(&self, name: &str) -> Option<SequenceIdx> {
        self.named_sequences
            .get(&SequenceName(name.to_owned()))
            .copied()
    }

    // ─── Committing predicates to evaluation ────────────────────────────

    /// Add a predicate to the top-level evaluation set for a mock, with an optional
    /// return value and a cardinality modifier.
    ///
    /// This is the ONLY way to attach a return value to a predicate tree.
    /// The predicate itself remains a pure condition in the arena.
    ///
    /// Macro equivalent: `expect(predicate) -> return_val;`
    ///
    /// Cardinality is NOT specified here — it must be part of the predicate tree.
    /// If you want `times(3)` semantics, wrap the predicate with `times()` before calling this.
    pub fn expect<Input, ReturnVal>(
        &mut self,
        mock_id: &MockId,
        predicate: PredicateIndex,
        return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
    ) {
        // Read initial state from the root predicate in the arena.
        let (completed, exhausted) = self
            .arena
            .get(predicate)
            .map(|p| (p.state.completed, p.state.exhausted))
            .unwrap_or((false, false));

        let root = Expectation {
            predicate,
            return_val: return_val_closure.map(|c| ReturnValDoublePointer::from_fn(c)),
            completed,
            exhausted,
        };
        self.expectations
            .entry(mock_id.clone())
            .or_default()
            .push(root);
    }

    // ─── Leaf predicate creation ────────────────────────────────────────

    /// Creates a leaf (Single) condition predicate in the arena and returns its index.
    /// Does NOT add it to the evaluation set or the named map — caller decides.
    pub fn create_single<Input>(
        &mut self,
        mock_id: &MockId,
        condition: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>,
    ) -> PredicateIndex {
        let single = SingleExpectation {
            mock_id: mock_id.clone(),
            condition: ConditionDoublePointer::from_fn(condition),
        };
        self.arena
            .insert(Predicate::new(PredicateKind::Single(single)))
    }

    /// Shorthand: create a leaf predicate and name it.
    pub fn create_named<Input>(
        &mut self,
        name: impl Into<PredicateName>,
        mock_id: &MockId,
        condition: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>,
    ) -> Result<PredicateIndex> {
        let index = self.create_single(mock_id, condition);
        self.name_predicate(name, index)
    }

    // ─── N-ary combinators ──────────────────────────────────────────────

    /// All predicates must be satisfied. Consumes operands from the arena.
    pub fn and(&mut self, operands: Vec<PredicateIndex>) -> PredicateIndex {
        let children: Vec<Predicate> = operands
            .into_iter()
            .map(|idx| {
                self.arena
                    .take(idx)
                    .expect("predicate already consumed or invalid")
            })
            .collect();
        let predicate = Predicate::new(PredicateKind::And(children));
        self.arena.insert(predicate)
    }

    /// At least one predicate must be satisfied. Consumes operands from the arena.
    pub fn or(&mut self, operands: Vec<PredicateIndex>) -> PredicateIndex {
        let children: Vec<Predicate> = operands
            .into_iter()
            .map(|idx| {
                self.arena
                    .take(idx)
                    .expect("predicate already consumed or invalid")
            })
            .collect();
        let predicate = Predicate::new(PredicateKind::Or(children));
        self.arena.insert(predicate)
    }

    /// Exactly one predicate must be satisfied. Consumes operands from the arena.
    pub fn xor(&mut self, operands: Vec<PredicateIndex>) -> PredicateIndex {
        let children: Vec<Predicate> = operands
            .into_iter()
            .map(|idx| {
                self.arena
                    .take(idx)
                    .expect("predicate already consumed or invalid")
            })
            .collect();
        let predicate = Predicate::new(PredicateKind::Xor(children));
        self.arena.insert(predicate)
    }

    /// Negate an existing predicate. Consumes the inner predicate from the arena.
    pub fn not(&mut self, inner: PredicateIndex) -> PredicateIndex {
        let child = self
            .arena
            .take(inner)
            .expect("predicate already consumed or invalid");
        let predicate = Predicate::new(PredicateKind::Not(Box::new(child)));
        self.arena.insert(predicate)
    }

    /// Create an ordering dependency: `then` only activates after the referenced
    /// expectation completes. The dependency is identified by (mock_id, expectation_index).
    /// Consumes the `then` predicate from the arena.
    pub fn after(&mut self, dependency: (MockId, usize), then: PredicateIndex) -> PredicateIndex {
        let then_pred = self
            .arena
            .take(then)
            .expect("predicate already consumed or invalid");
        let predicate = Predicate::new(PredicateKind::After {
            dependency,
            then: Box::new(then_pred),
        });
        self.arena.insert(predicate)
    }

    /// Wrap a predicate with a cardinality constraint (within the predicate tree).
    /// Consumes the inner predicate from the arena.
    pub fn times(&mut self, inner: PredicateIndex, modifier: TimesModifier) -> PredicateIndex {
        let child = self
            .arena
            .take(inner)
            .expect("predicate already consumed or invalid");
        let predicate = Predicate::new(PredicateKind::Times {
            inner: Box::new(child),
            modifier,
        });
        self.arena.insert(predicate)
    }

    // ─── Sequence operations ────────────────────────────────────────────

    /// Create a new sequence builder with a declared length and cardinality.
    /// Returns a `SequenceIdx` for referencing the builder.
    ///
    /// Steps are assigned to specific indices via `set_sequence_step()`.
    /// Call `finalize_sequences()` to convert all builders into finalized Sequences.
    pub fn create_sequence(&mut self, len: usize, modifier: TimesModifier) -> SequenceIdx {
        let idx = SequenceIdx(self.sequence_builders.len() as u32);
        self.sequence_builders
            .push(SequenceBuilder::new(len, modifier));
        idx
    }

    /// Shorthand: create a sequence builder and name it.
    pub fn create_named_sequence(
        &mut self,
        name: impl Into<SequenceName>,
        len: usize,
        modifier: TimesModifier,
    ) -> Result<SequenceIdx> {
        let idx = self.create_sequence(len, modifier);
        self.name_sequence(name, idx)
    }

    /// Set the entry predicate for a sequence builder.
    pub fn set_sequence_entry(
        &mut self,
        seq: SequenceIdx,
        entry_predicate: PredicateIndex,
    ) -> Result<()> {
        let builder = self
            .sequence_builders
            .get_mut(seq.0 as usize)
            .ok_or_else(|| format!("invalid sequence index {:?}", seq.raw()))?;
        builder.set_entry_predicate(entry_predicate);
        Ok(())
    }

    /// Assign a step to a specific index within a sequence builder.
    /// Consumes the predicate from the arena.
    /// Returns an error if the slot is already occupied or out of bounds.
    pub fn set_sequence_step<Input, ReturnVal>(
        &mut self,
        seq: SequenceIdx,
        index: usize,
        mock_id: &MockId,
        predicate: PredicateIndex,
        return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
    ) -> Result<()> {
        let pred = self.arena.take(predicate).ok_or_else(|| {
            format!(
                "predicate already consumed or invalid index {:?}",
                predicate.raw()
            )
        })?;
        let builder = self
            .sequence_builders
            .get_mut(seq.0 as usize)
            .ok_or_else(|| format!("invalid sequence index {:?}", seq.raw()))?;
        builder.set_step(index, mock_id, pred, return_val_closure)
    }

    /// Finalize all sequence builders into Sequences.
    /// Returns warnings for any sequences that had empty slots.
    /// After this call, sequences are ready for activation and evaluation.
    pub fn finalize_sequences(&mut self) -> Vec<(SequenceIdx, SequenceBuildWarning)> {
        let mut warnings = Vec::new();
        let builders = std::mem::take(&mut self.sequence_builders);

        for (i, builder) in builders.into_iter().enumerate() {
            let (sequence, warning) = builder.build();
            if let Some(w) = warning {
                warnings.push((SequenceIdx(i as u32), w));
            }
            self.sequences.push(sequence);
        }

        warnings
    }

    /// Explicitly activate a sequence, causing it to hijack evaluation for
    /// all mocks that appear in its steps.
    pub fn activate_sequence(&mut self, seq: SequenceIdx) -> Result<()> {
        let sequence = self
            .sequences
            .get_mut(seq.0 as usize)
            .ok_or_else(|| format!("invalid sequence index {:?}", seq.raw()))?;

        if sequence.is_active() {
            return Err(format!("sequence {:?} is already active", seq.raw()).into());
        }
        if sequence.steps.is_empty() {
            return Err(format!("sequence {:?} has no steps", seq.raw()).into());
        }

        sequence.activate();

        // Register all affected mocks as hijacked
        let affected: Vec<MockId> = sequence.steps.iter().map(|s| s.mock_id.clone()).collect();

        for mock_id in affected {
            self.hijacked_mocks.insert(mock_id, seq);
        }

        Ok(())
    }

    /// Deactivate a sequence and release all hijacked mocks.
    fn deactivate_sequence(&mut self, seq: SequenceIdx) {
        let Some(sequence) = self.sequences.get(seq.0 as usize) else {
            return;
        };

        let affected: Vec<MockId> = sequence.steps.iter().map(|s| s.mock_id.clone()).collect();

        for mock_id in &affected {
            self.hijacked_mocks.remove(mock_id);
        }
    }

    // ─── Evaluation ────────────────────────────────────────────────────

    /// Evaluate a mock call. If the mock is currently hijacked by a sequence,
    /// the sequence takes priority. Otherwise, normal expectations are checked.
    ///
    /// # Safety
    /// The caller must ensure that `Input` and `ReturnVal` match the types used
    /// when constructing the underlying closures.
    pub unsafe fn evaluate<Input, ReturnVal>(
        &mut self,
        mock_id: &MockId,
        input: Input,
    ) -> Result<Option<ReturnVal>> {
        // ─── Phase 1: Check if this mock is hijacked by an active sequence ──
        if let Some(&seq_idx) = self.hijacked_mocks.get(mock_id) {
            return unsafe { self.evaluate_sequence::<Input, ReturnVal>(seq_idx, mock_id, input) };
        }

        // ─── Phase 2: Normal expectation evaluation ─────────────────────────
        unsafe { self.evaluate_normal::<Input, ReturnVal>(mock_id, input) }
    }

    /// Evaluate a mock call against an active sequence.
    ///
    /// The current step must target this mock_id and its predicate must match.
    /// If successful, the step's return value is used and the sequence advances.
    /// If the sequence completes, hijacked mocks are released.
    ///
    /// # Safety
    /// Same type-safety requirements as `evaluate`.
    unsafe fn evaluate_sequence<Input, ReturnVal>(
        &mut self,
        seq_idx: SequenceIdx,
        mock_id: &MockId,
        input: Input,
    ) -> Result<Option<ReturnVal>> {
        let sequence = self
            .sequences
            .get(seq_idx.0 as usize)
            .ok_or_else(|| format!("invalid sequence index {:?}", seq_idx.raw()))?;

        let step = sequence.current_step().ok_or_else(|| {
            format!(
                "sequence {:?} is active but has no current step",
                seq_idx.raw()
            )
        })?;

        // Verify this step expects this mock
        if step.mock_id != *mock_id {
            return Err(format!(
                "sequence {:?} step {} expects mock {:?} but got {:?}",
                seq_idx.raw(),
                sequence.run_state.as_ref().unwrap().current_step,
                step.mock_id,
                mock_id,
            )
            .into());
        }

        // Evaluate the step's owned predicate directly
        // SAFETY: sequences and expectations are disjoint fields of Checkpoint.
        // We need &mut to sequences (for the predicate) and & to expectations (for After deps).
        let expectations_ptr: *const HashMap<MockId, Vec<Expectation>> =
            std::ptr::addr_of!(self.expectations);
        let sequence = self.sequences.get_mut(seq_idx.0 as usize).unwrap();
        let step_idx = sequence.run_state.as_ref().unwrap().current_step;
        let step_pred = &mut sequence.steps[step_idx].predicate;
        let matched = unsafe {
            Self::eval_predicate_ref::<Input>(step_pred, &*expectations_ptr, mock_id, &input)
        }?;

        if !matched {
            return Err(format!(
                "sequence {:?} step {}: predicate did not match for mock {:?}",
                seq_idx.raw(),
                self.sequences[seq_idx.0 as usize]
                    .run_state
                    .as_ref()
                    .unwrap()
                    .current_step,
                mock_id,
            )
            .into());
        }

        // Extract return value from the step
        let return_val = self.sequences[seq_idx.0 as usize]
            .current_step()
            .and_then(|s| s.return_val.as_ref())
            .map(|r| unsafe { r.into_fn::<Input, ReturnVal>() }(input));

        // Mark the owned predicate as matched
        let sequence = self.sequences.get_mut(seq_idx.0 as usize).unwrap();
        let state = sequence.run_state.as_ref().unwrap();
        let step_idx = state.current_step;
        Self::mark_matched_ref(&mut sequence.steps[step_idx].predicate);

        // Advance the sequence
        let still_active = self.sequences[seq_idx.0 as usize].advance();

        if !still_active {
            // Sequence iteration exhausted — release hijacked mocks
            self.deactivate_sequence(seq_idx);
        }

        Ok(return_val)
    }

    /// Normal evaluation path: check root expectations for a mock.
    ///
    /// Also checks if any matched expectation is a sequence entry predicate,
    /// triggering sequence activation if so.
    ///
    /// # Safety
    /// Same type-safety requirements as `evaluate`.
    unsafe fn evaluate_normal<Input, ReturnVal>(
        &mut self,
        mock_id: &MockId,
        input: Input,
    ) -> Result<Option<ReturnVal>> {
        let expectations = match self.expectations.get(mock_id) {
            Some(e) => e,
            None => {
                return Err(format!("no expectations registered for mock_id {:?}", mock_id).into());
            }
        };

        let candidate_indices: Vec<usize> = expectations
            .iter()
            .enumerate()
            .filter(|(_, root)| !self.is_expectation_exhausted(root))
            .map(|(i, _)| i)
            .collect();

        let mut errors: Vec<String> = Vec::new();

        for candidate_idx in candidate_indices {
            let predicate_idx = self.expectations[mock_id][candidate_idx].predicate;

            match unsafe { self.eval_predicate::<Input>(predicate_idx, mock_id, &input) } {
                Ok(true) => {
                    // Predicate matched — update arena state
                    self.mark_matched(predicate_idx);

                    // Sync cached state on the Expectation
                    if let Some(root_pred) = self.arena.get(predicate_idx) {
                        self.expectations.get_mut(mock_id).unwrap()[candidate_idx].completed =
                            root_pred.state.completed;
                        self.expectations.get_mut(mock_id).unwrap()[candidate_idx].exhausted =
                            root_pred.state.exhausted;
                    }

                    // Extract return value
                    let root = &self.expectations[mock_id][candidate_idx];
                    let return_val = root
                        .return_val
                        .as_ref()
                        .map(|r| unsafe { r.into_fn::<Input, ReturnVal>() }(input));

                    // Check if this predicate triggers any sequence activation
                    self.check_sequence_activation(predicate_idx);

                    return Ok(return_val);
                }
                Ok(false) => {
                    errors.push(format!(
                        "expectation[{}]: predicate did not match",
                        candidate_idx
                    ));
                }
                Err(e) => {
                    errors.push(format!("expectation[{}]: {}", candidate_idx, e));
                }
            }
        }

        Err(format!(
            "no valid expectation found for mock_id {:?}. Tried:\n  {}",
            mock_id,
            errors.join("\n  ")
        )
        .into())
    }

    /// After a predicate matches, check if it's an entry predicate for any sequence.
    /// If so, activate that sequence.
    fn check_sequence_activation(&mut self, matched_predicate: PredicateIndex) {
        // Find sequences whose entry_predicate matches what was just satisfied
        let to_activate: Vec<SequenceIdx> = self
            .sequences
            .iter()
            .enumerate()
            .filter(|(_, seq)| {
                !seq.is_active()
                    && !seq.is_exhausted()
                    && seq.entry_predicate == Some(matched_predicate)
            })
            .map(|(i, _)| SequenceIdx(i as u32))
            .collect();

        for seq_idx in to_activate {
            // Ignore errors here (e.g. empty sequences)
            let _ = self.activate_sequence(seq_idx);
        }
    }

    /// Recursively evaluate a predicate node. Returns `true` if the predicate matches.
    ///
    /// Entry point: looks up the root predicate in the arena, then delegates to
    /// `eval_predicate_ref` which walks owned children directly.
    ///
    /// # Safety
    /// The caller must ensure that `Input` matches the type used in condition closures.
    unsafe fn eval_predicate<Input>(
        &mut self,
        predicate_index: PredicateIndex,
        mock_id: &MockId,
        input: &Input,
    ) -> Result<bool> {
        // Split self into disjoint field borrows to satisfy the borrow checker.
        let predicate = self
            .arena
            .get_mut(predicate_index)
            .ok_or_else(|| format!("invalid predicate index {:?}", predicate_index.raw()))?;

        // SAFETY: `predicate` borrows self.arena mutably. We need &self.expectations
        // for After dependency checking. These are disjoint fields. We use a raw pointer
        // derived from the struct's field address to avoid the borrow checker complaining.
        let expectations_ptr: *const HashMap<MockId, Vec<Expectation>> =
            std::ptr::addr_of!(self.expectations);
        unsafe { Self::eval_predicate_ref::<Input>(predicate, &*expectations_ptr, mock_id, input) }
    }

    /// Recursively evaluate a predicate reference (walks owned children directly).
    /// Takes `&mut Predicate` so it can set `last_matched_child` for Or/Xor.
    ///
    /// # Safety
    /// The caller must ensure that `Input` matches the type used in condition closures.
    unsafe fn eval_predicate_ref<Input>(
        predicate: &mut Predicate,
        expectations: &HashMap<MockId, Vec<Expectation>>,
        mock_id: &MockId,
        input: &Input,
    ) -> Result<bool> {
        if predicate.state.exhausted {
            return Ok(false);
        }

        // Determine the variant tag without holding a borrow on kind.
        let is_or = matches!(&predicate.kind, PredicateKind::Or(_));
        let is_xor = matches!(&predicate.kind, PredicateKind::Xor(_));

        // Handle Or/Xor separately so we can set last_matched_child without borrow conflicts.
        if is_or {
            let len = match &predicate.kind {
                PredicateKind::Or(children) => children.len(),
                _ => unreachable!(),
            };
            for i in 0..len {
                let child = match &mut predicate.kind {
                    PredicateKind::Or(children) => &mut children[i],
                    _ => unreachable!(),
                };
                if unsafe {
                    Self::eval_predicate_ref::<Input>(child, expectations, mock_id, input)
                }? {
                    predicate.state.last_matched_child = Some(i as u32);
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        if is_xor {
            let len = match &predicate.kind {
                PredicateKind::Xor(children) => children.len(),
                _ => unreachable!(),
            };
            let mut match_count = 0u32;
            let mut matched_idx = 0u32;
            for i in 0..len {
                let child = match &mut predicate.kind {
                    PredicateKind::Xor(children) => &mut children[i],
                    _ => unreachable!(),
                };
                if unsafe {
                    Self::eval_predicate_ref::<Input>(child, expectations, mock_id, input)
                }? {
                    match_count += 1;
                    matched_idx = i as u32;
                    if match_count > 1 {
                        return Ok(false);
                    }
                }
            }
            if match_count == 1 {
                predicate.state.last_matched_child = Some(matched_idx);
                return Ok(true);
            } else {
                return Ok(false);
            }
        }

        match &mut predicate.kind {
            PredicateKind::Single(single) => {
                match unsafe { single.check::<Input>(mock_id, input) } {
                    Ok(()) => Ok(true),
                    Err(_) => Ok(false),
                }
            }

            PredicateKind::And(children) => {
                for child in children.iter_mut() {
                    if !unsafe {
                        Self::eval_predicate_ref::<Input>(child, expectations, mock_id, input)
                    }? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            PredicateKind::Not(inner) => {
                let inner_matched = unsafe {
                    Self::eval_predicate_ref::<Input>(inner, expectations, mock_id, input)
                }?;
                Ok(!inner_matched)
            }

            PredicateKind::After { dependency, then } => {
                let (ref dep_mock_id, dep_idx) = *dependency;
                let dep_completed = expectations
                    .get(dep_mock_id)
                    .and_then(|exps| exps.get(dep_idx))
                    .map(|exp| exp.completed)
                    .unwrap_or(false);
                if !dep_completed {
                    return Ok(false);
                }
                unsafe { Self::eval_predicate_ref::<Input>(then, expectations, mock_id, input) }
            }

            PredicateKind::Times { inner, modifier: _ } => unsafe {
                Self::eval_predicate_ref::<Input>(inner, expectations, mock_id, input)
            },

            // Or and Xor handled above
            PredicateKind::Or(_) | PredicateKind::Xor(_) => unreachable!(),
        }
    }

    /// After a successful match, walk the predicate tree and update call counts / completion.
    /// Entry point: looks up the root in the arena, then delegates to the recursive helper.
    fn mark_matched(&mut self, index: PredicateIndex) {
        let Some(predicate) = self.arena.get_mut(index) else {
            return;
        };

        Self::mark_matched_ref(predicate);
    }

    /// Recursively walk an owned predicate tree and update state after a successful match.
    fn mark_matched_ref(predicate: &mut Predicate) {
        // Times has special semantics: it only increments when inner completes.
        // Handle it separately before the generic call_count increment.
        if let PredicateKind::Times { inner, modifier } = &mut predicate.kind {
            // Recurse into inner first
            Self::mark_matched_ref(inner);

            // Check if inner completed a cycle (minimum met)
            if inner.state.completed {
                // Inner completed one cycle — increment Times counter and reset inner
                predicate.state.call_count += 1;

                let count = predicate.state.call_count;

                // completed = minimum requirement met
                predicate.state.completed = match modifier {
                    TimesModifier::Once => count >= 1,
                    TimesModifier::Times(n) => count >= *n,
                    TimesModifier::AtLeast(n) => count >= *n,
                    TimesModifier::AtMost(_) => true, // any count is valid
                    TimesModifier::Any => true,       // always satisfied
                    TimesModifier::Never => count == 0,
                };

                // exhausted = will not accept any more matches
                predicate.state.exhausted = modifier.is_exhausted(count);

                if !predicate.state.exhausted {
                    // Reset inner for the next cycle
                    Self::reset_predicate_ref(inner);
                }
            }
            return;
        }

        predicate.state.call_count += 1;

        match &mut predicate.kind {
            PredicateKind::Single(_) => {
                predicate.state.completed = true;
                predicate.state.exhausted = true;
            }
            PredicateKind::And(children) => {
                for child in children.iter_mut() {
                    Self::mark_matched_ref(child);
                }
                predicate.state.completed = children.iter().all(|c| c.state.completed);
                predicate.state.exhausted = children.iter().all(|c| c.state.exhausted);
            }
            PredicateKind::Or(children) => {
                // Or short-circuits — only the matched child should be marked.
                // We use last_matched_child (set during a pre-scan) or fall back to
                // marking the first completed child we find.
                if let Some(idx) = predicate.state.last_matched_child {
                    if let Some(child) = children.get_mut(idx as usize) {
                        Self::mark_matched_ref(child);
                    }
                } else {
                    // Fallback: mark all children (conservative but correct for most cases)
                    for child in children.iter_mut() {
                        Self::mark_matched_ref(child);
                    }
                }
                predicate.state.completed = children.iter().any(|c| c.state.completed);
                predicate.state.exhausted = children.iter().all(|c| c.state.exhausted);
            }
            PredicateKind::Xor(children) => {
                // Only the matched child should be marked
                if let Some(idx) = predicate.state.last_matched_child {
                    if let Some(child) = children.get_mut(idx as usize) {
                        Self::mark_matched_ref(child);
                    }
                } else {
                    for child in children.iter_mut() {
                        Self::mark_matched_ref(child);
                    }
                }
                let completed_count = children.iter().filter(|c| c.state.completed).count();
                predicate.state.completed = completed_count == 1;
                predicate.state.exhausted = children.iter().all(|c| c.state.exhausted);
            }
            PredicateKind::Not(inner) => {
                // Not doesn't mark inner as matched — inner was NOT matched.
                predicate.state.completed = !inner.state.completed;
                predicate.state.exhausted = inner.state.exhausted;
            }
            PredicateKind::After { then, .. } => {
                Self::mark_matched_ref(then);
                predicate.state.completed = then.state.completed;
                predicate.state.exhausted = then.state.exhausted;
            }
            PredicateKind::Times { .. } => unreachable!("handled above"),
        }
    }

    /// Recursively reset a predicate subtree's runtime state (call_count, completed, exhausted).
    /// Entry point: looks up root in the arena.
    /// Used by the outer Times node to reset its inner tree when starting a new cycle.
    fn reset_predicate(&mut self, index: PredicateIndex) {
        let Some(predicate) = self.arena.get_mut(index) else {
            return;
        };

        Self::reset_predicate_ref(predicate);
    }

    /// Recursively reset an owned predicate subtree's runtime state.
    /// Uses `PredicateState::initial_for` to compute correct initial values
    /// for each node (e.g., Any starts completed, Never starts exhausted).
    fn reset_predicate_ref(predicate: &mut Predicate) {
        // Reset children first so their state is correct when we compute our own.
        match &mut predicate.kind {
            PredicateKind::Single(_) => {}
            PredicateKind::And(children)
            | PredicateKind::Or(children)
            | PredicateKind::Xor(children) => {
                for child in children.iter_mut() {
                    Self::reset_predicate_ref(child);
                }
            }
            PredicateKind::Not(inner) => Self::reset_predicate_ref(inner),
            PredicateKind::After { then, .. } => Self::reset_predicate_ref(then),
            PredicateKind::Times { inner, .. } => Self::reset_predicate_ref(inner),
        }

        // Now compute our own initial state (which may depend on children's reset state).
        predicate.state = PredicateState::initial_for(&mut predicate.kind);
    }

    // ─── Queries ────────────────────────────────────────────────────────

    /// Check if an expectation's predicate tree is exhausted (will not accept more matches).
    /// Uses the cached state on the Expectation struct.
    fn is_expectation_exhausted(&self, expectation: &Expectation) -> bool {
        expectation.exhausted
    }

    /// Check if an expectation's predicate tree is completed (has met its minimum requirement).
    /// Uses the cached state on the Expectation struct.
    fn is_expectation_completed(&self, expectation: &Expectation) -> bool {
        expectation.completed
    }

    /// Check whether all expectations AND all sequences are satisfied.
    pub fn is_complete(&self) -> bool {
        // Check normal expectations
        for (_id, roots) in &self.expectations {
            for root in roots {
                if !self.is_expectation_completed(root) {
                    return false;
                }
            }
        }
        // Check sequences
        for seq in &self.sequences {
            if !seq.is_completed() {
                return false;
            }
        }
        true
    }

    /// Returns expectations that have NOT been satisfied yet.
    pub fn unsatisfied_expectations(&self) -> Vec<(&MockId, usize)> {
        let mut result = Vec::new();
        for (id, roots) in &self.expectations {
            for (i, root) in roots.iter().enumerate() {
                if !self.is_expectation_completed(root) {
                    result.push((id, i));
                }
            }
        }
        result
    }

    /// Returns sequence indices that have NOT been completed.
    pub fn unsatisfied_sequences(&self) -> Vec<SequenceIdx> {
        self.sequences
            .iter()
            .enumerate()
            .filter(|(_, seq)| !seq.is_completed())
            .map(|(i, _)| SequenceIdx(i as u32))
            .collect()
    }
}

// ─── Sequences ──────────────────────────────────────────────────────────────

/// Index into the checkpoint's sequence registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SequenceIdx(u32);

impl SequenceIdx {
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// A single step within a sequence. Each step targets a specific mock
/// and has a predicate that must match + an optional return value.
pub struct SequenceStep {
    /// Which mock this step expects a call on.
    pub mock_id: MockId,
    /// The owned predicate tree that the input must satisfy.
    pub predicate: Predicate,
    /// Optional return value for this step (each step can have its own).
    pub return_val: Option<ReturnValDoublePointer>,
}

/// Warning produced during sequence finalization when slots were left empty.
#[derive(Debug)]
pub struct SequenceBuildWarning {
    /// Indices of slots that were empty and removed during compaction.
    pub empty_slots: Vec<usize>,
    /// The declared length vs the actual occupied count.
    pub declared_len: usize,
    pub actual_len: usize,
}

/// Builder for constructing sequences with index-based slot assignment.
///
/// Created with a declared length. Steps are inserted at specific indices
/// (matching how the macro layer assigns positions). On `build()`, empty
/// slots produce a warning and are compacted out.
pub struct SequenceBuilder {
    /// Slot array — `None` means the position hasn't been assigned yet.
    slots: Vec<Option<SequenceStep>>,
    /// Optional entry predicate that triggers activation.
    pub entry_predicate: Option<PredicateIndex>,
    /// Cardinality for the entire sequence.
    pub modifier: TimesModifier,
}

impl SequenceBuilder {
    /// Create a new builder with a declared number of slots.
    pub fn new(len: usize, modifier: TimesModifier) -> Self {
        let mut slots = Vec::with_capacity(len);
        slots.resize_with(len, || None);
        Self {
            slots,
            entry_predicate: None,
            modifier,
        }
    }

    /// Insert a step at a specific index.
    /// Returns an error if the slot is already occupied.
    /// Takes an owned `Predicate` (consumed from the arena by the caller).
    pub fn set_step<Input, ReturnVal>(
        &mut self,
        index: usize,
        mock_id: &MockId,
        predicate: Predicate,
        return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
    ) -> Result<()> {
        if index >= self.slots.len() {
            return Err(format!(
                "sequence slot index {} out of bounds (declared length: {})",
                index,
                self.slots.len()
            )
            .into());
        }
        if self.slots[index].is_some() {
            return Err(format!("sequence slot {} is already occupied", index).into());
        }
        self.slots[index] = Some(SequenceStep {
            mock_id: mock_id.clone(),
            predicate,
            return_val: return_val_closure.map(|c| ReturnValDoublePointer::from_fn(c)),
        });
        Ok(())
    }

    /// Set the entry predicate for automatic activation.
    pub fn set_entry_predicate(&mut self, predicate: PredicateIndex) {
        self.entry_predicate = Some(predicate);
    }

    /// How many slots are currently occupied.
    pub fn occupied_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// How many slots are still empty.
    pub fn empty_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_none()).count()
    }

    /// Finalize the builder into a `Sequence`.
    ///
    /// Empty slots are removed (compacted) and a warning is returned if any
    /// were present. The resulting `Sequence` contains only the occupied steps
    /// in their original relative order.
    pub fn build(self) -> (Sequence, Option<SequenceBuildWarning>) {
        let declared_len = self.slots.len();
        let empty_slots: Vec<usize> = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_none())
            .map(|(i, _)| i)
            .collect();

        let steps: Vec<SequenceStep> = self.slots.into_iter().flatten().collect();

        let warning = if !empty_slots.is_empty() {
            Some(SequenceBuildWarning {
                actual_len: steps.len(),
                declared_len,
                empty_slots,
            })
        } else {
            None
        };

        let sequence = Sequence {
            steps,
            entry_predicate: self.entry_predicate,
            modifier: self.modifier,
            run_state: None,
        };

        (sequence, warning)
    }
}

/// Runtime state for a single iteration of a sequence.
pub struct SequenceRunState {
    /// Index of the current step within the sequence (0-based).
    pub current_step: usize,
    /// How many full iterations have been completed.
    pub iterations_completed: u32,
}

impl SequenceRunState {
    pub fn new() -> Self {
        Self {
            current_step: 0,
            iterations_completed: 0,
        }
    }
}

/// A finalized sequence: an ordered list of steps that must be matched in order,
/// potentially across multiple different mocks.
///
/// Constructed from a `SequenceBuilder` via `build()`.
///
/// Activation: when the `entry_predicate` (if set) is matched, the sequence
/// becomes active and "hijacks" evaluation for all mocks in `affected_mocks()`.
/// While active, those mocks MUST follow the sequence order or evaluation fails.
///
/// Cardinality: the sequence can be expected to run N times via its `modifier`.
pub struct Sequence {
    /// Ordered steps that must be matched sequentially.
    pub steps: Vec<SequenceStep>,
    /// Optional entry predicate — when this predicate is matched (from normal
    /// evaluation), the sequence activates. If `None`, the sequence must be
    /// activated explicitly.
    pub entry_predicate: Option<PredicateIndex>,
    /// Cardinality for the entire sequence (how many full iterations).
    pub modifier: TimesModifier,
    /// Runtime state — `None` if the sequence has not been activated yet.
    pub run_state: Option<SequenceRunState>,
}

impl Sequence {
    /// All mock_ids that participate in this sequence.
    pub fn affected_mocks(&self) -> Vec<&MockId> {
        let mut mocks: Vec<&MockId> = self.steps.iter().map(|s| &s.mock_id).collect();
        mocks.dedup();
        mocks
    }

    /// Is this sequence currently active (hijacking evaluation)?
    pub fn is_active(&self) -> bool {
        self.run_state.is_some()
    }

    /// Is this sequence fully completed (all iterations done per modifier)?
    pub fn is_completed(&self) -> bool {
        match &self.run_state {
            None => false,
            Some(state) => {
                let done = state.iterations_completed;
                match &self.modifier {
                    TimesModifier::Once => done >= 1,
                    TimesModifier::Times(n) => done >= *n,
                    TimesModifier::AtLeast(n) => done >= *n,
                    TimesModifier::AtMost(_) => true,
                    TimesModifier::Any => true,
                    TimesModifier::Never => done == 0,
                }
            }
        }
    }

    /// Is the cardinality exhausted (no more iterations allowed)?
    pub fn is_exhausted(&self) -> bool {
        match &self.run_state {
            None => false,
            Some(state) => {
                let done = state.iterations_completed;
                match &self.modifier {
                    TimesModifier::Once => done >= 1,
                    TimesModifier::Times(n) => done >= *n,
                    TimesModifier::AtMost(n) => done >= *n,
                    TimesModifier::Never => true,
                    TimesModifier::Any | TimesModifier::AtLeast(_) => false,
                }
            }
        }
    }

    /// Get the current step, if the sequence is active and not finished.
    pub fn current_step(&self) -> Option<&SequenceStep> {
        let state = self.run_state.as_ref()?;
        self.steps.get(state.current_step)
    }

    /// Advance to the next step. If we've completed all steps, increment
    /// iterations_completed and reset to step 0 (if cardinality allows).
    /// Returns `true` if the sequence is still active after advancing.
    pub fn advance(&mut self) -> bool {
        let Some(state) = self.run_state.as_mut() else {
            return false;
        };

        state.current_step += 1;

        // Check if we've completed one full iteration
        if state.current_step >= self.steps.len() {
            state.iterations_completed += 1;
            drop(state);

            if self.is_exhausted() {
                return false;
            }

            // Reset for next iteration
            self.run_state.as_mut().unwrap().current_step = 0;
        }

        true
    }

    /// Activate this sequence (begin execution from step 0).
    pub fn activate(&mut self) {
        self.run_state = Some(SequenceRunState::new());
    }
}
