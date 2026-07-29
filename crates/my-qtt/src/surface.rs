// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! # `surface` — named-variable elaboration onto the QTT core
//!
//! Coupling **step 2**: the bridge from a *named* surface to the de Bruijn
//! [`crate::Tm`] that the verified [`crate::check`] consumes, so the running
//! compiler can enforce the QTT resource axis on real programs (today its
//! `checker.rs` has none — `proofs/STATUS.md`).
//!
//! [`SExpr`]/[`STy`] are the **resource-relevant fragment** of the my-lang
//! surface `ast::Expr`/`ast::Type` (variables, λ with a quantity, application,
//! the two products, sums, `let`, the echo residue) — the schema the my-lang
//! frontend lowers onto (see the `ast::Expr → SExpr` lowering, the thin
//! mechanical follow-on). The non-resource surface (AI models, prompts,
//! records, …) carries no usage content and is out of this fragment.
//!
//! Elaboration is a scope-resolution pass: names become de Bruijn indices
//! against a [`Scope`] (innermost binder last, matching [`crate::Tctx`]). Types
//! have no binders, so [`elaborate_ty`] is purely structural. After elaboration
//! [`check_surface`] runs the verified usage-walk — so a linear binder dropped
//! at the *surface* is rejected by the *machine-checked* algorithm.
//!
//! Quantities are explicit on [`SExpr`] binders. Because the current my-lang
//! `Param` has no quantity syntax, the `ast` lowering will default binders
//! (documented at the lowering site); this engine already threads whatever
//! quantity the surface supplies, so adding the syntax later needs no change
//! here.

use crate::{check, Mode, Q, Ty, Tm, Uvec};

/// Resource-relevant surface types (named-free; mirror of the QTT core types).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum STy {
    Unit,
    With(Box<STy>, Box<STy>),
    Tensor(Box<STy>, Box<STy>),
    Sum(Box<STy>, Box<STy>),
    /// `(q x : a) -> b`
    Arr(Q, Box<STy>, Box<STy>),
    Echo(Mode, Box<STy>, Box<STy>),
}

/// Resource-relevant surface terms with **named** binders.
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Var(String),
    Unit,
    /// `|q x : ty| body`
    Lam {
        q: Q,
        param: String,
        ty: STy,
        body: Box<SExpr>,
    },
    App(Box<SExpr>, Box<SExpr>),
    /// additive pair `<e1, e2>` (shared usage)
    With(Box<SExpr>, Box<SExpr>),
    Fst(Box<SExpr>),
    Snd(Box<SExpr>),
    /// multiplicative pair `(e1, e2)` (split usage)
    Tensor(Box<SExpr>, Box<SExpr>),
    /// `let (x, y) = pair in body`
    LetPair {
        x: String,
        y: String,
        pair: Box<SExpr>,
        body: Box<SExpr>,
    },
    /// `inl[other] e` — `other` is the RIGHT summand annotation
    Inl(STy, Box<SExpr>),
    /// `inr[other] e` — `other` is the LEFT summand annotation
    Inr(STy, Box<SExpr>),
    /// `case scrut of inl xl => arm_l | inr xr => arm_r`
    Case {
        scrut: Box<SExpr>,
        xl: String,
        arm_l: Box<SExpr>,
        xr: String,
        arm_r: Box<SExpr>,
    },
    /// `let (q name) = value in body`
    Let {
        q: Q,
        name: String,
        value: Box<SExpr>,
        body: Box<SExpr>,
    },
    /// echo residue `[mode] echo<a => b>(witness)`
    Echo {
        mode: Mode,
        a: STy,
        b: STy,
        witness: Box<SExpr>,
    },
    Weaken(Box<SExpr>),
}

/// A lexical scope: binder names, **innermost last** (so the de Bruijn index of
/// a name is its distance from the end). Aligns with [`crate::Tctx`].
pub type Scope = Vec<String>;

/// Elaboration failures (the surface fragment is otherwise total).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElabError {
    /// A name with no binder in scope.
    Unbound(String),
}

/// Translate a surface type to the core (purely structural — no binders).
pub fn elaborate_ty(t: &STy) -> Ty {
    match t {
        STy::Unit => Ty::Unit,
        STy::With(a, b) => Ty::With(Box::new(elaborate_ty(a)), Box::new(elaborate_ty(b))),
        STy::Tensor(a, b) => Ty::Tensor(Box::new(elaborate_ty(a)), Box::new(elaborate_ty(b))),
        STy::Sum(a, b) => Ty::Sum(Box::new(elaborate_ty(a)), Box::new(elaborate_ty(b))),
        STy::Arr(q, a, b) => Ty::Arr(*q, Box::new(elaborate_ty(a)), Box::new(elaborate_ty(b))),
        STy::Echo(m, a, b) => Ty::Echo(*m, Box::new(elaborate_ty(a)), Box::new(elaborate_ty(b))),
    }
}

