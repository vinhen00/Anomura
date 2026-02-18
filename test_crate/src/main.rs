use itertools::Itertools;
use std::assert;
use test_crate::foo;
fn main() {
    let b = foo::bar(42);
    //assert!(b == 42);
    let s = std::env::args().collect_vec();
    println!("Hello, FooBar! {:?} {:?}", b, s);
}
