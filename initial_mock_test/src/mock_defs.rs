
struct Stroo {
    x: i32,
    name: String,
}


impl Stroo {
    fn stroop(&mut self) {
        self.name = "Jeremy".to_string();
        println!(" and my x is {}", self.x);
        self.stroop_original();
    }
}

fn main() {
    //let x = foo(1);
    let y = 88;
    println!("Mocked: {}", y);
}
