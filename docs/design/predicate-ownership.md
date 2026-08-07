# Predicate Ownership Refactor

## Summary

Refactor the predicate system so that combinators and modifiers **own** their child
predicates rather than referencing them by index into a shared arena. This eliminates
shared mutable state between nested predicates, allowing modifiers (e.g. `Times`) to
freely manage the lifecycle of their inner predicates without corrupting sibling state.

Additionally, split `PredicateState` into distinct `completed` and `exhausted` fields
to properly model predicates that have met their minimum requirement but can still
accept additional matches.

---

## Motivation

In the current design, all predicates live in a flat arena and reference each other via
`PredicateIndex`. This causes problems:

- `mark_matched` walks a shared tree and mutates shared state — a modifier on one branch
  can inadvertently affect another branch that references the same node.
- `Times` has to avoid recursing into children to prevent marking inner `Single` predicates
  as completed, preventing re-evaluation. This is fragile.
- `reset_predicate` must be carefully scoped to avoid resetting state that other predicates
  depend on.
- `completed` is currently a single bool that conflates "minimum met" with "maximum reached",
  making it impossible for `After` to observe satisfaction without blocking further matches.

---

## Design

### 1. PredicateState: completed vs exhausted

```rust
pub struct PredicateState {
    /// Number of times this predicate has been successfully matched.
    pub call_count: u32,
    /// Minimum cardinality met — the predicate is "satisfied".
    /// Used by `After` to decide whether its dependency condition is met.
    /// A predicate can be completed but still accept more matches.
    pub completed: bool,
    /// Maximum cardinality reached — no more matches allowed.
    /// Used by evaluation to skip this predicate entirely.
    pub exhausted: bool,
}

impl PredicateState {
    pub fn new() -> Self {
        Self { call_count: 0, completed: false, exhausted: false }
    }

    pub fn is_completed(&self) -> bool { self.completed }
    pub fn is_exhausted(&self) -> bool { self.exhausted }
}
```

#### State transitions

| State | Meaning | Example |
|-------|---------|---------|
| `!completed && !exhausted` | Still working toward minimum requirement | `Times(3)` after 1 match |
| `completed && !exhausted` | Minimum met, can still accept more | `AtLeast(2)` after 2 matches; `Any` always |
| `completed && exhausted` | Done — will not match anymore | `Once` after 1 match; `Times(3)` after 3 |
| `!completed && exhausted` | Error state — over-matched a `Never` | `Never` that somehow got a match |

#### How completed/exhausted are computed per TimesModifier

```rust
fn update_state(state: &mut PredicateState, modifier: &TimesModifier) {
    let count = state.call_count;
    state.completed = match modifier {
        TimesModifier::Once      => count >= 1,
        TimesModifier::Times(n)  => count >= *n,
        TimesModifier::AtLeast(n)=> count >= *n,
        TimesModifier::AtMost(_) => true,   // any count in range is valid
        TimesModifier::Any       => true,   // always satisfied
        TimesModifier::Never     => count == 0,
    };
    state.exhausted = match modifier {
        TimesModifier::Once      => count >= 1,
        TimesModifier::Times(n)  => count >= *n,
        TimesModifier::AtMost(n) => count >= *n,
        TimesModifier::Never     => true,   // never allows any matches
        TimesModifier::Any | TimesModifier::AtLeast(_) => false,
    };
}
```

#### How completed/exhausted propagate per PredicateKind

