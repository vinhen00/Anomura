# Generics Model Refactor + Pipeline Fixes Session — 2026-11-19

## Summary

Completed the generics model refactor (migrating from `GenericsModel` to `GenericParams`) and fixed critical/medium issues found during a pipeline review. All changes compile cleanly; e2e tests not re-run this session.

## Context (where we started)

Phase 6 cleanup was done before this session. The `crate_api.rs` had been restructured with `Item`/`ItemKind` wrapping, and a flat `ItemStore` with `ItemId` references. A new `GenericParams` model had been designed in `generics_model.rs` but not propagated to the rest of the code — the crate had ~20 compilation errors.

## Part 1: GenericParams Propagation

### What changed

**`generics_model.rs`** — already had the new model (no changes this session):
- `GenericParams` with single `Vec<GenericParam>` (declaration order) + `Vec<WherePredicate>`
- `GenericParam` enum: `Lifetime(LifetimeParam)`, `Type(TypeParam)`, `Const(ConstParam)`
- `Bound` enum: `Trait(TraitBound)` with `ForLifetimes` + `is_maybe`, `Lifetime(String)`
- `Receiver` enum: `None`, `Ref`, `RefLifetime`, `RefMut`, `RefMutLifetime`, `Owned`, `Arbitrary(TypeRepr)`
- `FnQualifiers`: `is_async`, `is_unsafe`, `is_const`, `abi: Option<String>`

**`crate_api.rs`** — updated:
- All `GenericsModel` → `GenericParams` on StructData, EnumData, TraitData, FunctionData, ImplData, MethodData
- `FunctionData`/`MethodData`: replaced individual `is_async`/`is_unsafe`/`abi` with `qualifiers: FnQualifiers`
- `MethodData`: replaced `ReceiverKind` with `Receiver` (from generics_model)
- Removed `ReceiverKind` enum entirely
- Removed `TraitImplInfo` from `StructData`/`EnumData` (`trait_impls` field removed)
- `has_default()` returns `false` (placeholder — TraitImplInfo tracking removed)
- `TraitData.supertraits`: changed from `Vec<TypeRepr>` to `Vec<Bound>`
- `AssociatedTypeData.bounds`: changed from `Vec<TypeRepr>` to `Vec<Bound>`
- `VariantData`: added `kind: StructKind` field for enum variant shape tracking

**`analysis_pass.rs`** — rewrote collection logic:
- `collect_generics` builds single `Vec<GenericParam>` in declaration order (was separate vecs)
- Added `hir_bound_to_bound()` producing `Bound` with HRTB + `?Sized` support
- Added `collect_fn_qualifiers(header)` extracting async/unsafe/const/ABI from HIR `FnHeader`
- `detect_receiver` rewritten: uses HIR `ImplicitSelfKind` instead of string matching. Handles lifetime receivers and arbitrary self types (`self: Pin<&mut Self>`)
- Added `extract_ref_lifetime()` for `&'a self` / `&'a mut self`
- `collect_method_from_impl` now reads actual qualifiers from HIR (was `FnQualifiers::default()`)
- `collect_function` now reads ABI from `sig.header`
- `collect_supertraits` uses `hir_bound_to_bound` (captures lifetime + HRTB supertraits)
- `collect_enum` records `StructKind` on each variant
- Removed `bound_to_type_repr` (replaced by `hir_bound_to_bound`)
- Removed `enrich_trait_impls` and all `TraitImplInfo` usage

**`crate_mock_gen.rs`** — updated codegen:
- All `ReceiverKind` → `Receiver` (31 occurrences), using `has_self()`, `is_ref()`, `is_ref_mut()`, `is_none()` helpers
- `GenericsModel` → `GenericParams`
- Receiver matching handles new variants (`RefLifetime`, `RefMutLifetime`, `Arbitrary`) via `to_source()`
- `gen_mock_fn_body` emits `{qualifiers}` prefix
- `gen_mock_method_body` emits `{qualifiers}` prefix + takes `is_trait_impl` param for visibility control
- `gen_trait_def` emits qualifiers on method signatures
- Removed `strip_prefix("pub ")` hack for trait impls
- `gen_enum_def` uses `vd.kind` for struct-like vs tuple vs unit variant syntax

**`type_repr.rs`** — added:
- `GenericArg::BoundConstraint(String, String)` for `Item: Display + Debug` constraints
- Same variant on `RawGenericArg` with pass-through `resolve()`

## Part 2: Pipeline Review + Fixes

### Issues found and fixed (all CRITICAL + MEDIUM)

| ID | Issue | Fix |
|----|-------|-----|
| C1 | FnQualifiers not emitted in codegen | Added qualifiers prefix to gen_mock_fn_body, gen_mock_method_body, gen_trait_def |
| M4 | ABI never collected | collect_fn_qualifiers reads header.abi; "Rust" → None, others stored |
| C2 | `&'a mut self` misclassified as Ref | Rewrote detect_receiver with HIR ImplicitSelfKind |
| M5 | Receiver::Arbitrary never produced | detect_receiver now falls back to HIR type for arbitrary self |
| M6 | String-based receiver detection fragile | Eliminated entirely — uses HIR ImplicitSelfKind |
| C6 | Struct-like enum variants emitted as tuple | Added StructKind to VariantData, gen_enum_def uses it |
| M1 | Lifetime supertraits dropped | supertraits now Vec<Bound>, collect_supertraits uses hir_bound_to_bound |
| M2 | HRTB on supertraits lost | Same fix — hir_bound_to_bound captures ForLifetimes |
| M3 | Const args/defaults use debug formatting | Uses expr_to_string/qpath_to_string instead of format!("{:?}") |
| M7 | AssocItemConstraintKind::Bound not handled | Added BoundConstraint variant, extract_generic_args handles Bound |

