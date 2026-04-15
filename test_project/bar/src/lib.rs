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
}

#[derive(Debug, Clone)]
pub struct TestStruct1 {
    pub text: String,
}
impl std::fmt::Display for TestStruct1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
pub struct TestStruct2 {
    pub n: u32,
    pub m: u32,
}
impl std::fmt::Display for TestStruct2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.n, self.m)
    }
}

// pub fn mut_string(str: &mut String) {

// }