fn lookup(scope: &Scope, name: &str) -> Option<usize> {
    // de Bruijn index = distance from the innermost (last) binder.
    scope.iter().rev().position(|n| n == name)
}

/// Resolve names to de Bruijn indices, producing a core [`Tm`]. `scope` holds
/// the enclosing binders (innermost last); it is restored on return.
pub fn elaborate(e: &SExpr, scope: &mut Scope) -> Result<Tm, ElabError> {
    Ok(match e {
        SExpr::Var(name) => {
            Tm::Var(lookup(scope, name).ok_or_else(|| ElabError::Unbound(name.clone()))?)
        }
        SExpr::Unit => Tm::UnitT,
        SExpr::Lam { q, param, ty, body } => {
            scope.push(param.clone());
            let body = elaborate(body, scope);
            scope.pop();
            Tm::Lam(*q, elaborate_ty(ty), Box::new(body?))
        }
        SExpr::App(f, a) => Tm::App(
            Box::new(elaborate(f, scope)?),
            Box::new(elaborate(a, scope)?),
        ),
        SExpr::With(x, y) => Tm::With(
            Box::new(elaborate(x, scope)?),
            Box::new(elaborate(y, scope)?),
        ),
        SExpr::Fst(p) => Tm::Fst(Box::new(elaborate(p, scope)?)),
        SExpr::Snd(p) => Tm::Snd(Box::new(elaborate(p, scope)?)),
        SExpr::Tensor(x, y) => Tm::Tensor(
            Box::new(elaborate(x, scope)?),
            Box::new(elaborate(y, scope)?),
        ),
        SExpr::LetPair { x, y, pair, body } => {
            let pair = elaborate(pair, scope)?;
            scope.push(x.clone()); // index 1 in body
            scope.push(y.clone()); // index 0 in body (innermost)
            let body = elaborate(body, scope);
            scope.pop();
            scope.pop();
            Tm::LetPair(Box::new(pair), Box::new(body?))
        }
        SExpr::Inl(other, e) => Tm::Inl(elaborate_ty(other), Box::new(elaborate(e, scope)?)),
        SExpr::Inr(other, e) => Tm::Inr(elaborate_ty(other), Box::new(elaborate(e, scope)?)),
        SExpr::Case { scrut, xl, arm_l, xr, arm_r } => {
            let scrut = elaborate(scrut, scope)?;
            scope.push(xl.clone());
            let arm_l = elaborate(arm_l, scope);
            scope.pop();
            scope.push(xr.clone());
            let arm_r = elaborate(arm_r, scope);
            scope.pop();
            Tm::Case(Box::new(scrut), Box::new(arm_l?), Box::new(arm_r?))
        }
        SExpr::Let { q, name, value, body } => {
            let value = elaborate(value, scope)?;
            scope.push(name.clone());
            let body = elaborate(body, scope);
            scope.pop();
            Tm::Let(*q, Box::new(value), Box::new(body?))
        }
        SExpr::Echo { mode, a, b, witness } => Tm::MkEcho(
            *mode,
            elaborate_ty(a),
            elaborate_ty(b),
            Box::new(elaborate(witness, scope)?),
        ),
        SExpr::Weaken(e) => Tm::Weaken(Box::new(elaborate(e, scope)?)),
    })
}

/// Outcome of resource-checking a closed surface term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    /// Elaboration failed (e.g. an unbound name).
    Elab(ElabError),
    /// Elaborated, but the verified usage-walk rejected it (ill-typed or a
    /// linear binder dropped/duplicated).
    Untypable,
}

