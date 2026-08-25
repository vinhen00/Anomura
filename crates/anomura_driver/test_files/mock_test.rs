fn foo(x:i32, y:i32) -> i32 {
    x*y
}

struct Stroop {
    waf: String
}

impl Stroop {
    fn bar(&mut self, new: String) {
        self.waf = new;
    }
}


fn main() {
    foo(4,5);
    let mut waffle = Stroop{waf: "George".to_string()};
    waffle.bar("Liam".to_string());
    println!("{}", waffle.waf);
}
