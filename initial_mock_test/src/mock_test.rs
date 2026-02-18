

fn main() {
    let x = 100;
    let xyt = foo(4);
    println!("Main printed: {}", xyt);
}

fn foo(xy: i32) -> i32 {
    let temp = xy * 10;
    let temp0 = xy * 10;
    let temp9 = xy * 10;
    let temp8 = xy * 10;
    let temp7 = xy * 10;
    let temp6 = xy * 10;
    let temp5 = xy * 10;
    let temp4 = xy * 10;
    let temp3 = xy * 10;
    let temp2 = xy * 10;
    let temp1 = xy * 10;
    return bar(temp + temp4);
}

fn bar(x: i32) -> i32 {
    return x;
}