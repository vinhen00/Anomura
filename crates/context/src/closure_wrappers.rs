use crate::errors::PredicateResult;

#[derive(Debug, Clone)]
pub struct ReturnValDoublePointer {
    thin_ptr: *const (),
}
impl ReturnValDoublePointer {
    pub fn from_fn<Input, ReturnVal>(closure: Box<dyn Fn(Input) -> ReturnVal>) -> Self {
        let cloref = Box::leak(closure);
        let wrapped = Box::new(cloref);
        let wraref = Box::into_raw(wrapped);
        let thin_ptr = wraref as *const _;
        Self { thin_ptr }
    }
    /// Casts the a raw double pointer created with from_fn into a closure ` Result<&dyn Fn() -> ReturnVal>`
    /// # Safety
    /// You must guarantee that `ReturnVal` is the exact same Type as was used when you used `from_fn` to create the value.
    pub unsafe fn into_fn<Input, ReturnVal>(&self) -> &dyn Fn(Input) -> ReturnVal {
        let wraref: *mut &mut dyn Fn(Input) -> ReturnVal = self.thin_ptr as _;
        let cloref: &mut dyn Fn(Input) -> ReturnVal = unsafe { *wraref };
        cloref
    }
}

#[derive(Debug, Clone)]
pub struct ConditionDoublePointer {
    thin_ptr: *const (),
}
impl ConditionDoublePointer {
    pub fn from_fn<Input>(closure: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>) -> Self {
        let cloref = Box::leak(closure);
        let wrapped = Box::new(cloref);
        let wraref = Box::into_raw(wrapped);
        let thin_ptr = wraref as *const _;
        Self { thin_ptr }
    }
    /// Casts the a raw double pointer created with from_fn into a closure with type `Fn(&Input) -> Result<()>`
    /// # Safety
    /// You must guarantee that `Input` is the exact same Type as was used when you used `from_fn` to create the value.
    pub unsafe fn into_fn<Input>(&self) -> &dyn Fn(&Input) -> PredicateResult<()> {
        let wraref: *mut &mut dyn Fn(&Input) -> PredicateResult<()> = self.thin_ptr as _;
        let cloref: &mut dyn Fn(&Input) -> PredicateResult<()> = unsafe { *wraref };
        cloref
    }
}
//Our DoublePointers will live through the remainder of the program, and they will not be modified in any way.
unsafe impl Send for ConditionDoublePointer {}
unsafe impl Sync for ConditionDoublePointer {}
unsafe impl Send for ReturnValDoublePointer {}
unsafe impl Sync for ReturnValDoublePointer {}
