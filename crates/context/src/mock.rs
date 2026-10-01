use crate::{AdtIdNumber, ReturnValDoublePointer};

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
pub enum MockId {
    AdtInstance { adt_id: AdtId, fn_id: FnId },
    AdtStatic { adt_path: AdtPath, fn_id: FnId },
    Fn(FnId),
}

impl MockId {
    /// Create a MockId for a standalone function.
    /// `id` is typically `"{crate}_{fn_name}"`.
    pub fn new_fn(id: impl Into<String>) -> Self {
        MockId::Fn(FnId::new(id))
    }

    /// Create a MockId for a static ADT method (constructors, associated fns, or
    /// instance methods on all-public structs that share a single mock ID).
    /// `adt_path` identifies the ADT (e.g. `"{crate}_{Struct}"`).
    /// `fn_id` identifies the method (e.g. `"{method_name}"`).
    pub fn new_adt_static(adt_path: impl Into<String>, fn_id: impl Into<String>) -> Self {
        MockId::AdtStatic {
            adt_path: AdtPath::new(adt_path),
            fn_id: FnId::new(fn_id),
        }
    }

    /// Create a MockId for a per-instance ADT method.
    /// `adt_path` identifies the ADT (e.g. `"{crate}_{Struct}"`).
    /// `fn_id` identifies the method (e.g. `"{method_name}"`).
    /// `number` is the instance counter from `adt_mock_id`.
    pub fn new_adt_instance(
        adt_path: impl Into<String>,
        fn_id: impl Into<String>,
        number: AdtIdNumber,
    ) -> Self {
        MockId::AdtInstance {
            adt_id: AdtId::new(adt_path, number),
            fn_id: FnId::new(fn_id),
        }
    }

    /// Get the FnId from any variant.
    pub fn fn_id(&self) -> &FnId {
        match self {
            MockId::Fn(fn_id) => fn_id,
            MockId::AdtStatic { fn_id, .. } => fn_id,
            MockId::AdtInstance { fn_id, .. } => fn_id,
        }
    }
}

impl std::fmt::Display for MockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MockId::Fn(fn_id) => write!(f, "{}", fn_id),
            MockId::AdtStatic { adt_path, fn_id } => write!(f, "{}_{}", adt_path, fn_id),
            MockId::AdtInstance { adt_id, fn_id } => write!(f, "{}_{}", adt_id, fn_id),
        }
    }
}

/// Contains the path + instance number for a per-instance ADT mock.
#[derive(Debug, Clone, Hash, core::cmp::Eq, PartialEq)]
pub struct AdtId {
    pub(crate) path: AdtPath,
    pub(crate) number: AdtIdNumber,
}

impl From<FnId> for MockId {
    fn from(value: FnId) -> Self {
        MockId::Fn(value)
    }
}

impl AdtId {
    pub fn new(path: impl Into<String>, number: AdtIdNumber) -> Self {
        AdtId {
            path: AdtPath::new(path),
            number,
        }
    }
    pub fn path(&self) -> &AdtPath {
        &self.path
    }
    pub fn number(&self) -> &AdtIdNumber {
        &self.number
    }
}

impl std::fmt::Display for AdtId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.path, self.number)
    }
}

#[derive(Debug, Clone, Hash, core::cmp::Eq, PartialEq)]
/// Contains the path to an ADT (struct/enum).
pub struct AdtPath(String);

impl AdtPath {
    pub fn new(path: impl Into<String>) -> Self {
        AdtPath(path.into())
    }
}

impl std::fmt::Display for AdtPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Just the path to the standalone function.
#[derive(Debug, Clone, Hash, core::cmp::Eq, PartialEq)]
pub struct FnId(String);

impl FnId {
    pub fn new(id: impl Into<String>) -> Self {
        FnId(id.into())
    }
}

impl std::fmt::Display for FnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
