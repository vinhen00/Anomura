//! Generics, bounds, and type-info model types for the two-pass analysis architecture.
//!
//! These types represent the full Rust generics system: generic parameters (lifetimes,
//! type params, const params), bounds (trait bounds, lifetime bounds, higher-ranked),
//! where-clause predicates, `#[repr(...)]` information, and function qualifiers.

use crate::type_repr::TypeRepr;

// ═══════════════════════════════════════════════════════════════════════════════
// Generic parameters
// ═══════════════════════════════════════════════════════════════════════════════

/// Generic parameters on a function, struct, enum, trait, impl, or associated type (GATs).
///
/// Parameters are stored in declaration order via a single `Vec<GenericParam>`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GenericParams {
    pub params: Vec<GenericParam>,
    pub where_clauses: Vec<WherePredicate>,
}

/// A single generic parameter declaration.
#[derive(Debug, Clone, serde::Serialize)]
pub enum GenericParam {
    Lifetime(LifetimeParam),
    Type(TypeParam),
    Const(ConstParam),
}

/// A lifetime parameter: `'a`, `'a: 'b + 'c`
#[derive(Debug, Clone, serde::Serialize)]
pub struct LifetimeParam {
    pub name: String,
    /// Outlives bounds: `'a: 'b + 'c` → `["'b", "'c"]`
    pub bounds: Vec<String>,
}

/// A type parameter: `T`, `T: Clone + 'a`, `T = i32`
#[derive(Debug, Clone, serde::Serialize)]
pub struct TypeParam {
    pub name: String,
    /// Inline bounds: `T: Clone + 'a`
    pub bounds: Vec<Bound>,
    /// Default type: `T = i32`
    pub default: Option<TypeRepr>,
}

/// A const generic parameter: `const N: usize`, `const N: usize = 3`
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConstParam {
    pub name: String,
    /// The type of the const parameter.
    pub ty: TypeRepr,
    /// Default value as source text: `"3"`, `"{ 1 + 2 }"`.
    pub default: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Bounds
// ═══════════════════════════════════════════════════════════════════════════════

/// A single bound on a type or lifetime.
#[derive(Debug, Clone, serde::Serialize)]
pub enum Bound {
    /// A trait bound: `Clone`, `Iterator<Item = u32>`, `?Sized`, `for<'a> Fn(&'a str)`
    Trait(TraitBound),
    /// A lifetime bound: `'a`, `'static`
    Lifetime(String),
}

/// A trait bound, optionally higher-ranked and/or relaxed.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TraitBound {
    /// The trait path with generic arguments.
    pub path: TypeRepr,
    /// Higher-ranked lifetime parameters: `for<'a, 'b>`.
    pub for_lifetimes: ForLifetimes,
    /// Whether this is a relaxed bound (`?Sized`).
    pub is_maybe: bool,
}

/// Higher-ranked lifetime quantifier: `for<'a, 'b>`.
///
/// Empty means no higher-ranked lifetimes (the common case).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ForLifetimes(pub Vec<String>);

// ═══════════════════════════════════════════════════════════════════════════════
// Where-clause predicates
// ═══════════════════════════════════════════════════════════════════════════════

