use std::sync::Arc;
use std::sync::Mutex;
use mock_macro::{end_mock_setup, mock_fn, start_mock_setup, mock_method, mock_struct};

fn main() {
    // start_mock_setup!();

    // mock_struct!(
    //     fns,
    //     struct Food {
    //         outer: String,
    //     }
    //     fn new(n: String) -> Food {
    //         default_return( { 
    //             Food{ 
    //                 inner: n, 
    //                 outer: "YOOOOOOO".into() } 
    //         } )
    //     }
    //     [
    //         fn food_fun (&mut self, n: String) {
    //             default_return({
    //                 println!("Changing inner from {} to {}", self.inner, n);
    //                 self.drink(5);
    //                 self.inner = n;

    //             })
    //         },
    //         fn drink (&self, i: i32) -> i32 {
    //             default_return(self.drink_original(i));
    //         }
    //     ]
    // );


    // end_mock_setup!();

    // let mut food = fns::Food::new("hello".into());
    // let mut mood = fns::Food::new("mellow".into());
    // food.food_fun("rom".into());
    // mood.food_fun("YAAAOOOIIII!!!".into());

}


#[test]
fn reference_parameters() {
    start_mock_setup!();
    mock_fn!(
        fns,
        fn ref_param(x: &u32) {
            default_return(());
        }
    );
    end_mock_setup!();
    fns::ref_param(&1);
}

#[test]
fn consume_parameters(){
    start_mock_setup!();
    mock_fn!(
        fns,
        fn cons_param(x: Box<u32>) {
            default_return({
                let dest: Arc<Mutex<Option<Box<u32>>>> = Arc::new(Mutex::new(None));
                let dest2 = dest.clone();
                *dest2.lock().unwrap() = Some(x);
                assert!(dest.lock().unwrap().is_some());
            })
        }
    );
    end_mock_setup!();
    fns::cons_param(Box::new(42));
}

#[test]
fn consume_self(){
    start_mock_setup!();
    mock_method!(
        fns,
        ConsSelfStruct,
        fn consume_self(self) {
            default_return(());
        }
    );
    end_mock_setup!();
    let a = fns::ConsSelfStruct;
    a.consume_self();
}

#[test]
fn foreign(){
    start_mock_setup!();
    mock_fn!(
        fns,
        fn foreign(){
            default_return(());
        }
    );
    end_mock_setup!();
    unsafe { fns::foreign(); }

}


#[test]
fn mock_struct(){
    start_mock_setup!();
    mock_struct!(
        fns,
        struct MockStruct{
            newfield: u32,
        }
        fn new() -> MockStruct {
            default_return(
                MockStruct {
                    pubfield: 10,
                    privfield: 11,
                    newfield: 12,
                }
            );
        }
        []
    );
    end_mock_setup!();
    fns::MockStruct::new();
}


#[test]
fn return_call_with_args(){
    start_mock_setup!();
    mock_fn!(
        fns,
        fn ret_call_w_args (x: i16) -> i16{
            default_return(x*2);
        }
    );
    end_mock_setup!();
    assert_eq!(fns::ret_call_w_args(2), 4);
}


#[test]
fn return_reference(){
    start_mock_setup!();
    mock_fn!(
        fns,
        fn ret_ref() -> &'static u32 {
            default_return(&5u32);
        }
    );
    end_mock_setup!();
    assert_eq!(5, *fns::ret_ref());
}