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
                    println!("Changing inner from {} to {}", self.inner, n);
                    self.drink();
                    self.inner = n;

                })
            },
            fn drink (&self) {
                default_return(());
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