### Known issues not yet fixed

| ID | Issue | Severity |
|----|-------|----------|
| C3 | gen_fn_wrappers doesn't propagate generics for generic free functions | CRITICAL (affects generic fn mocking) |
| C5 | Method-level generics not propagated to on_call_* wrappers | CRITICAL (affects generic method mocking) |
| C7 | PublicFoo struct for generic types missing generic params | CRITICAL (affects generic constructors) |
| M8 | PhantomData for private fields changes variance | MEDIUM |
| C4 | Generic mock dispatch TypeId mismatch | Believed non-issue (raw pointer casting bypasses TypeId) |

C3, C5, C7 are all related to the same design limitation: generic items need their type parameters carried through to wrapper types and on_call functions. This is a larger design discussion.

## Files Modified This Session

- `crates/anomura_driver/src/generics_model.rs` — (pre-session, reviewed only)
- `crates/anomura_driver/src/crate_api.rs` — type changes, removed ReceiverKind/TraitImplInfo
- `crates/anomura_driver/src/analysis_pass.rs` — rewrote collect_generics, detect_receiver, added helpers
- `crates/anomura_driver/src/crate_mock_gen.rs` — adapted all codegen to new types
- `crates/anomura_driver/src/type_repr.rs` — added BoundConstraint variant

## Key Technical Details

### HIR ImplicitSelfKind mapping
```
ImplicitSelfKind::Imm / Mut    → Receiver::Owned
ImplicitSelfKind::RefImm       → Receiver::Ref or RefLifetime (if explicit lifetime)
ImplicitSelfKind::RefMut       → Receiver::RefMut or RefMutLifetime (if explicit lifetime)
ImplicitSelfKind::None + self  → Receiver::Arbitrary (HIR type inspection)
ImplicitSelfKind::None + !self → Receiver::None
```

### FnHeader.abi
- `header.abi.name()` returns the ABI name string
- `"Rust"` → `None` (default ABI, don't emit)
- Everything else → `Some("C")`, `Some("system")`, etc.

### Constness on nightly-2026-10-06
- `Constness::Const { always: bool }` — struct variant, not unit
- Match with `Constness::Const { .. }`

### ConstArgKind for const generic args
- `ConstArgKind::Anon(anon)` → `tcx.hir_body(anon.body)` → `expr_to_string(ann, body.value)`
- `ConstArgKind::Path(qpath)` → `qpath_to_string(ann, qpath)`
- Other variants → debug fallback (rare in practice)

### TyKind::Ref on nightly-2026-10-06
- `TyKind::Ref(&Lifetime, &Ty, Mutability)` — 3 fields, not 2

## Continuation Notes

### Next steps
1. Consider C3/C5/C7 (generic wrapper types) — needs design discussion about how to handle type parameters in Predicate*/Return* structs and on_call_* methods
2. Run e2e tests (`cargo mock test -p mocks`) to verify nothing regressed
3. External trait impl support (from previous session — still commented out test)
4. Phase 6 cleanup items if any remain (old dead code)

## Part 3: Codegen Generics Review + Fixes

### Issues found and fixed

| ID | Issue | Fix |
|----|-------|-----|
| C1 | `gen_mock_method_body` self type missing generics in type tuple | Added `struct_type_path` param, uses `Wrapper<T>` not bare `Wrapper` |
| M3 | Missing space before `=` in assoc type defaults | Format string fix: `type Foo = bar` not `type Foo= bar` |
| M1 | Tuple struct private fields not PhantomData-wrapped | Applied same PhantomData wrapping as named structs |
| M2 | Trackable tuple structs missing `adt_mock_id` | Appended `pub context::AdtMockId` as final tuple field |
| C2 | `gen_fn_wrappers` ignores function generics | Reads `func.generics`, skips Predicate/Return for generic fns, adds generics to on_call/expect/sequence |
| C3 | `gen_method_wrappers` ignores method-level generics | `is_generic` now checks both impl + method generics, on_call_ gets method generics |
| C4 | `gen_public_struct` + `gen_constructor_body` missing generics | `PublicFoo<T>` with generics on struct, Into impl, and run_mock turbofish |
| C5 | GATs not supported (AssociatedTypeData lacks generics) | Added `generics: GenericParams` field, collection from HIR, rendering with generics + where clauses |

### Known remaining issues
| ID | Issue | Severity |
|----|-------|----------|
| M4 | PhantomData fails for unsized private fields (dyn Trait) | MEDIUM (very rare) |
| L1 | `#[repr(...)]` attributes not emitted on regenerated structs/enums | LOW |
| L2 | Double-underscore in submodule mock IDs | LOW (works but fragile) |
| L3 | Submodule functions lack `expect_`/`sequence_` convenience functions | LOW |
| — | Tuple struct `adt_mock_id` not accessible by name (`self.N`) | Pre-existing limitation |

### Compilation status
- `cargo check -p anomura_driver` ✅ (warnings only)
- `cargo check -p anomura_plugins` ✅ (clean)
- e2e tests: not run this session
