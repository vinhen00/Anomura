//! Compiler-independent representation of Rust types.
//!
//! This module provides two type representations following a "parse, don't validate" approach:
//!
//! - [`RawTypeRepr`] — used during AST collection (e.g., in `after_expansion`), where
//!   inferred types (`_`) may still be present. This is the "unvalidated" form.
//! - [`TypeRepr`] — the fully resolved form used in the analysis model and code generation.
//!   Has no `Inferred` variant — all types must be known. Produced by calling
//!   [`RawTypeRepr::resolve()`].
//!
//! This module has **no** dependencies on `rustc_ast` or any other compiler
//! crate — it is pure, stable Rust.

/// A compiler-independent representation of a Rust type.
///
/// Every variant is structured such that [`to_source`](TypeRepr::to_source)
/// always produces valid Rust syntax. Invalid states (empty paths, empty bounds,
/// etc.) are unrepresentable by construction.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeRepr {
    /// A named type path, e.g. `std::vec::Vec<T>`.
    ///
    /// `first` is the leading segment (guaranteed non-empty by construction).
    /// `rest` contains any additional `::` segments.
    Path {
        first: String,
        rest: Vec<String>,
        generics: Vec<TypeRepr>,
    },

    /// A reference type: `&'a mut T`.
    Reference {
        lifetime: Option<Lifetime>,
        mutable: bool,
        inner: Box<TypeRepr>,
    },

    /// A tuple type, e.g. `(A, B, C)`. An empty vec represents the unit type `()`.
    Tuple(Vec<TypeRepr>),

    /// A slice type: `[T]`.
    Slice(Box<TypeRepr>),

    /// An array type: `[T; N]`.
    Array {
        element: Box<TypeRepr>,
        length: ArrayLength,
    },

    /// A raw pointer: `*const T` or `*mut T`.
    RawPointer {
        mutable: bool,
        inner: Box<TypeRepr>,
    },

    /// A function pointer: `unsafe extern "C" fn(A, B) -> R`.
    FnPointer {
        params: Vec<TypeRepr>,
        return_type: Box<TypeRepr>,
        is_unsafe: bool,
        abi: Option<Abi>,
    },

    /// A trait object: `dyn Bound1 + Bound2 + 'a`.
    ///
    /// `first_bound` guarantees at least one trait bound exists.
    DynTrait {
        first_bound: Box<TypeRepr>,
        additional_bounds: Vec<TypeRepr>,
        lifetime: Option<Lifetime>,
    },

    /// An `impl Trait` type: `impl Bound1 + Bound2`.
    ///
    /// `first_bound` guarantees at least one bound exists.
    ImplTrait {
        first_bound: Box<TypeRepr>,
        additional_bounds: Vec<TypeRepr>,
    },

    /// The never type: `!`.
    Never,

    /// The `Self` type.
    SelfType,
}

/// A non-empty lifetime name (e.g., `"a"` for `'a`, `"static"` for `'static`).
/// Stored without the leading `'`.
#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime(String);

impl Lifetime {
    /// Create a new lifetime. Panics if `name` is empty.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(!name.is_empty(), "lifetime name must not be empty");
        Self(name)
    }

    /// The lifetime name without the leading `'`.
    pub fn name(&self) -> &str {
        &self.0
    }

    /// Formats as `'name`.
    pub fn to_source(&self) -> String {
        format!("'{}", self.0)
    }
}

/// A non-empty array length expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLength(String);

