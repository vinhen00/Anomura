mod builder;
mod closure_wrappers;
mod errors;
pub mod time_mod;
mod types;
mod unit_tests;
use errors::Result;
use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, OnceLock},
    usize,
};

use derive_more::{AsMut, AsRef, FromStr};

pub use crate::closure_wrappers::{ConditionDoublePointer, ReturnValDoublePointer};
use crate::{
    errors::{MockError, PredicateError},
    types::{
        edges::{Edge, EdgeTransitionInfo},
        mock::{MockHead, MockId, MockState},
        nodes::{Node, NodeIndex, Nodes},
        sequences::{SequenceHead, SequenceHeads, SequenceIndex},
        slices::Slices,
    },
};

pub static GLOBAL_CONTEXT: OnceLock<Mutex<GlobalContext>> = OnceLock::new();
//Slices are (for now) defined as sequences with a fixed start and end point.
//
#[derive(Debug)]
pub struct GlobalContext {
    slices: Slices,
    sequences: SequenceHeads,
    mock_heads: HashMap<MockId, MockHead>,
    nodes: Nodes,
}

impl GlobalContext {
    pub fn get_node_mut(&mut self, node_index: NodeIndex) -> Option<&mut Node> {
        self.nodes.get_node_mut(node_index)
    }
    pub fn get_node_ref(&mut self, node_index: NodeIndex) -> Option<&Node> {
        self.nodes.get_node_ref(node_index)
    }
    pub fn get_dot(&self) -> String {
        //let dot = petgraph::dot::Dot::new(&self.graph);
        //format!("{dot:?}")
        todo!()
    }
    pub fn get_sequence_head(&self, index: SequenceIndex) -> Option<&SequenceHead> {
        self.sequences.edge_ref(index)
    }
    pub fn mut_sequence_head(&mut self, index: SequenceIndex) -> Option<&mut SequenceHead> {
        self.sequences.edge_mut(index)
    }

    pub fn enter_sequence(&mut self, seq_head_index: SequenceIndex) -> Result<()> {
        let seq_head_vec = self
            .get_sequence_head(seq_head_index)
            .unwrap()
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
            let mut visited: HashSet<NodeIndex> = HashSet::new();
            let mut sequence_node_found = false;
            while !sequence_node_found && let Some(node_index) = instant_stack.pop() {
                let Some(node) = self.get_node_ref(node_index) else {
                    panic!(
                        "no node with id {:?} found when trying to enter sequence with index {:?}",
                        &id, seq_head_index
                    )
                };

                sequence_node_found = node.iter_conditions().any(|e| match e {
                    Edge::SequenceEnter(sequence_head_index)
                        if *sequence_head_index == seq_head_index =>
                    {
                        true
                    }
                    Edge::Instant { target, .. } => {
                        if !visited.contains(&target) {
                            instant_stack.push(*target);
                            visited.insert(*target);
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
        // All related mocks were in the correct position, so we move their heads and return success
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
    ) -> errors::Result<ReturnVal> {
        let (node_index, maybe_sequence_head_ids) = match self
            .mock_heads
            .get(&mock_id)
            .map(|mock_head| &mock_head.state)
        {
            Some(MockState::Locked {
                sequence_head_index,
            }) => {
                let seq_head = self
                    .get_sequence_head(*sequence_head_index)
                    .clone()
                    .unwrap();
                //This is bad performance and memory consumption wise... lets try to find a way to avoid cloning here.
                (seq_head.node_index, Some(seq_head.effected_mocks.clone()))
            }
            Some(MockState::Unlocked { mock_head_index }) => (*mock_head_index, None),
            None => return Err(MockError::NoMatchingId),
        };
        let mut sequence_head_indices: Vec<(NodeIndex, SequenceIndex)> = vec![];
        let mut sequence_stack_append: Vec<NodeIndex> = vec![];
        let mut instant_stack_append: Vec<NodeIndex> = vec![];
        let mut node_index_stack = vec![node_index];
        let mut visited = vec![];
        let mut failed_conditionals: Vec<PredicateError> = vec![];
        let mut successful_conditionals: Vec<_> = vec![];
        //traverses graph until we find a valid condition
        while let Some(node_index) = node_index_stack.pop() {
            if let Some(node) = self.get_node_ref(node_index) {
                if !node.contains_id(&mock_id) {
                    return Err(format!(
                        "node with index {:?} expected {:?} but received {:?}",
                        node_index, mock_id, node.ids
                    )
                    .into());
                };
                node.iter_conditions().for_each(|e| match e {
                    Edge::Instant { target, .. } => instant_stack_append.push(*target),
                    Edge::Condition(conditional_edge) => {
                        let condition = unsafe { conditional_edge.condition.into_fn::<Input>() };
                        let res = condition(&input);
                        match res {
                            Ok(()) => successful_conditionals.push(EdgeTransitionInfo {
                                priority: conditional_edge.priority,
                                return_val: conditional_edge.return_val.clone(),
                                target_node: conditional_edge.target,
                            }),
                            Err(e) => {
                                failed_conditionals.push(e);
                            }
                        }
                    }
                    Edge::SequenceEnter(index) => sequence_head_indices.push((node_index, *index)),
                    Edge::SequenceExit { id, target }
                        if maybe_sequence_head_ids
                            .clone()
                            .map(|ids| ids.contains(id))
                            .unwrap_or(false) =>
                    {
                        if *id == mock_id {
                            sequence_stack_append.push(*target)
                        }
                    }
                    _ => {}
                });
            } else {
                return Err("node not found".into());
            };

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
                if self.get_node_ref(new_node_index).is_none() {
                    return Err(format!("new node index {:?} is invalid", new_node_index).into());
                }
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
        head.state = MockState::Locked {
            sequence_head_index: new_node_index,
        };
        Ok(return_val.clone())
    }
}
