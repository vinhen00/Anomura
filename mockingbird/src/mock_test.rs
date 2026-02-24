
struct Stroo {
    x: i32,
    name: String,
}

impl Stroo {
    fn stroop(&mut self) {
        println!("My name is {}" , self.name)
    }
}


fn main() {
    let mut ss = Stroo { x: 67, name: "Jeffrey".to_string()};
    ss.stroop();
    println!("{}",ss.name);
}
