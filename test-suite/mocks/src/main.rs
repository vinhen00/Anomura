use std::sync::Arc;
use std::sync::Mutex;
use mock_macro::{mock_fn, mock_method, mock_struct};

fn main() {
    // 

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


    // context::finish_building_context();


    // let mut food = fns::Food::new("hello".into());
    // let mut mood = fns::Food::new("mellow".into());
    // food.food_fun("rom".into());
    // mood.food_fun("YAAAOOOIIII!!!".into());

}


#[test]
fn reference_parameters() {
    mock_fn!(
        fns,
        fn ref_param(x: &u32) {
            default_return(());
            expect(true, once());
        }
    );
    context::finish_building_context();

    fns::ref_param(&1);
}

#[test]
fn consume_parameters(){
    
    mock_fn!(
        fns,
        fn cons_param(x: Box<u32>) {
            default_return({
                let dest: Arc<Mutex<Option<Box<u32>>>> = Arc::new(Mutex::new(None));
                let dest2 = dest.clone();
                *dest2.lock().unwrap() = Some(x);
                assert!(dest.lock().unwrap().is_some());
            });
            expect(true, once());
        }
    );
    context::finish_building_context();

    fns::cons_param(Box::new(42));
}

#[test]
fn consume_self(){
    
    mock_method!(
        fns,
        ConsSelfStruct,
        fn consume_self(self) {
            default_return(());
        }
    );
    context::finish_building_context();

    let a = fns::ConsSelfStruct;
    a.consume_self();
}

#[test]
fn foreign(){
    
    mock_fn!(
        fns,
        fn foreign(){
            default_return(());
            expect(true, once());
        }
    );
    context::finish_building_context();

    unsafe { fns::foreign(); }

}


#[test]
fn mock_struct(){
    
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
    context::finish_building_context();

    fns::MockStruct::new();
}


#[test]
fn return_call_with_args(){
    
    mock_fn!(
        fns,
        fn ret_call_w_args (x: i16) -> i16{
            default_return(x*2);
            expect(true, once());
        }
    );
    context::finish_building_context();

    assert_eq!(fns::ret_call_w_args(2), 4);
}


#[test]
fn return_reference(){
    
    mock_method!(
        fns,
        Foo,
        fn ret_ref(&self) -> &'static u32 {
            default_return(&5u32);
        }
    );
    context::finish_building_context();

    let mut foo = fns::Foo { x: 5};
    assert_eq!(5, *foo.ret_ref());
}

#[test]
fn return_mutable_reference(){
    
    mock_method!(
        fns,
        Foo,
        fn ret_mut_ref(&mut self) -> &mut u32 {
            default_return({
                &mut self.x
            })
        }
    );
    context::finish_building_context();


    let mut foo = fns::Foo{ x: 5 };
    {
        let x = foo.ret_mut_ref();
        assert_eq!(5, *x);
        *x = 6;
    }
    {
        let y = foo.ret_mut_ref();
        assert_eq!(6, *y);
    }
}

#[test]
fn return_owned(){
    
    mock_method!(
        fns,
        Foo,
        fn ret_owned() -> Foo {
            default_return(Foo { x: 20 });
        }
    );
    context::finish_building_context();

    assert_eq!(fns::Foo::ret_owned(), fns::Foo{ x: 20 });
}

#[test]
fn return_parameters(){
    
    mock_fn!(
        fns,
        fn ret_param(x: &mut u32) {
            default_return({
                *x = 3;
                ()
            });
            expect(true, once());
        }
    );
    context::finish_building_context();

    let mut value: u32 = 5;
    fns::ret_param(&mut value);
    assert_eq!(value, 3);

}

#[test]
fn static_method(){
    
    mock_method!(
        fns,
        Foo,
        fn static_method () {
            default_return(());
        }
    );
    context::finish_building_context();
    
    fns::Foo::static_method();
}

#[test]
fn fallback(){
    
    mock_method!(
        fns,
        Foo,
        fn fallback(&self) -> u32 {
            default_return(self.fallback_original());
        }
    );
    context::finish_building_context();

    let foo = fns::Foo{ x: 9};
    assert_eq!(11, foo.fallback());
}

// #[test]
// fn modules(){
    
//     mock_fn!(
//         fns,
//         fn modules() -> u32 {
//             default_return(21);
//         }
//     );
//     context::finish_building_context();

//     assert_eq!(21, fns::a::modules());
// }

#[test]
fn return_const(){
    
    mock_fn!(
        fns,
        fn return_const() -> i16 {
            default_return(17i16);
            expect(true, once());
        }
    );
    context::finish_building_context();


    assert_eq!(17, fns::return_const());
}

#[test]
#[should_panic]
fn return_panic(){
    
    mock_fn!(
        fns,
        fn return_panic(){
            default_return({panic!();});
        }
    );
    context::finish_building_context();

    fns::return_panic();
}

#[test]
fn many_args(){
    
    mock_fn!(
        fns,
        fn foo(a: i8, b: i8, c: i8, d: i8, e: i8, f: i8, g: i8,
            h: i8, i: i8, j: i8, k: i8, l: i8, m: i8, n: i8, o: i8,
            p: i8) {
            default_return(());
            expect(true, once());
            }
    );
    context::finish_building_context();


    fns::foo(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

}

#[test]
fn times_once(){
    mock_fn!(
        fns,
        fn times_once(){
            default_return(());
            expect(true, once());
        }
    );
    context::finish_building_context();

    fns::times_once();
}

#[test]
fn times_any(){
    mock_fn!(
        fns,
        fn times_any(){
            default_return(());
            expect(true, any());
        }
    );
    context::finish_building_context();

    fns::times_any();
    fns::times_any();
}



#[test]
fn match_constant(){
    mock_fn!(
        fns,
        fn match_const(key: u32){
            default_return(());
            expect(*key == 5, once());
        }
    );
    context::finish_building_context();

    fns::match_const(5);

}

#[test]
fn match_operator(){
    mock_fn!(
        fns,
        fn match_operator(key: u32){
            default_return(());
            expect(*key == 5, once());  //eq
            expect(*key >= 10, once());  //ge
            expect(*key > 10, once());  //gt
            expect(*key <= 4, once());  //le
            expect(*key < 4, once());  //lt
            expect(*key != 5, once());  //ne
        }
    );
    context::finish_building_context();

    fns::match_operator(5);
    fns::match_operator(10);
    fns::match_operator(11);
    fns::match_operator(4);
    fns::match_operator(3);
    fns::match_operator(4);

}


// #[test]
// fn match_pattern(){
//     mock_fn!(
//         fns,
//         fn match_pattern(pattern: Pattern) {
//             default_return(());
//             expect(
//                 match pattern {
//                     Okay => true,
//                     NotOkay => false,
//                 }, 
//                 once()
//             );
//         }
//     );
// }