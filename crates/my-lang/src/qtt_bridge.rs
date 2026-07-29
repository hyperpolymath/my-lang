// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! # `qtt_bridge` — lower the my-lang surface AST onto the verified QTT core
//!
//! Coupling **step 2b**: the actual lowering of the running compiler's
//! [`crate::ast::Expr`]/[`crate::ast::Type`] onto [`my_qtt::surface::SExpr`], so
//! that the machine-checked QTT usage-walk (`my-qtt`, the faithful port of the
//! Coq `check`) can be run on real my-lang programs. With `my-qtt` now in
//! my-lang's dependency graph, the proofs' own algorithm is *in the compiler*.
//!
//! ## Scope (honest)
//! Only the **resource-relevant fragment** lowers: variables, λ, application,
//! `let`/block bindings, and literals (as opaque values). Everything else
//! (binary/unary ops, `match`, records, AI expressions, `try`, `restrict`, …)
//! returns [`BridgeError::Unsupported`] — these are follow-on work, not silent
//! passes. Base/opaque types (`Int`/`String`/…/records/effects) carry no
//! resource content and map to `Unit`; function and tuple types map
//! structurally (`->` to the affine arrow, tuples to `⊗`).
//!
//! ## The quantity default
//! The Coq `check` is **exact-usage**: a binder's declared quantity must equal
//! its realised usage. The my-lang `Param` has no quantity syntax yet, so this
//! bridge applies the **linear** reading ([`DEFAULT_Q`]` = One`): every binder
//! must be used *exactly once*. That is the strictest QTT discipline — it makes
//! [`check_expr`] genuinely *enforce* linearity on the real AST (a dropped or
//! duplicated binder is rejected by the verified walk), and it is the most
//! faithful demonstration of the coupling. Real per-binder quantities await
//! `Param` gaining quantity annotations (a small parser+AST follow-on); the
//! engine already threads whatever quantity it is given, so that change
//! activates here with no rewrite.

use crate::ast::{Block, Expr, LambdaBody, Stmt, Type};
use my_qtt::surface::{check_surface, CheckError, SExpr, STy};
use my_qtt::{Q, Ty, Uvec};

/// The quantity assigned to every surface binder pending quantity syntax — the
/// linear reading (`One` = used exactly once). See the module docs.
pub const DEFAULT_Q: Q = Q::One;

/// A surface construct outside the resource-relevant fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    Unsupported(&'static str),
}

/// Lower a surface type to a core resource type. Opaque base types map to
/// `Unit` (no resource content); `->` and tuples map structurally.
pub fn lower_ty(t: &Type) -> STy {
    match t {
        Type::Function { param, result, .. } => {
            STy::Arr(DEFAULT_Q, Box::new(lower_ty(param)), Box::new(lower_ty(result)))
        }
        Type::Tuple { elements, .. } => lower_tuple(elements),
        Type::Reference { inner, .. } => lower_ty(inner),
        // Primitive / Named / Effect / Ai / Array / Record / Constrained: opaque.
        _ => STy::Unit,
    }
}

fn lower_tuple(elems: &[Type]) -> STy {
    match elems {
        [] => STy::Unit,
        [t] => lower_ty(t),
        [t, rest @ ..] => STy::Tensor(Box::new(lower_ty(t)), Box::new(lower_tuple(rest))),
    }
}

/// Lower a surface expression to the named-variable core fragment.
pub fn lower_expr(e: &Expr) -> Result<SExpr, BridgeError> {
    Ok(match e {
        Expr::Ident(id) => SExpr::Var(id.name.clone()),
        // a literal is a value with no resource content.
        Expr::Literal(_) => SExpr::Unit,
        Expr::Lambda { params, body, .. } => {
            let inner = match body {
                LambdaBody::Expr(e) => lower_expr(e)?,
                LambdaBody::Block(b) => lower_block(b)?,
            };
            // right-fold params: `|x, y| body`  ->  `Lam x (Lam y body)`
            let mut acc = inner;
            for p in params.iter().rev() {
                acc = SExpr::Lam {
                    q: DEFAULT_Q,
                    param: p.name.name.clone(),
                    ty: lower_ty(&p.ty),
                    body: Box::new(acc),
                };
            }
            acc
        }
        Expr::Call { callee, args, .. } => {
            // `f(a, b)`  ->  `App (App f a) b`
            let mut acc = lower_expr(callee)?;
            for a in args {
                acc = SExpr::App(Box::new(acc), Box::new(lower_expr(a)?));
            }
            acc
        }
        Expr::Block(b) => lower_block(b)?,
        Expr::Binary { .. } => return Err(BridgeError::Unsupported("binary op")),
        Expr::Unary { .. } => return Err(BridgeError::Unsupported("unary op")),
        Expr::Field { .. } => return Err(BridgeError::Unsupported("field access")),
        Expr::Try { .. } => return Err(BridgeError::Unsupported("try")),
        Expr::Restrict { .. } => return Err(BridgeError::Unsupported("restrict")),
        Expr::Ai(_) => return Err(BridgeError::Unsupported("AI expression")),
        Expr::Match { .. } => return Err(BridgeError::Unsupported("match (use a 2-arm sum case)")),
        Expr::Array { .. } => return Err(BridgeError::Unsupported("array literal")),
        Expr::Record { .. } => return Err(BridgeError::Unsupported("record literal")),
    })
}

