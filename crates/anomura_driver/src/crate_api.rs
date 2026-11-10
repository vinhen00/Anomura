//! Data structures representing the public API of a crate, collected during AST walking.
//!
//! These mirror the structure of `mock_adt::MockAdtInput` but are built from the actual
//! crate source rather than user-provided macro input.
//!
//! All types use [`TypeRepr`] and [`String`] instead of `ast::Ty` and `Symbol`,
//! making them independent of the `rustc` AST and suitable for passing between
//! compiler invocations.

use crate::generics_model::*;
use crate::type_repr::TypeRepr;

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
    pub name: String,
    pub structs: Vec<StructModel>,
    pub enums: Vec<EnumModel>,
    pub traits: Vec<TraitModel>,
    pub functions: Vec<FunctionModel>,
    pub impls: Vec<ImplModel>,
    pub children: Vec<ModuleModel>,
}

impl ModuleModel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
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
    pub name: String,
    pub generics: GenericsModel,
    pub fields: Vec<FieldModel>,
    pub trackable: bool,
    pub repr: Option<ReprInfo>,
    /// Trait implementations, enriched from `after_analysis`.
    pub trait_impls: Vec<TraitImplInfo>,
}

impl StructModel {
    /// Whether all fields are public (determines trackability)
    pub fn all_public(&self) -> bool {
        self.fields.iter().all(|f| f.is_pub)
    }

    /// Whether this struct has a Default implementation (from derive or blanket).
    pub fn has_default(&self) -> bool {
        self.trait_impls.iter().any(|ti| {
            ti.trait_path == "Default"
                || ti.trait_path == "std::default::Default"
                || ti.trait_path == "core::default::Default"
        })
    }
}

/// A field in a struct.
#[derive(Debug, Clone)]
pub struct FieldModel {
    pub name: String,
    pub ty: TypeRepr,
    pub is_pub: bool,
}

/// A public enum with its variants.
#[derive(Debug, Clone)]
pub struct EnumModel {
    pub name: String,
    pub generics: GenericsModel,
    pub variants: Vec<VariantModel>,
    pub repr: Option<ReprInfo>,
    /// Trait implementations, enriched from `after_analysis`.
    pub trait_impls: Vec<TraitImplInfo>,
}

/// A variant in an enum.
#[derive(Debug, Clone)]
pub struct VariantModel {
    pub name: String,
    pub fields: Vec<TypeRepr>,
}

/// A public trait definition.
#[derive(Debug, Clone)]
pub struct TraitModel {
    pub name: String,
    pub generics: GenericsModel,
    pub supertraits: Vec<TypeRepr>,
    pub methods: Vec<MethodSigModel>,
    pub associated_types: Vec<AssociatedTypeModel>,
}

/// A method signature (used in traits, inherent impls, and trait impls).
#[derive(Debug, Clone)]
pub struct MethodSigModel {
    pub name: String,
    pub generics: GenericsModel,
    pub receiver: ReceiverKind,
    pub params: Vec<ParamModel>,
    pub return_type: TypeRepr,
    pub is_pub: bool,
    pub is_async: bool,
    pub is_unsafe: bool,
}

/// A function parameter.
#[derive(Debug, Clone)]
pub struct ParamModel {
    pub name: String,
    pub ty: TypeRepr,
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
    pub name: String,
    pub generics: GenericsModel,
    pub params: Vec<ParamModel>,
    pub return_type: TypeRepr,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub abi: Option<String>,
}

/// An impl block (inherent or trait).
#[derive(Debug, Clone)]
pub struct ImplModel {
    /// The type this impl is for, fully qualified.
    pub self_type: TypeRepr,
    /// Short name for matching (e.g., "Foo")
    pub self_type_name: String,
    /// If this is a trait impl, the trait ref; None for inherent impls.
    pub trait_ref: Option<TypeRepr>,
    /// Short trait name for matching (e.g., "Display")
    pub trait_name: Option<String>,
    /// Generics on the impl block itself.
    pub generics: GenericsModel,
    /// Methods in this impl block
    pub methods: Vec<MethodSigModel>,
    /// Associated type bindings (for trait impls).
    pub associated_types: Vec<AssociatedTypeBinding>,
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
