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
        fn food_fn (&mut self, n: string) {
            default_return({
                //let hash = self.mockhash;
                println!("hello");
            })
        }
    );
    end_mock_setup!();

    let mut food = bar::Food::new("hello".into());


}

#[test]
fn macro_test() {}
