use context::{ContextBuilder, MockId, Result, TimeModifier};

pub fn main() {
    println!("start of test");
    struct Foo(u32);
    struct Bar(String);
    let mock_id_foo = MockId::new("foo");
    let mock_id_bar = MockId::new("bar");
    let mut context_builder = ContextBuilder::new();
    assert!(
        context_builder
            .add_mock(mock_id_foo.clone(), Some(Box::new(|| Foo(42))))
            .is_ok()
    );
    let expectation1 =
        |a: &u32| -> Result<()> { if *a == 7 { Ok(()) } else { Err("not 7".into()) } };
    let expectation2 = |a: &u32| -> Result<()> {
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
                Box::new(expectation1),
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
    /*let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), result.0) else {
        panic!("failed second run");
    };*/
}
