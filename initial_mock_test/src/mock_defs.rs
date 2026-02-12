

fn main() {
    let x = foo(1);
    println!("Mocked: {}", x);
}

fn foo(x: i32) -> i32 {
    println!("hellowolrd");
    return 2;
}