/// Elaborate a **closed** surface term and run the verified usage-walk. On
/// success returns its type and exact realised usage (empty for a closed term).
/// This is the running compiler's hook into the machine-checked QTT discipline.
pub fn check_surface(e: &SExpr) -> Result<(Ty, Uvec), CheckError> {
    let mut scope = Scope::new();
    let tm = elaborate(e, &mut scope).map_err(CheckError::Elab)?;
    check(&[], &tm).ok_or(CheckError::Untypable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Q::*;

    fn b<T>(x: T) -> Box<T> {
        Box::new(x)
    }
    fn lam(q: Q, p: &str, ty: STy, body: SExpr) -> SExpr {
        SExpr::Lam { q, param: p.into(), ty, body: b(body) }
    }
    fn var(n: &str) -> SExpr {
        SExpr::Var(n.into())
    }

    /// `|1 x : Unit| x` — the linear identity elaborates and checks; the usage
    /// axis is now enforced on a NAMED surface term by the verified walk.
    #[test]
    fn linear_identity_ok() {
        let id = lam(One, "x", STy::Unit, var("x"));
        assert_eq!(
            check_surface(&id),
            Ok((Ty::Arr(One, b(Ty::Unit), b(Ty::Unit)), vec![]))
        );
    }

    /// `|1 x : Unit| unit` — a linear binder dropped at the surface is REJECTED
    /// (the coupling's whole point: the surface inherits the proof's discipline).
    #[test]
    fn linear_drop_rejected() {
        let drop = lam(One, "x", STy::Unit, SExpr::Unit);
        assert_eq!(check_surface(&drop), Err(CheckError::Untypable));
    }

    /// `|ω x : Unit| (x, x)` — the same body under an `ω` binder is accepted, so
    /// the rejection above is exactly the linearity check, not a parse error.
    #[test]
    fn omega_dup_ok() {
        let dup = lam(Omega, "x", STy::Unit, SExpr::Tensor(b(var("x")), b(var("x"))));
        assert!(matches!(check_surface(&dup), Ok((Ty::Arr(Omega, _, _), _))));
    }

    /// Nested binders resolve to the right de Bruijn indices. `|1 x| |1 y| (x, y)`
    /// uses BOTH exactly once → accepted; `|1 x| |1 y| y` drops the outer `x` →
    /// linear-rejected. (Each non-trivial term uses each linear binder exactly
    /// once, so resolution and the usage walk are both exercised.)
    #[test]
    fn scope_resolution() {
        let both = lam(
            One,
            "x",
            STy::Unit,
            lam(One, "y", STy::Unit, SExpr::Tensor(b(var("x")), b(var("y")))),
        );
        assert!(check_surface(&both).is_ok());
        let drop_outer = lam(One, "x", STy::Unit, lam(One, "y", STy::Unit, var("y")));
        assert_eq!(check_surface(&drop_outer), Err(CheckError::Untypable));
    }

    /// `let (1 z) = unit in z` — a linear `let` used once is accepted.
    #[test]
    fn linear_let_ok() {
        let e = SExpr::Let {
            q: One,
            name: "z".into(),
            value: b(SExpr::Unit),
            body: b(var("z")),
        };
        assert_eq!(check_surface(&e), Ok((Ty::Unit, vec![])));
    }

    /// An unbound name is an elaboration error, distinct from ill-typed.
    #[test]
    fn unbound_is_elab_error() {
        assert_eq!(
            check_surface(&var("nope")),
            Err(CheckError::Elab(ElabError::Unbound("nope".into())))
        );
    }

    // ----- echo modality wired into the surface + the verified checker (#3) -----

    fn echo(mode: Mode, witness: SExpr) -> SExpr {
        SExpr::Echo { mode, a: STy::Unit, b: STy::Unit, witness: b(witness) }
    }

    /// `[linear] echo<Unit => Unit>(unit)` elaborates and the verified checker
    /// assigns the echo residue type — echo flows surface → machine-checked walk.
    #[test]
    fn echo_intro_ok() {
        assert_eq!(
            check_surface(&echo(Mode::Linear, SExpr::Unit)),
            Ok((Ty::Echo(Mode::Linear, b(Ty::Unit), b(Ty::Unit)), vec![]))
        );
    }

    /// The echo discipline (`EchoLinear`): a LINEAR echo `weaken`s to an AFFINE
    /// one — checked by the verified walk on the surface term.
    #[test]
    fn echo_weaken_linear_to_affine() {
        let w = SExpr::Weaken(b(echo(Mode::Linear, SExpr::Unit)));
        assert_eq!(
            check_surface(&w),
            Ok((Ty::Echo(Mode::Affine, b(Ty::Unit), b(Ty::Unit)), vec![]))
        );
    }

    /// ...and weakening an AFFINE echo is REJECTED — the one-way `linear ⊑ affine`
    /// (the *no-section* fact) is enforced from the surface by the proof.
    #[test]
    fn echo_weaken_affine_rejected() {
        let w = SExpr::Weaken(b(echo(Mode::Affine, SExpr::Unit)));
        assert_eq!(check_surface(&w), Err(CheckError::Untypable));
    }

    /// Echo threads its witness's usage: `|1 x:Unit| [linear] echo<…>(unit)`
    /// drops the linear `x` inside the echo → rejected. So the echo residue does
    /// not launder away linearity.
    #[test]
    fn echo_witness_linearity_enforced() {
        let dropx = lam(One, "x", STy::Unit, echo(Mode::Linear, SExpr::Unit));
        assert_eq!(check_surface(&dropx), Err(CheckError::Untypable));
    }
}
