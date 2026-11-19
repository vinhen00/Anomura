//! Data structures representing the public API of a crate.
//!
//! Uses a flat `ItemStore` with `ItemId` references instead of nested ownership.
//! Every definition (struct, enum, trait, function, impl, module, field, variant)
//! is an `Item` with common metadata and a kind-specific payload.
//!
//! All types use [`TypeRepr`] and [`String`] instead of `ast::Ty` and `Symbol`,
//! making them independent of the `rustc` AST and suitable for passing between
//! compiler invocations.

use crate::generics_model::*;
use crate::type_repr::TypeRepr;

// ---------------------------------------------------------------------------
// ItemId — transparent u32 handle
// ---------------------------------------------------------------------------

/// A unique identifier for an item within a `CrateApiModel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct ItemId(pub u32);

impl std::fmt::Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ItemId({})", self.0)
    }
}

// ---------------------------------------------------------------------------
// Visibility
// ---------------------------------------------------------------------------

/// Visibility of an item.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Visibility {
    /// `pub`
    Public,
    /// `pub(crate)`
    Crate,
    /// `pub(super)`
    Super,
    /// `pub(in path)`
    Restricted(String),
    /// Private (no visibility keyword).
    Private,
}

// ---------------------------------------------------------------------------
// Attribute
// ---------------------------------------------------------------------------

/// A Rust attribute on an item.
#[derive(Debug, Clone, serde::Serialize)]
pub enum Attribute {
    /// `#[derive(Debug, Clone, ...)]`
    Derive(Vec<String>),
    /// `#[repr(C)]`, `#[repr(packed)]`, etc.
    Repr(ReprInfo),
    /// `#[cfg(...)]` — stored as the raw condition string.
    Cfg(String),
    /// `#[allow(...)]`, `#[warn(...)]`, `#[deny(...)]`, `#[forbid(...)]`
    Lint {
        level: LintLevel,
        lints: Vec<String>,
    },
    /// `#[must_use]` or `#[must_use = "reason"]`
    MustUse(Option<String>),
    /// `#[deprecated]` or `#[deprecated(since = "...", note = "...")]`
    Deprecated {
        since: Option<String>,
        note: Option<String>,
    },
    /// `#[doc = "..."]` — a single doc-comment line (each `///` line is one attribute).
    Doc(String),
    /// `#[inline]`, `#[inline(always)]`, `#[inline(never)]`
    Inline(InlineKind),
    /// `#[non_exhaustive]`
    NonExhaustive,
    /// `#[serde(...)]`, `#[tokio::test]`, or any other attribute not specifically modeled.
    /// Stored as the full attribute text (without the `#[` and `]`).
    Other(String),
}

/// Lint level for `#[allow(...)]` / `#[warn(...)]` / etc.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum LintLevel {
    Allow,
    Warn,
    Deny,
    Forbid,
}

/// Inline hint kind.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum InlineKind {
    /// `#[inline]`
    Hint,
    /// `#[inline(always)]`
    Always,
    /// `#[inline(never)]`
    Never,
}

// ---------------------------------------------------------------------------
// Item — the universal wrapper
// ---------------------------------------------------------------------------

/// An item in the crate API. Every definition gets one of these.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Item {
    /// Unique identifier within the crate model.
    pub id: ItemId,
    /// The item's name (e.g., struct name, function name, field name).
    pub name: String,
    /// Visibility of this item.
    pub visibility: Visibility,
    /// Attributes on this item.
    pub attrs: Vec<Attribute>,
    /// The kind-specific payload.
    pub kind: ItemKind,
}

// ---------------------------------------------------------------------------
// ItemKind — the definition variants
// ---------------------------------------------------------------------------

/// The kind of definition an `Item` represents.
#[derive(Debug, Clone, serde::Serialize)]
pub enum ItemKind {
    /// A module containing other items.
    Module(ModuleData),
    /// A struct definition.
    Struct(StructData),
    /// An enum definition.
    Enum(EnumData),
    /// An enum variant (child of an enum).
    Variant(VariantData),
    /// A struct, tuple-struct, or enum-variant field.
    Field(FieldData),
    /// A trait definition.
    Trait(TraitData),
    /// A free function.
    Function(FunctionData),
    /// An impl block (inherent or trait).
    Impl(ImplData),
    /// A method inside an impl or trait.
    Method(MethodData),
    /// An associated type declaration (in a trait) or binding (in an impl).
    AssociatedType(AssociatedTypeData),
}

