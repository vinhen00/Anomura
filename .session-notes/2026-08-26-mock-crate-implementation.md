# mock_crate! Implementation — Session Notes (2026-08-26)

## Summary

Implemented the scaffolding for `mock_crate!` — a macro that automatically mocks an entire crate's public API. The full pipeline from discover pass → IPC → substitution pass → AST collection is working end-to-end. What remains is the code generator (the large task).

## What Was Done

### 1. README update
- Rewrote README with correct instructions (must `cd test-suite/mocks`, then `cargo mock test`)
- Documented the `cargo install --path crates/anomura_plugins --force` requirement

### 2. Bug fixes
- Removed broken `self.` line in `expand_macro.rs` (mock_method code gen)
- Renamed struct field `mock_id: String` → `mock_hash: context::AdtMockId` in `expand_mock_struct`
- Fixed `RetvalFinder` to use `mock_hash: mock_hash` instead of `.to_string()`
- Added `Display` impl for `AdtMockId`

### 3. mock_crate! proc macro (crates/mock-macro/src/lib.rs)
- Added no-op `#[proc_macro] pub fn mock_crate` that returns empty TokenStream
- The discover pass driver intercepts the macro call at AST level (doesn't need actual expansion)

### 4. Discover pass changes (crates/anomura_driver/src/parse_mocks.rs)
- Added `mock_crate_targets: Vec<String>` field to `ParseMocks`
- Added `get_mock_crate_targets()` getter
- `handle_maccall`: recognizes `"mock_crate"`, extracts crate name, adds to both `crates` and `mock_crate_targets`
- `extract_path_value`: added `"mock_crate"` arm (consolidated all arms into one pattern)

### 5. IPC changes (crates/anomura_plugins/src/mock_discover_pass.rs)
- Added `CallBackMessage::MockCrateTargets(Vec<String>)` variant
- Added `mock_crate_targets: Vec<String>` to `DiscoverClientReturn`
- Listener thread handles `MockCrateTargets` message
- `send_back_results` sends mock_crate targets as separate IPC message
- `JoinHandle` type updated to `(Vec<String>, Vec<String>, Vec<String>)`
- `after_execution` populates `mock_crate_targets` in return value

### 6. Substitution pass routing (crates/anomura_plugins/src/substitution_pass.rs)
- `SubstitutePlugin` gained `mock_crate_targets: Vec<String>` field
- `modify_cargo` sets `MOCK_CRATE_TARGETS` env var (comma-separated list)
- `run()` reads env var, routes to `CrateIntercept` if crate_name is in mock_crate_targets, else `FunctionIntercept`
- Added `MOCK_CRATE_TARGETS_ENV` constant in `lib.rs`

### 7. cargo-mock.rs binary
- Updated to pass `res.mock_crate_targets` to `SubstitutePlugin::new()`

### 8. CrateApiModel (crates/anomura_driver/src/crate_api.rs) — NEW FILE
- `CrateApiModel` — top-level: crate_name + root ModuleModel
- `ModuleModel` — structs, enums, traits, functions, impls, children
- `StructModel` — name, fields, `all_public()` helper
- `FieldModel` — name, ty (Box<ast::Ty>), is_pub
- `EnumModel` — name, variants
- `VariantModel` — name, fields (Vec<Box<ast::Ty>>)
- `TraitModel` — name, methods
- `MethodSigModel` — name, receiver, params, return_type, is_pub
- `ParamModel` — name, ty
- `ReceiverKind` — None/Ref/RefMut/Owned
- `FunctionModel` — name, params, return_type
- `ImplModel` — self_type_name, trait_name (Option), methods

### 9. CrateIntercept driver (crates/anomura_driver/src/crate_intercept.rs) — NEW FILE
- Implements `rustc_driver::Callbacks`
- `after_crate_root_parsing`: walks AST, builds `CrateApiModel`, prints summary
- `collect_module`: walks `Vec<Box<Item>>` recursively
- `collect_function`, `collect_struct`, `collect_enum`, `collect_trait`, `collect_impl`
- `collect_method_sig`, `collect_params`, `collect_method_params`
- `extract_receiver`, `extract_type_name`, `extract_param_name`
- Handles: pub functions, pub structs (named fields only), pub enums, pub traits, all impls (inherent + trait), pub modules (recursive)

### 10. lib.rs (crates/anomura_driver/src/lib.rs)
- Added `pub mod crate_api;` and `pub mod crate_intercept;`

## End-to-end Test Result

Running `cargo mock test` from `test-suite/mocks/` with `mock_crate!(fns)`:
- Discover pass: correctly identifies `mock_crate_targets: ["fns"]` ✓
- Substitution pass: routes `fns` to `CrateIntercept` ✓
- AST collector output for `fns` crate:
  - 2 structs: `MockStruct` (2 fields, all_pub=false), `Foo` (1 field, all_pub=true)
  - 1 enum: `Pattern` (2 variants: Okay, NotOkay)
  - 0 traits (no pub traits defined in fns)
  - 18 free functions
  - 4 impls: `ConsSelfStruct` (1 method), `MockStruct` (1), `Foo` (5), `impl Debug for ClosureWrapper` (1)
  - 1 submodule: `a` (contains `pub fn modules()`)
- Compilation continues (no mock code injected yet → tests that reference `_original` functions fail)

## Next Steps: Code Generator (Task 7)

This is the substantial remaining work. The code generator needs to:

### Phase 1: Function body replacement (minimum viable)
For each collected function/method, replace the body with mock dispatch:
```rust
fn foo(x: u32) -> u32 {
    let mock_id = context::MockId::new(stringify!(crate_fn_name));
    if context::ctx_built_and_contains_id(&mock_id) {
        match context::run_mock::<(u32,), u32>(mock_id, (x,)) { ... }
    } else {
        return foo_original(x);
    }
}
```
And create `fn foo_original(x: u32) -> u32 { /* original body */ }`.

This is what `FunctionIntercept` already does — could potentially reuse/adapt that logic.

### Phase 2: Struct modifications
- Add `pub adt_mock_id: context::AdtMockId` to structs with private fields
- Modify constructors to initialize the field
- Add Drop impl for cleanup

### Phase 3: Convenience API (the mock_adt output)
For each method, generate:
- `pub struct PredicateStructMethod(...)` — wrapper newtype
- `pub struct ReturnStructMethod(...)` — wrapper newtype  
- `impl Struct { pub fn expect_method(...) }` — expectation setup
- `impl Struct { pub fn on_call_method(...) }` — convenience
- Sequence helpers

This is the largest part and mirrors the ~2800 lines in `mock_adt.rs`, but working at the `rustc_ast` level instead of `proc_macro2::TokenStream`.

### Key design decision for code gen
**Approach A**: Generate `rustc_ast` items directly (manipulate the AST programmatically)  
**Approach B**: Generate Rust source code as a string, parse it into AST items, inject them

Approach B is likely easier/faster to implement since we can reuse patterns from `expand_mock_fn`/`expand_mock_method` which already generate code via `quote!` → string → parse. The challenge is that we'd need to use `rustc_parse` to parse the generated string into `ast::Item` nodes.

### Files modified this session
- `/home/eli/anomura/README.md`
- `/home/eli/anomura/crates/context/src/lib.rs` (Display for AdtMockId)
- `/home/eli/anomura/crates/mock-macro/src/lib.rs` (mock_crate! proc macro)
- `/home/eli/anomura/crates/anomura_driver/src/lib.rs` (new modules)
- `/home/eli/anomura/crates/anomura_driver/src/parse_mocks.rs` (mock_crate handling)
- `/home/eli/anomura/crates/anomura_driver/src/expand_macro.rs` (mock_hash fix)
- `/home/eli/anomura/crates/anomura_driver/src/crate_api.rs` (NEW)
- `/home/eli/anomura/crates/anomura_driver/src/crate_intercept.rs` (NEW)
- `/home/eli/anomura/crates/anomura_plugins/src/lib.rs` (MOCK_CRATE_TARGETS_ENV)
- `/home/eli/anomura/crates/anomura_plugins/src/mock_discover_pass.rs` (IPC extension)
- `/home/eli/anomura/crates/anomura_plugins/src/substitution_pass.rs` (routing)
- `/home/eli/anomura/crates/anomura_plugins/src/bin/cargo-mock.rs` (pass targets)
- `/home/eli/anomura/test-suite/mocks/src/main.rs` (added mock_crate!(fns))
- `/home/eli/anomura/docs/design/mock_crate.md` (design doc)
