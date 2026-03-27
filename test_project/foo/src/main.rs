use std::sync::Mutex;

use mock_macro::{end_mock_setup, mock_fn, start_mock_setup, mock_method};

/*mock_fn!(
    name: random_bool,
    path: rand,
    input_types: [f64],
    input_ident: [a],
    ret_type: bool,
    ret_val: {
        false
    }
);*/

fn main() {
    start_mock_setup!();
    mock_fn!(
        rand,
        fn random_bool(prob: f64) -> bool {
            default_return({
                std::println!("greetings from context: {}", context::CONTEXT_CONST);
                context::context();
                true
            });
        }
    );

    mock_method!(
        bar,
        Food,
        fn food_fun(&mut self, n: String) {
            default_return({
                println!("Works");
            })
        }
    );

    
    end_mock_setup!();

    let fst: bool = rand::random_bool(0.0);
    println!("fst return val {fst}");
    if !fst {
        let snd = rand::random_bool(4.0);
        println!("second return val {snd}");
    }

    let mut food = bar::Food{ inner: "TRUUUUUUP".into()};
    food.food_fun("Test".into());

}

#[test]
fn macro_test() {}
