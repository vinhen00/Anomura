use std::sync::Arc;
use std::sync::Mutex;


pub fn ref_param(x: &u32){}
pub fn cons_param(x: Box<u32>){}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(PartialEq, Debug)]
pub struct Foo {
    pub x: u32
}

impl Foo {
    pub fn ret_ref(&self) -> &u32 {
        &1u32
    }

    pub fn ret_mut_ref(&mut self) -> &mut u32 {
       &mut self.x
    }

    //Foo doesnt implement clone so its a good use
    pub fn ret_owned() -> Foo{
        Foo { x: 10 }
    }

    pub fn static_method() {}

    pub fn fallback(&self) -> u32 {
        11
    }
}

pub fn ret_param( x: &mut u32 ) {}

pub mod a {
    pub fn modules() -> u32{0}
}

pub fn return_const() -> i16 {
    16i16
}

pub fn return_panic(){}

pub fn foo(a: i8, b: i8, c: i8, d: i8, e: i8, f: i8, g: i8,
    h: i8, i: i8, j: i8, k: i8, l: i8, m: i8, n: i8, o: i8,
    p: i8) {}

pub fn times_once(){}

pub fn times_any(){}

pub fn match_const(key: u32){}

pub fn match_operator(key: u32){}

pub enum Pattern {
    Okay,
    NotOkay,
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Okay => write!(f, "Okay"),
            Pattern::NotOkay => write!(f, "NotOkay"),
        }
    }
}

pub fn match_patter(pattern: Pattern) {}

pub fn match_range(key: u32) {}

pub fn match_wildcard(key: u32){}

pub fn match_function(key: u32){}

// #[derive(Debug)]
// pub struct ClosureWrapper(pub Box<dyn Fn(u32) -> u32>);


// pub fn closure_param(f: ClosureWrapper) -> u32 {
//     (f.0)(0)
// }