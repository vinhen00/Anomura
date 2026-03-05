use mock_macro::mock_def;
use mock_macro::mock_def;
use mock_macro::mocked;
mock_fn! {
    name: foo,
    path: bar::foo,
    input_types: [i32, i32],
    input_ident: [a, b],
    ret_type: i32,
    ret_val: {
        println!("foo printed a: {}, b: {}", a, b);
        foo_original(a,b) + a + b}
}

mock_method! {
    struct_name: Stroop,
    name: bar,
    path: kar::Stroop::bar,
    input_types: [String],
    input_ident: [n],
    ret_type: (),
    ret_val: {
        println!("Changing name from {} to {}", self.waf, n);
        self.waf = n;}
}
