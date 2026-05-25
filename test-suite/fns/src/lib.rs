use std::sync::Arc;
use std::sync::Mutex;


pub fn ref_param(x: &u32){}
pub fn cons_param(x: Box<u32>){}

pub struct ConsSelfStruct;

impl ConsSelfStruct {
    pub fn consume_self(self){}
}


mod ffi {
    unsafe extern "Rust" {
        pub fn foreign();
    }
}

pub fn foreign() {
    // implementation
}

pub struct MockStruct{
    pub pubfield: u32,
    privfield: u32,
}

impl MockStruct {
    pub fn new() -> Self {
        MockStruct{pubfield: 2, privfield: 1}
    }
}

pub fn ret_call_w_args(x: i16) -> i16 {
    x
}

struct Foo;

impl Foo {
    pub fn ret_ref(&self) -> &u32 {
    &1u32
    }

    pub fn ret_mut_ref(&self) -> &mut u32 {
        let mut x  = 1u32;
        x
    }
}
