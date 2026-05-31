use bar::{TestStruct1, TestStruct2, Bar};
use mock_macro::{mock_fn, mock_struct, mock_method};
use std::sync::Mutex;
fn main() {
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

    context::finish_building_context();


    let mut food = bar::Food::new("hello".into());
    let mut mood = bar::Food::new("mellow".into());
    food.food_fun("rom".into());
    mood.food_fun("YAAAOOOIIII!!!".into());

}
#[test]
fn macro_test() {
    mock_fn!(
        rand,
        fn random_bool(prob: f64) -> bool {
            default_return(true);
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
    context::finish_building_context();
    let fst = rand::random_bool(0.7);
    println!("fst return val {fst}");
    if !fst {
        let snd = rand::random_bool(0.89);
        println!("second return val {snd}");
    }
}




#[test]
fn struct_example() {
    mock_struct!(
        bar,
        struct Bar{
            pub new_field: u32
        }
        fn new() -> Bar {
            default_return(
                Bar{
                field1: 1,
                field2: 3,
                new_field: 4
            });
        }
        []
    );

    mock_method!(
        bar,
        Bar,
        fn method1(&self) {
            default_return(());
            expect(self.new_field == 4, once());
        }
    );

    mock_method!(
        bar,
        Bar,
        fn method2(&self) {
            default_return(());
            expect(self.mock_hash == 1.to_string(), once());
        }
    );
    context::finish_building_context();

    let obj1 = bar::Bar::new();
    let obj2 = bar::Bar::new();

    obj1.method1();
    obj2.method2();


}
