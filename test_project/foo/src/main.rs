use mock_macro::{end_mock_setup, mock_fn, start_mock_setup};
use std::sync::Mutex;

fn main() {
    start_mock_setup!();
    mock_fn!(
        rand,
        fn random_bool(prob: f64) -> bool {
            default_return({
                println!("greetings from context: {}", context::CONTEXT_CONST);
                context::context();
                true
            });
            expect(
                *prob < 0.5,
                once(),
                with_return({
                    println!("set return value false for first expect");
                    false
                }),
            );
            expect(*prob > 0.9, once());
        }
    );
    end_mock_setup!();
    let fst = rand::random_bool(0.0);
    println!("fst return val {fst}");
    if !fst {
        let snd = rand::random_bool(1.0);
        println!("second return val {snd}");
    }
    let test = std::fs::read_to_string("test.txt");
    if let Ok(text) = test {
        println!("Test file inlcuded string {}", text);
    } else {
        println!("Failed to open file");
    }
}

#[test]
fn macro_test() {
    start_mock_setup!();
    mock_fn!(
        rand,
        fn random_bool(prob: f64) -> bool {
            default_return({
                println!("greetings from context: {}", context::CONTEXT_CONST);
                context::context();
                true
            });
            expect(
                *prob < 0.5,
                once(),
                with_return({
                    println!("set return value false for first expect");
                    false
                }),
            );
            expect(*prob > 0.9, once());
        }
    );
    end_mock_setup!();
    let ctx = context::GLOBAL_CONTEXT
        .get()
        .expect(" couldn't fetch context");
    let mut guard = ctx.lock().expect("failed to fetch guard");
    assert!(
        guard
            .run_mock::<f64, bool>(rand_random_bool_mock_id.clone(), &0.3)
            .is_ok()
    );
    assert!(
        guard
            .run_mock::<f64, bool>(rand_random_bool_mock_id, &0.8)
            .is_err()
    );
}
