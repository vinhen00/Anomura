# Anomura (formerly project_mockingbird) — Session Notes (2026-08-25)

## Project Rename

The project was renamed from `project_mockingbird` to `anomura` (a taxonomy of crabs).

Crate renames:
- `mockingbird` → `anomura_driver` (at `crates/anomura_driver/`)
- `driver_test` → `anomura_plugins` (at `crates/anomura_plugins/`)
- `mockingbird_definitions` → `anomura_definitions` (at `crates/anomura_definitions/`)
- `mock-macro` and `context` crates keep their names

Project directory: `/home/eli/anomura`

---

## mock_adt! Macro — Complete Architecture

### Current Input Syntax

```rust
mock_adt! {
    krate,                    // crate name for mock ID prefix

    mod Mod {
        pub struct Example {
            a: f32,           // private → PhantomData, adt_mock_id added
            pub b: f32,
        }

        pub trait ExTrait {
            pub fn meth2(&mut self, text: String) -> bool;  // public → mocked
            fn private_helper(&self) -> u8;                 // private → excluded
        }
        impl ExTrait for Example {}

        impl Example {
            fn meth1(&self, a: f32, b: f32) -> usize;
            fn new(a: f32, b: f32) -> Self;   // constructor (Receiver::None)
        }

        trait From<(f32, f32)> {
            pub fn from(value: (f32, f32)) -> Self;  // trait constructor
        }
        impl From for Example {}
    }

    mod Pub {
        pub struct Point {   // all-public → no adt_mock_id, shared mock IDs
            pub x: f32,
            pub y: f32,
        }

        impl Point {
            fn distance(&self) -> f32;
            fn new(x: f32, y: f32) -> Self;
        }
    }

    mod CircleMod {
        pub struct Circle {
            radius: f32,     // private field → trackable
        }
        impl Circle {
            fn radius(&self) -> f32;
            fn new(radius: f32) -> Self;
        }
    }

    mod ShapeMod {
        pub enum Shape {
            Circle(Circle),
            Rect(Rect),
        }
        impl Shape {
            fn area(&self) -> f64;
            fn new_circle(inner: Circle) -> Self;
            fn new_rect(inner: Rect) -> Self;
        }
    }
}
```

### Architecture Overview

The macro works in these phases:

1. **Parse** — `MockAdtInput::parse()` reads crate ident + mod blocks recursively
2. **Flatten** — `flatten_modules()` walks the module tree, building `FlattenedStruct`, `FlattenedEnum`, and `FlattenedTrait` entries with full `syn::Path` (crate + module segments)
3. **Trackability Resolution** — fixpoint iteration determines which types can have per-instance mock IDs
4. **Classify** — `classify_methods()` splits methods into constructors (Receiver::None) vs mockable (has self receiver), carrying trait context
5. **Generate** — Per-type code gen:
   - Structs: `expand_single_struct()`
   - Enums: `expand_single_enum()`
   - Traits: `expand_trait_mock()`

### Generated Output per Mockable Method

For each method, the macro generates:
- **Wrapper newtypes**: `PredicateFoo`, `ReturnFoo`, `ExpectationFoo`
- **`Return::from_fn`**: closure `(self_ptr, params...) -> RetType`
- **`Predicate::from_fn`**: closure returning `Ok(())` or error
- **Mock method body**: dispatches to `context::run_mock`
- **`on_call_method`**: convenience for "always match, return this"
- **`create_predicate_method`**: instance-bound predicate
- **`expect_method`**: full expectation (predicate + return + times + checkpoint)
- **`method_times`**: wraps predicate in Times modifier
- **`expect_method_in_sequence`**: sequence support

---

## Changes Made This Session

### 1. Removed `from_impls` special case

- `FromImpl` struct and `from_impls: Vec<FromImpl>` removed from `MockAdtInput`
- `From` impls now handled as regular trait impls whose `from` method is a constructor
- Input syntax: `trait From<(f32, f32)> { pub fn from(value: (f32, f32)) -> Self; } impl From for Example {}`
- `TraitDef` gained `generics: Vec<Type>` for trait-level type params
- `classify_methods` routes `Receiver::None` trait methods to constructors with trait context
- `gen_constructors` returns `(inherent, trait_impls)` — trait constructors generate `impl Trait<Generics> for Struct` blocks

### 2. Restructured input: crate + mod blocks

- First argument: just the crate ident (not a full path)
- Content organized into `mod Name { ... }` blocks (supports multiple and nested modules)
- Data model:
  - `MockAdtInput { krate: Ident, modules: Vec<ModuleDef> }`
  - `ModuleDef { name, items: ModuleItems, children: Vec<ModuleDef> }`
  - `ModuleItems { vis, struct_name, fields, enum_name, variants, traits, trait_impls, inherent_impl }`
  - `FlattenedStruct` / `FlattenedEnum` / `FlattenedTrait` (internal, with computed paths)
- Module tree recursively flattened for code gen

### 3. All-public structs: no `adt_mock_id` field

