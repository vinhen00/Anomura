use api_macro::mock;

fn main() {
    let res = bar::foo(2);

    println!("Hello, barfoo! {res}");
    mock!(bar::foo("expr"), bar::Food::food_fun(food));
}
