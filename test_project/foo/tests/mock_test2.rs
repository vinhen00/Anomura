use mock_macro::{end_mock_setup, mock_fn, start_mock_setup};
use std::sync::Mutex;
#[test]
fn macro_test_2() {
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
                Once,
                with_return({
                    println!("set return value false for first expect");
                    false
                }),
            );
            expect(*prob < 0.9, TimeModifier::AtMostOnce);
            expect(*prob > 20.0, TimeModifier::Once);
        }
    );
    end_mock_setup!();
    let fst = rand::random_bool(0.7);
    println!("fst return val {fst}");
    if !fst {
        let snd = rand::random_bool(0.74);
        let third = rand::random_bool(0.74);
        let frth = rand::random_bool(21.0);
        //println!("second return val {snd}");
    }
}
