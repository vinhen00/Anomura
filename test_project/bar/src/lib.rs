pub fn foo(a: u32) -> u32 {
    a * 2
}

pub struct Food {
    pub inner: String,
}

impl Food {
    pub fn new(n: String) -> Self {
        Food{ inner: n }
    }
    pub fn food_fun(&mut self, n: String) {
        self.inner = n;
    }

    pub fn drink(&self, i: i32) -> i32 {
        println!{"Drinkning glug glug glug"};
        i * 2
    }
}


// pub fn mut_string(str: &mut String) {

// }