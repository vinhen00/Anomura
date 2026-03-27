pub fn foo(a: u32) -> u32 {
    a * 2
}

pub struct Food {
    pub inner: String,
}
impl Food {
    pub fn food_fun(&mut self, n: String) {
        self.inner = n;
    }
}


pub fn mut_string(str: &mut String) {

}