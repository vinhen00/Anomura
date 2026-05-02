use std::slice::{Iter, IterMut};

use crate::{
    ConditionDoublePointer, ReturnValDoublePointer,
    types::{mock::MockId, nodes::NodeIndex, sequences::SequenceIndex},
};

pub struct EdgeTransitionInfo {
    pub priority: u8,
    pub return_val: Option<ReturnValDoublePointer>,
    pub target_node: NodeIndex,
}
/*
an expectation can be a series of expectations,
There is a pool of mocks moving through the graph independently at the start.
If mock A enters a sequence where B is also dependent, A can take temporary ownership of B, assuming B is in a state where this is acceptable.
//if B is not in such a state, for example when moving through another sequence
*/

#[derive(Debug, Clone)]
pub struct ConditionalEdge {
    pub priority: u8,
    pub condition: ConditionDoublePointer,
    pub return_val: Option<ReturnValDoublePointer>,
    pub target: NodeIndex,
}

//all conditions must be matched but they can be matched in any order
#[derive(Debug, Clone)]
pub struct FreePermutationConditional {
    conditionals: Vec<(bool, ConditionalEdge)>,
}
#[derive(Debug, Clone)]
pub enum Edge {
    Instant { priority: u8, target: NodeIndex },
    Condition(ConditionalEdge),
    FreePermutation(FreePermutationConditional),
    SequenceEnter(SequenceIndex),
    SequenceExit { id: MockId, target: NodeIndex },
}
#[derive(Clone, Copy, Debug)]
pub struct EdgeIndex(usize);

#[derive(Debug, Clone)]
pub struct Edges(Vec<Edge>);

impl<U: Into<Vec<Edge>>> From<U> for Edges {
    fn from(value: U) -> Self {
        Self(value.into())
    }
}
impl Edges {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn add_edge(&mut self, edge: Edge) -> EdgeIndex {
        let index = EdgeIndex(self.0.len());
        self.0.push(edge);
        index
    }
    pub fn remove_edge(&mut self, index: EdgeIndex) -> Option<Edge> {
        if self.0.len() > index.0 {
            return None;
        }
        Some(self.0.remove(index.0))
    }
    pub fn edge_ref(&self, index: EdgeIndex) -> Option<&Edge> {
        self.0.get(index.0)
    }
    pub fn edge_mut(&mut self, index: EdgeIndex) -> Option<&mut Edge> {
        self.0.get_mut(index.0)
    }
    pub fn iter(&self) -> Iter<'_, Edge> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, Edge> {
        self.0.iter_mut()
    }
    pub fn into_iter(self) -> <std::vec::Vec<Edge> as std::iter::IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}
