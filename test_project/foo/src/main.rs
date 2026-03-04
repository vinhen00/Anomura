use api_macro::mock;

fn main() {
    let res = bar::foo(2);
    let sum = foobar::add(2, 3);
    println!("Hello, barfoo! {res}");
    mock!(bar::foo("expr"), bar::Food::food_fun(food));
}
