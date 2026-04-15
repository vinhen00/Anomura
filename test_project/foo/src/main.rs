use bar::{TestStruct1, TestStruct2};
use mock_macro::{end_mock_setup, mock_fn, start_mock_setup};
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
        bar,
        Food,
        fn new(n: String) -> Food {
            default_return( { 
                Food{ 
                    inner: n, 
                    outer: "YOOOOOOO".into() } 
            } )
        },
        []
    );

    mock_method!(
        bar,
        Food,
        fn food_fun (&mut self, n: String) {
            default_return({
                let hash = self.mock_hash.clone();
                println!("{}", hash);
            })
        }
    );
    end_mock_setup!();

    let mut food = bar::Food::new("hello".into());
    food.food_fun("rom".into());

}

#[test]
fn macro_test() {}
