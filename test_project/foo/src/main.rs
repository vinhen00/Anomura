use bar;
use mock_macro::{mock_fn, mock_method};

mock_fn! {
    name: foo,
    path: bar,
    input_types: [u32],
    input_ident: [a],
    ret_type: u32,
    ret_val: {
        println!("foo printed a: {}, b: {}", a);
        a*100 }
}

mock_method! {
    struct_name: Food,
    name: bar,
    path: bar,
    input_types: [String],
    input_ident: [n],
    ret_type: (),
    ret_val: {
        println!("Changing name from {} to {}", self.waf, n);
        self.waf = n;
    }
}

fn main() {
    println!("bar");
    bar::foo(67);
}
