
struct Stroo {
    x: i32,
    name: String,
}

impl Stroo {
    fn stroop(&mut self) {
        println!("My name is {} and my x is {}", self.name, self.x)
    }
}


fn main() {
    let mut ss = Stroo { x: 67, name: "Jeffrey".to_string()};
    ss.stroop();
}

