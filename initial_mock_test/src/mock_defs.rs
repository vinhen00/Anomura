
struct Stroo {
    x: i32,
    name: String,
}

impl Stroo {
    fn stroop(&mut self) {
        println!("My name is {} and my x is {} and Im mocked", self.name, self.x)
    }
}

fn main() {
    //let x = foo(1);
    let y = 88;
    println!("Mocked: {}", y);
}
