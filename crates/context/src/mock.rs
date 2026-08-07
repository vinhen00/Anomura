use crate::ReturnValDoublePointer;

/// Metadata for a registered mock.
#[derive(Debug)]
pub struct MockHead {
    /// Default return value used when no expectation provides one.
    pub default_return_val: Option<ReturnValDoublePointer>,
    /// Strictness level for uninteresting calls.
    pub strictness: StrictnessKind,
}

/// How to handle calls that don't match any expectation.
#[derive(Clone, Copy, Debug, Default)]
pub enum StrictnessKind {
    /// Warning on uninteresting call.
    #[default]
    Naggy,
    /// Error on uninteresting call.
    Strict,
    /// No warnings.
    Nice,
}

pub const CONTEXT_CONST: &str = "CONSTANT FROM CONTEXT";

#[derive(Debug, Clone, Hash, core::cmp::Eq, PartialEq)]
pub struct MockId(String);

impl MockId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}
