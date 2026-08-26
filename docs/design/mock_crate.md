# Design: `mock_crate!` macro

## Summary

`mock_crate!` is a "magical macro" that mocks an entire crate's public API automatically. Unlike `mock_fn!`/`mock_method!` (which require the user to manually specify each function signature), `mock_crate!` introspects the target crate during compilation and generates the full mock infrastructure for every public item.

## User-facing syntax

```rust
// In the test crate (e.g. test-suite/mocks/src/main.rs)
mock_crate!(fns);
// or with optional version (for disambiguation, not yet implemented):
mock_crate!(serde_json, "1.0.149");
```

This replaces needing any `mock_fn!`, `mock_method!`, or `mock_struct!` calls for the target crate. `mock_crate!` takes precedence over other mocking macros for the same crate.

## High-level flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Discover pass (compiles test crate with mock_discover_driver_exec)      │
│                                                                         │
│  1. ParseMocks walks AST, finds mock_crate!(fns)                        │
│  2. Extracts crate name "fns" → adds to crate_list                      │
│  3. Sends crate_list back via IPC (no program string needed for         │
│     mock_crate — the substitution pass will generate everything)        │
│                                                                         │
│  Output: crate_list = ["fns"], mock_crate_entries = ["fns"]             │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ Substitution pass (compiles target crate with mock_substitute_driver)   │
│                                                                         │
│  Uses a NEW driver mode: CrateIntercept (vs current FunctionIntercept)  │
│                                                                         │
│  Phase A: HIR Walk (collect crate API)                                  │
│    - Walk all pub items recursively through modules                      │
│    - Collect: pub structs (fields + visibility), pub enums (variants),  │
│      pub traits (methods), impl blocks (inherent + trait), pub fns      │
│    - Determine trackability (private fields → adt_mock_id)              │
│    - Build a CrateApiModel (mirrors MockAdtInput structure)             │
│                                                                         │
│  Phase B: Generate mock infrastructure                                  │
│    - From CrateApiModel, generate the same output as mock_adt!:         │
│      • Wrapper newtypes (PredicateFoo, ReturnFoo, ExpectationFoo)       │
│      • on_call_*, expect_*, create_predicate_* methods                  │
│      • Mock dispatch bodies (context::run_mock + fallback)              │
│      • adt_mock_id fields for trackable structs                         │
│      • Drop impls for cleanup                                           │
│      • Enum ID propagation                                              │
│      • Trait mock structs (MockTraitName)                               │
│                                                                         │
│  Phase C: AST rewrite                                                   │
│    - Replace function/method bodies with mock dispatch                   │
│    - Add adt_mock_id fields to structs with private fields              │
│    - Add _original copies for fallback                                  │
│    - Inject generated helper types/impls into the module tree           │
│    - Preserve module structure (pub mod foo { ... } stays intact)       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Detailed design

### 1. Discover pass changes

In `ParseMocks` (crates/anomura_driver/src/parse_mocks.rs):

- Add handling for `"mock_crate"` in `handle_maccall`:
  - Extract the crate name (first token)
  - Add to `self.crates` (for CrateFilter)
  - Mark it as a "full crate mock" (new field: `pub mock_crate_targets: Vec<String>`)
  - Do NOT generate any program string — the substitution pass handles everything

In `extract_path_value`:
- Add a `"mock_crate"` arm that extracts the crate name

The `CallBackMessage::NewMocks` payload or a new variant needs to carry the distinction between "these crates have individual mock_fn/mock_method defs" vs "these crates are full mock_crate targets".

**Proposed**: New `CallBackMessage` variant:
```rust
pub enum CallBackMessage {
    NewMocks(String, Vec<String>),          // existing: program string + crate list
    MockCrate(Vec<String>),                 // new: list of crates to fully mock
    Done,
}
```

Or simpler: add a `mock_crate_targets: Vec<String>` to `DiscoverClientReturn` alongside `mocked_fns` and `crate_list`.

### 2. Substitution pass changes

Currently `SubstitutePlugin::run()` calls `mock_map_from_program()` to parse the program string and get `MockObject`s, then creates a `FunctionIntercept` driver.

For `mock_crate` targets, we need a different path:

```
SubstitutePlugin::run(crate_name, compiler_args, plugin_args):
    if crate_name is in mock_crate_targets:
        use CrateIntercept driver (new)
    else:
        use FunctionIntercept driver (existing, for mock_fn/mock_method)
```

### 3. CrateIntercept driver (new)

This is the core new component. It implements `rustc_driver::Callbacks` and does:

**`after_crate_root_parsing` (AST level):**
- Not much — we need HIR for type resolution and trait impl info.

**`after_analysis` (HIR level):**
- Walk the HIR to collect the full public API:
  ```rust
  struct CrateApiModel {
      modules: Vec<ModuleModel>,  // preserves nesting
  }
  
  struct ModuleModel {
      name: Ident,
      structs: Vec<StructModel>,
      enums: Vec<EnumModel>,
      traits: Vec<TraitModel>,
      functions: Vec<FunctionModel>,
      children: Vec<ModuleModel>,
  }
  
  struct StructModel {
      name: Ident,
      fields: Vec<(Visibility, Ident, Type)>,
      inherent_methods: Vec<MethodModel>,
      trait_impls: Vec<(TraitName, Vec<MethodModel>)>,
  }
  // ... etc
  ```

**Problem**: `after_analysis` is too late to modify the AST (compilation has progressed). We need HIR data but AST modification.

