use crate::{
    ConditionDoublePointer, MockId, ReturnValDoublePointer, builder::ContextBuilder,
    errors::PredicateResult, time_mod::TimeModifier,
};

#[test]
fn pointers1() {
    let a: Box<dyn Fn(&u32) -> PredicateResult<()> + 'static> =
        Box::new(|a| if *a > 2 { Ok(()) } else { Err("error".into()) });
    let double_ptr = ConditionDoublePointer::from_fn(a);
    let casted = unsafe { double_ptr.into_fn::<u32>() };
    assert!(casted(&3).is_ok());
    assert!(casted(&2).is_err());
}

#[test]
fn pointers2() {
    struct TestStruct {
        pub string: String,
    }

    let a: Box<dyn Fn() -> TestStruct + 'static> = Box::new(|| {
        println!("this is a closure return val");
        TestStruct {
            string: String::from("hello pointers2"),
        }
    });
    let double_ptr = ReturnValDoublePointer::from_fn(a);
    let casted = unsafe { double_ptr.into_fn::<TestStruct>() };
    assert_eq!(casted().string, "hello pointers2");
    assert_ne!(casted().string, "goodbye pointer2");
}

#[test]
fn context1() {
    println!("start of test");
    struct Foo(u32);
    struct Bar(String);
    let mock_id_foo = MockId("foo".into());
    let mock_id_bar = MockId("bar".into());
    let mut context_builder = ContextBuilder::new();
    assert!(
        context_builder
            .add_mock(mock_id_foo.clone(), Some(Box::new(|| Foo(42))))
            .is_ok()
    );
    let expectation1: Box<dyn Fn(&u32) -> PredicateResult<()> + 'static> =
        Box::new(|a| if *a == 7 { Ok(()) } else { Err("not 7".into()) });
    let expectation2 = |a: &u32| -> PredicateResult<()> {
        if *a == 42 {
            Ok(())
        } else {
            Err("not 42".into())
        }
    };
    let return_clos = || Foo(100);
    assert!(
        context_builder
            .add_expectation::<u32, Foo>(
                &mock_id_foo,
                expectation1,
                None,
                TimeModifier::Once,
                false
            )
            .is_ok()
    );
    assert!(
        context_builder
            .add_expectation(
                &mock_id_foo,
                Box::new(expectation2),
                Some(Box::new(return_clos)),
                TimeModifier::Once,
                true
            )
            .is_ok()
    );

    let mut global_context = context_builder.finish();
    println!("here");
    let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), &7) else {
        panic!("failed first run");
    };
    let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), &result.0) else {
        panic!("failed first run");
    };
}
#[test]
fn context2() {
    println!("start of test");
    struct Foo(u32);
    struct Bar(String);
    let mock_id_foo = MockId("foo".into());
    let mock_id_bar = MockId("bar".into());
    let mut context_builder = ContextBuilder::new();
    assert!(
        context_builder
            .add_mock(mock_id_foo.clone(), Some(Box::new(|| Foo(42))))
            .is_ok()
    );
    assert!(
        context_builder
            .add_mock(mock_id_bar.clone(), Some(Box::new(|| Bar("getget".into()))))
            .is_ok()
    );

    let expectation1: Box<dyn Fn(&u32) -> PredicateResult<()> + 'static> =
        Box::new(|a| if *a == 7 { Ok(()) } else { Err("not 7".into()) });
    let expectation2 = |a: &u32| -> PredicateResult<()> {
        if *a == 42 {
            Ok(())
        } else {
            Err("not 42".into())
        }
    };
    let bar_expectation1: Box<dyn Fn(&Bar) -> PredicateResult<()> + 'static> = Box::new(|a| {
        if a.0 == "hello" {
            Ok(())
        } else {
            Err("not hello".into())
        }
    });
    let bar_expectation2 = |a: &Bar| -> PredicateResult<()> {
        if a.0 == "goodbye" {
            Ok(())
        } else {
            Err("bar not goodbye".into())
        }
    };
    let bar_ret1 = Box::new(|| Bar("goodbye".into()));

    let return_clos = || Foo(100);

    assert!(
        context_builder
            .add_expectation::<u32, Foo>(
                &mock_id_foo,
                expectation1,
                None,
                TimeModifier::Once,
                false
            )
            .is_ok()
    );
    assert!(
        context_builder
            .add_expectation(
                &mock_id_foo,
                Box::new(expectation2),
                Some(Box::new(return_clos)),
                TimeModifier::Once,
                true
            )
            .is_ok()
    );
    assert!(
        context_builder
            .add_expectation::<Bar, Bar>(
                &mock_id_bar,
                Box::new(bar_expectation1),
                Some(bar_ret1),
                TimeModifier::Once,
                false
            )
            .is_ok()
    );
    assert!(
        context_builder
            .add_expectation::<Bar, Bar>(
                &mock_id_bar,
                Box::new(bar_expectation2),
                None,
                TimeModifier::Once,
                true
            )
            .is_ok()
    );

    let mut global_context = context_builder.finish();
    println!("here");
    let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), &7) else {
        panic!("failed first run");
    };

    let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), &result.0) else {
        panic!("failed second run");
    };

    let Ok(goodbye) =
        global_context.run_mock::<Bar, Bar>(mock_id_bar.clone(), &Bar("hello".into()))
    else {
        panic!("failed third run");
    };

    let Ok(result) = global_context.run_mock::<Bar, Bar>(mock_id_bar.clone(), &goodbye) else {
        panic!("failed fourth run");
    };
}
