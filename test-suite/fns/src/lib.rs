pub fn ref_param(x: &u32) {}
pub fn cons_param(x: Box<u32>) {}

#[derive(Debug)]
pub struct ConsSelfStruct;

impl ConsSelfStruct {
    pub fn consume_self(self) {}
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
pub struct MockStruct {
    pub pubfield: u32,
    privfield: u32,
}

impl MockStruct {
    pub fn new() -> Self {
        MockStruct {
            pubfield: 2,
            privfield: 1,
        }
    }
    pub fn foo() {
        ()
    }
    pub fn get_value(&self) -> u32 {
        self.pubfield
    }
    fn private_helper(&self) -> u32 {
        self.privfield
    }
}

pub fn ret_call_w_args(x: i16) -> i16 {
    x
}

#[derive(PartialEq, Debug)]
pub struct Foo {
    pub x: u32,
}

impl Foo {
    pub fn ret_ref(&self) -> &u32 {
        &1u32
    }

    pub fn ret_mut_ref(&mut self) -> &mut u32 {
        &mut self.x
    }

    //Foo doesnt implement clone so its a good use
    pub fn ret_owned() -> Foo {
        Foo { x: 10 }
    }

    pub fn static_method() {}

    pub fn fallback(&self) -> u32 {
        11
    }
}

pub fn ret_param(x: &mut u32) {}

pub mod a {
    pub fn modules() -> u32 {
        0
    }

    pub mod nested {
        pub struct Inner {
            pub value: i32,
        }

        impl Inner {
            pub fn new(value: i32) -> Self {
                Inner { value }
            }

            pub fn double(&self) -> i32 {
                self.value * 2
            }
            pub fn tripple(&self) -> i32 {
                self.value * 3
            }
        }

        pub fn deep_fn() -> &'static str {
            "deep"
        }
    }
}


/// by default i don't panic. i don't do anything c:
pub fn return_panic() {}

#[derive(Debug)]
pub enum Pattern {
    Okay,
    NotOkay,
}

pub struct ClosureWrapper(pub Box<dyn Fn(u32) -> u32>);

impl std::fmt::Debug for ClosureWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClosureWrapper(<fn>)")
    }
}

pub fn closure_param(f: ClosureWrapper) -> u32 {
    (f.0)(0)
}


fn private_top_level_fn() -> u32 {
    42
}

struct PrivateStruct {
    data: u32,
}

trait InternalBehavior {
    fn secret(&self) -> u32;
}

impl InternalBehavior for Foo {
    fn secret(&self) -> u32 {
        self.x + 100
    }
}

pub trait Computable {
    fn compute(&self) -> u32;
}

impl Computable for Foo {
    fn compute(&self) -> u32 {
        self.x * 2
    }
}
