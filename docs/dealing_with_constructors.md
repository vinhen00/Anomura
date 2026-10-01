

for a struct A we create a struct PublicA 
pub struct A {
    pub a : u32,
    b : u32
}

impl A {
    new(...) -> A 
}


a constructor for a public struct has an on_call helper method where the user can define a closure that returns PublicA 

pub struct PublicA {
    pub a : u32,
} 

PublicA should implement Into<A>