// ---------------------------------------------------------------------------
// Kind-specific data types
// ---------------------------------------------------------------------------

/// Data for `ItemKind::Module`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleData {
    /// All direct child items (structs, enums, functions, impls, submodules, etc.).
    pub children: Vec<ItemId>,
}

/// The structural kind of a struct.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum StructKind {
    /// `struct Foo { x: u32, y: String }` — named fields.
    Plain,
    /// `struct Foo(u32, String)` — positional fields.
    Tuple,
    /// `struct Foo;` — no fields.
    Unit,
}

/// Data for `ItemKind::Struct`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StructData {
    pub generics: GenericParams,
    pub kind: StructKind,
    /// Field items (`ItemKind::Field`). Empty for unit structs.
    pub fields: Vec<ItemId>,
    /// Whether the struct has private fields (making it trackable for mock dispatch).
    pub trackable: bool,
}

/// Data for `ItemKind::Enum`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EnumData {
    pub generics: GenericParams,
    /// Variant items (`ItemKind::Variant`).
    pub variants: Vec<ItemId>,
}

/// Data for `ItemKind::Variant`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VariantData {
    /// The structural kind: unit, tuple, or struct-like.
    pub kind: StructKind,
    /// Field items for struct-like or tuple-like variants. Empty for unit variants.
    pub fields: Vec<ItemId>,
}

/// Data for `ItemKind::Field`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FieldData {
    pub ty: TypeRepr,
}

/// Data for `ItemKind::Trait`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TraitData {
    pub generics: GenericParams,
    pub supertraits: Vec<Bound>,
    /// Method items (`ItemKind::Method`).
    pub methods: Vec<ItemId>,
    /// Associated type items (`ItemKind::AssociatedType`).
    pub associated_types: Vec<ItemId>,
}

/// Data for `ItemKind::Function`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FunctionData {
    pub generics: GenericParams,
    pub qualifiers: FnQualifiers,
    pub params: Vec<ParamModel>,
    pub return_type: TypeRepr,
}

/// A function/method parameter. Inline (not an Item) since params are not
/// independently referenceable.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParamModel {
    pub name: String,
    pub ty: TypeRepr,
}

/// Data for `ItemKind::Impl`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImplData {
    /// The type this impl is for (e.g., `Wrapper<T>`).
    pub self_type: TypeRepr,
    /// Short name for matching (e.g., "Foo").
    pub self_type_name: String,
    /// If this is a trait impl, the trait reference; `None` for inherent impls.
    pub trait_ref: Option<TypeRepr>,
    /// Short trait name (e.g., "Display").
    pub trait_name: Option<String>,
    /// Generics on the impl block itself.
    pub generics: GenericParams,
    /// Method items (`ItemKind::Method`).
    pub methods: Vec<ItemId>,
    /// Associated type items (`ItemKind::AssociatedType`).
    pub associated_types: Vec<ItemId>,
}

/// Data for `ItemKind::Method`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MethodData {
    pub generics: GenericParams,
    pub qualifiers: FnQualifiers,
    pub receiver: Receiver,
    pub params: Vec<ParamModel>,
    pub return_type: TypeRepr,
}

/// Data for `ItemKind::AssociatedType`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AssociatedTypeData {
    /// Generic parameters (for GATs), e.g., `type Item<'a>` or `type Output<T: Clone>`.
    pub generics: GenericParams,
    /// Trait bounds (for declarations in traits), e.g., `type Item: Display`.
    pub bounds: Vec<Bound>,
    /// Concrete type (for bindings in impls), e.g., `type Item = u32`.
    pub ty: Option<TypeRepr>,
}

// ---------------------------------------------------------------------------
// ItemStore — flat storage with ID-based lookup
// ---------------------------------------------------------------------------

/// A flat store for all items in a crate. Items are allocated sequentially
/// and looked up by `ItemId`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ItemStore {
    items: Vec<Item>,
}

