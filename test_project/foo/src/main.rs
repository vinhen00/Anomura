use bar::{TestStruct1, TestStruct2};
use mock_macro::mock_fn;
use std::sync::Mutex;
fn main() {
<<<<<<< HEAD
    start_mock_setup!();

    mock_struct!(
        bar,
        struct Food {
            outer: String,
        }
        fn new(n: String) -> Food {
            default_return( { 
                Food{ 
                    inner: n, 
                    outer: "YOOOOOOO".into() } 
            } )
        }
        [
            fn food_fun (&mut self, n: String) {
                default_return({
                    println!("Changing inner from {} to {}", self.inner, n);
                    self.drink(5);
                    self.inner = n;

                })
            },
            fn drink (&self, i: i32) -> i32 {
                default_return(self.drink_original(i));
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
=======
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
            default_return({ true });
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
    context::finish_building_context();
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
>>>>>>> context2
