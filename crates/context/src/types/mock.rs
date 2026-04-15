use crate::{
    ReturnValDoublePointer,
    types::{nodes::NodeIndex, sequences::SequenceIndex},
};

#[derive(Clone, Debug)]
pub enum MockState {
    Locked { sequence_head_index: SequenceIndex },
    Unlocked { mock_head_index: NodeIndex },
}
#[derive(Debug)]
pub struct MockHead {
    pub state: MockState,
    pub default_return_val: Option<ReturnValDoublePointer>,
}

pub const CONTEXT_CONST: &str = "CONSTANT FROM CONTEXT";
#[derive(Debug, Clone, Hash, core::cmp::Eq, PartialEq)]
pub struct MockId(String);
impl MockId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}
