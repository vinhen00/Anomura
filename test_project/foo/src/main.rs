use mock_macro::{mock_fn, mock_method};

mock_fn! {
    name: foo,
    path: bar,
    input_types: [u32],
    input_ident: [test],
    ret_type: u32,
    ret_val: {
        println!("greetings from context: {}",context::CONTEXT_CONST);
        context::context();
        println!("foo printed a: {}", test);
        //println!("Foo original printed {}", foo_original(10));
        1
    }
}

mock_method! {
   struct_name: Food,
   name: food_fun,
   path: bar,
   input_types: [String],
   input_ident: [n],
   ret_type: (),
   ret_val: {
       println!("Changing name from {} to {}", self.inner, n);
       self.inner = n;
   }
}

fn main() {
    //context::context();
    println!("bar");
    bar::foo(67);
    let mut food = bar::Food {
        inner: "Hello world".to_string(),
    };
    food.food_fun("YO".into());
}
