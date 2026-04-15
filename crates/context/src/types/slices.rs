use std::slice::{Iter, IterMut};

use crate::types::nodes::{Node, NodeIndex, Nodes};

#[derive(Debug, Clone)]
pub struct SliceBuilder {
    start_node: Node,
    nodes: Nodes,
    current: NodeIndex,
}
impl SliceBuilder {
    pub fn iter_nodes(&self) -> Iter<'_, Node> {
        self.nodes.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, Node> {
        self.nodes.iter_mut()
    }
    pub fn into_iter(self) -> <std::vec::Vec<Node> as std::iter::IntoIterator>::IntoIter {
        self.nodes.into_iter()
    }
}
#[derive(Debug, Clone, Copy)]
pub struct SliceRef(usize);
#[derive(Debug, Clone, Default)]
pub struct Slices(Vec<SliceBuilder>);
impl Slices {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn add_slice(&mut self, slice: SliceBuilder) -> SliceRef {
        let index = SliceRef(self.0.len());
        self.0.push(slice);
        index
    }
    pub fn get_ref_slice(&mut self, slice_ref: SliceRef) -> Option<&SliceBuilder> {
        self.0.get(slice_ref.0)
    }
    pub fn get_mut_slice(&mut self, slice_ref: SliceRef) -> Option<&mut SliceBuilder> {
        self.0.get_mut(slice_ref.0)
    }

    pub fn iter(&self) -> Iter<'_, SliceBuilder> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, SliceBuilder> {
        self.0.iter_mut()
    }
    pub fn into_iter(self) -> <std::vec::Vec<SliceBuilder> as std::iter::IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}
