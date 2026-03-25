use std::{
    any::{Any, type_name},
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    marker::PhantomData,
    num::NonZero,
    result,
};

use derive_more::{AsMut, AsRef, Display, FromStr};
use petgraph::{
    Direction::Incoming,
    Graph,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
pub fn context() {
    println!("context says hello");
}

pub enum SequenceState {
    Inactive,
    Active,
}
pub struct SequenceHead {
    effected_mocks: Vec<MockId>,
    sequence_state: SequenceState,
    index: u32,
}
pub enum MockState {
    Locked { sequence_head_index: usize },
    Unlocked,
}
pub struct MockHead {
    state: MockState,
}

pub const CONTEXT_CONST: &str = "CONSTANT FROM CONTEXT";
#[derive(Debug, Clone, Hash, FromStr, AsRef, AsMut)]
pub struct MockId(String);
pub struct GlobalContext {
    sequence_heads: Vec<SequenceHead>,

    graph: DiGraph<MockNode, Edge>,

    mock_heads: HashMap<MockId, MockHead>,
}

impl GlobalContext {
    /*pub fn check_and_traverse_one_step(&self) -> Result<u32> {
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
    }*/
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
pub struct DefaultValuePtr {
    index: usize,
    size: usize,
}
pub struct ContextBuilder {
    default_value_data: Vec<u8>,
    start_index: NodeIndex,
    head: NodeIndex,
    environment: Environment,
    default_returns: HashMap<MockId, Option<DefaultValuePtr>>,
    graph: DiGraph<MockNode, Edge>,
}

impl ContextBuilder {
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
            graph,
            default_value_data: vec![],
            default_returns: HashMap::new(),
        }
    }

    pub fn add_expectation(
        &mut self,
        mock_id: MockId,
        condition: *const (),
        modifier: TimeModifier,
        exit: bool,
    ) -> Result<()> {
        let new_node_index = self.graph.add_node(MockNode { entry: false, exit });

        match modifier {
            TimeModifier::Once => {
                // add a single edge between nodes
                //       condition
                // (1) ---------------> (2)
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 0,
                    transition_cost: condition,
                });
                //consume input edge
                self.graph.add_edge(self.head, new_node_index, main_weight);
                self.head = new_node_index;
                Ok(())
            }
            TimeModifier::AtMostOnce => {
                //add two edges to new node, one with always ( lowest priority, one with condition)
                //      epsilon || condition
                // (1) ----------------------> (2)
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 1,
                    transition_cost: condition,
                });
                let instant_weight = Edge::Condition(ConditionalEdge {
                    priority: 0,
                    transition_cost: condition,
                });
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
                //       /   \
                //      |    |
                //       \  /         epsilon
                //        (1) ------------------> (2)
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 1,
                    transition_cost: condition,
                });
                let instant_weight = Edge::Condition(ConditionalEdge {
                    priority: 0,
                    transition_cost: condition,
                });
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
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 1,
                    transition_cost: condition,
                });

                let instant_weight = Edge::Condition(ConditionalEdge {
                    priority: 0,
                    transition_cost: condition,
                });
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

impl Default for ContextBuilder {
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

#[derive(Debug, Clone)]
pub struct ConditionalEdge {
    priority: u8,
    transition_cost: *const (),
}
#[derive(Debug, Clone)]
pub enum Edge {
    Instant { priority: u8 },
    Condition(ConditionalEdge),
}

pub struct MockNode {
    entry: bool,
    exit: bool,
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

//