- `FlattenedStruct.all_public: bool` — true when all fields are `pub`
- When `all_public`: no `adt_mock_id` field, path-only mock IDs, shared expectations
- When NOT `all_public`: instance-specific mock IDs with `adt_mock_id`
- Duplicate mock registration uses `let _ = ...` (ignore errors for shared IDs)

### 4. Bug fix: tuple destructure for single-element tuples

- `|(#(#param_names),*)|` → `|(#(#param_names,)*)|` (trailing comma per element)
- Fixes 1-tuple destructuring: `|(_0)|` (wrong) → `|(_0,)|` (correct)

### 5. Enum mocking support

**Trackability rules:**
- Structs: trackable if `!all_public`
- Enums: trackable if NO unit variants AND every variant has ≥1 trackable field
- Fixpoint iteration for recursive enum references
- Only crate-local types participate

**ID propagation (Option C):**
- Inner types own their IDs (constructors create normally)
- Enum extracts ID via generated `adt_mock_id()` match method
- For enum inner types: uses `.adt_mock_id()` (method call)
- For struct inner types: uses `.adt_mock_id` (field access)
- `trackable_field_is_enum: Vec<bool>` tracks which access pattern to use

**Constructor body generation:**
- Matches constructor param types to variant field types
- Supports single-field variants: `Self::Variant(param)`
- Supports multi-field variants: matches all fields to params
- Fallback: unit variant or Default::default()

**Generated code:**
- Enum definition (pass-through, no hidden fields)
- `adt_mock_id()` method (trackable only, match on variants)
- Drop impl (cleanup via ID extraction)
- All standard wrappers and helpers

### 6. Trait mock struct generation

For every `pub` trait, emits:
- `pub trait TraitName { /* only pub methods */ }` — actual trait definition
- `MockTraitName` struct with `adt_mock_id: context::AdtMockId`
- `impl TraitName for MockTraitName` with mock dispatch
- Full mocking infrastructure (wrappers, expect, on_call, etc.)
- Constructor: `MockTraitName::new()` registers mocks

**Visibility convention:**
- `pub fn meth(...)` → public: included in trait def, mocked everywhere
- `fn meth(...)` → private: excluded from trait def, NOT mocked

**MethodSig now has `vis: Visibility`** — filters private methods from:
- Emitted trait definitions
- Mock struct trait impls
- Struct/enum trait impls

### 7. Fixed `mock_fn` and `mock_method` macros for new context API

- Condition closures: wrapped in `ConditionDoublePointer::from_fn::<InputType>(...)`
  - Closure receives `&InputType`, destructured via `let (idents) = __input`
- Return closures: wrapped in `ReturnValDoublePointer::from_fn::<InputType, ReturnType>(...)`
- `add_expectation` call: now passes all 5 args (mock_id, condition, return_val, checkpoint_name=None, times_modifier)
- `TimesModifier` serialization: handles all variants (Once, Any, AtLeast, AtMost, Times, Never)
- `Expectation.ret` → `Expectation.ret_body: Option<Expr>` (cleaner separation)

### 8. Project rename: mockingbird → Anomura

- Directory: `/home/eli/project_mockingbird` → `/home/eli/anomura`
- `mockingbird` crate → `anomura_driver`
- `driver_test` crate → `anomura_plugins`
- `mockingbird_definitions` crate → `anomura_definitions`
- All Cargo.toml paths and dependencies updated
- All source `use` statements updated

---

## Key Design Decisions

- **Type erasure + monomorphic wrappers**: Context stores raw pointers; macro generates per-method wrapper types that know the correct `(Input, Ret)` tuple
- **Instance-specific vs shared mock IDs**: Private-field structs get unique per-instance IDs via `adt_mock_id`; all-public structs and non-trackable enums share a single ID per method
- **PhantomData for private fields**: Mock struct preserves type info without needing real values
- **Trait constructors**: Methods with `Receiver::None` in a trait are constructors, generating `impl Trait<Generics> for Struct` blocks
- **Checkpoint-based scoping**: All expectation registration goes through checkpoints
- **Option C for enum IDs**: Enum doesn't own an ID; extracts it from inner trackable field via match. No coordination needed at construction.
- **Trait visibility**: Public methods (`pub fn`) are mocked; unmarked methods are private and excluded entirely

---

## Key Files

- `crates/mock-macro/src/mock_adt.rs` — mock_adt! macro logic (~2800 lines)
- `crates/mock-macro/src/lib.rs` — proc macro entry points (mock_fn, mock_method, mock_adt)
- `crates/context/src/` — runtime mock context (expectations, predicates, closures)
- `test_project/foo/src/bin/example_macro.rs` — working example (structs, enums, traits, all cases)
- `test_project/foo/tests/visibility_test.rs` — tests for private method exclusion
- `test_project/foo/tests/mock_test1.rs` — tests for mock_fn macro
- `test_project/foo/tests/mock_test2.rs` — tests for mock_fn macro
- `test_project/foo/src/bin/mock_test3.rs` — tests for mock_fn + mock_method macros
- `.session-notes/` — this file