/// Lower a block: `let` statements become nested core `let`s; a trailing
/// expression statement is the block's value (an empty block is `unit`). Other
/// statement forms (control flow, non-final non-`let` statements) are out of
/// the resource fragment.
pub fn lower_block(b: &Block) -> Result<SExpr, BridgeError> {
    lower_stmts(&b.stmts)
}

fn lower_stmts(stmts: &[Stmt]) -> Result<SExpr, BridgeError> {
    match stmts {
        [] => Ok(SExpr::Unit),
        [Stmt::Expr(e)] => lower_expr(e),
        [Stmt::Let { name, value, .. }, rest @ ..] => Ok(SExpr::Let {
            q: DEFAULT_Q,
            name: name.name.clone(),
            value: Box::new(lower_expr(value)?),
            body: Box::new(lower_stmts(rest)?),
        }),
        _ => Err(BridgeError::Unsupported("block statement form")),
    }
}

/// Lowering or checking failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeCheckError {
    /// The surface construct is outside the resource fragment.
    Bridge(BridgeError),
    /// Lowered, but the verified usage-walk rejected it (ill-typed, or — under
    /// the linear default — a binder dropped or duplicated).
    Check(CheckError),
}

/// Lower a surface expression and run the **machine-checked** QTT usage-walk on
/// it. The running compiler's hook into the proofs' own resource discipline.
pub fn check_expr(e: &Expr) -> Result<(Ty, Uvec), BridgeCheckError> {
    let s = lower_expr(e).map_err(BridgeCheckError::Bridge)?;
    check_surface(&s).map_err(BridgeCheckError::Check)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Ident, Literal, Param, PrimitiveType};
    use crate::token::Span;
    use my_qtt::surface::ElabError;

    fn sp() -> Span {
        Span { start: 0, end: 0, line: 0, column: 0 }
    }
    fn id(n: &str) -> Ident {
        Ident::new(n, sp())
    }
    fn param(n: &str, ty: Type) -> Param {
        Param { name: id(n), ty, span: sp() }
    }

    /// A real `ast::Expr::Lambda` `|x: Int| x` lowers and is accepted: the
    /// linear binder is used exactly once. The verified engine now runs on the
    /// compiler's actual AST.
    #[test]
    fn real_linear_lambda_accepted() {
        let lam = Expr::Lambda {
            params: vec![param("x", Type::Primitive(PrimitiveType::Int))],
            body: LambdaBody::Expr(Box::new(Expr::Ident(id("x")))),
            span: sp(),
        };
        let r = check_expr(&lam);
        assert!(r.is_ok(), "expected ok, got {:?}", r);
        // Int is opaque (Unit); the arrow carries the linear quantity.
        assert!(matches!(r.unwrap().0, Ty::Arr(Q::One, _, _)));
    }

    /// `|x: Int| { 0 }` drops the linear binder `x` → rejected by the verified
    /// walk. Linearity is enforced on the real AST.
    #[test]
    fn real_dropping_lambda_rejected() {
        let lam = Expr::Lambda {
            params: vec![param("x", Type::Primitive(PrimitiveType::Int))],
            body: LambdaBody::Block(Block {
                stmts: vec![Stmt::Expr(Expr::Literal(Literal::Int(0, sp())))],
                span: sp(),
            }),
            span: sp(),
        };
        assert_eq!(
            check_expr(&lam),
            Err(BridgeCheckError::Check(CheckError::Untypable))
        );
    }

    /// A block `{ let z = 0; z }` lowers to a core `let` and checks (z used once).
    #[test]
    fn real_block_let_accepted() {
        let blk = Expr::Block(Block {
            stmts: vec![
                Stmt::Let {
                    mutable: false,
                    name: id("z"),
                    ty: None,
                    value: Expr::Literal(Literal::Int(0, sp())),
                    span: sp(),
                },
                Stmt::Expr(Expr::Ident(id("z"))),
            ],
            span: sp(),
        });
        assert!(check_expr(&blk).is_ok());
    }

    /// A free identifier is an elaboration error, surfaced through the bridge.
    #[test]
    fn real_unbound_ident() {
        assert_eq!(
            check_expr(&Expr::Ident(id("free"))),
            Err(BridgeCheckError::Check(CheckError::Elab(ElabError::Unbound(
                "free".into()
            ))))
        );
    }

    /// Out-of-fragment constructs are reported, not silently accepted.
    #[test]
    fn unsupported_is_reported() {
        let field = Expr::Field {
            object: Box::new(Expr::Ident(id("r"))),
            field: id("f"),
            span: sp(),
        };
        assert_eq!(
            check_expr(&field),
            Err(BridgeCheckError::Bridge(BridgeError::Unsupported("field access")))
        );
    }

    /// Function and tuple types lower structurally; base types are opaque `Unit`.
    #[test]
    fn type_lowering() {
        assert_eq!(lower_ty(&Type::Primitive(PrimitiveType::Int)), STy::Unit);
        let f = Type::Function {
            param: Box::new(Type::Primitive(PrimitiveType::Int)),
            result: Box::new(Type::Primitive(PrimitiveType::Bool)),
            span: sp(),
        };
        assert_eq!(
            lower_ty(&f),
            STy::Arr(Q::One, Box::new(STy::Unit), Box::new(STy::Unit))
        );
    }
}