**Solution**: Two-phase approach within a single compilation:
1. Use `after_crate_root_parsing` to do a *preliminary* AST walk (we can still gather struct fields, function signatures, impl blocks from the AST — we don't strictly need HIR for that).
2. Generate all mock infrastructure as new AST items and inject them.
3. Let compilation continue with the modified AST.

**Alternative**: Do a pre-compilation pass (similar to how `CompileMocks` works today) — compile the crate once just to collect metadata, then compile it again with modifications. This is closer to the existing two-pass architecture.

**Recommended approach**: Single-pass AST-level introspection. The AST already contains:
- All struct/enum definitions with fields and visibility
- All trait definitions with method signatures
- All impl blocks (inherent and trait)
- All function signatures
- Module structure

We don't strictly need HIR for the mock generation. The only thing HIR gives us that AST doesn't is resolved types (e.g., which trait impl belongs to which trait across crate boundaries) — but for items *within* the target crate, AST-level info is sufficient.

### 4. AST walk: collecting the CrateApiModel

In `after_crate_root_parsing`, walk the crate's items:

```rust
fn collect_module(items: &[P<Item>]) -> ModuleModel {
    for item in items {
        match &item.kind {
            ItemKind::Fn(fn_data) if is_pub(item) => {
                // collect free function
            }
            ItemKind::Struct(name, generics, fields) if is_pub(item) => {
                // collect struct with fields + visibility
            }
            ItemKind::Enum(name, generics, variants) if is_pub(item) => {
                // collect enum with variants
            }
            ItemKind::Trait(trait_data) if is_pub(item) => {
                // collect trait with method sigs
            }
            ItemKind::Impl(impl_data) => {
                // collect impl block, associate with struct/enum
            }
            ItemKind::Mod(_, _, mod_kind) if is_pub(item) => {
                // recurse into submodule
            }
            _ => {}
        }
    }
}
```

### 5. Code generation

Once we have the `CrateApiModel`, generate mock code equivalent to `mock_adt!` output. This can reuse/share logic with `mock_adt.rs` or be a parallel implementation in the driver.

**Key difference from mock_adt**: mock_adt generates code for the *test crate* (it's a proc macro that expands in the test crate). For mock_crate, the generated helpers (PredicateFoo, expect_*, on_call_*) go into the *target crate* itself as additional public items. The test crate then uses them via `fns::Example::expect_meth1(...)`.

Generated items per module (injected into the AST):
- Modified struct definitions (add `pub adt_mock_id: context::AdtMockId` if trackable)
- `_original` function copies
- Replaced function/method bodies with mock dispatch
- Wrapper newtypes: `pub struct PredicateStructMethod(...)`
- `impl Struct { pub fn expect_method(...) }` blocks
- `impl Struct { pub fn on_call_method(...) }` blocks
- `impl Drop for Struct { ... }` (for trackable types)
- `pub struct MockTraitName { ... }` + `impl Trait for MockTraitName`
- Enum `adt_mock_id()` methods

### 6. Module structure preservation

All generated items are injected into the same module where the original item lives. This means:
- `pub mod foo { pub fn bar() {} }` becomes:
  ```rust
  pub mod foo {
      pub fn bar() { /* mock dispatch */ }
      fn bar_original() { /* original body */ }
      // + helper types for bar
  }
  ```

### 7. What needs to be built

| Component | Location | Description |
|-----------|----------|-------------|
| `mock_crate!` proc macro | `crates/mock-macro/src/lib.rs` | No-op macro (like current `mock_struct!`), just exists so the discover pass can find it |
| Discover pass: `mock_crate` handling | `crates/anomura_driver/src/parse_mocks.rs` | Extract crate name, mark as full-crate mock |
| IPC extension | `crates/anomura_plugins/src/mock_discover_pass.rs` | Carry mock_crate targets to substitution pass |
| Substitution routing | `crates/anomura_plugins/src/main_sub.rs` | Route mock_crate targets to new driver |
| CrateApiModel | `crates/anomura_driver/src/crate_api.rs` (new) | Data structures for collected crate API |
| AST collector | `crates/anomura_driver/src/crate_intercept.rs` (new) | Walk AST, build CrateApiModel |
| Code generator | `crates/anomura_driver/src/crate_mock_gen.rs` (new) | Generate mock infrastructure from CrateApiModel |
| AST injector | part of `crate_intercept.rs` | Inject generated items back into the AST |

### 8. Implementation order

1. **Add `mock_crate!` no-op proc macro** (trivial)
2. **Extend discover pass** to recognize `mock_crate` and propagate it
3. **Build the CrateApiModel** and AST collector (can test in isolation)
4. **Build the code generator** (the bulk of the work — port mock_adt logic to work at the driver level, outputting AST items instead of proc_macro2 TokenStreams)
5. **Wire it into the substitution pass** (routing + AST injection)
6. **Test with `fns` crate** as the first target

### 9. Decisions made

- **External trait impls**: Yes, mock them. If `fns::Foo` implements `Display`, that impl's methods get mock dispatch too.
- **Generic types**: Skip for now. Generic structs, enums, and methods are not mocked in v1.
- **Re-exports**: Skip for now. `pub use other_mod::Thing` — we don't have the resolved info in the AST to follow these. Only items defined directly in the crate.
- **Derive macros**: Skip for now. Derive-generated impls (Clone, Debug, etc.) won't be in the AST at `after_crate_root_parsing` time, so we can't mock them. This is a known limitation.
- **Macro-generated items**: Same as derives — anything generated by proc macros won't be visible. Possible future solution: use `after_expansion` callback if available, or a two-pass approach where we expand macros first then collect. For now, accept this limitation.
