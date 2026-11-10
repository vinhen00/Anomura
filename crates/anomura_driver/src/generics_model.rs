//! Generics and type-info model types for the two-pass analysis architecture.
//!
//! These types represent generic parameters (lifetimes, type params, const params),
//! where-clause predicates, `#[repr(...)]` information, trait implementation metadata,
//! and associated type models. They form a rustc-independent intermediate representation
//! that both the discovery pass and substitution pass can produce and consume.

use crate::type_repr::TypeRepr;

/// Generic parameters on a function, struct, enum, trait, or impl block.
#[derive(Debug, Clone)]
pub struct GenericsModel {
    pub lifetimes: Vec<LifetimeParam>,
    pub type_params: Vec<TypeParam>,
    pub const_params: Vec<ConstParam>,
    pub where_clauses: Vec<WherePredicate>,
}

#[derive(Debug, Clone)]
pub struct LifetimeParam {
    /// e.g. `'a`
    pub name: String,
    /// Lifetime bounds, e.g. `'a: 'b` means bounds = vec!["'b"]
    pub bounds: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TypeParam {
    /// e.g. `T`
    pub name: String,
    /// Trait bounds, e.g. `T: Clone + Send` means bounds = vec![Clone, Send]
    pub bounds: Vec<TypeRepr>,
    /// Default type, e.g. `T = i32`
    pub default: Option<TypeRepr>,
}

#[derive(Debug, Clone)]
pub struct ConstParam {
    pub name: String,
    pub ty: TypeRepr,
}

#[derive(Debug, Clone)]
pub enum WherePredicate {
    TypeBound {
        ty: TypeRepr,
        bounds: Vec<TypeRepr>,
    },
    LifetimeBound {
        lifetime: String,
        bounds: Vec<String>,
    },
}

/// Trait implementation info collected from `TyCtxt` (after_analysis).
#[derive(Debug, Clone)]
pub struct TraitImplInfo {
    pub trait_path: String,
    pub is_auto: bool,
    pub is_blanket: bool,
    pub is_derived: bool,
}

/// The primary memory layout strategy. These are mutually exclusive in Rust.
#[derive(Debug, Clone, PartialEq)]
pub enum PrimaryLayout {
    /// Default Rust layout (no explicit repr).
    Rust,
    /// `#[repr(C)]`
    C,
    /// `#[repr(transparent)]`
    Transparent,
    /// `#[repr(u8)]`, `#[repr(i32)]`, etc. — integer-sized enums.
    Int(String),
}

/// Alignment modifier. `packed` and `align` are mutually exclusive.
#[derive(Debug, Clone, PartialEq)]
pub enum AlignmentModifier {
    /// `#[repr(packed)]` or `#[repr(packed(N))]`
    Packed(u32),
    /// `#[repr(align(N))]`
    Align(u32),
}

/// `#[repr(...)]` information, modeled with proper mutual exclusivity.
///
/// - `layout` determines the primary layout strategy (C, transparent, int, or default Rust).
/// - `alignment` optionally overrides alignment (packed or align, never both).
///
/// Note: `repr(C)` can combine with `repr(packed)` or `repr(align)`, but `packed` and `align`
/// cannot combine with each other. `repr(transparent)` and `repr(Int)` cannot combine with
/// alignment modifiers.
#[derive(Debug, Clone)]
pub struct ReprInfo {
    pub layout: PrimaryLayout,
    pub alignment: Option<AlignmentModifier>,
}

#[derive(Debug, Clone)]
pub struct AssociatedTypeModel {
    pub name: String,
    pub bounds: Vec<TypeRepr>,
    pub default: Option<TypeRepr>,
}

#[derive(Debug, Clone)]
pub struct AssociatedTypeBinding {
    pub name: String,
    pub ty: TypeRepr,
}

// ---------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------

impl GenericsModel {
    /// Returns a `GenericsModel` with all `Vec`s empty.
    pub fn empty() -> Self {
        Self {
            lifetimes: Vec::new(),
            type_params: Vec::new(),
            const_params: Vec::new(),
            where_clauses: Vec::new(),
        }
    }