/// A where-clause predicate.
#[derive(Debug, Clone, serde::Serialize)]
pub enum WherePredicate {
    /// `where T: Display + Send` or `where for<'a> T: Fn(&'a str)`
    TypeBound {
        for_lifetimes: ForLifetimes,
        ty: TypeRepr,
        bounds: Vec<Bound>,
    },
    /// `where 'a: 'b + 'c`
    LifetimeBound {
        lifetime: String,
        bounds: Vec<String>,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Function/method qualifiers and receiver
// ═══════════════════════════════════════════════════════════════════════════════

/// Qualifiers on a function or method.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct FnQualifiers {
    pub is_async: bool,
    pub is_unsafe: bool,
    pub is_const: bool,
    /// Extern ABI string: `"C"`, `"system"`. `None` for default Rust ABI.
    pub abi: Option<String>,
}

/// The kind of self receiver on a method.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Receiver {
    /// No self — static method or constructor.
    None,
    /// `&self`
    Ref,
    /// `&'a self`
    RefLifetime(String),
    /// `&mut self`
    RefMut,
    /// `&'a mut self`
    RefMutLifetime(String),
    /// `self` (owned)
    Owned,
    /// `self: Pin<&mut Self>` or other arbitrary self types.
    Arbitrary(TypeRepr),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Repr info
// ═══════════════════════════════════════════════════════════════════════════════

/// The primary memory layout strategy. Mutually exclusive in Rust.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum PrimaryLayout {
    Rust,
    C,
    Transparent,
    /// `#[repr(u8)]`, `#[repr(i32)]`, etc.
    Int(String),
}

/// Alignment modifier. `packed` and `align` are mutually exclusive.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum AlignmentModifier {
    Packed(u32),
    Align(u32),
}

/// `#[repr(...)]` information with proper mutual exclusivity.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReprInfo {
    pub layout: PrimaryLayout,
    pub alignment: Option<AlignmentModifier>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementations — GenericParams
// ═══════════════════════════════════════════════════════════════════════════════

