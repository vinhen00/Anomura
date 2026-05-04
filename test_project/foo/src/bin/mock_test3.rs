use bar::{TestStruct1, TestStruct2, handler1, handler2};
use mock_macro::mock_fn;
use std::sync::Mutex;

fn gab() -> TestStruct2 {
    println!("hello gab");
    TestStruct2 { n: 4, m: 5 }
}

fn main() {
    mock_fn!(
        bar,
        fn handler1(input: TestStruct1) -> TestStruct2 {
            default_return(gab());
            expect(
                input.text.starts_with("hello"),
                Once,
                with_return(TestStruct2 { n: 100, m: 58 }),
            );
            expect(
                input.text.len() <= 7,
                TimeModifier::Any,
                with_return({ TestStruct2 { n: 0, m: 1 } }),
            );

            expect(input.text == "01234567", TimeModifier::Once);
            expect(
                input.text.ends_with("goodbye"),
                TimeModifier::AtLeastOnce,
                with_return(TestStruct2 { n: 1, m: 33 }),
            );
        }
    );

    mock_fn!(
        bar,
        fn handler2(input: TestStruct2) -> TestStruct1 {
            default_return({
                TestStruct1 {
                    text: "default_return".into(),
                }
            });
            expect(input.n > 200 || input.m < 10, Any);
            expect((input.n + input.m) == 158, AtMostOnce);
            expect(
                input.n * input.m == input.m,
                AtLeastOnce,
                with_return(TestStruct1 {
                    text: ":c goodbye".into(),
                }),
            );
        }
    );
    context::finish_building_context();

    let struct1 = TestStruct1 {
        text: "hello world".into(),
    };
    let struct2_res = handler1(struct1);
    handler2(TestStruct2 { n: 300, m: 14 });
    handler2(TestStruct2 { n: 0, m: 7 });
    if rand::random_bool(0.5) {
        handler2(struct2_res);
    }

    let mut any = TestStruct1 {
        text: String::with_capacity(8),
    };
    for n in 0..=7 {
        any.text.push_str(&format!("{}", n));
        handler1(any.clone());
    }
    let struct1_res = handler2(TestStruct2 { n: 1, m: 44 });
    let struct2_res2 = handler1(struct1_res);
    dbg!(&struct2_res2);
    handler2(struct2_res2);
}
