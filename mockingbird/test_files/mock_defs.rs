extern crate mock_macro;
use mock_macro::mock_def;
use mock_macro::mocked;

mock_def! {
    name: foo,
    input_types: [i32, i32],
    input_ident: [a, b],
    ret_type: (),
    ret_val: {
        println!("Hello World");
        println!("I'm printing from inside a mocked macro {} {}", a, b);
    }
}


