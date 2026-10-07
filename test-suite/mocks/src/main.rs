use fns::Computable;
use mock_macro::mock_crate;

// Invoke the `mock_crate!` proc macro for the `fns` crate. During the discover pass,
// the custom rustc driver intercepts this macro invocation and identifies `fns` as a
// crate whose public API should be mocked. During the substitution pass, every public
// function and method body in `fns` is replaced with mock-dispatch logic that checks
// the runtime context for registered mocks. The macro also generates convenience types
// and helpers in the `fns` namespace:
//   - `fns::on_call_<fn_name>(...)` — registers a mock closure for a free function
//   - `fns::Return<FnName>::from_fn(...)` — wraps a closure into the expected return-type wrapper
//   - `fns::Foo::on_call_<method>(...)` — static helper to register a mock for a method on an all-public struct
//   - `instance.on_call_<method>(...)` — instance method to register a mock on a trackable (private-field) struct
//   - `fns::sequence_<fn_name>(...)` — registers one step of a sequenced mock
//   - `fns::ClosureWrapper::on_call_fmt(...)` — registers a mock for a trait-impl method
mock_crate!(fns);

fn main() {}

// ─── Free function tests ──────────────────────────────────────────────────────

/// Validates that a free function with no arguments can be mocked to return a constant value.
///
/// This is the most basic mocking scenario: intercept a zero-argument function and override
/// its return value. If this fails, the entire mock dispatch pipeline is broken — the
/// substitution pass isn't replacing function bodies, or the runtime context isn't routing
/// calls through registered closures. This test is the canary for the end-to-end mock system.
///
/// Uses `a::nested::deep_fn() -> &'static str` — a zero-arg function inside a nested
/// submodule, which also validates that the mock system handles nested module paths.
#[test]
fn mock_crate_return_const() {
    // Register a mock for `fns::a::nested::deep_fn()`. The real implementation returns "deep";
    // we override it to return "mocked". `ReturnNested_Deep_fn` is the generated wrapper type
    // that adapts our closure to the mock dispatch interface. `from_fn` wraps the closure.
    fns::on_call_nested_deep_fn(|| true, fns::ReturnNested_Deep_fn::from_fn(|| "mocked"));

    // Transition from build phase to active phase. After this call, no more mocks can be
    // registered, and all subsequent function calls will be dispatched through the mock context.
    context::finish_building_context();

    // Call the real function path — the substituted body routes through our mock closure.
    let result = fns::a::nested::deep_fn();

    // Verify the mock's return value was used instead of the real implementation's "deep".
    assert_eq!(result, "mocked");
}

/// Validates that a free function receiving arguments correctly forwards them to the mock closure.
///
/// This tests argument passthrough: the mock closure receives the same arguments the caller
/// passed to the original function. If this fails, the generated dispatch code is dropping
/// or mishandling function parameters during the mock routing.
#[test]
fn mock_crate_ret_call_w_args() {
    // Register a mock for `fns::ret_call_w_args(x: i16) -> i16`. The real implementation
    // returns `x` unchanged; we override it with `x * 3` to prove our closure receives `x`.
    fns::on_call_ret_call_w_args(
        |_: &i16| true,
        fns::ReturnRet_call_w_args::from_fn(|x| x * 3),
    );

    // Lock the context — all mocks are now active.
    context::finish_building_context();

    // Call with argument 5. The mock closure computes 5 * 3 = 15.
    let result = fns::ret_call_w_args(5);

    // Confirm the argument was correctly forwarded and the mock's computation was used.
    assert_eq!(result, 15);
}

