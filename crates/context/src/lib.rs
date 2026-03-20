use std::{any::Any, collections::HashMap, fmt::Display, hash::Hash, result};

use derive_more::{AsMut, AsRef, Display, FromStr};
use petgraph::{
    Graph,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
pub fn context() {
    println!("context says hello");
}

pub const CONTEXT_CONST: &str = "CONSTANT FROM CONTEXT";
#[derive(Debug, Clone, Hash, FromStr, AsRef, AsMut)]
pub struct MockId(String);
pub struct GlobalContext {
    mock_context: HashMap<MockId, Box<dyn Any>>,
}



pub struct MockContext<Input, ReturnValue: Clone> {
    //our current position in the graph
    index: u32,
    graph: DiGraph<MockNode, MockEdge<Input, ReturnValue>>,
}

impl<Input: Display, ReturnValue: Clone> MockContext<Input, ReturnValue> {
    pub fn check_and_traverse_one_step(&self, input: Input) -> Result<u32> {
        let mut errors = vec![];
        let mut valid_neighbors = vec![];

        self.graph
            .edges(self.index.into())
            .for_each(|e| match e.weight().check(&input) {
                Ok(_) => valid_neighbors.push(e),
                Err(e) => errors.push(e.0),
            });
        if valid_neighbors.is_empty() {
            return Err(format!(
                "input {} didn't match any of the following cases: {:?}",
                input, errors
            )
            .into());
        }
        valid_neighbors.sort_by(|e, f| e.weight().priority.cmp(&f.weight().priority));
        let edge_index = valid_neighbors[0].id();
        let edge_endpoints = self
            .graph
            .edge_endpoints(edge_index)
            .expect("nonexistent edge");
        Ok(edge_endpoints.1.index() as u32)
    }
}
/*
an expectation can be a series of expectations,
There is a pool of mocks moving through the graph independently at the start. 
If mock A enters a sequence where B is also dependent, A can take temporary ownership of B, assuming B is in a state where this is acceptable.
//if B is not in such a state, for example when moving through another sequence 
*/




pub struct ExpectationRef(u32);

#[derive(Debug, Clone, Display, FromStr)]
pub struct EnvId(String);
pub enum EnvVal {
    ExpectationRef(ExpectationRef),
    Int(u32),
}
pub type Environment = HashMap<EnvId, EnvVal>;
pub struct MockContextBuilder<Input, ReturnValue: Clone> {
    start_index: NodeIndex,
    head: NodeIndex,
    environment: Environment,
    default_return: Option<ReturnValue>,
    graph: DiGraph<MockNode, MockEdge<Input, ReturnValue>>,
}

impl<Input, ReturnValue: Clone> MockContextBuilder<Input, ReturnValue> {
    pub fn new() -> Self {
        let mut graph = Graph::new();
        let start_index = graph.add_node(MockNode {
            entry: true,
            exit: true,
        });
        Self {
            start_index,
            head: start_index,
            environment: HashMap::new(),
            default_return: None,
            graph,
        }
    }

    pub fn default_return_value(&mut self, return_value: ReturnValue) -> Result<()> {
        if self.default_return.is_some() {
            return Err("default return value set twice".into());
        }
        self.default_return = Some(return_value);
        Ok(())
    }

    pub fn add_expectation<F: Fn(&Input) -> Result<()> + 'static>(
        &mut self,
        condition: Condition<Input>,
        modifier: TimeModifier,
        return_value: Option<ReturnValue>,
        exit: bool,
    ) -> Result<()> { 
        let new_node_index = self.graph.add_node(MockNode { entry: false, exit });
        let transition_cost = || {
            let Some(ret_resolved) = return_value.or(self.default_return.clone()) else {
                return Err(MockError(
                    "no return value or default return value found".into(),
                ));
            };
            Ok(TransitionCost::ConsumeInput {
                condition,
                return_value: ret_resolved,
            })
        };

        match modifier {
            TimeModifier::Once => {
                // add a single edge between nodes
                //       condition
                // (1) ---------------> (2)
                let main_weight = MockEdge {
                    priority: 0,
                    transition_cost: transition_cost()?,
                };
                //consume input edge
                self.graph.add_edge(self.head, new_node_index, main_weight);
                self.head = new_node_index;
                Ok(())
            }
            TimeModifier::AtMostOnce => {
                //add two edges to new node, one with always ( lowest priority, one with condition)
                //      epsilon || condition
                // (1) ----------------------> (2)
                let main_weight = MockEdge {
                    priority: 1,
                    transition_cost: transition_cost()?,
                };
                let instant_weight = MockEdge {
                    priority: 0,
                    transition_cost: TransitionCost::<Input, ReturnValue>::Instant,
                };
                //consume input edge
                self.graph.add_edge(self.head, new_node_index, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(self.head, new_node_index, instant_weight);
                self.head = new_node_index;
                Ok(())
            }

            TimeModifier::Any => {
                //add two edges, one from Node n to n and one instant edge to edge n+1
                //
                //    condition
                //       /  \
                //      |    |
                //       \  /         epsilon
                //        (1) ------------------> (2)
                let main_weight = MockEdge {
                    priority: 1,
                    transition_cost: transition_cost()?,
                };
                let instant_weight = MockEdge {
                    priority: 0,
                    transition_cost: TransitionCost::<Input, ReturnValue>::Instant,
                };
                //consume input edge
                self.graph.add_edge(self.head, self.head, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(self.head, new_node_index, instant_weight);
                self.head = new_node_index;
                Ok(())
            }
            TimeModifier::AtLeastOnce => {
                //add two edges, one from Node n to n and one instant edge to edge n+1
                //
                //                 condition
                //                   /  \
                //                  |    |
                //     condition     \  /         epsilon
                //  (n) ----------> (n+1) ------------------> (n+2)

                let n_plus_one = self.graph.add_node(MockNode { entry: false, exit });
                let main_weight = MockEdge {
                    priority: 1,
                    transition_cost: transition_cost()?,
                };

                let instant_weight = MockEdge {
                    priority: 0,
                    transition_cost: TransitionCost::<Input, ReturnValue>::Instant,
                };
                //once
                self.graph
                    .add_edge(self.head, n_plus_one, main_weight.clone());

                //consume input edge
                self.graph.add_edge(n_plus_one, n_plus_one, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(n_plus_one, new_node_index, instant_weight);
                Ok(())
            }
            TimeModifier::Until(env_id) => todo!(),
            TimeModifier::Times(times_value) => todo!(),
            TimeModifier::After(env_id) => todo!(),
        }
    }
}

impl<Input, ReturnValue: Clone> Default for MockContextBuilder<Input, ReturnValue> {
    fn default() -> Self {
        Self::new()
    }
}

pub enum TimesValue {
    Explicit(u32),
    Implicit(EnvId),
}
pub enum TimeModifier {
    Once,
    AtMostOnce,
    Any,
    AtLeastOnce,
    Until(EnvId),
    Times(TimesValue),
    After(EnvId),
}
#[derive(Debug)]
pub struct MockEdge<Input, ReturnVal: Clone> {
    priority: u8,
    transition_cost: TransitionCost<Input, ReturnVal>,
}
impl<Input, ReturnVal: Clone> Clone for MockEdge<Input, ReturnVal> {
    fn clone(&self) -> Self {
        Self {
            priority: self.priority,
            transition_cost: self.transition_cost.clone(),
        }
    }
}
#[derive(Debug)]
pub enum TransitionCost<Input, ReturnVal: Clone> {
    ConsumeInput {
        condition: Condition<Input>,
        return_value: ReturnVal,
    },
    Instant,
}

impl<Input, ReturnVal: Clone> Clone for TransitionCost<Input, ReturnVal> {
    fn clone(&self) -> Self {
        match self {
            Self::ConsumeInput {
                condition,
                return_value,
            } => Self::ConsumeInput {
                condition: match condition {
                    Condition::Closure(c) => Condition::Closure(c.clone()),
                },
                return_value: return_value.clone(),
            },
            Self::Instant => Self::Instant,
        }
    }
}

pub struct MockNode {
    entry: bool,
    exit: bool,
}
#[derive(Debug)]
pub enum Condition<I> {
    Closure(fn(&I) -> Result<()>),
}

impl<Input, ReturnVal: Clone> MockEdge<Input, ReturnVal> {
    fn check(&self, input: &Input) -> Result<()> {
        match &self.transition_cost {
            TransitionCost::Instant => Ok(()),
            TransitionCost::ConsumeInput { condition, .. } => match &condition {
                Condition::Closure(closure) => (*closure)(input),
            },
        }
    }
    fn execute(
        closure: &dyn Fn(&Input) -> Result<()>,
        input: &Input,
        return_value: ReturnVal,
    ) -> Result<ReturnVal> {
        match closure(input) {
            Ok(_) => Ok(return_value.clone()),
            Err(e) => Err(e),
        }
    }
}

pub type Result<T> = result::Result<T, MockError>;

#[derive(Debug, Clone, Display, FromStr)]
pub struct MockError(String);

impl From<String> for MockError {
    fn from(value: String) -> Self {
        MockError(value)
    }
}
impl From<&str> for MockError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
