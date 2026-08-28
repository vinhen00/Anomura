use mock_macro::mock_crate;
use fns::Computable;

mock_crate!(fns);

fn main() {}

// ─── Free function tests ──────────────────────────────────────────────────────

#[test]
fn mock_crate_return_const() {
    fns::on_call_return_const(fns::ReturnReturn_const::from_fn(|| 42i16));
    context::finish_building_context();

    let result = fns::return_const();
    assert_eq!(result, 42);
}

#[test]
fn mock_crate_ret_call_w_args() {
    fns::on_call_ret_call_w_args(fns::ReturnRet_call_w_args::from_fn(|x| x * 3));
    context::finish_building_context();

    let result = fns::ret_call_w_args(5);
    assert_eq!(result, 15);
}

#[test]
fn mock_crate_match_const() {
    fns::on_call_match_const(fns::ReturnMatch_const::from_fn(|_key| ()));
    context::finish_building_context();

    fns::match_const(99);
}

// ─── Struct method tests (using generated on_call helpers) ────────────────────

#[test]
fn mock_crate_foo_fallback_with_helper() {
    // Use the generated on_call helper for Foo::fallback
    fns::Foo::on_call_fallback(fns::ReturnFooFallback::from_fn(|_self_ref| 999u32));
    context::finish_building_context();

    let foo = fns::Foo { x: 5 };
    let result = foo.fallback();
    assert_eq!(result, 999);
}

#[test]
fn mock_crate_foo_static_method_with_helper() {
    // Use the generated on_call helper for Foo::static_method
    fns::Foo::on_call_static_method(fns::ReturnFooStatic_method::from_fn(|| ()));
    context::finish_building_context();

    fns::Foo::static_method(); // should not panic
}

#[test]
fn mock_crate_foo_constructor_registers_mocks() {
    // ret_owned is a constructor — it creates a Foo and registers mocks for its methods.
    // Must be called during build phase (before finish_building_context).
    let foo = fns::Foo::ret_owned();
    context::finish_building_context();

    // The constructor registered mocks for ret_ref, ret_mut_ref, fallback
    // Foo has x: Default::default() = 0
    assert_eq!(foo.x, 0);
}

// ─── Test unmocked functions panic with clear message ─────────────────────────

#[test]
#[should_panic(expected = "mock_crate: no mock context built for return_panic")]
fn mock_crate_unmocked_panics() {
    context::finish_building_context();
    fns::return_panic();
}

// ─── Multiple mocks in one test ──────────────────────────────────────────────

#[test]
fn mock_crate_multiple_mocks() {
    fns::on_call_return_const(fns::ReturnReturn_const::from_fn(|| 100i16));
    fns::on_call_ret_call_w_args(fns::ReturnRet_call_w_args::from_fn(|x| x + 10));
    context::finish_building_context();

    assert_eq!(fns::return_const(), 100);
    assert_eq!(fns::ret_call_w_args(5), 15);
}

// ─── Submodule function test ─────────────────────────────────────────────────

#[test]
fn mock_crate_submodule_function() {
    fns::on_call_a_modules(fns::ReturnA_Modules::from_fn(|| 777u32));
    context::finish_building_context();

    let result = fns::a::modules();
    assert_eq!(result, 777);
}

// ─── Sequence test ───────────────────────────────────────────────────────────

#[test]
fn mock_crate_sequence() {
    // Create a sequence: calls return different values in order
    context::new_sequence("counting", 3, context::TimesModifier::Once, None).unwrap();

    fns::sequence_ret_call_w_args("counting", 0, |_x| Ok(()), |x| x + 100);
    fns::sequence_ret_call_w_args("counting", 1, |_x| Ok(()), |x| x + 200);
    fns::sequence_ret_call_w_args("counting", 2, |_x| Ok(()), |x| x + 300);

    context::finish_building_context();
    context::activate_sequence("counting").unwrap();

    assert_eq!(fns::ret_call_w_args(1), 101);  // step 0: 1 + 100
    assert_eq!(fns::ret_call_w_args(1), 201);  // step 1: 1 + 200
    assert_eq!(fns::ret_call_w_args(1), 301);  // step 2: 1 + 300
}

// ─── Checkpoint test ─────────────────────────────────────────────────────────

#[test]
fn mock_crate_checkpoints() {
    // Set up two checkpoints with different return values for the same function.
    // We need to place expectations before creating the next checkpoint,
    // since on_call_* adds to the latest checkpoint.

    // Register the mock first
    fns::on_call_return_const(fns::ReturnReturn_const::from_fn(|| 10i16));
    // ^ this goes into the default/first checkpoint

    context::new_checkpoint("phase2").unwrap();
    fns::on_call_return_const(fns::ReturnReturn_const::from_fn(|| 20i16));
    // ^ this goes into phase2

    context::finish_building_context();

    // In first checkpoint: should return 10
    assert_eq!(fns::return_const(), 10);

    // Advance to phase2
    context::control_checkpoint().unwrap();
    assert_eq!(fns::return_const(), 20);
}

// ─── Trait impl test ─────────────────────────────────────────────────────────

#[test]
fn mock_crate_trait_impl_debug() {
    // Mock the Debug::fmt implementation for ClosureWrapper using the generated helper
    fns::ClosureWrapper::on_call_fmt(fns::ReturnClosureWrapperFmt::from_fn(|_self_ref, f| {
        f.write_str("MOCKED!")
    }));
    context::finish_building_context();

    let cw = fns::ClosureWrapper(Box::new(|x| x));
    let debug_output = format!("{:?}", cw);
    assert_eq!(debug_output, "MOCKED!");
}

// ─── Struct initialization + expectations ────────────────────────────────────

#[test]
fn mock_crate_mock_struct_instance_mock() {
    // MockStruct is trackable (has private fields) → instance-specific mock IDs.
    // Constructor registers mocks, on_call_get_value is an instance method.
    let ms = fns::MockStruct::new();
    ms.on_call_get_value(fns::ReturnMockStructGet_value::from_fn(|_self_ref| 42u32));
    context::finish_building_context();

    assert_eq!(ms.get_value(), 42);
}

#[test]
fn mock_crate_foo_constructor_and_expectations() {
    // Foo is all-public → shared mock IDs, on_call is static.
    let _foo = fns::Foo::ret_owned(); // constructor registers mocks
    fns::Foo::on_call_fallback(fns::ReturnFooFallback::from_fn(|_self_ref| 123u32));
    context::finish_building_context();

    let foo = fns::Foo { x: 5 };
    assert_eq!(foo.fallback(), 123);
}

#[test]
fn mock_crate_foo_computable_trait_mock() {
    // Mock a crate-local trait impl via on_call
    fns::Foo::on_call_compute(fns::ReturnFooCompute::from_fn(|_self_ref| 999u32));
    context::finish_building_context();

    let foo = fns::Foo { x: 1 };
    assert_eq!(foo.compute(), 999);
}

#[test]
fn mock_crate_two_mock_struct_instances() {
    // Two instances of MockStruct should have independent mock IDs
    let ms1 = fns::MockStruct::new();
    let ms2 = fns::MockStruct::new();

    ms1.on_call_get_value(fns::ReturnMockStructGet_value::from_fn(|_self_ref| 100u32));
    ms2.on_call_get_value(fns::ReturnMockStructGet_value::from_fn(|_self_ref| 200u32));
    context::finish_building_context();

    assert_eq!(ms1.get_value(), 100);
    assert_eq!(ms2.get_value(), 200);
}
