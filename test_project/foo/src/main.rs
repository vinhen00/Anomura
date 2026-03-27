use std::sync::Mutex;

use mock_macro::{end_mock_setup, mock_fn, start_mock_setup};

/*mock_fn!(
    name: random_bool,
    path: rand,
    input_types: [f64],
    input_ident: [prob],
    ret_type: bool,
    ret_val: {
        println!("greetings from context: {}",context::CONTEXT_CONST);
        context::context();
        true
    }
);*/

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
            expect(|prob| prob < 0.5).once().with_return({
                println!("set return value false for first expect");
                false
            });
            expect(|prob| prob > 0.9).once();
        }
    );
    end_mock_setup!();

    let fst = rand::random_bool(0.0);
    println!("fst return val {fst}");
    if !fst {
        let snd = rand::random_bool(4.0);
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
fn macro_test() {}
