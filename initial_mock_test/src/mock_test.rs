

fn main() {
    let foo = foo(4);
    let bar = bar(99);
    println!("Foo returned {}, Bar returned {}", foo, bar);
}

fn foo(xy: i32) -> i32 {
    println!("Original foo printed this");
    return bar(xy);
}

fn bar(x: i32) -> i32 {
    return x;
}

fn test() {
    println!("This is printing in test")
}