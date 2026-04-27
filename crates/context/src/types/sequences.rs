use std::slice::{Iter, IterMut};

use derive_more::Eq;

use crate::types::{mock::MockId, nodes::NodeIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SequenceIndex(usize);

#[derive(Debug, Clone)]
pub struct SequenceBuilder {
    seq_head_index: SequenceIndex,
    effected_mocks: Vec<MockId>,
    enter_sequence: NodeIndex,
    current_index: NodeIndex,
}

impl SequenceBuilder {
    pub fn to_sequence_head(self) -> SequenceHead {
        SequenceHead {
            seq_head_index: self.seq_head_index,
            effected_mocks: self.effected_mocks,
            node_index: self.enter_sequence,
            enter_sequence: self.enter_sequence,
            exit_sequence: self.current_index,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Sequences(Vec<SequenceBuilder>);

impl<U: Into<Vec<SequenceBuilder>>> From<U> for Sequences {
    fn from(value: U) -> Self {
        Self(value.into())
    }
}
impl Sequences {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn add_edge(&mut self, edge: SequenceBuilder) -> SequenceIndex {
        let index = SequenceIndex(self.0.len());
        self.0.push(edge);
        index
    }
    pub fn remove_edge(&mut self, index: SequenceIndex) -> Option<SequenceBuilder> {
        if self.0.len() > index.0 {
            return None;
        }
        Some(self.0.remove(index.0))
    }
    pub fn edge_ref(&self, index: SequenceIndex) -> Option<&SequenceBuilder> {
        self.0.get(index.0)
    }
    pub fn edge_mut(&mut self, index: SequenceIndex) -> Option<&mut SequenceBuilder> {
        self.0.get_mut(index.0)
    }
    pub fn iter(&self) -> Iter<'_, SequenceBuilder> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, SequenceBuilder> {
        self.0.iter_mut()
    }
    pub fn into_iter(
        self,
    ) -> <std::vec::Vec<SequenceBuilder> as std::iter::IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceHead {
    pub seq_head_index: SequenceIndex,
    pub effected_mocks: Vec<MockId>,
    pub node_index: NodeIndex,
    pub enter_sequence: NodeIndex,
    pub exit_sequence: NodeIndex,
}

#[derive(Debug, Clone)]
pub struct SequenceHeads(Vec<SequenceHead>);

impl<U: Into<Vec<SequenceHead>>> From<U> for SequenceHeads {
    fn from(value: U) -> Self {
        Self(value.into())
    }
}
impl SequenceHeads {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn add_edge(&mut self, edge: SequenceHead) -> SequenceIndex {
        let index = SequenceIndex(self.0.len());
        self.0.push(edge);
        index
    }
    pub fn remove_edge(&mut self, index: SequenceIndex) -> Option<SequenceHead> {
        if self.0.len() > index.0 {
            return None;
        }
        Some(self.0.remove(index.0))
    }
    pub fn edge_ref(&self, index: SequenceIndex) -> Option<&SequenceHead> {
        self.0.get(index.0)
    }
    pub fn edge_mut(&mut self, index: SequenceIndex) -> Option<&mut SequenceHead> {
        self.0.get_mut(index.0)
    }
    pub fn iter(&self) -> Iter<'_, SequenceHead> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, SequenceHead> {
        self.0.iter_mut()
    }
    pub fn into_iter(self) -> <std::vec::Vec<SequenceHead> as std::iter::IntoIterator>::IntoIter {
        self.0.into_iter()
    }
    pub fn contains_id(&self, index: SequenceIndex, id: &MockId) -> bool {
        self.edge_ref(index)
            .map(|e| e.effected_mocks.contains(id))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct SequenceExit {
    priority: u8,
    id: MockId,
}
