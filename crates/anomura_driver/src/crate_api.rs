//! Data structures representing the public API of a crate, collected during AST walking.
//! These mirror the structure of `mock_adt::MockAdtInput` but are built from the actual
//! crate source rather than user-provided macro input.

use rustc_ast as ast;
use rustc_span::symbol::Symbol;

/// The full public API of a crate, organized by module.
#[derive(Debug, Clone)]
pub struct CrateApiModel {
    /// The crate name (used as prefix for mock IDs)
    pub crate_name: String,
    /// Top-level module (represents the crate root)
    pub root: ModuleModel,
}

/// A module and its public contents.
#[derive(Debug, Clone)]
pub struct ModuleModel {
    pub name: Symbol,
    pub structs: Vec<StructModel>,
    pub enums: Vec<EnumModel>,
    pub traits: Vec<TraitModel>,
    pub functions: Vec<FunctionModel>,
    pub impls: Vec<ImplModel>,
    pub children: Vec<ModuleModel>,
}

impl ModuleModel {
    pub fn new(name: Symbol) -> Self {
        Self {
            name,
            structs: Vec::new(),
            enums: Vec::new(),
            traits: Vec::new(),
            functions: Vec::new(),
            impls: Vec::new(),
            children: Vec::new(),
        }
    }
}

/// A public struct with its fields.
#[derive(Debug, Clone)]
pub struct StructModel {
    pub name: Symbol,
    pub fields: Vec<FieldModel>,
    pub trackable: bool,
}

impl StructModel {
    /// Whether all fields are public (determines trackability)
    pub fn all_public(&self) -> bool {
        self.fields.iter().all(|f| f.is_pub)
    }
}

/// A field in a struct.
#[derive(Debug, Clone)]
pub struct FieldModel {
    pub name: Symbol,
    pub ty: Box<ast::Ty>,
    pub is_pub: bool,
}

/// A public enum with its variants.
#[derive(Debug, Clone)]
pub struct EnumModel {
    pub name: Symbol,
    pub variants: Vec<VariantModel>,
}

/// A variant in an enum.
#[derive(Debug, Clone)]
pub struct VariantModel {
    pub name: Symbol,
    pub fields: Vec<Box<ast::Ty>>,
}

/// A public trait definition.
#[derive(Debug, Clone)]
pub struct TraitModel {
    pub name: Symbol,
    pub methods: Vec<MethodSigModel>,
}

/// A method signature (used in traits, inherent impls, and trait impls).
#[derive(Debug, Clone)]
pub struct MethodSigModel {
    pub name: Symbol,
    pub receiver: ReceiverKind,
    pub params: Vec<ParamModel>,
    pub return_type: Option<Box<ast::Ty>>,
    pub is_pub: bool,
}

/// A function parameter.
#[derive(Debug, Clone)]
pub struct ParamModel {
    pub name: Symbol,
    pub ty: Box<ast::Ty>,
}

/// The kind of self receiver on a method.
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiverKind {
    /// No self — static method or constructor
    None,
    /// `&self`
    Ref,
    /// `&mut self`
    RefMut,
    /// `self` (owned)
    Owned,
}

/// A public free function.
#[derive(Debug, Clone)]
pub struct FunctionModel {
    pub name: Symbol,
    pub params: Vec<ParamModel>,
    pub return_type: Option<Box<ast::Ty>>,
}

/// An impl block (inherent or trait).
#[derive(Debug, Clone)]
pub struct ImplModel {
    /// The type this impl is for (e.g., "Foo")
    pub self_type_name: Symbol,
    /// If this is a trait impl, the trait name; None for inherent impls
    pub trait_name: Option<Symbol>,
    /// Methods in this impl block
    pub methods: Vec<MethodSigModel>,
}

/// A module path represented as a crate name and a list of module segments.
/// Provides conversions to both the mock ID prefix format (underscore-separated)
/// and the Rust module path format (`::` separated).
#[derive(Debug, Clone)]
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
