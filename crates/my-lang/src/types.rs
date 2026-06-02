//! Type system definitions for My Language
//!
//! Defines the internal representation of types used during type checking.

use std::fmt;

/// Linearity mode of an echo residue.
///
/// Mirrors `Mode` in the `echo-types` Agda library (`EchoLinear.agda`):
/// the thin two-point poset `linear ⊑ affine`. `Linear` keeps the full,
/// proof-relevant fiber witness; `Affine` keeps only the collapsed
/// residue (all affine echoes over a point are equal — `affine-all-equal`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EchoMode {
    /// Full fiber retained; distinctions preserved.
    Linear,
    /// Collapsed residue; distinctions weakened away.
    Affine,
}

impl EchoMode {
    /// The mode ordering `_≤m_` from `EchoLinear.agda`
    /// (`linear ≤m linear`, `linear ≤m affine`, `affine ≤m affine`).
    ///
    /// `self.weakens_to(other)` is true when an echo at mode `self` may be
    /// *weakened* to mode `other`. Crucially `Affine` does **not** weaken to
    /// `Linear`: weakening is lossy and has no section (`no-section-weaken`).
    pub fn weakens_to(self, other: EchoMode) -> bool {
        matches!(
            (self, other),
            (EchoMode::Linear, EchoMode::Linear)
                | (EchoMode::Linear, EchoMode::Affine)
                | (EchoMode::Affine, EchoMode::Affine)
        )
    }
}

impl fmt::Display for EchoMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EchoMode::Linear => write!(f, "linear"),
            EchoMode::Affine => write!(f, "affine"),
        }
    }
}

/// Internal type representation used during type checking
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    /// Primitive types
    Int,
    Float,
    String,
    Bool,

    /// Unit type (void)
    Unit,

    /// Named type (struct, effect, etc.)
    Named(String),

    /// Function type
    Function {
        params: Vec<Ty>,
        result: Box<Ty>,
    },

    /// Array type
    Array(Box<Ty>),

    /// Reference type
    Ref {
        mutable: bool,
        inner: Box<Ty>,
    },

    /// Tuple type
    Tuple(Vec<Ty>),

    /// Record type
    Record(Vec<(String, Ty)>),

    /// AI effect type
    AI(Box<Ty>),

    /// Effect type
    Effect(Box<Ty>),

    /// Echo type — the proof-relevant residue of a lossy map, after the
    /// `echo-types` library: `Echo f y = Σ (x : domain) (f x ≡ y)`. We track
    /// the collapsing map's `domain` and `codomain` plus the linearity
    /// `mode`. It models "loss that is not total erasure": a typed witness of
    /// what a `domain` value retained after collapsing into `codomain`.
    /// `mode` records whether the full fiber (`Linear`) or only the collapsed
    /// residue (`Affine`) is observable. See the affine bridge in
    /// `proofs/verification/{coq,idris}/solo-core/EchoResidue.*`.
    Echo {
        mode: EchoMode,
        domain: Box<Ty>,
        codomain: Box<Ty>,
    },

    /// Type variable (for inference)
    Var(usize),

    /// Error type (for error recovery)
    Error,

    /// Unknown type (not yet inferred)
    Unknown,
}

impl Ty {
    /// Check if this type is numeric (Int or Float)
    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float)
    }

    /// Check if this type is a primitive
    pub fn is_primitive(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float | Ty::String | Ty::Bool | Ty::Unit)
    }

    /// Check if this type is an error or unknown
    pub fn is_error_or_unknown(&self) -> bool {
        matches!(self, Ty::Error | Ty::Unknown)
    }

    /// Check if two types are compatible for assignment
    pub fn is_assignable_from(&self, other: &Ty) -> bool {
        if self == other {
            return true;
        }

        // Error types are compatible with everything (for error recovery)
        if self.is_error_or_unknown() || other.is_error_or_unknown() {
            return true;
        }

        // AI<T> is assignable from T
        if let Ty::AI(inner) = self {
            if inner.as_ref() == other {
                return true;
            }
        }

        match (self, other) {
            (Ty::Array(a), Ty::Array(b)) => a.is_assignable_from(b),
            (Ty::Ref { inner: a, .. }, Ty::Ref { inner: b, .. }) => a.is_assignable_from(b),
            (Ty::AI(a), Ty::AI(b)) => a.is_assignable_from(b),
            (Ty::Effect(a), Ty::Effect(b)) => a.is_assignable_from(b),
            (Ty::Tuple(a), Ty::Tuple(b)) if a.len() == b.len() => {
                a.iter().zip(b.iter()).all(|(x, y)| x.is_assignable_from(y))
            }
            (Ty::Function { params: p1, result: r1 }, Ty::Function { params: p2, result: r2 }) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2.iter()).all(|(x, y)| y.is_assignable_from(x)) // contravariant
                    && r1.is_assignable_from(r2) // covariant
            }
            (
                Ty::Echo { mode: m_t, domain: d_t, codomain: c_t },
                Ty::Echo { mode: m_s, domain: d_s, codomain: c_s },
            ) => {
                // Echo weakening (EchoLinear `weaken` / `_≤m_`): a more-
                // informative source echo may be supplied where a less-
                // informative target is expected — `Linear` weakens to
                // `Affine`, never the reverse (the weakening has no section,
                // `no-section-weaken`). Domain and codomain are invariant.
                m_s.weakens_to(*m_t) && d_t == d_s && c_t == c_s
            }
            _ => false,
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => write!(f, "Int"),
            Ty::Float => write!(f, "Float"),
            Ty::String => write!(f, "String"),
            Ty::Bool => write!(f, "Bool"),
            Ty::Unit => write!(f, "()"),
            Ty::Named(name) => write!(f, "{}", name),
            Ty::Function { params, result } => {
                if params.len() == 1 {
                    write!(f, "{} -> {}", params[0], result)
                } else {
                    write!(f, "({}) -> {}",
                        params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "),
                        result)
                }
            }
            Ty::Array(inner) => write!(f, "[{}]", inner),
            Ty::Ref { mutable, inner } => {
                if *mutable {
                    write!(f, "&mut {}", inner)
                } else {
                    write!(f, "&{}", inner)
                }
            }
            Ty::Tuple(types) => {
                write!(f, "({})", types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "))
            }
            Ty::Record(fields) => {
                write!(f, "{{ {} }}",
                    fields.iter().map(|(n, t)| format!("{}: {}", n, t)).collect::<Vec<_>>().join(", "))
            }
            Ty::AI(inner) => write!(f, "AI<{}>", inner),
            Ty::Effect(inner) => write!(f, "Effect<{}>", inner),
            Ty::Echo { mode, domain, codomain } => {
                write!(f, "{} Echo<{} => {}>", mode, domain, codomain)
            }
            Ty::Var(id) => write!(f, "?{}", id),
            Ty::Error => write!(f, "<error>"),
            Ty::Unknown => write!(f, "<unknown>"),
        }
    }
}

