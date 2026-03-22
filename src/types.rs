//! Type system definitions for My Language
//!
//! Defines the internal representation of types used during type checking.

use std::fmt;

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
