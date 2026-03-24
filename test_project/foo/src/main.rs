use mock_macro::{mock_fn, mock_method};
use rand;


mock_method!{
    struct_name: Food,
    name: food_fun,
    path: bar,
    self_receiver: RefMut,
    input_types: [String],
    input_ident: [miew],
    ret_type: (),
    ret_val: {
        println!("Changing name from {} to {}", self.inner, miew);
        self.inner = miew;
    }
}

mock_fn!{
    name: random_bool,
    path: rand,
    input_types: [f64],
    input_ident: [a],
    ret_type: bool,
    ret_val: {
        false
    }
}




fn main() {
    let mut food = bar::Food {
        inner: "Hello world".to_string(),
    };
    food.food_fun("YO".into());
    let test = rand::random_bool(1.0);
}
