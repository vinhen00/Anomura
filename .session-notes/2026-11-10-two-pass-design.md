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

## Files Modified This Session

- `/home/eli/anomura/docs/design/two_pass_analysis_substitution.md` (NEW — design doc)

## Key Files (reference)

- `crates/anomura_driver/src/crate_intercept.rs` — current CrateIntercept (to be refactored)
- `crates/anomura_driver/src/crate_api.rs` — current CrateApiModel (to be updated with TypeRepr)
- `crates/anomura_driver/src/crate_mock_gen.rs` — code generation (to use TypeRepr::to_source())
- `crates/anomura_plugins/src/substitution_pass.rs` — SubstitutePlugin::run() (to add analysis pass)
- `crates/anomura_plugins/src/bin/mock_substitute_driver_exec.rs` — driver binary entry point
- `docs/design/post_expansion_substitution.md` — old approach (superseded)
- `docs/design/rust_language_items.md` — language item handling notes (trait design)
