use mock_macro::mock_def;
use mock_macro::mocked;

mock_def! {
    name: foo,
    input_types: [i32, i32],
    input_ident: [a, b],
    ret_type: i32,
    ret_val: a * b + 10
}


// fn full() {
//     println!("hello");
// }