impl ArrayLength {
    /// Create a new array length. Panics if `expr` is empty.
    pub fn new(expr: impl Into<String>) -> Self {
        let expr = expr.into();
        assert!(!expr.is_empty(), "array length expression must not be empty");
        Self(expr)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A non-empty ABI string (e.g., `"C"`, `"system"`).
#[derive(Debug, Clone, PartialEq)]
pub struct Abi(String);

impl Abi {
    /// Create a new ABI. Panics if `abi` is empty.
    pub fn new(abi: impl Into<String>) -> Self {
        let abi = abi.into();
        assert!(!abi.is_empty(), "ABI string must not be empty");
        Self(abi)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TypeRepr {
    /// Convert this type representation to valid Rust source code.
    pub fn to_source(&self) -> String {
        match self {
            TypeRepr::Path { first, rest, generics } => {
                let mut path = first.clone();
                for seg in rest {
                    path.push_str("::");
                    path.push_str(seg);
                }
                if generics.is_empty() {
                    path
                } else {
                    let generic_args: Vec<String> =
                        generics.iter().map(|g| g.to_source()).collect();
                    format!("{}<{}>", path, generic_args.join(", "))
                }
            }

            TypeRepr::Reference {
                lifetime,
                mutable,
                inner,
            } => {
                let mut s = String::from("&");
                if let Some(lt) = lifetime {
                    s.push_str(&lt.to_source());
                    s.push(' ');
                }
                if *mutable {
                    s.push_str("mut ");
                }
                s.push_str(&inner.to_source());
                s
            }

            TypeRepr::Tuple(elements) => {
                if elements.is_empty() {
                    "()".to_string()
                } else if elements.len() == 1 {
                    format!("({},)", elements[0].to_source())
                } else {
                    let inner: Vec<String> = elements.iter().map(|e| e.to_source()).collect();
                    format!("({})", inner.join(", "))
                }
            }

            TypeRepr::Slice(inner) => {
                format!("[{}]", inner.to_source())
            }

            TypeRepr::Array { element, length } => {
                format!("[{}; {}]", element.to_source(), length.as_str())
            }

            TypeRepr::RawPointer { mutable, inner } => {
                if *mutable {
                    format!("*mut {}", inner.to_source())
                } else {
                    format!("*const {}", inner.to_source())
                }
            }

            TypeRepr::FnPointer {
                params,
                return_type,
                is_unsafe,
                abi,
            } => {
                let mut s = String::new();
                if *is_unsafe {
                    s.push_str("unsafe ");
                }
                if let Some(abi_val) = abi {
                    s.push_str(&format!("extern \"{}\" ", abi_val.as_str()));
                }
                let param_list: Vec<String> = params.iter().map(|p| p.to_source()).collect();
                s.push_str(&format!("fn({})", param_list.join(", ")));
                if !return_type.is_unit() {
                    s.push_str(&format!(" -> {}", return_type.to_source()));
                }
                s
            }

            TypeRepr::DynTrait { first_bound, additional_bounds, lifetime } => {
                let mut parts = vec![first_bound.to_source()];
                for b in additional_bounds {
                    parts.push(b.to_source());
                }
                if let Some(lt) = lifetime {
                    parts.push(lt.to_source());
                }
                format!("dyn {}", parts.join(" + "))
            }

            TypeRepr::ImplTrait { first_bound, additional_bounds } => {
                let mut parts = vec![first_bound.to_source()];
                for b in additional_bounds {
                    parts.push(b.to_source());
                }
                format!("impl {}", parts.join(" + "))
            }

            TypeRepr::Never => "!".to_string(),

            TypeRepr::SelfType => "Self".to_string(),
        }
    }

    /// Create the unit type `()`.
    pub fn unit() -> Self {
        TypeRepr::Tuple(vec![])
    }

    /// Returns `true` if this type is the unit type `()`.
    pub fn is_unit(&self) -> bool {
        matches!(self, TypeRepr::Tuple(elements) if elements.is_empty())
    }

    /// Convenience constructor for a single-segment path with no generics.
    ///
    /// Panics if `name` is empty.
    pub fn simple(name: &str) -> Self {
        assert!(!name.is_empty(), "type name must not be empty");
        TypeRepr::Path {
            first: name.to_string(),
            rest: vec![],
            generics: vec![],
        }
    }

    /// Convenience constructor for a path with generic parameters.
    ///
    /// `segments` must be non-empty. Panics if empty.
    pub fn with_generics(segments: Vec<String>, generics: Vec<TypeRepr>) -> Self {
        assert!(!segments.is_empty(), "path segments must not be empty");
        let mut iter = segments.into_iter();
        let first = iter.next().unwrap();
        assert!(!first.is_empty(), "first path segment must not be empty");
        TypeRepr::Path {
            first,
            rest: iter.collect(),
            generics,
        }
    }

    /// Returns all path segments as a collected Vec (convenience for iteration).
    pub fn path_segments(&self) -> Option<Vec<&str>> {
        match self {
            TypeRepr::Path { first, rest, .. } => {
                let mut segs = vec![first.as_str()];
                segs.extend(rest.iter().map(|s| s.as_str()));
                Some(segs)
            }
            _ => None,
        }
    }
}

/// A type representation used during AST collection, before full resolution.
///
/// Unlike [`TypeRepr`], this includes an `Inferred` variant for `_` types that
/// haven't been resolved yet. Call [`resolve()`](RawTypeRepr::resolve) to convert
/// to a [`TypeRepr`], which will fail if any `Inferred` types remain.
#[derive(Debug, Clone, PartialEq)]
pub enum RawTypeRepr {
    Path {
        first: String,
        rest: Vec<String>,
        generics: Vec<RawTypeRepr>,
    },
    Reference {
        lifetime: Option<Lifetime>,
        mutable: bool,
        inner: Box<RawTypeRepr>,
    },
    Tuple(Vec<RawTypeRepr>),
    Slice(Box<RawTypeRepr>),
    Array {
        element: Box<RawTypeRepr>,
        length: ArrayLength,
    },
    RawPointer {
        mutable: bool,
        inner: Box<RawTypeRepr>,
    },
    FnPointer {
        params: Vec<RawTypeRepr>,
        return_type: Box<RawTypeRepr>,
        is_unsafe: bool,
        abi: Option<Abi>,
    },
    DynTrait {
        first_bound: Box<RawTypeRepr>,
        additional_bounds: Vec<RawTypeRepr>,
        lifetime: Option<Lifetime>,
    },
    ImplTrait {
        first_bound: Box<RawTypeRepr>,
        additional_bounds: Vec<RawTypeRepr>,
    },
    Never,
    /// An inferred type (`_`). Must be resolved before the type can be used
    /// in the analysis model.
    Inferred,
    SelfType,
}

/// Error returned when [`RawTypeRepr::resolve()`] encounters an unresolved `Inferred` type.
#[derive(Debug, Clone)]
pub struct UnresolvedTypeError {
    pub message: String,
}

impl std::fmt::Display for UnresolvedTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unresolved inferred type: {}", self.message)
    }
}

impl RawTypeRepr {
    /// Convert to a fully resolved [`TypeRepr`].
    ///
    /// Returns `Err` if any `Inferred` variant is found in the type tree.
    pub fn resolve(self) -> Result<TypeRepr, UnresolvedTypeError> {
        match self {
            RawTypeRepr::Path { first, rest, generics } => {
                let resolved_generics = generics
                    .into_iter()
                    .map(|g| g.resolve())
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeRepr::Path {
                    first,
                    rest,
                    generics: resolved_generics,
                })
            }
            RawTypeRepr::Reference { lifetime, mutable, inner } => {
                Ok(TypeRepr::Reference {
                    lifetime,
                    mutable,
                    inner: Box::new(inner.resolve()?),
                })
            }
            RawTypeRepr::Tuple(elements) => {
                let resolved = elements
                    .into_iter()
                    .map(|e| e.resolve())
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeRepr::Tuple(resolved))
            }
            RawTypeRepr::Slice(inner) => {
                Ok(TypeRepr::Slice(Box::new(inner.resolve()?)))
            }
            RawTypeRepr::Array { element, length } => {
                Ok(TypeRepr::Array {
                    element: Box::new(element.resolve()?),
                    length,
                })
            }
            RawTypeRepr::RawPointer { mutable, inner } => {
                Ok(TypeRepr::RawPointer {
                    mutable,
                    inner: Box::new(inner.resolve()?),
                })
            }
            RawTypeRepr::FnPointer { params, return_type, is_unsafe, abi } => {
                let resolved_params = params
                    .into_iter()
                    .map(|p| p.resolve())
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeRepr::FnPointer {
                    params: resolved_params,
                    return_type: Box::new(return_type.resolve()?),
                    is_unsafe,
                    abi,
                })
            }
            RawTypeRepr::DynTrait { first_bound, additional_bounds, lifetime } => {
                let resolved_additional = additional_bounds
                    .into_iter()
                    .map(|b| b.resolve())
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeRepr::DynTrait {
                    first_bound: Box::new(first_bound.resolve()?),
                    additional_bounds: resolved_additional,
                    lifetime,
                })
            }
            RawTypeRepr::ImplTrait { first_bound, additional_bounds } => {
                let resolved_additional = additional_bounds
                    .into_iter()
                    .map(|b| b.resolve())
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeRepr::ImplTrait {
                    first_bound: Box::new(first_bound.resolve()?),
                    additional_bounds: resolved_additional,
                })
            }
            RawTypeRepr::Never => Ok(TypeRepr::Never),
            RawTypeRepr::Inferred => Err(UnresolvedTypeError {
                message: "encountered `_` type that was not resolved during analysis".to_string(),
            }),
            RawTypeRepr::SelfType => Ok(TypeRepr::SelfType),
        }
    }

    /// Convenience: convert to source string (for debugging during collection).
    pub fn to_source(&self) -> String {
        match self {
            RawTypeRepr::Inferred => "_".to_string(),
            other => {
                match other.clone().resolve() {
                    Ok(resolved) => resolved.to_source(),
                    Err(_) => "<unresolved>".to_string(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_path() {
        assert_eq!(TypeRepr::simple("u32").to_source(), "u32");
    }

    #[test]
    fn multi_segment_path() {
        let ty = TypeRepr::with_generics(
            vec!["std".into(), "vec".into(), "Vec".into()],
            vec![TypeRepr::simple("u32")],
        );
        assert_eq!(ty.to_source(), "std::vec::Vec<u32>");
    }

    #[test]
    fn with_generics_constructor() {
        let ty = TypeRepr::with_generics(
            vec!["Option".into()],
            vec![TypeRepr::simple("String")],
        );
        assert_eq!(ty.to_source(), "Option<String>");
    }

    #[test]
    fn reference_immutable() {
        let ty = TypeRepr::Reference {
            lifetime: None,
            mutable: false,
            inner: Box::new(TypeRepr::simple("str")),
        };
        assert_eq!(ty.to_source(), "&str");
    }

    #[test]
    fn reference_mutable_with_lifetime() {
        let ty = TypeRepr::Reference {
            lifetime: Some(Lifetime::new("a")),
            mutable: true,
            inner: Box::new(TypeRepr::simple("u8")),
        };
        assert_eq!(ty.to_source(), "&'a mut u8");
    }

    #[test]
    fn unit_type() {
        assert_eq!(TypeRepr::unit().to_source(), "()");
        assert!(TypeRepr::unit().is_unit());
    }

    #[test]
    fn non_unit_is_not_unit() {
        assert!(!TypeRepr::simple("u32").is_unit());
    }

    #[test]
    fn single_element_tuple() {
        let ty = TypeRepr::Tuple(vec![TypeRepr::simple("u32")]);
        assert_eq!(ty.to_source(), "(u32,)");
    }

    #[test]
    fn multi_element_tuple() {
        let ty = TypeRepr::Tuple(vec![
            TypeRepr::simple("u32"),
            TypeRepr::simple("String"),
            TypeRepr::simple("bool"),
        ]);
        assert_eq!(ty.to_source(), "(u32, String, bool)");
    }

    #[test]
    fn slice_type() {
        let ty = TypeRepr::Slice(Box::new(TypeRepr::simple("u8")));
        assert_eq!(ty.to_source(), "[u8]");
    }

    #[test]
    fn array_type() {
        let ty = TypeRepr::Array {
            element: Box::new(TypeRepr::simple("u8")),
            length: ArrayLength::new("16"),
        };
        assert_eq!(ty.to_source(), "[u8; 16]");
    }

    #[test]
    fn raw_pointer_const() {
        let ty = TypeRepr::RawPointer {
            mutable: false,
            inner: Box::new(TypeRepr::simple("u8")),
        };
        assert_eq!(ty.to_source(), "*const u8");
    }

    #[test]
    fn raw_pointer_mut() {
        let ty = TypeRepr::RawPointer {
            mutable: true,
            inner: Box::new(TypeRepr::simple("u8")),
        };
        assert_eq!(ty.to_source(), "*mut u8");
    }

    #[test]
    fn fn_pointer_simple() {
        let ty = TypeRepr::FnPointer {
            params: vec![TypeRepr::simple("u32")],
            return_type: Box::new(TypeRepr::simple("bool")),
            is_unsafe: false,
            abi: None,
        };
        assert_eq!(ty.to_source(), "fn(u32) -> bool");
    }

    #[test]
    fn fn_pointer_unit_return() {
        let ty = TypeRepr::FnPointer {
            params: vec![TypeRepr::simple("u32")],
            return_type: Box::new(TypeRepr::unit()),
            is_unsafe: false,
            abi: None,
        };
        assert_eq!(ty.to_source(), "fn(u32)");
    }

    #[test]
    fn fn_pointer_unsafe_extern() {
        let ty = TypeRepr::FnPointer {
            params: vec![TypeRepr::simple("i32"), TypeRepr::simple("i32")],
            return_type: Box::new(TypeRepr::simple("i32")),
            is_unsafe: true,
            abi: Some(Abi::new("C")),
        };
        assert_eq!(ty.to_source(), "unsafe extern \"C\" fn(i32, i32) -> i32");
    }

    #[test]
    fn dyn_trait() {
        let ty = TypeRepr::DynTrait {
            first_bound: Box::new(TypeRepr::simple("Display")),
            additional_bounds: vec![TypeRepr::simple("Send")],
            lifetime: Some(Lifetime::new("a")),
        };
        assert_eq!(ty.to_source(), "dyn Display + Send + 'a");
    }

    #[test]
    fn dyn_trait_single_bound() {
        let ty = TypeRepr::DynTrait {
            first_bound: Box::new(TypeRepr::simple("Iterator")),
            additional_bounds: vec![],
            lifetime: None,
        };
        assert_eq!(ty.to_source(), "dyn Iterator");
    }

    #[test]
    fn impl_trait() {
        let ty = TypeRepr::ImplTrait {
            first_bound: Box::new(TypeRepr::simple("Iterator")),
            additional_bounds: vec![],
        };
        assert_eq!(ty.to_source(), "impl Iterator");
    }

    #[test]
    fn never_type() {
        assert_eq!(TypeRepr::Never.to_source(), "!");
    }

    #[test]
    fn self_type() {
        assert_eq!(TypeRepr::SelfType.to_source(), "Self");
    }

    #[test]
    fn nested_generics() {
        let ty = TypeRepr::with_generics(
            vec!["Option".into()],
            vec![TypeRepr::with_generics(
                vec!["Vec".into()],
                vec![TypeRepr::Reference {
                    lifetime: Some(Lifetime::new("a")),
                    mutable: false,
                    inner: Box::new(TypeRepr::simple("str")),
                }],
            )],
        );
        assert_eq!(ty.to_source(), "Option<Vec<&'a str>>");
    }

    #[test]
    fn path_segments() {
        let ty = TypeRepr::with_generics(
            vec!["std".into(), "vec".into(), "Vec".into()],
            vec![],
        );
        assert_eq!(ty.path_segments(), Some(vec!["std", "vec", "Vec"]));
    }

    #[test]
    fn path_segments_single() {
        let ty = TypeRepr::simple("u32");
        assert_eq!(ty.path_segments(), Some(vec!["u32"]));
    }

    #[test]
    fn path_segments_non_path() {
        assert_eq!(TypeRepr::Never.path_segments(), None);
    }

    // --- Newtype validation tests ---

    #[test]
    #[should_panic(expected = "type name must not be empty")]
    fn simple_empty_name_panics() {
        TypeRepr::simple("");
    }

    #[test]
    #[should_panic(expected = "path segments must not be empty")]
    fn with_generics_empty_segments_panics() {
        TypeRepr::with_generics(vec![], vec![]);
    }

    #[test]
    #[should_panic(expected = "first path segment must not be empty")]
    fn with_generics_empty_first_segment_panics() {
        TypeRepr::with_generics(vec!["".into()], vec![]);
    }

    #[test]
    #[should_panic(expected = "lifetime name must not be empty")]
    fn empty_lifetime_panics() {
        Lifetime::new("");
    }

    #[test]
    #[should_panic(expected = "array length expression must not be empty")]
    fn empty_array_length_panics() {
        ArrayLength::new("");
    }

    #[test]
    #[should_panic(expected = "ABI string must not be empty")]
    fn empty_abi_panics() {
        Abi::new("");
    }

    // --- RawTypeRepr tests ---

    #[test]
    fn raw_resolve_simple() {
        let raw = RawTypeRepr::Path {
            first: "u32".into(),
            rest: vec![],
            generics: vec![],
        };
        let resolved = raw.resolve().unwrap();
        assert_eq!(resolved.to_source(), "u32");
    }

    #[test]
    fn raw_resolve_inferred_fails() {
        let raw = RawTypeRepr::Inferred;
        assert!(raw.resolve().is_err());
    }

    #[test]
    fn raw_resolve_nested_inferred_fails() {
        let raw = RawTypeRepr::Reference {
            lifetime: None,
            mutable: false,
            inner: Box::new(RawTypeRepr::Inferred),
        };
        assert!(raw.resolve().is_err());
    }

    #[test]
    fn raw_resolve_complex() {
        let raw = RawTypeRepr::FnPointer {
            params: vec![
                RawTypeRepr::Path { first: "u32".into(), rest: vec![], generics: vec![] },
            ],
            return_type: Box::new(RawTypeRepr::Path {
                first: "bool".into(),
                rest: vec![],
                generics: vec![],
            }),
            is_unsafe: false,
            abi: None,
        };
        let resolved = raw.resolve().unwrap();
        assert_eq!(resolved.to_source(), "fn(u32) -> bool");
    }

    #[test]
    fn raw_inferred_to_source() {
        assert_eq!(RawTypeRepr::Inferred.to_source(), "_");
    }
}