impl GenericParams {
    pub fn empty() -> Self {
        Self { params: Vec::new(), where_clauses: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty() && self.where_clauses.is_empty()
    }

    /// Generates the `<...>` generic parameter clause.
    pub fn to_source(&self) -> String {
        let real_params: Vec<&GenericParam> = self.params.iter()
            .filter(|p| match p {
                GenericParam::Lifetime(lp) => lp.name != "'_" && lp.name != "''_",
                _ => true,
            })
            .collect();

        if real_params.is_empty() { return String::new(); }

        let parts: Vec<String> = real_params.iter().map(|p| p.to_source()).collect();
        format!("<{}>", parts.join(", "))
    }

    /// Generates the `where` clause.
    pub fn where_clause_to_source(&self) -> String {
        if self.where_clauses.is_empty() { return String::new(); }

        let predicates: Vec<String> = self.where_clauses.iter()
            .map(|pred| pred.to_source())
            .collect();

        format!("where {}", predicates.join(", "))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementations — GenericParam variants
// ═══════════════════════════════════════════════════════════════════════════════

impl GenericParam {
    pub fn to_source(&self) -> String {
        match self {
            GenericParam::Lifetime(lp) => lp.to_source(),
            GenericParam::Type(tp) => tp.to_source(),
            GenericParam::Const(cp) => cp.to_source(),
        }
    }
}

impl LifetimeParam {
    pub fn to_source(&self) -> String {
        if self.bounds.is_empty() { self.name.clone() }
        else { format!("{}: {}", self.name, self.bounds.join(" + ")) }
    }
}

impl TypeParam {
    pub fn to_source(&self) -> String {
        let mut s = self.name.clone();
        if !self.bounds.is_empty() {
            s.push_str(": ");
            s.push_str(&self.bounds.iter().map(|b| b.to_source()).collect::<Vec<_>>().join(" + "));
        }
        if let Some(ref d) = self.default {
            s.push_str(" = ");
            s.push_str(&d.to_source());
        }
        s
    }
}

impl ConstParam {
    pub fn to_source(&self) -> String {
        let mut s = format!("const {}: {}", self.name, self.ty.to_source());
        if let Some(ref d) = self.default {
            s.push_str(" = ");
            s.push_str(d);
        }
        s
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementations — Bound / TraitBound / ForLifetimes
// ═══════════════════════════════════════════════════════════════════════════════

impl Bound {
    pub fn to_source(&self) -> String {
        match self {
            Bound::Trait(tb) => tb.to_source(),
            Bound::Lifetime(lt) => lt.clone(),
        }
    }

    pub fn trait_bound(path: TypeRepr) -> Self {
        Bound::Trait(TraitBound::simple(path))
    }

    pub fn lifetime(name: impl Into<String>) -> Self {
        Bound::Lifetime(name.into())
    }
}

impl TraitBound {
    pub fn to_source(&self) -> String {
        let mut s = self.for_lifetimes.to_source_prefix();
        if self.is_maybe { s.push('?'); }
        s.push_str(&self.path.to_source());
        s
    }

    pub fn simple(path: TypeRepr) -> Self {
        Self { path, for_lifetimes: ForLifetimes::default(), is_maybe: false }
    }
}

impl ForLifetimes {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Renders `for<'a, 'b> ` prefix. Empty string if no lifetimes.
    pub fn to_source_prefix(&self) -> String {
        if self.0.is_empty() { String::new() }
        else { format!("for<{}> ", self.0.join(", ")) }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementations — WherePredicate
// ═══════════════════════════════════════════════════════════════════════════════

impl WherePredicate {
    pub fn to_source(&self) -> String {
        match self {
            WherePredicate::TypeBound { for_lifetimes, ty, bounds } => {
                let bound_strs: Vec<String> = bounds.iter().map(|b| b.to_source()).collect();
                format!("{}{}: {}", for_lifetimes.to_source_prefix(), ty.to_source(), bound_strs.join(" + "))
            }
            WherePredicate::LifetimeBound { lifetime, bounds } => {
                format!("{}: {}", lifetime, bounds.join(" + "))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementations — Receiver
// ═══════════════════════════════════════════════════════════════════════════════

impl Receiver {
    pub fn to_source(&self) -> String {
        match self {
            Receiver::None => String::new(),
            Receiver::Ref => "&self".to_string(),
            Receiver::RefLifetime(lt) => format!("&{} self", lt),
            Receiver::RefMut => "&mut self".to_string(),
            Receiver::RefMutLifetime(lt) => format!("&{} mut self", lt),
            Receiver::Owned => "self".to_string(),
            Receiver::Arbitrary(ty) => format!("self: {}", ty.to_source()),
        }
    }

    pub fn is_none(&self) -> bool { matches!(self, Receiver::None) }
    pub fn has_self(&self) -> bool { !self.is_none() }
    pub fn is_ref(&self) -> bool { matches!(self, Receiver::Ref | Receiver::RefLifetime(_)) }
    pub fn is_ref_mut(&self) -> bool { matches!(self, Receiver::RefMut | Receiver::RefMutLifetime(_)) }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementations — FnQualifiers
// ═══════════════════════════════════════════════════════════════════════════════

impl FnQualifiers {
    pub fn is_default(&self) -> bool {
        !self.is_async && !self.is_unsafe && !self.is_const && self.abi.is_none()
    }

    /// Renders `"const async unsafe extern \"C\" "` etc.
    pub fn to_source_prefix(&self) -> String {
        let mut s = String::new();
        if self.is_const { s.push_str("const "); }
        if self.is_async { s.push_str("async "); }
        if self.is_unsafe { s.push_str("unsafe "); }
        if let Some(ref abi) = self.abi { s.push_str(&format!("extern \"{}\" ", abi)); }
        s
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementations — ReprInfo
// ═══════════════════════════════════════════════════════════════════════════════

impl Default for ReprInfo {
    fn default() -> Self { Self { layout: PrimaryLayout::Rust, alignment: None } }
}

impl ReprInfo {
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
                    if *n == 1 { parts.push("packed".to_string()); }
                    else { parts.push(format!("packed({})", n)); }
                }
                AlignmentModifier::Align(n) => parts.push(format!("align({})", n)),
            }
        }
        if parts.is_empty() { String::new() }
        else { format!("#[repr({})]", parts.join(", ")) }
    }

    pub fn is_default(&self) -> bool {
        self.layout == PrimaryLayout::Rust && self.alignment.is_none()
    }
}
