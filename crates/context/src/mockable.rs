use std::collections::HashMap;

///trait that should be implemented by all mocked ADTs, and derived by all mocked traits
/// 
pub trait Mockable  {
    /*
    we need to be able to add expectations in various ways (duh)



     */
    // as proof of concept, a special initializtion method.
}




type Exp<Input, Ret> = fn(Input) -> Ret;


// let our exp pointers implement a special drop that uses 