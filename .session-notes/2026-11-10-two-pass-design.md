# Two-Pass Analysis + Substitution Design Session — 2026-11-10

## Summary

Designed a new two-pass architecture for `mock_crate!` that replaces the fragile post-expansion query override approach (`post_expansion_substitution.md`). The core idea: separate reading (analysis pass) from writing (substitution pass) to avoid all the DUMMY_NODE_ID / DefId registration issues.

## Key Design Decisions

### 1. Two-pass architecture (analysis → substitution)
- **Analysis pass:** Full compilation through `after_analysis` (read-only). Collects the complete API model from the expanded AST (`after_expansion`) and type-level info from `TyCtxt` (`after_analysis`). Returns `Compilation::Stop` after analysis — no codegen needed.
- **Substitution pass:** `after_crate_root_parsing` (pre-expansion). Receives the model in memory, retains scaffolding items, removes everything else, regenerates mockable items from the model. Rustc handles expansion/resolution/codegen of the regenerated crate.

### 2. In-memory model transfer
Both passes run as sequential `rustc_run()` calls within the same `SubstitutePlugin::run()` process. No serialization needed — the `AnalysisModel` is moved from one callback struct to the next.

### 3. Regeneration strategy (middle-ground, not nuclear)
- **Keep:** `use`, `type` aliases, `const`, `static`, `extern crate`, `extern` blocks, `macro_rules!` defs
- **Remove + regenerate:** `fn`, `struct`, `enum`, `trait`, `impl`, `mod`, macro calls (`MacCall`)
- Macro-generated items are in the model (collected from expanded AST) and get regenerated as concrete items. The macro invocations are simply removed.

### 4. TypeRepr — custom type representation
Replaces both `Box<ast::Ty>` (not serializable) and raw strings (no structure). Enum with variants: Path, Reference, Tuple, Slice, Array, RawPointer, FnPointer, DynTrait, ImplTrait, Never, SelfType, Inferred. Fully qualified paths from the expanded AST. `to_source()` method for code generation.

### 5. Module tree preserved
Regenerated crate preserves original `pub mod` structure so downstream code paths (`fns::sub_module::some_fn`) remain valid.