/// Validates mocking a function whose return type is unit `()` and that takes an argument.
///
/// Not all mocked functions return meaningful values. This test ensures the framework handles
/// unit-return functions without panicking, and that the argument is still forwarded to the
/// mock closure (even if the closure ignores it). If this breaks, unit-return function mocking
/// is unsound.
///
/// Uses `ref_param(x: &u32)` — a function taking a reference argument and returning `()`.
#[test]
fn mock_crate_match_const() {
    // Register a mock for `fns::ref_param(x: &u32)` which returns `()`.
    // The closure accepts `_x` (ignored) and returns unit.
    fns::on_call_ref_param(|_: &&u32| true, fns::ReturnRef_param::from_fn(|_x| ()));

    // Finalize the mock context.
    context::finish_building_context();

    // Call with an arbitrary argument. Success means no panic — the mock dispatch
    // handled the unit return type correctly.
    fns::ref_param(&99);
}

// ─── Struct method tests (using generated on_call helpers) ────────────────────

/// Validates mocking an instance method on a struct with all-public fields (shared mock ID).
///
/// `Foo` has only public fields (`pub x: u32`), so it uses a shared/static mock ID — all
/// instances of `Foo` share the same mock registrations. The generated `Foo::on_call_fallback`
/// is a static helper (not instance-specific). This test confirms that:
///   1. The static `on_call_*` helper correctly registers a mock for instance methods.
///   2. The `&self` receiver is forwarded to the mock closure.
///   3. Any `Foo` instance dispatches through the shared mock.
/// If this fails, all-public-struct method mocking is broken.
#[test]
fn mock_crate_foo_fallback_with_helper() {
    // Register a mock for `Foo::fallback(&self) -> u32` using the static helper.
    // The real implementation returns 11; we override it to return 999.
    // The closure receives `_self_ref` (the &self reference) which we ignore here.
    fns::Foo::on_call_fallback(
        |_: &fns::Foo| true,
        fns::ReturnFooFallback::from_fn(|_self_ref| 999u32),
    );

    // Finalize — mocks are now active.
    context::finish_building_context();

    // Create a Foo directly (not via constructor). Since Foo is all-public, we can
    // construct it with struct literal syntax. The shared mock still applies.
    let foo = fns::Foo { x: 5 };

    // Call the mocked method. The substituted body routes through the shared mock.
    let result = foo.fallback();

    // Verify the mock's return value overrides the real implementation's return of 11.
    assert_eq!(result, 999);
}

/// Validates mocking a static method (no `self` receiver) on a struct.
///
/// Static methods have no instance — they're called as `Foo::static_method()`. The mock
/// system must handle the absence of a `self` parameter. If this fails, the code generator
/// incorrectly assumes all methods have a receiver, or the dispatch logic can't route
/// receiverless calls.
#[test]
fn mock_crate_foo_static_method_with_helper() {
    // Register a mock for `Foo::static_method()`. The real implementation is a no-op;
    // we mock it with another no-op to confirm dispatch works without a self receiver.
    fns::Foo::on_call_static_method(|| true, fns::ReturnFooStatic_method::from_fn(|| ()));

    // Finalize the mock context.
    context::finish_building_context();

    // Call the static method. If dispatch fails, this would panic with a "no mock context" error.
    fns::Foo::static_method();
}

/// Validates that a constructor function (returning `Self`) correctly registers mocks
/// during the build phase and produces a valid instance.
///
/// `Foo::ret_owned()` is a constructor that returns `Foo`. When mocked, it may register
/// mocks for the struct's methods as part of its substituted body. This test verifies:
///   1. The constructor can be called during the build phase.
///   2. The returned instance has default-initialized fields (x: 0 via `Default::default()`).
///   3. Calling the constructor doesn't panic — meaning the mock registration within it succeeded.
/// If this fails, constructor-time mock registration for all-public structs is broken.
#[test]
fn mock_crate_foo_constructor_registers_mocks() {
    // Call the constructor during the build phase. The substituted `ret_owned` body
    // creates a `Foo` with `x: Default::default()` (which is 0 for u32) and registers
    // mocks for Foo's methods (ret_ref, ret_mut_ref, fallback, etc.).
    let foo = fns::Foo::ret_owned();

    // Finalize — mocks registered by the constructor are now active.
    context::finish_building_context();

    // Verify the constructed Foo has default-initialized fields, not the real
    // implementation's value of 10. This proves the substituted constructor body ran.
    assert_eq!(foo.x, 0);
}

