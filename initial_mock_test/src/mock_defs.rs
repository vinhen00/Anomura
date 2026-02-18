

fn main() {
    //let x = foo(1);
    let y = 88;
    println!("Mocked: {}", y);
}

fn foo(romeo: i32) -> i32 {
    let y = 66;
    println!("Im pritning from the mock");
    foo_original(y);
    return romeo * 5;
}

fn bar(bebe: i32) -> i32 {
    println!("This is printed in bar");
    return bebe+9;
}