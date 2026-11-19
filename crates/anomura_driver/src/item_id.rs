//! Item identifier for cross-referencing definitions across crates.

/// A unique identifier for an item within a [`CrateApiModel`](crate::crate_api::CrateApiModel).
///
/// References an item across potentially multiple crates:
/// - `crate_idx = 0` is always the primary (compiled) crate.
/// - Higher indices refer to external dependency crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ItemId {
    /// Index into `CrateApiModel.crates`. 0 = the primary crate.
    pub crate_idx: u16,
    /// Index into that crate's `ItemStore`.
    pub item_idx: u32,
}

impl ItemId {
    /// Create an ItemId for the primary crate.
    pub fn local(item_idx: u32) -> Self {
        Self { crate_idx: 0, item_idx }
    }

    /// Whether this item is in the primary (compiled) crate.
    pub fn is_local(&self) -> bool {
        self.crate_idx == 0
    }
}

impl std::fmt::Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.crate_idx == 0 {
            write!(f, "ItemId({})", self.item_idx)
        } else {
            write!(f, "ItemId({}:{})", self.crate_idx, self.item_idx)
        }
    }
}
