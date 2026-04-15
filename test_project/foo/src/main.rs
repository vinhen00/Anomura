use std::sync::Mutex;

use mock_macro::{end_mock_setup, mock_fn, start_mock_setup, mock_method, mock_struct};

fn main() {
    start_mock_setup!();

    mock_struct!(
        bar,
        struct Food {
            inner: String,
            outer: String,
        },
        fn new(n: String) -> Food {
            default_return( { 
                Food{ 
                    inner: n, 
                    outer: "YOOOOOOO".into() } 
            } )
        },
        [
            fn food_fun (&mut self, n: String) {
                default_return({
                    let hash = self.mock_hash.clone();
                    println!("Object with id {} called function food_fun", hash);
                })
            }
        ]
    );
    end_mock_setup!();

    let mut food = bar::Food::new("hello".into());
    let mut mood = bar::Food::new("mellow".into());
    food.food_fun("rom".into());
    mood.food_fun("YAAAOOOIIII!!!".into());

}

#[test]
fn macro_test() {}