// ─── Test unmocked functions panic with clear message ─────────────────────────

/// Validates that calling a function with no registered mock produces a clear panic message.
///
/// When no mock is registered for a function and `finish_building_context()` has been called,
/// the substituted body should panic with a descriptive error identifying the unmocked function.
/// This is a critical developer-experience feature: without it, unmocked calls would silently
/// return garbage or cause undefined behavior. The panic message must include the function name
/// so the developer knows exactly which mock is missing.
#[test]
#[should_panic(expected = "mock_crate: no mock context built for return_panic")]
fn mock_crate_unmocked_panics() {
    // Finalize the context WITHOUT registering any mocks. This means every function
    // in the `fns` crate will hit the "no mock context" branch in the substituted body.
    context::finish_building_context();

    // Call an unmocked function. The substituted body detects that no mock was registered
    // for `return_panic` and panics with the expected message.
    fns::return_panic();
}

// ─── Multiple mocks in one test ──────────────────────────────────────────────

/// Validates that multiple different functions can be mocked simultaneously in a single test.
///
/// Real-world tests will mock several functions at once. This test ensures that the mock
/// context correctly stores and dispatches multiple independent function mocks without
/// interference. If this fails, the context's internal map is colliding on mock IDs, or
/// only the last registered mock takes effect.
#[test]
fn mock_crate_multiple_mocks() {
    // Register two independent mocks: one for deep_fn, one for ret_call_w_args.
    // Both are registered before finish_building_context — the context must hold both.
    fns::on_call_nested_deep_fn(|| true, fns::ReturnNested_Deep_fn::from_fn(|| "multi"));
    fns::on_call_ret_call_w_args(
        |_: &i16| true,
        fns::ReturnRet_call_w_args::from_fn(|x| x + 10),
    );

    // Finalize — both mocks are now active simultaneously.
    context::finish_building_context();

    // Verify each mock operates independently with its own return logic.
    assert_eq!(fns::a::nested::deep_fn(), "multi");
    assert_eq!(fns::ret_call_w_args(5), 15);
}

// ─── Submodule function test ─────────────────────────────────────────────────

/// Validates that functions inside submodules of the mocked crate are correctly intercepted.
///
/// The `fns` crate has a submodule `pub mod a` containing `pub fn modules() -> u32`. The
/// mock system must recursively discover and substitute functions in nested modules, not
/// just top-level ones. The `on_call_a_modules` helper uses a flattened naming convention
/// (module path joined with underscores). If this fails, the discover pass or code generator
/// doesn't recurse into submodules.
#[test]
fn mock_crate_submodule_function() {
    // Register a mock for `fns::a::modules()`. The generated helper name flattens the
    // module path: `on_call_a_modules` corresponds to `fns::a::modules`. The return type
    // wrapper follows the same convention: `ReturnA_Modules`.
    fns::on_call_a_modules(|| true, fns::ReturnA_Modules::from_fn(|| 777u32));

    // Finalize the mock context.
    context::finish_building_context();

    // Call through the real module path `fns::a::modules()`. The substituted body
    // in the `a` submodule dispatches through the mock registered above.
    let result = fns::a::modules();

    // Verify the mock overrode the real implementation (which returns 0).
    assert_eq!(result, 777);
}

// ─── Sequence test ───────────────────────────────────────────────────────────

