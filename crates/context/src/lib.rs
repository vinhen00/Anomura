use std::{any::Any, collections::HashMap};

use petgraph::{csr::DefaultIx, graph::DiGraph};
use derive_more::{FromStr,AsRef,AsMut};
pub fn context() {
    println!("context says hello");
}

pub const CONTEXT_CONST: &str = "CONSTANT FROM CONTEXT";

#[derive(Debug, Clone, FromStr, AsRef, AsMut)]
pub struct MockId(String);
pub struct Context {
    mock_context: HashMap<MockId, dyn Any>,
}

pub struct MockContext<Input,ReturnValue> {
    //our current position in the graph
    index : DefaultIx, 
    graph: DiGraph< MockNode,MockEdge<Input,ReturnValue>>
}


impl <Input,ReturnValue> MockContext<Input,ReturnValue> {
pub fn check_and_traverse_one_step(&self, input : Input)  {
        let mut valid_neighbors :Vec<_> = self.graph.edges(self.index).filter(|e| e.weight().check(input)).collect();
        valid_neighbors.sort_by(|e , f| e.weight().priority > f.weight().priority);
    }
}
pub struct MockContextBuilder<Input,ReturnValue> {
    default_return : Option<ReturnValue>,
    graph : DiGraph<Condition<Input>,MockContext<Input,ReturnValue>>
}






impl <Input,ReturnValue> MockContextBuilder<Input,ReturnValue> {
    pub fn add_expectation(&self, condition: Condition<Input>, modifier : Modifier< ) {

    }
}

pub struct MockEdge<Input, ReturnVal> where ReturnVal : Clone {
    condition : Condition,
    Priority : u8,
    return_value : ReturnVal
}


pub struct MockNode;
struct Condition<I>  {
    closure : Fn(&I) -> bool
}

impl MockEdge<I,ReturnVal> {

    fn check(&self, input : &I) -> bool {
        self.condition.closure(input)
    }
    fn execute(&self, input : &I) -> Result<ReturnVal> {
        if self.condition.close(input) { Ok(self.return_value.clone()) }
        else { Err(format!(""))}
    }

}




#[cfg(test)]
mod tests {
    use super::*;
}

pub type Result<T> = result::Result<T,MockError>;


#[derive(Debug,Clone,Error,FromStr)]
pub struct MockError(String);