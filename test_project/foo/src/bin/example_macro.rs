use std::marker::PhantomData;

use context::{TimesModifier, new_expectations::SingleExpectation};

// in crate named krate
mod Mod {
    use context::{ConditionDoublePointer, Expectation};
    use std::marker::PhantomData;

    mock_macro::mock_adt! {
        krate,

        mod Mod {
            pub struct Example {
                a: f32,
                pub b: f32,
            }

            pub trait ExTrait {
                pub fn meth2(&mut self, text: String) -> bool;
                fn private_helper(&self) -> u8;
            }
            impl ExTrait for Example {}

            impl Example {
                fn meth1(&self, a: f32, b: f32) -> usize;
                fn new(a: f32, b: f32) -> Self;
            }

            trait From<(f32, f32)> {
                pub fn from(value: (f32, f32)) -> Self;
            }
            impl From for Example {}
        }

        mod Pub {
            pub struct Point {
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
                radius: f32,
            }

            impl Circle {
                fn radius(&self) -> f32;
                fn new(radius: f32) -> Self;
            }
        }

        mod RectMod {
            pub struct Rect {
                w: f32,
                h: f32,
            }

            impl Rect {
                fn width(&self) -> f32;
                fn new(w: f32, h: f32) -> Self;
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

        mod PairMod {
            pub enum Pair {
                Both(Circle, Rect),
            }

            impl Pair {
                fn describe(&self) -> usize;
                fn new_both(a: Circle, b: Rect) -> Self;
            }
        }

        mod MaybeShapeMod {
            pub enum MaybeShape {
                Some(Circle),
                None,
            }

            impl MaybeShape {
                fn label(&self) -> usize;
                fn new_some(inner: Circle) -> Self;
                fn new_none() -> Self;
            }
        }

        mod ContainerMod {
            pub enum Container {
                Wrapped(Shape),
            }

            impl Container {
                fn size(&self) -> usize;
                fn new_wrapped(inner: Shape) -> Self;
            }
        }
    }
}

use crate::Mod::*;

pub fn main() {
    context::teardown();

    let mut ex = Example::new(1.0, 2.0);
    let ret = ReturnExampleMeth1::from_fn(|_self_ptr, _a, _b| 42);
    let pred = PredicateExampleMeth1::from_fn(|_self_ptr, _a, _b| Ok(()));
    ex.expect_meth1(None::<String>, pred, ret, None);

    let ret = ReturnExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| true);
    let pred = PredicateExampleImplExTraitMeth2::from_fn(|_self_ptr, _text| Ok(()));
    ex.expect_meth2(None::<String>, pred, ret, None);

    // Test From impl
    let ex2: Example = (3.0_f32, 4.0_f32).into();
    let ret = ReturnExampleMeth1::from_fn(|_self_ptr, a, b| (a + b) as usize);
    let pred = PredicateExampleMeth1::from_fn(|_self_ptr, _a, _b| Ok(()));
    ex2.expect_meth1(None::<String>, pred, ret, None);

    // Test all-public struct (Point)
    let pt = Point::new(3.0, 4.0);
    let ret = ReturnPointDistance::from_fn(|_self_ptr| 5.0);
    let pred = PredicatePointDistance::from_fn(|_self_ptr| Ok(()));
    pt.expect_distance(None::<String>, pred, ret, None);

    // Test enum mocking
    let circle = Circle::new(5.0);
    let shape = Shape::new_circle(circle);
    let ret = ReturnShapeArea::from_fn(|_self_ptr| 78.5);
    let pred = PredicateShapeArea::from_fn(|_self_ptr| Ok(()));
    shape.expect_area(None::<String>, pred, ret, None);

    // Test enum with Rect variant
    let rect = Rect::new(3.0, 4.0);
    let shape2 = Shape::new_rect(rect);
    let ret2 = ReturnShapeArea::from_fn(|_self_ptr| 12.0);
    let pred2 = PredicateShapeArea::from_fn(|_self_ptr| Ok(()));
    shape2.expect_area(None::<String>, pred2, ret2, None);

    // Test multi-field variant (Pair::Both(Circle, Rect)) — trackable via Circle
    let circle2 = Circle::new(2.0);
    let rect2 = Rect::new(5.0, 6.0);
    let pair = Pair::new_both(circle2, rect2);
    let ret_pair = ReturnPairDescribe::from_fn(|_self_ptr| 99);
    let pred_pair = PredicatePairDescribe::from_fn(|_self_ptr| Ok(()));
    pair.expect_describe(None::<String>, pred_pair, ret_pair, None);

    // Test enum with unit variant (MaybeShape) — should be non-trackable (shared IDs)
    let circle3 = Circle::new(1.0);
    let maybe = MaybeShape::new_some(circle3);
    let ret_maybe = ReturnMaybeShapeLabel::from_fn(|_self_ptr| 42);
    let pred_maybe = PredicateMaybeShapeLabel::from_fn(|_self_ptr| Ok(()));
    maybe.expect_label(None::<String>, pred_maybe, ret_maybe, Some(context::TimesModifier::Any));

    // Second MaybeShape instance — should share the same mock (non-trackable)
    let maybe2 = MaybeShape::new_none();

    // Test nested enum (Container wraps Shape, which is trackable) — Container should be trackable too
    let circle4 = Circle::new(10.0);
    let inner_shape = Shape::new_circle(circle4);
    let container = Container::new_wrapped(inner_shape);
    let ret_container = ReturnContainerSize::from_fn(|_self_ptr| 777);
    let pred_container = PredicateContainerSize::from_fn(|_self_ptr| Ok(()));
    container.expect_size(None::<String>, pred_container, ret_container, None);

    // Test trait mock (MockExTrait)
    let mut mock_trait = MockExTrait::new();
    let ret_trait = ReturnMockExTraitMeth2::from_fn(|_self_ptr, _text| true);
    let pred_trait = PredicateMockExTraitMeth2::from_fn(|_self_ptr, _text| Ok(()));
    mock_trait.expect_meth2(None::<String>, pred_trait, ret_trait, None);

    context::finish_building_context();

    // Test the mock calls
    assert_eq!(ex.meth1(1.0, 2.0), 42);
    assert!(ex.meth2("hello".into()));
    assert_eq!(ex2.meth1(3.0, 7.0), 10);

    // Test Point (all-public) mock
    assert_eq!(pt.distance(), 5.0);

    // Verify public fields work
    assert_eq!(ex.b, 2.0);
    assert_eq!(ex2.b, 4.0);
    assert_eq!(pt.x, 3.0);
    assert_eq!(pt.y, 4.0);

    // Test enum mocks
    assert_eq!(shape.area(), 78.5);
    assert_eq!(shape2.area(), 12.0);

    // Multi-field variant — independently tracked
    assert_eq!(pair.describe(), 99);

    // Non-trackable enum (has unit variant) — shared mock ID
    assert_eq!(maybe.label(), 42);
    // Second instance shares the same mock expectation (non-trackable)
    assert_eq!(maybe2.label(), 42);

    // Nested enum (Container wraps trackable Shape)
    assert_eq!(container.size(), 777);

    // Trait mock (MockExTrait)
    assert!(mock_trait.meth2("hello trait".into()));

    println!("All assertions passed!");
}