/// Validates that sequenced mocks return different values on successive calls in order.
///
/// Sequences allow a single function to behave differently on each invocation — essential
/// for testing stateful protocols, retry logic, or iterative algorithms. This test creates
/// a 3-step sequence where `ret_call_w_args` returns increasingly larger offsets on each call.
/// If this fails, the sequence state machine (step tracking, advancement, exhaustion) is broken,
/// or sequenced mocks don't integrate with the dispatch pipeline.
#[test]
fn mock_crate_sequence() {
    // Create a named sequence "counting" with 3 steps. `TimesModifier::Once` means the
    // entire sequence plays through exactly once (each step fires once, in order).
    // `None` for the `after` parameter means this sequence has no prerequisite sequence.
    context::new_sequence("counting", 3, context::TimesModifier::Once, None).unwrap();

    // Register each step of the sequence. Each call to `sequence_ret_call_w_args` binds
    // a step index (0, 1, 2) to a matcher closure and a return closure:
    //   - The first closure `|_x| Ok(())` is the argument matcher (always matches here).
    //   - The second closure is the return-value computation for that step.
    fns::sequence_ret_call_w_args("counting", 0, |_x| Ok(()), |x| x + 100);
    fns::sequence_ret_call_w_args("counting", 1, |_x| Ok(()), |x| x + 200);
    fns::sequence_ret_call_w_args("counting", 2, |_x| Ok(()), |x| x + 300);
    //
    // Finalize mock registrations.
    context::finish_building_context();

    // Activate the sequence. Before activation, sequence steps are inert. This transitions
    // the sequence into its "playing" state, starting from step 0.
    context::activate_sequence("counting").unwrap();

    // Each call advances the sequence to the next step:
    // Step 0: closure is |x| x + 100, so ret_call_w_args(1) = 1 + 100 = 101
    assert_eq!(fns::ret_call_w_args(1), 101);
    // Step 1: closure is |x| x + 200, so ret_call_w_args(1) = 1 + 200 = 201
    assert_eq!(fns::ret_call_w_args(1), 201);
    // Step 2: closure is |x| x + 300, so ret_call_w_args(1) = 1 + 300 = 301
    assert_eq!(fns::ret_call_w_args(1), 301);
}

// ─── Checkpoint test ─────────────────────────────────────────────────────────

/// Validates that checkpoints partition mock expectations into sequential phases.
///
/// Checkpoints allow re-defining a function's mock behavior at runtime boundaries. This is
/// useful for tests with distinct setup/execution/teardown phases, or state machines where
/// the same function should behave differently after a certain event. Mocks registered before
/// a checkpoint belong to the current phase; mocks registered after `new_checkpoint` belong
/// to the next phase. `control_checkpoint()` advances the active phase at runtime.
/// If this fails, the checkpoint system doesn't correctly scope expectations or advance phases.
#[test]
fn mock_crate_checkpoints() {
    // Register a mock for `deep_fn` in the default (first) checkpoint phase.
    // This mock returns "phase1" and will be active until we advance past this phase.
    fns::on_call_nested_deep_fn(|| true, fns::ReturnNested_Deep_fn::from_fn(|| "phase1"));

    // Create a new checkpoint named "phase2". All subsequent mock registrations will
    // be associated with this new phase, not the default one.
    context::new_checkpoint("phase2").unwrap();

    // Register a different mock for the same function in phase2. This mock returns "phase2"
    // and will only become active after `control_checkpoint()` is called.
    fns::on_call_nested_deep_fn(|| true, fns::ReturnNested_Deep_fn::from_fn(|| "phase2"));

    // Finalize — both phases' mocks are stored, but only the first phase is active.
    context::finish_building_context();

    // In the first checkpoint phase: the mock returns "phase1".
    assert_eq!(fns::a::nested::deep_fn(), "phase1");

    // Advance to phase2. The first phase's mock for `deep_fn` is replaced by phase2's.
    context::control_checkpoint().unwrap();

    // In phase2: the same function now returns "phase2".
    assert_eq!(fns::a::nested::deep_fn(), "phase2");
}

// ─── Trait impl test ─────────────────────────────────────────────────────────

