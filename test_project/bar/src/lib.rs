
#[derive(Debug)]
pub struct Bar {
    pub field1: u32,
    field2: u32
}


impl Bar {
    pub fn new() -> Bar{
        Bar {field1: 0, field2: 1}
    }

    pub fn method1 (&self) {}

    pub fn method2 (&self) {}
}