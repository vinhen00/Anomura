

fn main() {
    let xy = foo(1);
    println!("Real: {}", xy);
}

fn foo(x: i32) -> i32 {
    let test = 9;
    return x * 6;
}