/// Validates mocking a trait implementation method (`Debug::fmt`) on a struct.
///
/// Trait-impl methods like `fmt` are dispatched differently from inherent methods — they
/// go through vtable/trait resolution. The mock system must intercept them at the impl level.
/// `ClosureWrapper` implements `Debug` manually, and we override `fmt` to write custom output.
/// If this fails, trait-impl method interception in the substitution pass is broken, or the
/// generated `on_call_fmt` helper doesn't correctly target the trait impl.
#[test]
fn mock_crate_trait_impl_debug() {
    // Register a mock for the `Debug::fmt` impl on `ClosureWrapper`. The closure receives
    // `_self_ref` (the &ClosureWrapper reference) and `f` (the formatter), matching the
    // real `fmt(&self, f: &mut Formatter) -> fmt::Result` signature.
    fns::ClosureWrapper::on_call_fmt(
        |_: &fns::ClosureWrapper, _: &&mut std::fmt::Formatter| true,
        fns::ReturnClosureWrapperFmt::from_fn(|_self_ref, f| f.write_str("MOCKED!")),
    );

    // Finalize the mock context.
    context::finish_building_context();

    // Construct a real ClosureWrapper. The inner closure is irrelevant — we're testing
    // the Debug impl, not the closure's behavior.
    let cw = fns::ClosureWrapper(Box::new(|x| x));

    // Trigger Debug::fmt via the `format!("{:?}", ...)` macro. This calls the substituted
    // `fmt` method, which routes through our mock closure.
    let debug_output = format!("{:?}", cw);

    // Verify the mock's output replaced the real Debug impl (which would output
    // "ClosureWrapper(<fn>)").
    assert_eq!(debug_output, "MOCKED!");
}

// ─── Struct initialization + expectations ────────────────────────────────────

/// Validates instance-specific mocking for trackable structs (structs with private fields).
///
/// `MockStruct` has a private field (`privfield`), making it "trackable" — each instance
/// gets a unique `AdtMockId` assigned by the constructor. This allows per-instance mock
/// registrations via `instance.on_call_*()` instead of the static `Struct::on_call_*()`.
/// This test confirms that:
///   1. The constructor assigns a unique mock ID to the instance.
///   2. `instance.on_call_get_value(...)` registers a mock scoped to that specific instance.
///   3. The mock dispatch correctly resolves the instance's ID at call time.
/// If this fails, trackable struct ID assignment or instance-scoped dispatch is broken.
#[test]
fn mock_crate_mock_struct_instance_mock() {
    // Create a MockStruct via its constructor. Because MockStruct has private fields,
    // the substituted constructor injects a unique `adt_mock_id` into the instance.
    // This ID is used to scope all subsequent mock registrations to this specific object.
    let ms = fns::MockStruct::new();

    // Register a mock for `get_value` on this specific instance. Unlike all-public structs
    // (where `Struct::on_call_*` is static), trackable structs use instance methods:
    // `ms.on_call_get_value(...)`. The mock is keyed to `ms`'s unique mock ID.
    ms.on_call_get_value(
        |_: &fns::MockStruct| true,
        fns::ReturnMockStructGet_value::from_fn(|_self_ref| 42u32),
    );

    // Finalize the mock context.
    context::finish_building_context();

    // Call the mocked method. The substituted body reads the instance's mock ID and
    // finds the registered closure. The real implementation returns `self.pubfield` (2).
    assert_eq!(ms.get_value(), 42);
}

