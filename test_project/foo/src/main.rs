use bar::{TestStruct1, TestStruct2};
use mock_macro::{end_mock_setup, mock_fn, start_mock_setup};
use std::sync::Mutex;
fn main() {
    start_mock_setup!();
    mock_fn!(
        foo,
        fn handler1(input: TestStruct1) -> TestStruct2 {
            default_return({ TestStruct2 { n: 42, m: 7 } });
        }
    );
    mock_fn!(
        foo,
        fn handler2(input: TestStruct2) -> TestStruct1 {
            default_return({
                TestStruct1 {
                    text: "default_return".into(),
                }
            });
        }
    );
    mock_fn!(
        rand,
        fn random_bool(prob: f64) -> bool {
            default_return({
                std::println!("greetings from context: {}", context::CONTEXT_CONST);
                true
            });
            expect(
                *prob > 0.5,
                Once,
                with_return({
                    println!("set return value false for first expect");
                    false
                }),
            );
            expect(*prob < 0.9, once());
        }
    );
    end_mock_setup!();
    let fst = rand::random_bool(1.0);
    println!("fst return val {fst}");
    if !fst {
        let snd = rand::random_bool(0.7);
        println!("second return val {snd}");
    }
}
/*
#[test]
fn macro_test() {
    start_mock_setup!();
    mock_fn!(
        rand,
        fn random_bool(prob: f64) -> bool {
            default_return({
                std::println!("greetings from context: {}", context::CONTEXT_CONST);
                context::context();
                true
            });
            expect(
                *prob > 0.5,
                once(),
                with_return({
                    println!("set return value false for first expect");
                    false
                }),
            );
            expect(*prob < 0.9, once());
        }
    );
    end_mock_setup!();
    let fst = rand::random_bool(0.7);
    println!("fst return val {fst}");
    if !fst {
        let snd = rand::random_bool(0.89);
        println!("second return val {snd}");
    }
}*/