    /// Returns `true` if there are no generic parameters and no where-clause predicates.
    pub fn is_empty(&self) -> bool {
        self.lifetimes.is_empty()
            && self.type_params.is_empty()
            && self.const_params.is_empty()
            && self.where_clauses.is_empty()
    }

    /// Generates the `<...>` generic parameter clause.
    ///
    /// Returns an empty string when there are no parameters.
    ///
    /// Ordering: lifetimes, then type params, then const params.
    ///
    /// # Examples
    ///
    /// ```text
    /// <'a, T: Clone, const N: usize>
    /// <'a: 'b, T: Clone + Send, U = i32>
    /// ```
    pub fn to_source(&self) -> String {
        if self.lifetimes.is_empty() && self.type_params.is_empty() && self.const_params.is_empty()
        {
            return String::new();
        }

        let mut parts: Vec<String> = Vec::new();

        // Lifetimes first: 'a: 'b
        for lt in &self.lifetimes {
            if lt.bounds.is_empty() {
                parts.push(lt.name.clone());
            } else {
                parts.push(format!("{}: {}", lt.name, lt.bounds.join(" + ")));
            }
        }

        // Type params: T: Clone + Send  or  T = DefaultType
        for tp in &self.type_params {
            let mut s = tp.name.clone();
            if !tp.bounds.is_empty() {
                let bound_strs: Vec<String> = tp.bounds.iter().map(|b| b.to_source()).collect();
                s.push_str(": ");
                s.push_str(&bound_strs.join(" + "));
            }
            if let Some(ref default) = tp.default {
                s.push_str(" = ");
                s.push_str(&default.to_source());
            }
            parts.push(s);
        }

        // Const params: const N: usize
        for cp in &self.const_params {
            parts.push(format!("const {}: {}", cp.name, cp.ty.to_source()));
        }

        format!("<{}>", parts.join(", "))
    }

    /// Generates the `where` clause.
    ///
    /// Returns an empty string when there are no where-clause predicates.
    ///
    /// # Examples
    ///
    /// ```text
    /// where T: Clone + Send, 'a: 'b
    /// ```
    pub fn where_clause_to_source(&self) -> String {
        if self.where_clauses.is_empty() {
            return String::new();
        }

        let predicates: Vec<String> = self
            .where_clauses
            .iter()
            .map(|pred| match pred {
                WherePredicate::TypeBound { ty, bounds } => {
                    let bound_strs: Vec<String> = bounds.iter().map(|b| b.to_source()).collect();
                    format!("{}: {}", ty.to_source(), bound_strs.join(" + "))
                }
                WherePredicate::LifetimeBound { lifetime, bounds } => {
                    format!("{}: {}", lifetime, bounds.join(" + "))
                }
            })
            .collect();

        format!("where {}", predicates.join(", "))
    }
}

impl Default for ReprInfo {
    fn default() -> Self {
        Self {
            layout: PrimaryLayout::Rust,
            alignment: None,
        }
    }
}

impl ReprInfo {
    /// Generates the `#[repr(...)]` attribute string.
    ///
    /// Returns an empty string for the default Rust layout with no alignment modifier.
    pub fn to_source(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        match &self.layout {
            PrimaryLayout::Rust => {}
            PrimaryLayout::C => parts.push("C".to_string()),
            PrimaryLayout::Transparent => parts.push("transparent".to_string()),
            PrimaryLayout::Int(int_ty) => parts.push(int_ty.clone()),
        }

        if let Some(ref alignment) = self.alignment {
            match alignment {
                AlignmentModifier::Packed(n) => {
                    if *n == 1 {
                        parts.push("packed".to_string());
                    } else {
                        parts.push(format!("packed({})", n));
                    }
                }
                AlignmentModifier::Align(n) => {
                    parts.push(format!("align({})", n));
                }
            }
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("#[repr({})]", parts.join(", "))
        }
    }

    /// Whether this is the default Rust repr with no modifiers.
    pub fn is_default(&self) -> bool {
        self.layout == PrimaryLayout::Rust && self.alignment.is_none()
    }
}