/// Validates that calling a constructor during build phase + registering a static mock
/// both work together for all-public structs.
///
/// For all-public structs like `Foo`, the constructor (`ret_owned`) and the static
/// `on_call_*` helpers both operate on the same shared mock ID. This test confirms they
/// compose correctly: call the constructor first (which may register internal mocks),
/// then override a specific method via the static helper. Any `Foo` instance — even one
/// constructed separately — will use the statically-registered mock.
/// If this fails, there's a conflict between constructor-registered and explicitly-registered mocks.
#[test]
fn mock_crate_foo_constructor_and_expectations() {
    // Call the constructor during build phase. This registers default mocks for Foo's methods.
    // We assign to `_foo` because we don't use this instance — we're testing that the
    // static mock registered below takes precedence for any Foo instance.
    let _foo = fns::Foo::ret_owned();

    // Override the `fallback` mock via the static helper. Since Foo is all-public,
    // this shared mock applies to ALL Foo instances, including ones not yet created.
    fns::Foo::on_call_fallback(
        |_: &fns::Foo| true,
        fns::ReturnFooFallback::from_fn(|_self_ref| 123u32),
    );

    // Finalize — both the constructor's mocks and our explicit override are active.
    context::finish_building_context();

    // Create a completely separate Foo instance via struct literal syntax.
    // Because Foo uses shared mock IDs, this instance also dispatches through the
    // statically-registered mock.
    let foo = fns::Foo { x: 5 };

    // Verify the explicit mock (returning 123) is used, not the real implementation (11)
    // or any default the constructor may have registered.
    assert_eq!(foo.fallback(), 123);
}

/// Validates mocking a crate-local public trait implementation on a struct.
///
/// The `Computable` trait is defined in `fns` with `fn compute(&self) -> u32`, and `Foo`
/// implements it as `self.x * 2`. The mock system must intercept trait-impl methods for
/// crate-local traits just like inherent methods. The generated helper `Foo::on_call_compute`
/// targets the `impl Computable for Foo` block specifically.
/// If this fails, the discover pass misses crate-local trait impls, or the substitution pass
/// doesn't replace their method bodies.
#[test]
fn mock_crate_foo_computable_trait_mock() {
    // Register a mock for `Foo`'s implementation of `Computable::compute`. The generated
    // helper `on_call_compute` targets the trait-impl method, not an inherent method.
    fns::Foo::on_call_compute(
        |_: &fns::Foo| true,
        fns::ReturnFooCompute::from_fn(|_self_ref| 999u32),
    );

    // Finalize the mock context.
    context::finish_building_context();

    // Create a Foo with x: 1. The real `compute()` would return 1 * 2 = 2.
    let foo = fns::Foo { x: 1 };

    // Verify the mock overrides the trait impl — returns 999 instead of 2.
    assert_eq!(foo.compute(), 999);
}

/// Validates that two instances of a trackable struct have independent, non-interfering mocks.
///
/// This is the key differentiator between trackable structs (private fields → per-instance IDs)
/// and all-public structs (shared IDs). Two `MockStruct` instances must each get their own
/// unique `AdtMockId`, so mocks registered on one don't affect the other. This test is critical
/// because without proper ID isolation, mocking one instance would silently corrupt another —
/// a subtle, hard-to-debug correctness violation.
#[test]
fn mock_crate_two_mock_struct_instances() {
    // Create two separate MockStruct instances. Each constructor call generates a unique
    // mock ID, so ms1 and ms2 are independently mockable.
    let ms1 = fns::MockStruct::new();
    let ms2 = fns::MockStruct::new();

    // Register different return values for the same method on different instances.
    // ms1's `get_value` returns 100; ms2's `get_value` returns 200.
    // These registrations must not interfere because they're keyed to different mock IDs.
    ms1.on_call_get_value(
        |_: &fns::MockStruct| true,
        fns::ReturnMockStructGet_value::from_fn(|_self_ref| 100u32),
    );
    ms2.on_call_get_value(
        |_: &fns::MockStruct| true,
        fns::ReturnMockStructGet_value::from_fn(|_self_ref| 200u32),
    );

    // Finalize — both instances' mocks are active.
    context::finish_building_context();

    // Verify each instance dispatches through its own mock, not the other's.
    // If mock IDs were shared (like all-public structs), both would return whichever
    // was registered last — but trackable structs guarantee isolation.
    assert_eq!(ms2.get_value(), 200);
    assert_eq!(ms1.get_value(), 100);
}