### 6. No fallback to original implementations
Functions/methods with no registered mock **panic**. Original bodies are not preserved (they're gone after regeneration).

### 7. Visibility filtering
Only fully `pub` items collected and regenerated. `pub(crate)`, `pub(super)`, and other restricted visibility items excluded.

### 8. Trait handling
- Private traits: kept with empty bodies
- Public traits: all methods mockable, default bodies replaced with mock dispatch
- Both trait-level and type-level mock expectations supported
- Trait object identity recovery flagged as open design question (marker trait / supertrait approach proposed but needs prototyping)

## Brainstorming Process

### Alternatives considered for macro expansion:
1. **cargo expand** — rejected: output not guaranteed to be re-parseable
2. **Two-pass (chosen)** — analysis reads expanded AST, substitution writes pre-expansion
3. **Manual macro expansion in after_crate_root_parsing** — rejected: reimplements expansion driver, fragile
4. **User-declared API surface** — rejected: not automatic enough for mock_crate! goals
5. **Don't mock macro-generated items** — considered as fallback but not needed with two-pass
6. **Nuclear regeneration (throw away entire crate)** — discussed extensively, settled on middle-ground (keep scaffolding)

### Why not nuclear regeneration:
- `const` and `static` values hard to reconstruct from type checker alone (easier from expanded AST, but adds model complexity)
- `macro_rules!` definitions exist only in syntax, not typed representation
- `extern` blocks, type aliases, extern crate — all easier to just keep
- Middle-ground gives same macro handling benefit (macro calls removed, expanded items regenerated) with less risk

### Why keep `use` statements:
- Belt-and-suspenders with fully qualified types — if a type path from the expanded AST isn't fully qualified, the `use` imports provide fallback resolution
- Cleaner debugging (generated code more readable)

## Design Doc

Written at: `/home/eli/anomura/docs/design/two_pass_analysis_substitution.md`

Covers:
- Full architecture diagram
- Execution flow (SubstitutePlugin::run changes)
- AnalysisModel data structures
- TypeRepr definition and conversion
- Regeneration strategy (what's kept vs removed vs regenerated)
- Data flow between passes
- Item matching between after_expansion and after_analysis (via DefId)
- Impact on existing code
- 6-phase implementation plan
- Risk assessment
- Trait handling design (section 14)
- Resolved and remaining open questions

## Implementation Plan (from doc)

1. **Phase 1:** TypeRepr + model update (replace ast::Ty with TypeRepr, add GenericsModel)
2. **Phase 2:** AnalysisPass scaffold (after_expansion callback, wire into SubstitutePlugin)
3. **Phase 3:** Regenerative CrateIntercept (scaffolding retention, module-tree regeneration)
4. **Phase 4:** after_analysis enrichment (trait impls, repr info, blanket impls from TyCtxt)
5. **Phase 5:** Macro-generated items testing
6. **Phase 6:** Cleanup (remove old approach, Derives struct, fetch_derives)

## Open Questions Remaining

1. Generic impl blocks — code generation for `impl<T: Clone> Foo<T> { ... }`
2. Orphan rules — blanket impls from external crates don't need regeneration
3. Conditional compilation — should be automatic (same compiler_args for both passes)
4. Trait object mock identity recovery — supertrait bounds vs thread-local vs other approaches

## Implementation Progress

### Phase 1: TypeRepr + model update ✅
- Created `type_repr.rs` with `TypeRepr` (11 variants, no `Inferred`) and `RawTypeRepr` (12 variants, includes `Inferred`). "Parse, don't validate" pattern: `RawTypeRepr::resolve() -> Result<TypeRepr, _>`.
- Invalid states made unrepresentable: `Lifetime`, `ArrayLength`, `Abi` newtypes (non-empty by construction). `DynTrait`/`ImplTrait` have `first_bound` field guaranteeing at least one bound. `Path` has `first: String` + `rest: Vec<String>`.
- Created `generics_model.rs` with `GenericsModel`, `ReprInfo` (with `PrimaryLayout` enum and `AlignmentModifier` enum for mutual exclusivity), `TraitImplInfo`, etc.
- Updated `crate_api.rs`: `Symbol` → `String`, `Box<ast::Ty>` → `TypeRepr`, removed `Derives`, added generics/repr/trait_impls fields.
- Updated `crate_mock_gen.rs`: removed rustc dependencies, uses `.to_source()` everywhere.
- Updated `crate_intercept.rs`: collect_* methods produce String/TypeRepr, `ty_to_type_repr` bridge.
- All 22 e2e tests pass.

### Phase 2: AnalysisPass scaffold ✅
- Created `analysis_pass.rs` with `AnalysisPass` implementing `Callbacks::after_expansion`.
- Confirmed callback signatures: `after_expansion(compiler, tcx: TyCtxt)` and `after_analysis(compiler, tcx: TyCtxt)` — both receive TyCtxt, not raw AST.
- Accesses expanded AST via `tcx.resolver_for_lowering_raw(()).borrow()`.
- Wired into `SubstitutePlugin::run()`: two sequential `rustc_run` calls in the same process.
- `CrateIntercept::new()` now takes `CrateApiModel` as second parameter.
- All 22 e2e tests pass.

### Phase 3: Regenerative CrateIntercept ✅
- Added to `crate_mock_gen.rs`: `gen_crate_from_model()`, `gen_module_items()`, `gen_struct_def()`, `gen_enum_def()`, `gen_trait_def()`, `gen_impl_block()`.
- Rewrote `after_crate_root_parsing`: retains scaffolding (use, const, static, type alias, extern crate, extern blocks, macro defs), removes everything else, regenerates from model.
- Macro calls are removed from source; their expanded items come from the analysis model.
- All 22 e2e tests pass.

### Phase 5: Macro-generated item testing ✅ (done before Phase 4)
- **macro_rules! tests**: Added `define_fn!` macro to fns crate generating `macro_generated_add()` and `macro_generated_greeting()`. Both mocked successfully.
- **Derive proc macro tests**: Created `test-suite/test-macros/` crate with `#[derive(Describe)]` that generates `impl Describable { fn describe() }`. Mocked successfully.
- **Attribute proc macro tests**: Added `#[generate_doubled]` attribute macro that generates companion function `magic_number_doubled()`. Both original and generated functions mocked successfully.
- All macro types work: declarative macros, derive proc macros, attribute proc macros.

### Issues found and fixed during Phase 3/5:
1. **Expanded AST collects too many impls** — derive-generated impls (Debug, Default, PartialEq, StructuralPartialEq) and private trait impls were leaking into the model. Fixed with two-pass collection: collect types/traits first, then filter impls to only known public types.
2. **Duplicate items** — `gen_convenience_api` was a separate pass generating items alongside `gen_crate_from_model`. Merged convenience API generation inline into `gen_crate_from_model` — single source of truth, single pass.
3. **Trait impl visibility** — `gen_mock_method_body` always emitted `pub fn`, but trait impl methods can't have `pub`. Fixed with `replacen("pub fn ", "fn ", 1)` for trait impls.
4. **Tuple/unit structs unsupported** — `collect_struct` only handled named structs. `ClosureWrapper(pub Box<dyn Fn(u32) -> u32>)` wasn't collected. Added tuple struct and unit struct support to both collection and code generation.
5. **Submodule convenience wrappers placed inside modules** — wrappers for `a::nested::Inner` methods were generated inside `pub mod a { ... }` but referenced `a::nested::Inner` from outside. Fixed by collecting `root_wrappers` separately and emitting them at crate root.
6. **External trait impl mocking** — `impl Debug for ClosureWrapper` (hand-written external trait impl) is not yet regenerated. Test temporarily commented out. Needs external trait impl support.

### Current test results (2026-11-11):
- **22 pass**: Including 5 macro-generated item tests (2 macro_rules, 1 derive proc macro, 2 attribute proc macro)
- **4 ignored**: Constructor tests requiring `has_default()` type enrichment (Phase 4)
- **1 commented out**: External trait impl mocking (`mock_crate_trait_impl_debug`)

### Remaining phases:
- Phase 4: after_analysis enrichment (trait impls for has_default, repr info, generics)
- Phase 6: Cleanup (remove old collect_* methods, apply_mock_bodies, gen_convenience_api, etc.)
- External trait impl support (impl Debug/Clone/etc. for crate types)

## Continuation Notes (for next session)

### What's working now
- Two-pass architecture: AnalysisPass (after_expansion) → CrateIntercept (after_crate_root_parsing)
- Both passes run as sequential `rustc_run()` calls in the same driver process (SubstitutePlugin::run)
- CrateIntercept retains scaffolding (use/const/static/type alias/extern crate/extern blocks/macro defs), regenerates everything else from model
- All 22 existing e2e tests pass

### Callback signatures confirmed on nightly-2026-10-06
```rust
fn after_expansion<'tcx>(&mut self, compiler: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation
fn after_analysis<'tcx>(&mut self, compiler: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation
```
Both receive `TyCtxt`, NOT raw AST. Expanded AST accessed via `tcx.resolver_for_lowering_raw(()).borrow()`. HIR accessed via `tcx.hir_crate_items(()).free_items()` and `tcx.hir_node(item_id.hir_id())`.

### Phase 4: after_analysis enrichment ✅ (partial — repr + trait impls done, generics placeholder)
- **Rewrote analysis_pass.rs** from AST-based `after_expansion` to HIR-based `after_analysis` — single callback, single pass
- **No more rustc_ast/rustc_ast_pretty dependencies** in analysis_pass.rs — only rustc_hir and rustc_middle::ty::TyCtxt
- **Type info from TyCtxt**:
  - Repr info via `tcx.adt_def(def_id).repr()` → `ReprInfo` with `PrimaryLayout` + `AlignmentModifier`
  - Trait impls via `tcx.all_local_trait_impls(())` for comprehensive coverage
  - Function signatures from `tcx.fn_sig(def_id).skip_binder().skip_binder()`
  - Field types from `tcx.type_of(field.def_id).skip_binder()`
  - Visibility from `tcx.visibility(def_id).is_public()`
  - Parameter names from `tcx.fn_arg_idents(def_id)`
  - Item names from `tcx.item_name(def_id)`
- **Receiver detection** via `tcx.fn_arg_idents()` checking for "self" name + type string inspection
- **Parent filtering** — `tcx.local_parent(local_def_id) == CRATE_DEF_ID` prevents double-counting submodule items from `free_items()`
- **Module path prefix stripping** — recursive post-processing: types from TyCtxt use crate-relative paths (e.g., `a::nested::Inner`), stripped to short names when inside the matching module context
- **External trait impls recorded as TraitImplInfo** — `has_default()` works correctly
- Generics still placeholder (`GenericsModel::empty()`) — will be populated when needed
- All 26 tests pass

### Key technical findings from Phase 4:
- `tcx.hir_crate_items(()).free_items()` returns ALL items including those nested in modules — must filter with `tcx.local_parent()`
- `tcx.impl_trait_ref(def_id)` returns `EarlyBinder<TraitRef>` (not Option) — use `.skip_binder()` directly, only call for trait impls (`impl_data.of_trait.is_some()`)
- `format!("{}", ty)` for `ty::Ty` gives crate-relative paths (e.g., `a::nested::Inner` not `Inner`)
- `tcx.fn_arg_idents()` returns `&[Ident]` with the source parameter names
- `AssocItemKind::Fn { has_self }` is the pattern for function items in HIR impl/trait items
- HIR `ItemKind::Fn { sig, body, .. }` is a struct variant (not tuple)
- The expanded AST (`resolver_for_lowering_raw`) is likely stolen by `after_analysis` time — HIR is the way to go

### Phase 6: Cleanup
- Remove old `collect_*` methods from `crate_intercept.rs` (dead code)
- Remove `apply_mock_bodies`, `replace_fn_body`, `handle_impl_methods`, etc. from `crate_intercept.rs`
- Remove `gen_convenience_api` and `gen_module_convenience_wrappers` from `crate_mock_gen.rs` (replaced by inline generation)
- Remove `gen_full_mock_source` from `crate_mock_gen.rs` (unused stub)
- Keep in `crate_intercept.rs`: `inject_source_items`, `parse_fn_item`, and `after_crate_root_parsing` callback

### External trait impl support (new task)
- Need to regenerate hand-written impls for external traits (e.g., `impl Debug for ClosureWrapper`)
- Currently filtered out because `Debug` is not a local trait
- Options: keep all trait impls for known types (but then derive-generated ones also come back), or distinguish hand-written from derive-generated
- The `mock_crate_trait_impl_debug` test is commented out pending this

### Key technical details to remember
- `TypeRepr::Path` has `first: String, rest: Vec<String>` (NOT `segments: Vec<String>`)
- `TypeRepr::DynTrait/ImplTrait` have `first_bound + additional_bounds` (NOT `bounds: Vec`)
- `Lifetime`, `ArrayLength`, `Abi` are newtypes that panic on empty construction
- `RawTypeRepr` has `Inferred` variant; `TypeRepr` does not. `RawTypeRepr::resolve() -> Result<TypeRepr, _>` is the boundary.
- `ReprInfo` uses `PrimaryLayout` enum (Rust/C/Transparent/Int) + `AlignmentModifier` enum (Packed/Align) for mutual exclusivity
- `StructModel.has_default()` replaces old `derives.default` check — currently always false (needs Phase 4)
- The `ty_to_type_repr` bridge in both `analysis_pass.rs` and `crate_intercept.rs` uses `TypeRepr::simple(&pprust::ty_to_string(ty))` — flattens the type to a single Path segment. Works but loses structure.
- `gen_crate_from_model` now generates EVERYTHING (definitions + mock dispatch + convenience API) in a single pass
- `gen_module_items` takes `root_wrappers: &mut String` parameter — convenience wrappers bubble up to crate root
- Impl blocks for trait impls strip `pub` from method signatures via `replacen("pub fn ", "fn ", 1)`
- Analysis pass uses two-pass collection: types/traits first, then impls filtered to known types only

## Files Modified (both sessions)

- `/home/eli/anomura/docs/design/two_pass_analysis_substitution.md` (NEW — design doc)
- `/home/eli/anomura/crates/anomura_driver/src/type_repr.rs` (NEW — TypeRepr + RawTypeRepr)
- `/home/eli/anomura/crates/anomura_driver/src/generics_model.rs` (NEW — GenericsModel, ReprInfo, etc.)
- `/home/eli/anomura/crates/anomura_driver/src/analysis_pass.rs` (NEW — AnalysisPass with impl filtering)
- `/home/eli/anomura/crates/anomura_driver/src/crate_api.rs` (rewritten — TypeRepr + String)
- `/home/eli/anomura/crates/anomura_driver/src/crate_mock_gen.rs` (updated — regeneration functions, inline convenience API)
- `/home/eli/anomura/crates/anomura_driver/src/crate_intercept.rs` (updated — receives model, regenerative callback)
- `/home/eli/anomura/crates/anomura_driver/src/lib.rs` (added module declarations)
- `/home/eli/anomura/crates/anomura_plugins/src/substitution_pass.rs` (two-pass wiring)
- `/home/eli/anomura/crates/anomura_plugins/src/lib.rs` (fixed attribute ordering)
- `/home/eli/anomura/test-suite/fns/src/lib.rs` (added macro_rules + proc macro items)
- `/home/eli/anomura/test-suite/fns/Cargo.toml` (added test-macros dependency)
- `/home/eli/anomura/test-suite/test-macros/` (NEW — proc macro test crate with Describe derive + generate_doubled attr)
- `/home/eli/anomura/test-suite/mocks/src/main.rs` (added macro/proc macro tests, ignored constructor tests, commented out Debug test)
- `/home/eli/anomura/Cargo.toml` (added fns + test-macros to workspace)

## Key Files (reference)

- `crates/anomura_driver/src/crate_intercept.rs` — current CrateIntercept (to be refactored)
- `crates/anomura_driver/src/crate_api.rs` — current CrateApiModel (to be updated with TypeRepr)
- `crates/anomura_driver/src/crate_mock_gen.rs` — code generation (to use TypeRepr::to_source())
- `crates/anomura_plugins/src/substitution_pass.rs` — SubstitutePlugin::run() (to add analysis pass)
- `crates/anomura_plugins/src/bin/mock_substitute_driver_exec.rs` — driver binary entry point
- `docs/design/post_expansion_substitution.md` — old approach (superseded)
- `docs/design/rust_language_items.md` — language item handling notes (trait design)
