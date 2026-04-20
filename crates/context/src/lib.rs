mod builder;
mod closure_wrappers;
mod errors;
mod time_mod;
mod unit_tests;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::{Mutex, OnceLock},
};

use derive_more::{AsMut, AsRef, FromStr};

pub use crate::closure_wrappers::{ConditionDoublePointer, ReturnValDoublePointer};
use crate::errors::{MockError, PredicateError, Result};

pub static GLOBAL_CONTEXT: OnceLock<Mutex<GlobalContext>> = OnceLock::new();
#[derive(Debug, Clone, Copy)]
pub struct NodeIndex(usize);
#[derive(Debug, Clone)]
pub enum SequenceState {
    Inactive,
    Active,
}
#[derive(Debug, Clone)]
pub struct Nodes(Vec<Node>);
impl Nodes {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn add(&mut self, node: Node) -> NodeIndex {
        let index = NodeIndex(self.0.len());
        self.0.push(node);
        index
    }
    pub fn get_node_ref(&self, node_index: NodeIndex) -> Option<&Node> {
        self.0.get(node_index.0)
    }
    pub fn get_node_mut(&mut self, node_index: NodeIndex) -> Option<&mut Node> {
        self.0.get_mut(node_index.0)
    }
    pub fn remove_node(&mut self, node_index: NodeIndex) -> Option<Node> {
        if node_index.0 >= self.0.len() {
            return None;
        }
        Some(self.0.remove(node_index.0))
    }
}
//Slices are (for now) defined as sequences with a fixed start and end point.
#[derive(Debug, Clone)]
pub struct Slice {
    start_index: NodeIndex,
    nodes: Nodes,
    end_index: NodeIndex,
}
#[derive(Debug, Clone, Copy)]
pub struct SliceRef(usize);
#[derive(Debug, Clone)]
pub struct Slices(Vec<Slice>);
impl Slices {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn add_slice(&mut self, slice: Slice) -> SliceRef {
        let index = SliceRef(self.0.len());
        self.0.push(slice);
        index
    }
    pub fn get_ref_slice(&mut self, slice_ref: SliceRef) -> Option<&Slice> {
        self.0.get(slice_ref.0)
    }
    pub fn get_mut_slice(&mut self, slice_ref: SliceRef) -> Option<&mut Slice> {
        self.0.get_mut(slice_ref.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequenceHeadIndex(usize);
#[derive(Clone, Debug)]
pub struct SequenceHead {
    seq_head_index: SequenceHeadIndex,
    effected_mocks: Vec<MockId>,
    sequence_state: SequenceState,
    node_index: NodeIndex,
    enter_sequence: NodeIndex,
    exit_sequence: NodeIndex,
}

#[derive(Debug, Clone)]
pub struct SequenceExit {
    priority: u8,
    id: MockId,
}
#[derive(Debug, Clone)]
pub struct SequenceNode {
    entry: bool,
    exit: bool,
    id: MockId,
    ids: Vec<MockId>,
}
#[derive(Clone, Debug)]
pub enum MockState {
    Locked {
        sequence_head_index: SequenceHeadIndex,
    },
    Unlocked {
        mock_head_index: NodeIndex,
    },
}
pub struct MockHead {
    state: MockState,
    default_return_val: Option<ReturnValDoublePointer>,
}

pub const CONTEXT_CONST: &str = "CONSTANT FROM CONTEXT";
#[derive(Debug, Clone, Hash, FromStr, AsRef, AsMut, core::cmp::Eq, PartialEq)]
pub struct MockId(String);
impl MockId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

pub struct EdgeTransitionInfo {
    priority: u8,
    return_val: Option<ReturnValDoublePointer>,
    target_node: NodeIndex,
}
/*
an expectation can be a series of expectations,
There is a pool of mocks moving through the graph independently at the start.
If mock A enters a sequence where B is also dependent, A can take temporary ownership of B, assuming B is in a state where this is acceptable.
//if B is not in such a state, for example when moving through another sequence
*/

#[derive(Debug, Clone)]
pub struct ConditionalEdge {
    priority: u8,
    condition: ConditionDoublePointer,
    return_val: Option<ReturnValDoublePointer>,
}

//all conditions must be matched but they can be matched in any order
#[derive(Debug, Clone)]
pub struct FreePermutationConditional {
    conditionals: Vec<(bool, ConditionalEdge)>,
}
#[derive(Debug, Clone)]
pub enum Edge {
    Instant { priority: u8 },
    Condition(ConditionalEdge),
    FreePermutation(FreePermutationConditional),
    SequenceEnter(SequenceHeadIndex),
    SequenceExit(MockId),
}
#[derive(Debug, Clone)]
pub struct Node {
    node_kind: NodeKind,
    ids: HashSet<MockId>,
    conditions: Edge,
}
#[derive(Debug, Clone)]
pub enum NodeKind {
    Mock,
    Sequence,
}
//

pub struct GlobalContext {
    slices: Slices,
    sequences: Vec<SequenceHead>,
    mock_heads: HashMap<MockId, MockHead>,
    nodes: Nodes,
    edges: Vec<Edge>,
}

impl GlobalContext {
    pub fn get_dot(&self) -> String {
        let dot = petgraph::dot::Dot::new(&self.graph);
        format!("{dot:?}")
    }
    pub fn get_sequence_head(&self, index: SequenceHeadIndex) -> &SequenceHead {
        &self.sequence_heads[index.0]
    }
    pub fn mut_sequence_head(&mut self, index: SequenceHeadIndex) -> &mut SequenceHead {
        &mut self.sequence_heads[index.0]
    }

    pub fn enter_sequence(&mut self, seq_head_index: SequenceHeadIndex) -> Result<()> {
        let seq_head_vec = self
            .get_sequence_head(seq_head_index)
            .effected_mocks
            .clone();
        for id in &seq_head_vec {
            let mock_head_index = match self.mock_heads.get(id).map(|h| &h.state) {
                Some(MockState::Locked {
                    sequence_head_index,
                }) => {
                    return Err(format!(
                        "mock with id {:?} wanted to enter sequence with index {:?} 
                        but it was already in sequence with index{:?}",
                        id, seq_head_index, sequence_head_index
                    )
                    .into());
                }
                Some(MockState::Unlocked { mock_head_index }) => *mock_head_index,
                None => {
                    return Err(format!(
                        "mock_id {:?} not found when trying to enter sequence with index {:?}",
                        id, seq_head_index
                    )
                    .into());
                }
            };
            let mut instant_stack: Vec<NodeIndex> = vec![mock_head_index];
            let mut visited = HashSet::new();
            let mut sequence_node_found = false;
            while !sequence_node_found && let Some(node_index) = instant_stack.pop() {
                assert!(
                    self.graph.node_weight(node_index).is_some(),
                    "no node with id {:?} found when trying to enter sequence with index {:?}",
                    &id,
                    seq_head_index
                );

                sequence_node_found = self
                    .graph
                    .edges_directed(node_index, petgraph::Direction::Outgoing)
                    .any(|e| match e.weight() {
                        Edge::SequenceEnter(sequence_head_index)
                            if *sequence_head_index == seq_head_index =>
                        {
                            assert!(
                                e.target()
                                    == self.get_sequence_head(*sequence_head_index).enter_sequence
                            );
                            true
                        }
                        Edge::Instant { .. } => {
                            let target = e.target();
                            if !visited.contains(&target) {
                                instant_stack.push(e.target());
                                visited.insert(e.target());
                            }
                            false
                        }
                        _ => false,
                    });
            }
            if !sequence_node_found {
                return Err(format!(
                    "Mock with id {:?} couldn't find a valid entry point for entering sequence{:?}",
                    id, seq_head_index
                )
                .into());
            }
        }
        // All related mock was in the correct position, so we move their heads and return success
        for id in seq_head_vec {
            self.mock_heads.entry(id.clone()).and_modify(|h| {
                h.state = MockState::Locked {
                    sequence_head_index: seq_head_index,
                }
            });
        }
        Ok(())
    }

    pub fn run_mock<Input, ReturnVal>(
        &mut self,
        mock_id: MockId,
        input: Input,
    ) -> Result<ReturnVal> {
        let (node_index, maybe_sequence_head) = match self
            .mock_heads
            .get(&mock_id)
            .map(|mock_head| &mock_head.state)
        {
            Some(MockState::Locked {
                sequence_head_index,
            }) => {
                let seq_head = self.get_sequence_head(*sequence_head_index).clone();
                (seq_head.node_index, Some(seq_head))
            }
            Some(MockState::Unlocked { mock_head_index }) => (*mock_head_index, None),
            None => return Err(MockError::NoMatchingId),
        };
        let mut sequence_head_indices = vec![];
        let mut sequence_stack_append = vec![];
        let mut instant_stack_append = vec![];
        let mut node_index_stack = vec![node_index];
        let mut visited = vec![];
        let mut failed_conditionals: Vec<PredicateError> = vec![];
        let mut successful_conditionals: Vec<_> = vec![];
        //traverses graph until
        while let Some(node_index) = node_index_stack.pop() {
            let Some(res) = self.graph.node_weight(node_index) else {
                return Err("node not found".into());
            };
            if res.ids.contains(&mock_id) {
                return Err(format!(
                    "node with index {:?} expected {:?} but received {:?}",
                    node_index, mock_id, res.ids
                )
                .into());
            };
            self.graph
                .edges_directed(node_index, petgraph::Direction::Outgoing)
                .for_each(|e| match e.weight() {
                    Edge::Instant { .. } => instant_stack_append.push(e.target()),
                    Edge::Condition(conditional_edge) => {
                        let condition = unsafe { conditional_edge.condition.into_fn::<Input>() };
                        let res = condition(&input);
                        match res {
                            Ok(()) => successful_conditionals.push(EdgeTransitionInfo {
                                priority: conditional_edge.priority,
                                return_val: conditional_edge.return_val.clone(),
                                target_node: e.target(),
                            }),
                            Err(e) => {
                                failed_conditionals.push(e);
                            }
                        }
                    }
                    Edge::SequenceEnter(index) => sequence_head_indices.push((node_index, *index)),
                    Edge::SequenceExit(exit_mock_id)
                        if maybe_sequence_head
                            .clone()
                            .map(|head| head.effected_mocks.contains(exit_mock_id))
                            .unwrap_or(false) =>
                    {
                        if *exit_mock_id == mock_id {
                            sequence_stack_append.push(e.target())
                        }
                        self.mock_heads
                            .entry(exit_mock_id.clone())
                            .and_modify(|head| {
                                head.state = MockState::Unlocked {
                                    mock_head_index: e.target(),
                                }
                            });
                    }
                    _ => {}
                });
            for (target, seq_head) in sequence_head_indices.drain(0..) {
                if let Ok(()) = self.enter_sequence(seq_head) {
                    sequence_stack_append.push(target)
                }
            }
            //if valid conditions were found, pick the edge with the highest priority
            if !successful_conditionals.is_empty() {
                successful_conditionals.sort_by(|e, f| e.priority.cmp(&f.priority));
                let edge = successful_conditionals
                    .last()
                    .expect("we have already checked that edges is not empty");
                let return_val_ptr = edge.return_val.as_ref().unwrap_or(
                    self.mock_heads[&mock_id]
                        .default_return_val
                        .as_ref()
                        .expect("no return value found"),
                );
                let return_val = unsafe { return_val_ptr.into_fn::<Input, ReturnVal>()(input) };
                let new_node_index = edge.target_node;
                if self.graph.node_weight(new_node_index).is_none() {
                    return Err(format!("new node index {:?} is invalid", new_node_index).into());
                };
                self.mock_heads.entry(mock_id.clone()).and_modify(|h| {
                    h.state = MockState::Unlocked {
                        mock_head_index: new_node_index,
                    }
                });
                return Ok(return_val);
            }
            node_index_stack.append(&mut instant_stack_append);
            node_index_stack.append(&mut sequence_stack_append);
            visited.push(node_index);
        }
        //failed to find any valid conditions
        let joined = failed_conditionals
            .into_iter()
            .map(|e| e.0)
            .collect::<Vec<_>>()
            .join(",\n");
        Err(format!(
            "No matching condition found for input tried the following:  {}",
            joined
        )
        .into())
    }
}