/// Convert AST type to internal type representation
pub fn ast_type_to_ty(ty: &crate::ast::Type) -> Ty {
    use crate::ast::{Type, PrimitiveType};

    match ty {
        Type::Primitive(p) => match p {
            PrimitiveType::Int => Ty::Int,
            PrimitiveType::Float => Ty::Float,
            PrimitiveType::String => Ty::String,
            PrimitiveType::Bool => Ty::Bool,
        },
        Type::Named(ident) => Ty::Named(ident.name.clone()),
        Type::Function { param, result, .. } => Ty::Function {
            params: vec![ast_type_to_ty(param)],
            result: Box::new(ast_type_to_ty(result)),
        },
        Type::Effect { inner, .. } => Ty::Effect(Box::new(ast_type_to_ty(inner))),
        Type::Ai { inner, .. } => Ty::AI(Box::new(ast_type_to_ty(inner))),
        Type::Reference { mutable, inner, .. } => Ty::Ref {
            mutable: *mutable,
            inner: Box::new(ast_type_to_ty(inner)),
        },
        Type::Array { element, .. } => Ty::Array(Box::new(ast_type_to_ty(element))),
        Type::Record { fields, .. } => Ty::Record(
            fields.iter().map(|f| (f.name.name.clone(), ast_type_to_ty(&f.ty))).collect()
        ),
        Type::Tuple { elements, .. } => Ty::Tuple(
            elements.iter().map(ast_type_to_ty).collect()
        ),
        Type::Constrained { base, .. } => ast_type_to_ty(base),
    }
}

#[cfg(test)]
mod echo_tests {
    use super::*;

    fn echo(mode: EchoMode) -> Ty {
        Ty::Echo {
            mode,
            domain: Box::new(Ty::Bool),
            codomain: Box::new(Ty::Unit),
        }
    }

    // EchoLinear `_≤m_`: linear ⊑ linear, linear ⊑ affine, affine ⊑ affine,
    // and crucially affine ⋢ linear.
    #[test]
    fn mode_ordering_is_thin_poset() {
        assert!(EchoMode::Linear.weakens_to(EchoMode::Linear));
        assert!(EchoMode::Linear.weakens_to(EchoMode::Affine));
        assert!(EchoMode::Affine.weakens_to(EchoMode::Affine));
        assert!(!EchoMode::Affine.weakens_to(EchoMode::Linear));
    }

    // EchoLinear `weaken`: a linear echo may be supplied where an affine
    // echo is expected (target.is_assignable_from(source)).
    #[test]
    fn linear_weakens_to_affine() {
        let affine = echo(EchoMode::Affine);
        let linear = echo(EchoMode::Linear);
        assert!(affine.is_assignable_from(&linear));
    }

    // EchoLinear `no-section-weaken`: weakening is irreversible — an affine
    // echo is NOT acceptable where a linear echo is required.
    #[test]
    fn affine_has_no_section_back_to_linear() {
        let linear = echo(EchoMode::Linear);
        let affine = echo(EchoMode::Affine);
        assert!(!linear.is_assignable_from(&affine));
    }

    // Same mode is reflexively assignable; mismatched domain/codomain is not.
    #[test]
    fn echo_invariant_in_domain_and_codomain() {
        let a = echo(EchoMode::Linear);
        assert!(a.is_assignable_from(&a));
        let other_codomain = Ty::Echo {
            mode: EchoMode::Linear,
            domain: Box::new(Ty::Bool),
            codomain: Box::new(Ty::Int),
        };
        assert!(!a.is_assignable_from(&other_codomain));
    }

    #[test]
    fn echo_display_renders_mode_and_arrow() {
        assert_eq!(echo(EchoMode::Linear).to_string(), "linear Echo<Bool => ()>");
        assert_eq!(echo(EchoMode::Affine).to_string(), "affine Echo<Bool => ()>");
    }
}