| Variant | `completed` when... | `exhausted` when... |
|---------|---------------------|---------------------|
| `Single` | matched at least once (`call_count >= 1`) | same as completed (single is always "once") |
| `And(children)` | ALL children are completed | ALL children are exhausted |
| `Or(children)` | ANY child is completed | ALL children are exhausted |
| `Xor(children)` | exactly one child is completed | ALL children are exhausted |
| `Not(inner)` | inner is NOT completed (negation of semantics) | inner is exhausted (can't try anymore) |
| `Times { inner, modifier }` | see TimesModifier table above (uses own `call_count`) | see TimesModifier table above |
| `After { dependency, then }` | `then` is completed (dependency is just a gate) | `then` is exhausted |

**Key insight for `Times`:** The `Times` node's `call_count` tracks how many times the
*entire inner subtree* has been satisfied (i.e., how many "cycles" have completed).
When the inner predicate becomes **completed** after a match, `Times` resets the inner
tree and increments its own `call_count`. The trigger is completion (minimum met), not
exhaustion. This is what makes `Times(3, Times(2, single))` equivalent to
`Times(6, single)`, and avoids breaking modifiers that never exhaust (`Any`, `AtLeast`).

---

### 2. PredicateName as a lightweight handle

`PredicateName` remains a simple string identifier. It maps to a slot in the arena.
Predicates are created in the arena and assigned a name. When a combinator consumes a
predicate, the name is removed from the registry and the predicate is **taken** from the
arena (replaced with a tombstone).

Users cannot directly reference predicates by index — only by name. This ensures the
ownership model is enforceable: once a name is consumed, it cannot be used again.

---

### 3. Owned predicate tree

```rust
pub enum PredicateKind {
    /// Leaf — checks a single condition against a specific mock.
    Single(SingleExpectation),

    // ── Combinators: own their children ──────────────────────────────
    /// All children must match.
    And(Vec<Predicate>),
    /// At least one child must match.
    Or(Vec<Predicate>),
    /// Exactly one child must match.
    Xor(Vec<Predicate>),
    /// Child must NOT match.
    Not(Box<Predicate>),

    // ── Modifiers: own the inner predicate ───────────────────────────
    /// Cardinality constraint on the inner predicate.
    Times { inner: Box<Predicate>, modifier: TimesModifier },

    // ── Ordering: dependency is a reference, "then" is owned ─────────
    /// Active only after the dependency predicate has completed.
    /// `dependency` stays in the arena (read-only observation of `completed`).
    /// `then` is owned — this predicate controls its lifecycle.
    After { dependency: PredicateIndex, then: Box<Predicate> },
}
```

---

### 4. Arena with tombstones

```rust
pub struct PredicateArena {
    predicates: Vec<Option<Predicate>>,
}

impl PredicateArena {
    /// Insert a predicate, returning its index.
    pub fn insert(&mut self, predicate: Predicate) -> PredicateIndex {
        let index = PredicateIndex(self.predicates.len() as u32);
        self.predicates.push(Some(predicate));
        index
    }

    /// Read-only access (for `After` dependency checks).
    pub fn get(&self, index: PredicateIndex) -> Option<&Predicate> {
        self.predicates.get(index.0 as usize)?.as_ref()
    }

    /// Mutable access (for updating state of `After` dependencies).
    pub fn get_mut(&mut self, index: PredicateIndex) -> Option<&mut Predicate> {
        self.predicates.get_mut(index.0 as usize)?.as_mut()
    }

    /// Take ownership of a predicate, leaving a tombstone (None).
    /// Returns None if already taken or index is invalid.
    pub fn take(&mut self, index: PredicateIndex) -> Option<Predicate> {
        self.predicates.get_mut(index.0 as usize)?.take()
    }
}
```

---

### 5. Cloning predicates

Cloning is an explicit operation on the checkpoint:

```rust
impl Checkpoint {
    /// Deep-clone a named predicate and insert the copy into the arena
    /// under a new name. The original remains untouched.
    /// The clone has fresh PredicateState (zeroed call_count, not completed, not exhausted).
    pub fn clone_predicate(
        &mut self,
        source_name: &str,
        new_name: impl Into<PredicateName>,
    ) -> Result<PredicateIndex> { ... }
}
```

This creates a fully independent copy — separate state, separate lifecycle. The user
must explicitly opt in to duplication. By default, composition consumes.

---

### 6. Consuming API

```rust
impl Checkpoint {
    /// Take a predicate out of the arena by name. The name becomes invalid after this.
    pub fn take_predicate(&mut self, name: &str) -> Result<Predicate> {
        let index = self.resolve_predicate(name)
            .ok_or("predicate not found")?;
        self.named_predicates.remove(&PredicateName(name.to_owned()));
        self.arena.take(index)
            .ok_or("predicate already consumed")
    }

    // ── Combinators: consume their operands by name ─────────────────

    /// All operands must match. Consumes operands from the arena by name.
    pub fn and_owned(&mut self, names: Vec<&str>) -> Result<PredicateIndex> {
        let children: Vec<Predicate> = names.iter()
            .map(|n| self.take_predicate(n))
            .collect::<Result<_>>()?;
        let predicate = Predicate::new(PredicateKind::And(children));
        Ok(self.arena.insert(predicate))
    }

    /// At least one operand must match. Consumes operands by name.
    pub fn or_owned(&mut self, names: Vec<&str>) -> Result<PredicateIndex> { ... }

    /// Exactly one operand must match. Consumes operands by name.
    pub fn xor_owned(&mut self, names: Vec<&str>) -> Result<PredicateIndex> { ... }

    /// Negate. Consumes the inner predicate by name.
    pub fn not_owned(&mut self, name: &str) -> Result<PredicateIndex> { ... }

    /// Cardinality constraint. Consumes the inner predicate by name.
    pub fn times_owned(
        &mut self,
        name: &str,
        modifier: TimesModifier,
    ) -> Result<PredicateIndex> { ... }

    /// Ordering constraint. Does NOT consume the dependency (read-only reference).
    /// Consumes the "then" predicate by name.
    pub fn after_owned(
        &mut self,
        dependency_name: &str,
        then_name: &str,
    ) -> Result<PredicateIndex> { ... }
}
```

---

### 7. After dependency semantics

`After` does NOT reference its dependency via `PredicateIndex` into the arena.
Instead, it references a committed top-level **expectation** by `(MockId, usize)`:

```rust
PredicateKind::After {
    dependency: (MockId, usize),  // (mock_id, expectation_index)
    then: PredicateIndex,
}
```

This design choice ensures:
- `After` can only observe top-level committed expectations, never inner nodes
  buried inside another predicate tree (which may be reset by a parent `Times` node).
- Multiple `After` predicates can observe the same dependency expectation.
- The dependency expectation is evaluated independently as its own commitment.
- `After` checks completion via `Checkpoint::is_expectation_completed()`, which
  reads the root predicate's `state.completed` from the arena.

The dependency expectation must be committed (via `expect()`) **before** the `After`
node is created — the index must already exist. This is a natural ordering constraint
that mirrors semantic intent: "B happens after A" requires A to be defined first.

```rust
impl Checkpoint {
    pub fn after(&mut self, dependency: (MockId, usize), then: PredicateIndex) -> PredicateIndex {
        let predicate = Predicate::new(PredicateKind::After { dependency, then });
        self.arena.insert(predicate)
    }
}
```

---

### 8. Sequences take ownership

Sequence steps own their predicates directly:

```rust
pub struct SequenceStep {
    pub mock_id: MockId,
    /// Owned predicate tree for this step (consumed from the arena on build).
    pub predicate: Predicate,
    /// Optional return value for this step.
    pub return_val: Option<ReturnValDoublePointer>,
}
```

The builder's `set_step` method consumes the predicate by name, same pattern as
combinators.

---

### 9. Times: inner reset cycle

When a `Times` node wraps another predicate (including nested `Times`), the lifecycle is:

1. Evaluate inner predicate — if it matches, mark it matched.
2. After marking, check if the inner predicate is now **completed** (minimum met).
3. If inner is completed:
   a. Increment `Times` node's own `call_count`.
   b. Reset the entire inner subtree (zero `call_count`, clear `completed`/`exhausted`).
   c. Recompute `Times`'s own `completed`/`exhausted` from its modifier.
4. If `Times` itself is now exhausted, stop accepting matches.

The trigger is **completion**, not exhaustion. This is important because:
- It allows `Times(n, AtLeast(m, P))` to cycle after `m` matches per round.
- Exhaustion-based cycling would break modifiers that never exhaust (e.g. `Any`, `AtLeast`).
- Completion universally means "minimum requirement met" — exactly when a parent should
  consider one cycle done.

This makes `Times(3, Times(2, single))` behave as:
- Inner `Times(2)` completes after 2 matches.
- Outer `Times(3)` resets the inner tree and cycles 3 times.
- Total: 2 × 3 = 6 matches.

Note on degenerate and nested cases:

When a `Times` node wraps an inner predicate that **starts completed** (e.g. `Any`,
`AtMost`), the cycling logic runs at construction time (`PredicateState::initial_for`).
This "fast-forwards" through iterations without requiring real matches:

- The outer loops while `inner.state.completed && !inner.state.exhausted`.
- Each iteration increments the inner's own `call_count` (tracking consumed capacity)
  and recomputes its state. This lets bounded inners (like `AtMost(m)`) properly exhaust.
- The loop breaks when:
  (a) the outer's modifier is exhausted (cap reached), or
  (b) the outer's modifier is completed for unbounded modifiers (`Any`, `AtLeast`), or
  (c) the inner exhausts (ran out of capacity).

Concrete nesting behaviors:

| Outer | Inner | At birth | Runtime | Total calls |
|-------|-------|----------|---------|-------------|
| `Times(n)` | `Any(P)` | completed + exhausted | 0 calls | 0 |
| `Times(n)` | `AtMost(m)` (n≤m) | completed + exhausted | 0 calls | 0 |
| `Times(n)` | `AtMost(m)` (n>m) | NOT completed + not exhausted | fails (inner stuck) | 0 |
| `Times(n)` | `AtLeast(m)` | not completed | exhausts at n×m | n×m |
| `Times(n)` | `Times(m)` | not completed | exhausts at n×m | n×m |
| `AtLeast(n)` | `Any(P)` | completed, not exhausted | unlimited | ∞ |
| `AtLeast(n)` | `AtMost(m)` (n≤m) | completed, not exhausted | unlimited | ∞ |
| `AtLeast(n)` | `Times(m)` | not completed | unlimited after n×m | ∞ |
| `AtMost(n)` | `Times(m)` | completed, not exhausted | exhausts at n×m | n×m |
| `AtMost(n)` | `AtLeast(m)` | completed, not exhausted | exhausts at n×m | n×m |
| `Any` | `Times(n)` | completed, not exhausted | unlimited | ∞ |
| `Never` | (anything) | completed + exhausted | 0 calls | 0 |
| `Once` | `AtLeast(m)` | not completed | exhausts at m | m |

```rust
// Pseudocode for Times evaluation + mark_matched
fn eval_times(times_pred: &Predicate, ...) -> bool {
    if times_pred.state.exhausted { return false; }
    eval_predicate(&times_pred.inner, ...)
}

fn mark_matched_times(times_pred: &mut Predicate) {
    let PredicateKind::Times { inner, modifier } = &mut times_pred.kind else { unreachable!() };

    // Mark the inner subtree
    mark_matched(inner);

    // Check if inner completed a cycle (minimum met)
    if inner.state.completed {
        times_pred.state.call_count += 1;
        reset_predicate(inner);  // zero out the inner tree for next cycle
        update_state(&mut times_pred.state, modifier);
    }
}
```

---

### 10. Evaluation (revised)

Since the tree is owned, evaluation walks owned children directly. The arena is only
needed for `After` dependency lookups.

```rust
/// Evaluate a predicate tree. Returns true if it matches.
/// `arena` is passed for After dependency resolution only.
fn eval_predicate<Input>(
    predicate: &Predicate,
    arena: &PredicateArena,
    mock_id: &MockId,
    input: &Input,
) -> Result<bool> {
    if predicate.state.exhausted {
        return Ok(false);
    }

    match &predicate.kind {
        PredicateKind::Single(single) => {
            match unsafe { single.check::<Input>(mock_id, input) } {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        }

        PredicateKind::And(children) => {
            for child in children {
                if !eval_predicate(child, arena, mock_id, input)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }

        PredicateKind::Or(children) => {
            for child in children {
                if eval_predicate(child, arena, mock_id, input)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        PredicateKind::Xor(children) => {
            let mut count = 0u32;
            for child in children {
                if eval_predicate(child, arena, mock_id, input)? {
                    count += 1;
                    if count > 1 { return Ok(false); }
                }
            }
            Ok(count == 1)
        }

        PredicateKind::Not(inner) => {
            Ok(!eval_predicate(inner, arena, mock_id, input)?)
        }

        PredicateKind::Times { inner, modifier: _ } => {
            // Times itself checks exhaustion at the top of this function.
            // Delegate to inner — if inner matches, the match is valid.
            eval_predicate(inner, arena, mock_id, input)
        }

        PredicateKind::After { dependency, then } => {
            // dependency is (MockId, usize) — look up the committed expectation
            let (dep_mock_id, dep_idx) = dependency;
            let dep_completed = checkpoint.expectations
                .get(dep_mock_id)
                .and_then(|exps| exps.get(*dep_idx))
                .map(|exp| checkpoint.is_expectation_completed(exp))
                .unwrap_or(false);
            if !dep_completed {
                return Ok(false);  // gate closed
            }
            eval_predicate(then, arena, mock_id, input)
        }
    }
}
```

---

### 11. mark_matched (revised)

Walks the owned tree, updates state, and handles `Times` reset cycles.

```rust
fn mark_matched(predicate: &mut Predicate) {
    match &mut predicate.kind {
        PredicateKind::Single(_) => {
            predicate.state.call_count += 1;
            predicate.state.completed = true;
            predicate.state.exhausted = true;  // Single is always "once"
        }

        PredicateKind::And(children) => {
            for child in children {
                mark_matched(child);
            }
            predicate.state.call_count += 1;
            predicate.state.completed = children.iter().all(|c| c.state.completed);
            predicate.state.exhausted = children.iter().all(|c| c.state.exhausted);
        }

        PredicateKind::Or(children) => {
            // Only mark the first matched child? Or all?
            // Semantics: Or short-circuits — only the matched child is marked.
            // This requires knowing WHICH child matched. See note below.
            predicate.state.call_count += 1;
            predicate.state.completed = children.iter().any(|c| c.state.completed);
            predicate.state.exhausted = children.iter().all(|c| c.state.exhausted);
        }

        PredicateKind::Xor(children) => {
            predicate.state.call_count += 1;
            let completed_count = children.iter().filter(|c| c.state.completed).count();
            predicate.state.completed = completed_count == 1;
            predicate.state.exhausted = children.iter().all(|c| c.state.exhausted);
        }

        PredicateKind::Not(inner) => {
            // Not doesn't mark inner as matched — inner was NOT matched.
            predicate.state.call_count += 1;
            predicate.state.completed = !inner.state.completed;
            predicate.state.exhausted = inner.state.exhausted;
        }

        PredicateKind::Times { inner, modifier } => {
            // Mark inner subtree
            mark_matched(inner);

            // Check if inner completed a cycle (minimum met)
            if inner.state.completed {
                predicate.state.call_count += 1;
                reset_predicate(inner);  // fresh inner tree for next cycle
            }

            // Update own completed/exhausted based on modifier and call_count
            update_state(&mut predicate.state, modifier);
        }

        PredicateKind::After { dependency: _, then } => {
            // Only mark the "then" branch.
            // dependency is (MockId, usize) — an external expectation reference,
            // not part of this predicate tree.
            mark_matched(then);
            predicate.state.call_count += 1;
            predicate.state.completed = then.state.completed;
            predicate.state.exhausted = then.state.exhausted;
        }
    }
}

/// Reset an entire predicate subtree to fresh state.
fn reset_predicate(predicate: &mut Predicate) {
    predicate.state = PredicateState::new();
    match &mut predicate.kind {
        PredicateKind::Single(_) => {}
        PredicateKind::And(children)
        | PredicateKind::Or(children)
        | PredicateKind::Xor(children) => {
            for child in children { reset_predicate(child); }
        }
        PredicateKind::Not(inner) => reset_predicate(inner),
        PredicateKind::Times { inner, .. } => reset_predicate(inner),
        PredicateKind::After { then, .. } => reset_predicate(then),
        // Note: After's dependency is (MockId, usize) — an external expectation, not reset here
    }
}
```

**Note on Or/Xor mark_matched:** When `Or` matches, only one child actually matched.
We may need `mark_matched` to accept information about *which* child matched (from
`eval_predicate`). One approach: `eval_predicate` returns the index of the matched child
for `Or`/`Xor`, and `mark_matched` only recurses into that child. Otherwise we'd mark
non-matching children as matched, corrupting their state.

---

### 12. Expectation (replaces RootExpectation)

`RootExpectation` is removed. The predicate tree is fully self-contained — it owns
its cardinality via `Times` nodes. The top-level commitment is now just a predicate
reference + a return value:

```rust
pub struct Expectation {
    /// The predicate tree that must match for this expectation to fire.
    /// Controls its own cardinality and lifecycle internally via Times nodes.
    pub predicate: PredicateIndex,
    /// Optional return value closure, invoked when the predicate matches.
    /// Only expectations carry return values — inner predicates are pure conditions.
    pub return_val: Option<ReturnValDoublePointer>,
}
```

Completion and exhaustion queries live on `Checkpoint` (since they need arena access):

```rust
impl Checkpoint {
    /// Delegates to the root predicate's state in the arena.
    fn is_expectation_exhausted(&self, expectation: &Expectation) -> bool {
        self.arena.get(expectation.predicate)
            .map(|p| p.state.completed)
            .unwrap_or(true)
    }

    fn is_expectation_completed(&self, expectation: &Expectation) -> bool {
        self.arena.get(expectation.predicate)
            .map(|p| p.state.completed)
            .unwrap_or(false)
    }
}
```

If the user wants cardinality, they wrap their predicate in a `Times` node before
committing. The macro layer handles this transparently — the user writes
`expect(...).times(3)` and the macro emits a `Times` wrapper around the predicate.

This eliminates the dual-cardinality problem (root modifier vs inner modifier) and
makes the predicate tree the single source of truth for lifecycle.

---

### 13. expect (commit to evaluation)

The `expect` method commits a predicate to evaluation. No modifier is needed —
cardinality is already encoded in the predicate tree itself (via `Times` wrapping).

```rust
impl Checkpoint {
    /// Commit a predicate to evaluation.
    ///
    /// Cardinality is NOT specified here — it must be part of the predicate tree.
    /// If you want `times(3)` semantics, wrap the predicate with `times()`
    /// before calling this.
    pub fn expect<Input, ReturnVal>(
        &mut self,
        mock_id: &MockId,
        predicate: PredicateIndex,
        return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
    ) {
        let expectation = Expectation {
            predicate,
            return_val: return_val_closure.map(|c| ReturnValDoublePointer::from_fn(c)),
        };
        self.expectations
            .entry(mock_id.clone())
            .or_default()
            .push(expectation);
    }
}
```

---

### 14. User-facing flow (macro-generated)

```rust
// 1. Create leaf predicates (live in arena, referenced by PredicateIndex)
let check_positive = checkpoint.create_single::<i32>(&mock_id, Box::new(|x| {
    if *x > 0 { Ok(()) } else { Err("not positive".into()) }
}));

let check_even = checkpoint.create_single::<i32>(&mock_id, Box::new(|x| {
    if *x % 2 == 0 { Ok(()) } else { Err("not even".into()) }
}));

// 2. Compose — combines predicates into a new arena node
let combined = checkpoint.and(vec![check_positive, check_even]);

// 3. Apply cardinality — wraps in a Times node in the predicate tree
let timed = checkpoint.times(combined, TimesModifier::Times(3));

// 4. Commit to evaluation — no modifier here, Times(3) in the tree handles cardinality
checkpoint.expect::<i32, ()>(
    &mock_id,
    timed,
    None,  // no return value
);

// 5. (Optional) Create an After dependency referencing this expectation
//    The committed expectation is at index 0 for this mock_id.
let later_pred = checkpoint.create_single::<i32>(&mock_id, Box::new(|_| Ok(())));
let after_first = checkpoint.after((mock_id.clone(), 0), later_pred);
let after_timed = checkpoint.times(after_first, TimesModifier::Once);
checkpoint.expect::<i32, ()>(&mock_id, after_timed, None);
```

---

## Implementation status

The following changes have been implemented:

- ✅ `RootExpectation` renamed to `Expectation` (predicate + return val only).
- ✅ `modifier` and `state` fields removed from `Expectation` — cardinality lives
  entirely in the predicate tree via `Times` nodes.
- ✅ `expect()` no longer takes a `TimesModifier` parameter.
- ✅ `is_expectation_completed` / `is_expectation_exhausted` live on `Checkpoint`
  and delegate to the root predicate's `state.completed` in the arena.
- ✅ `Times` uses "increment on inner completion + reset" semantics:
  - Recurses into inner, checks if inner `completed`, increments own counter, resets inner.
  - `Times(3, Times(2, P))` correctly yields 6 total matches.
- ✅ `After` references a committed expectation by `(MockId, usize)` — not a raw
  `PredicateIndex`. This prevents referencing inner nodes that may be reset by a
  parent `Times`.
- ✅ `after()` builder takes `(MockId, usize)` dependency.
- ✅ `add_expectation` (the public sugar in `lib.rs`) still accepts a `TimesModifier`
  but wraps it into the predicate tree internally.
- ✅ All tests pass with the new semantics.

## Remaining migration notes

- The `_owned` variants described earlier in this doc (consuming by name) are not yet
  implemented — the current API uses `PredicateIndex` directly.
- ✅ `PredicateState` now has separate `completed` and `exhausted` fields. `completed`
  means "minimum met" (monotonic — stays true once set for some modifiers).
  `exhausted` means "will not accept more" (modifier-based via `TimesModifier::is_exhausted`).
- ✅ `Or`/`Xor` `mark_matched` now tracks `last_matched_child` (set during eval) and
  only recurses into the matched child.
- ✅ `TimesModifier::is_exhausted(count)` is a pure function of the modifier's cap —
  no inner state dependency. `Never` always exhausts. `Any`/`AtLeast` never exhaust.
- ✅ Construction-time cycling in `PredicateState::initial_for` properly handles all
  nesting combinations without infinite loops. Bounded inners track consumed capacity
  via `call_count` during cycling.
- Sequences still use the builder pattern with `PredicateIndex`; they don't consume
  predicates by name yet.
