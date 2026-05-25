pub fn foo(a: u32) -> u32 {
    a * 2
}

pub struct Food {
    pub inner: String,
}
impl Food {
    pub fn new(n: String) -> Food {
        Food { inner: n }
    }
    pub fn food_fun(&mut self, n: String) {
        self.inner = n;
    }

    pub fn drink(&self, i: i32) -> i32 {
        println! {"Drinkning glug glug glug"};
        i * 2
    }
}

pub fn handler1(input: TestStruct1) -> TestStruct2 {
    let n = input.text.len() as u32;
    let m = input.text.as_ptr() as u32;
    TestStruct2 { n, m }
}

pub fn handler2(input: TestStruct2) -> TestStruct1 {
    TestStruct1 {
        text: format!("({}, {})", input.n, input.m),
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
#[derive(Debug, Clone)]
pub struct TestStruct2 {
    pub n: u32,
    pub m: u32,
}
impl std::fmt::Display for TestStruct2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.n, self.m)
    }
}