impl ItemStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Allocate a new item and return its `ItemId`.
    pub fn alloc(
        &mut self,
        name: impl Into<String>,
        visibility: Visibility,
        kind: ItemKind,
    ) -> ItemId {
        let id = ItemId(self.items.len() as u32);
        self.items.push(Item {
            id,
            name: name.into(),
            visibility,
            attrs: Vec::new(),
            kind,
        });
        id
    }

    /// Get a reference to an item by ID.
    pub fn get(&self, id: ItemId) -> &Item {
        &self.items[id.0 as usize]
    }

    /// Get a mutable reference to an item by ID.
    pub fn get_mut(&mut self, id: ItemId) -> &mut Item {
        &mut self.items[id.0 as usize]
    }

    /// Total number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterate over all items.
    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }
}

impl Default for ItemStore {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// CrateApiModel — the top-level model
// ---------------------------------------------------------------------------

/// The full public API of a crate.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CrateApiModel {
    /// The crate name (used as prefix for mock IDs).
    pub crate_name: String,
    /// Flat store of all items.
    pub items: ItemStore,
    /// The root module's `ItemId`.
    pub root: ItemId,
}

// ---------------------------------------------------------------------------
// ModPath — used by code generation
// ---------------------------------------------------------------------------

/// A module path represented as a crate name and a list of module segments.
/// Provides conversions to both the mock ID prefix format (underscore-separated)
/// and the Rust module path format (`::` separated).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModPath {
    pub crate_name: String,
    pub segments: Vec<String>,
}

impl ModPath {
    /// Create a root-level path (no modules).
    pub fn root(crate_name: &str) -> Self {
        Self {
            crate_name: crate_name.to_string(),
            segments: Vec::new(),
        }
    }

    /// Create a child path by appending a module segment.
    pub fn child(&self, segment: &str) -> Self {
        let mut segments = self.segments.clone();
        segments.push(segment.to_string());
        Self {
            crate_name: self.crate_name.clone(),
            segments,
        }
    }

    /// Mock ID prefix with struct name: crate + modules + struct joined by `_`.
    /// e.g. crate "fns", segments ["a", "nested"], struct "Inner" → "fns_a_nested_Inner"
    pub fn mock_prefix(&self, struct_name: &str) -> String {
        let mut parts = vec![self.crate_name.clone()];
        parts.extend(self.segments.clone());
        if !struct_name.is_empty() {
            parts.push(struct_name.to_string());
        }
        parts.join("_")
    }

    /// Rust module path: module segments joined by `::`.
    /// Returns empty string for root level.
    /// e.g. segments ["a", "nested"] → "a::nested"
    pub fn mod_path(&self) -> String {
        self.segments.join("::")
    }
}

// ---------------------------------------------------------------------------
// Convenience helpers
// ---------------------------------------------------------------------------

impl Item {
    /// Returns `true` if this item is public.
    pub fn is_pub(&self) -> bool {
        matches!(self.visibility, Visibility::Public)
    }
}

impl CrateApiModel {
    /// Get the root module data.
    pub fn root_module(&self) -> &ModuleData {
        match &self.items.get(self.root).kind {
            ItemKind::Module(m) => m,
            _ => panic!("root ItemId does not point to a Module"),
        }
    }

    /// Count items by kind for debug logging.
    pub fn count_by_kind(&self) -> (usize, usize, usize, usize, usize) {
        let mut structs = 0;
        let mut enums = 0;
        let mut functions = 0;
        let mut traits = 0;
        let mut impls = 0;
        for item in self.items.iter() {
            match &item.kind {
                ItemKind::Struct(_) => structs += 1,
                ItemKind::Enum(_) => enums += 1,
                ItemKind::Function(_) => functions += 1,
                ItemKind::Trait(_) => traits += 1,
                ItemKind::Impl(_) => impls += 1,
                _ => {}
            }
        }
        (structs, enums, functions, traits, impls)
    }
}

impl StructData {
    /// Whether this struct has a Default implementation.
    /// TODO: Reimplement once trait impl tracking is redesigned without TraitImplInfo.
    pub fn has_default(&self) -> bool {
        false
    }
}
