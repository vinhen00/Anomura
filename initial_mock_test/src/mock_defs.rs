fn main() {
    let x = foo(1);
    println!("Mocked: {}", x);
}

fn foo(x: i32) -> i32 {
    println!("helloworld");
    return 2;
}
