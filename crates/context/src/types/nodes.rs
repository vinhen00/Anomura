use std::{
    collections::HashSet,
    slice::{Iter, IterMut},
};

use crate::types::{
    edges::{Edge, EdgeIndex, Edges},
    mock::MockId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn iter(&self) -> Iter<'_, Node> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, Node> {
        self.0.iter_mut()
    }
    pub fn into_iter(self) -> <std::vec::Vec<Node> as std::iter::IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub node_kind: NodeKind,
    pub ids: HashSet<MockId>,
    conditions: Edges,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            node_kind: kind,
            ids: [].into(),
            conditions: Edges::new(),
        }
    }
    pub fn add_id(&mut self, id: MockId) -> bool {
        self.ids.insert(id)
    }
    pub fn add_edge(&mut self, edge: Edge) -> EdgeIndex {
        self.conditions.add_edge(edge)
    }
    pub fn get_edge_ref(&self, index: EdgeIndex) -> Option<&Edge> {
        self.conditions.edge_ref(index)
    }
    pub fn get_edge_mut(&mut self, index: EdgeIndex) -> Option<&mut Edge> {
        self.conditions.edge_mut(index)
    }
    pub fn contains_id(&self, id: &MockId) -> bool {
        self.ids.contains(id)
    }
    pub fn iter_conditions(&self) -> Iter<'_, Edge> {
        self.conditions.iter()
    }
}
#[derive(Debug, Clone)]
pub enum NodeKind {
    Mock,
    Sequence,
}
