

fn main() {
    let trips = trip(5);
    let message = dub(8);
    println!("{trips}");
}

fn trip(x: i32) -> i32{
    let y = dub(2);
    return y*3;
}

fn dub(x: i32) -> i32{
    return x*2;
}

fn mocked_dub(x: i32) -> i8 {
    return 2;
}