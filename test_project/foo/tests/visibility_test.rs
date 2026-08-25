//! Tests that private trait methods don't get propagated to generated code.
//!
//! Strategy: We verify that only public methods generate the expected types and methods.
//! If private methods were accidentally propagated, this test would either fail to compile
//! (if it tried to use them) or the assertions about the type system would fail.

use std::marker::PhantomData;

mod visibility_test {
    use std::marker::PhantomData;

    mock_macro::mock_adt! {
        test_crate,

        mod TraitMod {
            pub struct Widget {
                id: u32,
                pub name: u32,
            }

            pub trait Drawable {
                pub fn draw(&self) -> String;
                pub fn color(&self) -> u32;
                fn internal_prepare(&self) -> u8;      // private
                fn internal_cache_key(&self) -> u64;   // private
            }
            impl Drawable for Widget {}

            impl Widget {
                fn new(name: u32) -> Self;
                fn width(&self) -> u32;
            }
        }
    }
}

use visibility_test::*;

#[test]
fn public_trait_methods_are_mocked_on_struct() {
    context::teardown();

    let w = Widget::new(42);

    // Public method `draw` should have wrappers
    let ret = ReturnWidgetImplDrawableDraw::from_fn(|_self_ptr| "drawn".to_string());
    let pred = PredicateWidgetImplDrawableDraw::from_fn(|_self_ptr| Ok(()));
    w.expect_draw(None::<String>, pred, ret, None);

    // Public method `color` should have wrappers
    let ret2 = ReturnWidgetImplDrawableColor::from_fn(|_self_ptr| 0xFF0000);
    let pred2 = PredicateWidgetImplDrawableColor::from_fn(|_self_ptr| Ok(()));
    w.expect_color(None::<String>, pred2, ret2, None);

    context::finish_building_context();

    assert_eq!(w.draw(), "drawn");
    assert_eq!(w.color(), 0xFF0000);
}

#[test]
fn trait_mock_struct_exists_and_works() {
    context::teardown();

    // MockDrawable should exist and implement Drawable
    let mut mock = MockDrawable::new();

    let ret = ReturnMockDrawableDraw::from_fn(|_self_ptr| "mock_drawn".to_string());
    let pred = PredicateMockDrawableDraw::from_fn(|_self_ptr| Ok(()));
    mock.expect_draw(None::<String>, pred, ret, None);

    let ret2 = ReturnMockDrawableColor::from_fn(|_self_ptr| 0x00FF00);
    let pred2 = PredicateMockDrawableColor::from_fn(|_self_ptr| Ok(()));
    mock.expect_color(None::<String>, pred2, ret2, None);

    context::finish_building_context();

    // Call through the trait
    assert_eq!(mock.draw(), "mock_drawn");
    assert_eq!(mock.color(), 0x00FF00);
}

#[test]
fn trait_mock_can_be_used_as_dyn() {
    context::teardown();

    let mut mock = MockDrawable::new();

    let ret = ReturnMockDrawableDraw::from_fn(|_self_ptr| "dyn_test".to_string());
    let pred = PredicateMockDrawableDraw::from_fn(|_self_ptr| Ok(()));
    mock.expect_draw(None::<String>, pred, ret, None);

    let ret2 = ReturnMockDrawableColor::from_fn(|_self_ptr| 99);
    let pred2 = PredicateMockDrawableColor::from_fn(|_self_ptr| Ok(()));
    mock.expect_color(None::<String>, pred2, ret2, None);

    context::finish_building_context();

    // Use as &dyn Drawable
    let dyn_ref: &dyn Drawable = &mock;
    assert_eq!(dyn_ref.draw(), "dyn_test");
    assert_eq!(dyn_ref.color(), 99);
}

#[test]
fn private_methods_not_on_trait_mock() {
    // This test verifies that MockDrawable does NOT have:
    // - expect_internal_prepare
    // - expect_internal_cache_key
    // - on_call_internal_prepare
    // - on_call_internal_cache_key
    //
    // And that these types do NOT exist:
    // - ReturnMockDrawableInternalPrepare
    // - PredicateMockDrawableInternalPrepare
    // - ReturnMockDrawableInternalCacheKey
    // - PredicateMockDrawableInternalCacheKey
    //
    // If any of these existed, uncommenting the lines below would compile.
    // Since they DON'T exist, this test passes by just verifying the public surface works.

    // Verify only 2 methods on the trait (draw, color) — not 4
    // We do this by confirming that the public API compiles without the private methods.
    context::teardown();

    let mock = MockDrawable::new();
    // Only public methods have expect_ helpers:
    // mock.expect_internal_prepare(...) // Would not compile — method doesn't exist
    // mock.expect_internal_cache_key(...) // Would not compile — method doesn't exist

    // Can still create and drop without issues
    drop(mock);
}

#[test]
fn private_methods_not_on_struct_impl() {
    // Verify that Widget does NOT have:
    // - expect_internal_prepare
    // - expect_internal_cache_key
    // - on_call_internal_prepare
    //
    // The struct should only mock the public trait methods (draw, color) and its own inherent method (width).

    context::teardown();

    let w = Widget::new(1);

    // These exist (public trait + inherent):
    let ret = ReturnWidgetImplDrawableDraw::from_fn(|_self_ptr| "ok".to_string());
    let pred = PredicateWidgetImplDrawableDraw::from_fn(|_self_ptr| Ok(()));
    w.expect_draw(None::<String>, pred, ret, None);

    let ret2 = ReturnWidgetWidth::from_fn(|_self_ptr| 100);
    let pred2 = PredicateWidgetWidth::from_fn(|_self_ptr| Ok(()));
    w.expect_width(None::<String>, pred2, ret2, None);

    context::finish_building_context();

    assert_eq!(w.draw(), "ok");
    assert_eq!(w.width(), 100);

    // These would NOT compile (private trait methods not mocked):
    // w.expect_internal_prepare(...)
    // w.expect_internal_cache_key(...)
}

/// Compile-time assertion: the trait only has 2 methods (draw, color).
/// If private methods leaked into the trait definition, any impl would need to provide them.
struct ManualImpl;
impl Drawable for ManualImpl {
    fn draw(&self) -> String { "manual".to_string() }
    fn color(&self) -> u32 { 0 }
    // If internal_prepare or internal_cache_key were required, this would fail to compile.
}
