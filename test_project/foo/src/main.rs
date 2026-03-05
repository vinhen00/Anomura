use mock_macro::{mock_fn, mock_method};

fn main() {
    mock_fn! {
        name: bar::foo,
        input_types: [u32],
        input_ident: [a],
        ret_type: u32,
        ret_val: {
            println!("foo printed a: {}, b: {}", a);
            a*100 }
    }

    mock_method! {
        struct_name: bar::Food,
        name: bar,
        input_types: [String],
        input_ident: [n],
        ret_type: (),
        ret_val: {
            println!("Changing name from {} to {}", self.waf, n);
            self.waf = n;
        }
    }

    let res = bar::foo(2);
    let sum = foobar::add(2, 3);
    println!("Hello, barfoo! {res}");
    // mock!(bar::foo("expr"), bar::Food::food_fun(food));
}
