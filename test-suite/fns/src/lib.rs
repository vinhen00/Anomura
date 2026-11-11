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

#[derive(Debug, Default)]
pub struct MockStruct {
    pub pubfield: u32,
    privfield: u32,
}

impl MockStruct {
    pub fn new(name: u32) -> Self {
        MockStruct {
            pubfield: name,
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

#[derive(PartialEq, Debug, Default)]
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

/// A struct that intentionally does NOT derive Default.
/// Used to test that constructors panic when no mock is registered.
#[derive(Debug)]
pub struct NoDefaultStruct {
    pub value: String,
}

impl NoDefaultStruct {
    pub fn new(value: String) -> Self {
        NoDefaultStruct { value }
    }

    pub fn get_value(&self) -> &str {
        &self.value
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

// ─── Macro-generated items (tests the two-pass analysis architecture) ─────────

/// A macro that generates a public function. Before the two-pass architecture,
/// this function would be invisible to the mock system because macro expansion
/// hadn't happened yet at the point of API collection.
macro_rules! define_fn {
    ($name:ident, $ret:ty, $body:expr) => {
        pub fn $name() -> $ret {
            $body
        }
    };
}

define_fn!(macro_generated_add, u32, 42);
define_fn!(macro_generated_greeting, &'static str, "hello from macro");

// ─── Proc-macro-generated items (tests proc macro expansion in two-pass architecture) ───

/// A struct that uses a derive proc macro to generate a `describe()` method.
/// The `Describe` derive macro generates `impl Describable { pub fn describe(&self) -> &'static str }`.
/// This tests that proc-macro-generated impl blocks are visible to the analysis pass.
#[derive(test_macros::Describe)]
pub struct Describable {
    pub label: String,
}

impl Describable {
    pub fn new(label: String) -> Self {
        Describable { label }
    }
}

/// A function decorated with an attribute proc macro that generates a companion function.
/// `#[generate_doubled]` on `magic_number() -> u32` generates `magic_number_doubled() -> u32`.
/// This tests that attribute-macro-generated functions are visible to the analysis pass.
#[test_macros::generate_doubled]
pub fn magic_number() -> u32 {
    21
}
