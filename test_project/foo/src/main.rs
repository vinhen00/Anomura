use mock_macro::mock_fn;
use rand;

mock_fn!(
    name: random_bool,
    path: rand,
    input_types: [f64],
    input_ident: [prob],
    ret_type: bool,
    ret_val: {
        println!("greetings from context: {}",context::CONTEXT_CONST);
        context::context();
        true
    }
);

fn main() {
    let random = rand::random_bool(0.0);
    println!("random returned {random}");
    let test = std::fs::read_to_string("test.txt");
    if let Ok(text) = test {
        println!("Test file inlcuded string {}", text);
    } else {
        println!("Failed to open file");
    }
}
