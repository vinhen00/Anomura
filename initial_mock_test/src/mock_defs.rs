

fn main() {
    //let x = foo(1);
    let y = 88;
    println!("Mocked: {}", y);
}

fn foo(xy: i32) -> i32 {
    let y = 66;
    println!("Im pritning from the mock");
    return xy * 5;
}
