use mock_macro::mock_crate;

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
    fns::Foo::on_call_fallback(fns::ReturnFooFallback::from_fn(|_self_ref, | 999u32));
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
fn mock_crate_foo_ret_owned_with_helper() {
    // Static method that returns Foo
    fns::Foo::on_call_ret_owned(fns::ReturnFooRet_owned::from_fn(|| fns::Foo { x: 77 }));
    context::finish_building_context();

    let foo = fns::Foo::ret_owned();
    assert_eq!(foo.x, 77);
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
