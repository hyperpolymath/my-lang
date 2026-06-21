// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! # `my-qtt` — the QTT verified-core checker
//!
//! A **faithful Rust port** of the machine-checked Coq usage-walk synthesiser
//! `check` (rung **R5**) and the affine-budget decision `aff_type_dec` (**R5b**)
//! from `proofs/verification/coq/solo-core/SoloCore.v`. This is the first
//! concrete step of the *fundamentals ⇄ implementation coupling*: the QTT
//! resource discipline the proofs establish, executed by the compiler rather
//! than re-implemented ad hoc. (Today the conventional `crates/my-lang`
//! checker has no usage axis — see `proofs/STATUS.md` §"Implementation ⇄ spec
//! coupling".)
//!
//! [`check`] : `Tctx -> Tm -> Option<(Ty, Uvec)>` is a one-pass synthesiser
//! that returns BOTH the type and the EXACT usage vector a term realises. The
//! Coq `check_correct` theorem proves `has_type G D t a  <->  check G t =
//! Some (a, D)` (axiom-free, CI-gated). The four `reflexivity` examples in
//! `SoloCore.v` (`check_id_unit` / `check_drop_linear` / `check_dup_linear` /
//! `check_dup_omega`) are reproduced below as `#[test]`s, so this port is
//! checked against the verified algorithm's own closed computations.
//!
//! ## Representation note (de Bruijn ⇄ `Vec`)
//! The Coq context is a snoc-list `TSnoc G a` where de Bruijn index `0` is the
//! most-recently-bound (innermost) variable. Here [`Tctx`]` = Vec<Ty>` with the
//! **last** element innermost: entering a binder is `push`, index `n` reads
//! `g[g.len()-1-n]`, and a body's synthesised usage carries the binder's
//! quantity as its **last** element (the Coq `USnoc D qb` shape, recovered by
//! `pop`).
//!
//! ## Scope
//! Exactly the mechanised QTT solo core (`tm`). Wiring the full my-lang surface
//! AST onto this core (so the running compiler enforces the resource axis) is
//! the next coupling step; this crate is the verified engine it will call.

#![forbid(unsafe_code)]

pub mod surface;

/// Quantities: the three-point affine semiring `{0, 1, ω}` (Coq `Quantity.Q`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Q {
    Zero,
    One,
    Omega,
}

/// Echo linearity mode — the thin poset `linear ⊑ affine` (Coq `Mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Linear,
    Affine,
}

/// QTT core types (Coq `ty`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Unit,
    /// additive product `a & b` (shared usage, projected by `Fst`/`Snd`)
    With(Box<Ty>, Box<Ty>),
    /// multiplicative product `a ⊗ b` (split usage, eliminated by `LetPair`)
    Tensor(Box<Ty>, Box<Ty>),
    Sum(Box<Ty>, Box<Ty>),
    /// `(q x : a) -> b`
    Arr(Q, Box<Ty>, Box<Ty>),
    /// echo residue `[m] Echo<a => b>`
    Echo(Mode, Box<Ty>, Box<Ty>),
}

/// QTT core terms, de Bruijn (Coq `tm`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tm {
    Var(usize),
    UnitT,
    Lam(Q, Ty, Box<Tm>),
    App(Box<Tm>, Box<Tm>),
    With(Box<Tm>, Box<Tm>),
    Fst(Box<Tm>),
    Snd(Box<Tm>),
    Tensor(Box<Tm>, Box<Tm>),
    /// `let (x, y) = e1 in e2` — `e2` binds two vars (index 1 = `x`, 0 = `y`)
    LetPair(Box<Tm>, Box<Tm>),
    /// annotation = the OTHER summand
    Inl(Ty, Box<Tm>),
    Inr(Ty, Box<Tm>),
    /// scrutinee, left arm (binds 1), right arm (binds 1)
    Case(Box<Tm>, Box<Tm>, Box<Tm>),
    /// `let (q x) = e1 in e2`
    Let(Q, Box<Tm>, Box<Tm>),
    MkEcho(Mode, Ty, Ty, Box<Tm>),
    Weaken(Box<Tm>),
}

/// Type context; last element = innermost binder (de Bruijn `0`).
pub type Tctx = Vec<Ty>;
/// Usage vector, aligned slot-for-slot with a [`Tctx`].
pub type Uvec = Vec<Q>;

// ----- the Q semiring (Coq `Quantity.v` qadd/qmul/qle) -----

/// Additive accounting: `0` identity, `1+1 = ω`, anything with `ω` is `ω`.
pub fn qadd(a: Q, b: Q) -> Q {
    use Q::*;
    match (a, b) {
        (Zero, q) => q,
        (One, Zero) => One,
        (One, One) => Omega,
        (One, Omega) => Omega,
        (Omega, _) => Omega,
    }
}

/// Multiplicative scaling: `0` absorbing, `1` identity, `ω·ω = ω`.
pub fn qmul(a: Q, b: Q) -> Q {
    use Q::*;
    match (a, b) {
        (Zero, _) => Zero,
        (One, q) => q,
        (Omega, Zero) => Zero,
        (Omega, One) => Omega,
        (Omega, Omega) => Omega,
    }
}

/// Subquantity order: a value of quantity `a` may be used where `b` is
/// expected. The affine weakening `qle Zero One = true` is the discard.
pub fn qle(a: Q, b: Q) -> bool {
    use Q::*;
    match (a, b) {
        (Zero, _) => true,
        (_, Omega) => true,
        (One, One) => true,
        (One, Zero) => false,
        (Omega, Zero) => false,
        (Omega, One) => false,
    }
}

// ----- usage vectors (Coq uzero/onehot/uadd/uscale/ule) -----

/// All-`Zero` usage of length `n` (Coq `uzero`).
pub fn uzero(n: usize) -> Uvec {
    vec![Q::Zero; n]
}

/// `One` at the de Bruijn-`n` slot (`len-1-n`), `Zero` elsewhere (Coq `onehot`).
pub fn onehot(len: usize, n: usize) -> Uvec {
    let mut d = vec![Q::Zero; len];
    if n < len {
        d[len - 1 - n] = Q::One;
    }
    d
}

/// Pointwise add; `None` on length mismatch (Coq `uadd` is partial on shape).
pub fn uadd(d1: &[Q], d2: &[Q]) -> Option<Uvec> {
    if d1.len() != d2.len() {
        return None;
    }
    Some(d1.iter().zip(d2).map(|(&x, &y)| qadd(x, y)).collect())
}

/// Scale every slot by `q` (Coq `uscale`).
pub fn uscale(q: Q, d: &[Q]) -> Uvec {
    d.iter().map(|&x| qmul(q, x)).collect()
}

/// Pointwise order: equal length and `qle` at every slot (Coq `ule`).
pub fn ule(d: &[Q], d2: &[Q]) -> bool {
    d.len() == d2.len() && d.iter().zip(d2).all(|(&x, &y)| qle(x, y))
}

/// `n`-th type from the innermost end (Coq `tnth`).
fn tnth(g: &[Ty], n: usize) -> Option<&Ty> {
    if n < g.len() {
        Some(&g[g.len() - 1 - n])
    } else {
        None
    }
}

// ----- the usage-walk synthesiser (Coq `check`, SoloCore.v:2128) -----

/// One-pass synthesiser: returns the type and the EXACT realised usage, or
/// `None` if the term is ill-typed or mis-used (a linear binder dropped or
/// duplicated). Faithful to the Coq `check`; `check_correct` proves it agrees
/// with the declarative `has_type` judgement.
pub fn check(g: &[Ty], t: &Tm) -> Option<(Ty, Uvec)> {
    match t {
        Tm::Var(n) => tnth(g, *n).map(|a| (a.clone(), onehot(g.len(), *n))),

        Tm::UnitT => Some((Ty::Unit, uzero(g.len()))),

        Tm::Lam(q, a, body) => {
            let mut g2 = g.to_vec();
            g2.push(a.clone());
            let (b, mut d) = check(&g2, body)?;
            let qb = d.pop()?; // innermost binder's usage (the `USnoc _ qb` top)
            if qb == *q {
                Some((Ty::Arr(*q, Box::new(a.clone()), Box::new(b)), d))
            } else {
                None
            }
        }

        Tm::App(t1, t2) => {
            let (f, d1) = check(g, t1)?;
            let (a2, d2) = check(g, t2)?;
            match f {
                Ty::Arr(q, a, b) if a2 == *a => uadd(&d1, &uscale(q, &d2)).map(|d| (*b, d)),
                _ => None,
            }
        }

        Tm::With(t1, t2) => {
            let (a, d1) = check(g, t1)?;
            let (b, d2) = check(g, t2)?;
            // additive product: both components SHARE the same usage.
            if d1 == d2 {
                Some((Ty::With(Box::new(a), Box::new(b)), d1))
            } else {
                None
            }
        }

        Tm::Fst(t1) => match check(g, t1)? {
            (Ty::With(a, _), d) => Some((*a, d)),
            _ => None,
        },

        Tm::Snd(t1) => match check(g, t1)? {
            (Ty::With(_, b), d) => Some((*b, d)),
            _ => None,
        },

        Tm::Tensor(t1, t2) => {
            let (a, d1) = check(g, t1)?;
            let (b, d2) = check(g, t2)?;
            // multiplicative product: usage is SPLIT (added).
            uadd(&d1, &d2).map(|d| (Ty::Tensor(Box::new(a), Box::new(b)), d))
        }

        Tm::LetPair(t1, t2) => {
            let (ta, d1) = check(g, t1)?;
            if let Ty::Tensor(a, b) = ta {
                let mut g2 = g.to_vec();
                g2.push(*a); // x : a  (index 1 in body)
                g2.push(*b); // y : b  (index 0 in body, innermost)
                let (c, mut d) = check(&g2, t2)?;
                let q1 = d.pop()?; // innermost (y) usage — must be linear
                let q2 = d.pop()?; // next (x) usage — must be linear
                if q1 == Q::One && q2 == Q::One {
                    uadd(&d1, &d).map(|dd| (c, dd))
                } else {
                    None
                }
            } else {
                None
            }
        }

        Tm::Inl(b, t1) => check(g, t1).map(|(a, d)| (Ty::Sum(Box::new(a), Box::new(b.clone())), d)),

        Tm::Inr(a, t1) => check(g, t1).map(|(b, d)| (Ty::Sum(Box::new(a.clone()), Box::new(b)), d)),

        Tm::Case(t1, tl, tr) => {
            let (ts, d1) = check(g, t1)?;
            if let Ty::Sum(a, b) = ts {
                let mut gl = g.to_vec();
                gl.push(*a);
                let mut gr = g.to_vec();
                gr.push(*b);
                let (cl, mut dl) = check(&gl, tl)?;
                let (cr, mut dr) = check(&gr, tr)?;
                let ql = dl.pop()?; // left arm binder usage — linear
                let qr = dr.pop()?; // right arm binder usage — linear
                // both arms: bind linearly, agree on result type AND residual usage.
                if ql == Q::One && qr == Q::One && cl == cr && dl == dr {
                    uadd(&d1, &dl).map(|d| (cl, d))
                } else {
                    None
                }
            } else {
                None
            }
        }

        Tm::Let(q, t1, t2) => {
            let (a, d1) = check(g, t1)?;
            let mut g2 = g.to_vec();
            g2.push(a);
            let (b, mut d2) = check(&g2, t2)?;
            let qb = d2.pop()?;
            if qb == *q {
                uadd(&uscale(*q, &d1), &d2).map(|d| (b, d))
            } else {
                None
            }
        }

        Tm::MkEcho(m, a, b, t1) => {
            let (a2, d) = check(g, t1)?;
            if a2 == *a {
                Some((Ty::Echo(*m, Box::new(a.clone()), Box::new(b.clone())), d))
            } else {
                None
            }
        }

        Tm::Weaken(t1) => match check(g, t1)? {
            // a Linear echo weakens to an Affine one (one-way; no section).
            (Ty::Echo(Mode::Linear, a, b), d) => Some((Ty::Echo(Mode::Affine, a, b), d)),
            _ => None,
        },
    }
}

/// Affine-budget decision (Coq `aff_type_dec`, via `aff_type_iff`): the term
/// has type `expected` within usage `budget` iff the UNIQUE synthesised usage
/// fits (`ule realised budget`). The affine **discard** lives here — a
/// `One`-budget resource may be left unused (`Zero <= One`), which strict
/// linear [`check`] rejects.
pub fn aff_check(g: &[Ty], t: &Tm, expected: &Ty, budget: &[Q]) -> bool {
    match check(g, t) {
        Some((a0, d0)) => &a0 == expected && ule(&d0, budget),
        None => false,
    }
}

// ===== the call-by-value evaluator (Coq `step1`, Eval.v) =====
//
// A FAITHFUL port of the Coq functional one-step evaluator `step1`, which is
// proved SOUND + COMPLETE against the reference `step` RELATION (Eval.v:
// `step1_sound`/`step1_complete`, axiom-free). Coupling #2 (interpreter
// adequacy vs Coq `step`) is then closed by differential conformance:
// `conformance/run.sh` checks `my_qtt::step1` == extracted Coq `step1` on a
// random corpus (one step AND iterated to normal form, which cross-checks the
// substitution below against Coq's extracted `subst`).

/// de Bruijn shift (Coq `shift`): increment every free var `>= c`.
pub fn shift(c: usize, t: &Tm) -> Tm {
    match t {
        Tm::Var(k) => if *k < c { Tm::Var(*k) } else { Tm::Var(k + 1) },
        Tm::UnitT => Tm::UnitT,
        Tm::Lam(q, a, b) => Tm::Lam(*q, a.clone(), Box::new(shift(c + 1, b))),
        Tm::App(f, x) => Tm::App(Box::new(shift(c, f)), Box::new(shift(c, x))),
        Tm::With(a, b) => Tm::With(Box::new(shift(c, a)), Box::new(shift(c, b))),
        Tm::Fst(t0) => Tm::Fst(Box::new(shift(c, t0))),
        Tm::Snd(t0) => Tm::Snd(Box::new(shift(c, t0))),
        Tm::Tensor(a, b) => Tm::Tensor(Box::new(shift(c, a)), Box::new(shift(c, b))),
        Tm::LetPair(a, b) => Tm::LetPair(Box::new(shift(c, a)), Box::new(shift(c + 2, b))),
        Tm::Inl(ty, t0) => Tm::Inl(ty.clone(), Box::new(shift(c, t0))),
        Tm::Inr(ty, t0) => Tm::Inr(ty.clone(), Box::new(shift(c, t0))),
        Tm::Case(s, l, r) => Tm::Case(
            Box::new(shift(c, s)),
            Box::new(shift(c + 1, l)),
            Box::new(shift(c + 1, r)),
        ),
        Tm::Let(q, a, b) => Tm::Let(*q, Box::new(shift(c, a)), Box::new(shift(c + 1, b))),
        Tm::MkEcho(m, a, b, t0) => Tm::MkEcho(*m, a.clone(), b.clone(), Box::new(shift(c, t0))),
        Tm::Weaken(t0) => Tm::Weaken(Box::new(shift(c, t0))),
    }
}

/// Replace de Bruijn index `j` by `u`, decrementing free vars `> j` (Coq `subst_at`).
pub fn subst_at(j: usize, u: &Tm, t: &Tm) -> Tm {
    match t {
        Tm::Var(k) => {
            if *k < j {
                Tm::Var(*k)
            } else if *k == j {
                u.clone()
            } else {
                Tm::Var(k - 1)
            }
        }
        Tm::UnitT => Tm::UnitT,
        Tm::Lam(q, a, b) => Tm::Lam(*q, a.clone(), Box::new(subst_at(j + 1, &shift(0, u), b))),
        Tm::App(f, x) => Tm::App(Box::new(subst_at(j, u, f)), Box::new(subst_at(j, u, x))),
        Tm::With(a, b) => Tm::With(Box::new(subst_at(j, u, a)), Box::new(subst_at(j, u, b))),
        Tm::Fst(t0) => Tm::Fst(Box::new(subst_at(j, u, t0))),
        Tm::Snd(t0) => Tm::Snd(Box::new(subst_at(j, u, t0))),
        Tm::Tensor(a, b) => Tm::Tensor(Box::new(subst_at(j, u, a)), Box::new(subst_at(j, u, b))),
        Tm::LetPair(a, b) => Tm::LetPair(
            Box::new(subst_at(j, u, a)),
            Box::new(subst_at(j + 2, &shift(0, &shift(0, u)), b)),
        ),
        Tm::Inl(ty, t0) => Tm::Inl(ty.clone(), Box::new(subst_at(j, u, t0))),
        Tm::Inr(ty, t0) => Tm::Inr(ty.clone(), Box::new(subst_at(j, u, t0))),
        Tm::Case(s, l, r) => Tm::Case(
            Box::new(subst_at(j, u, s)),
            Box::new(subst_at(j + 1, &shift(0, u), l)),
            Box::new(subst_at(j + 1, &shift(0, u), r)),
        ),
        Tm::Let(q, a, b) => {
            Tm::Let(*q, Box::new(subst_at(j, u, a)), Box::new(subst_at(j + 1, &shift(0, u), b)))
        }
        Tm::MkEcho(m, a, b, t0) => Tm::MkEcho(*m, a.clone(), b.clone(), Box::new(subst_at(j, u, t0))),
        Tm::Weaken(t0) => Tm::Weaken(Box::new(subst_at(j, u, t0))),
    }
}

/// Substitute for the index-0 binder (Coq `subst0`).
pub fn subst0(u: &Tm, t: &Tm) -> Tm {
    subst_at(0, u, t)
}

/// Two-variable substitution for `LetPair` bodies (Coq `subst2`).
pub fn subst2(u1: &Tm, u2: &Tm, t: &Tm) -> Tm {
    subst0(u1, &subst0(&shift(0, u2), t))
}

/// Decidable value predicate (Coq `is_value`).
pub fn is_value(t: &Tm) -> bool {
    match t {
        Tm::UnitT | Tm::Lam(..) => true,
        Tm::With(a, b) | Tm::Tensor(a, b) => is_value(a) && is_value(b),
        Tm::Inl(_, a) | Tm::Inr(_, a) => is_value(a),
        Tm::MkEcho(_, _, _, a) => is_value(a),
        _ => false,
    }
}

/// One call-by-value step, or `None` if normal/stuck (Coq `step1`; proved
/// sound + complete vs the reference `step` relation in Eval.v).
pub fn step1(t: &Tm) -> Option<Tm> {
    match t {
        Tm::Var(_) | Tm::UnitT | Tm::Lam(..) => None,
        Tm::App(t1, t2) => {
            if let Some(p) = step1(t1) {
                Some(Tm::App(Box::new(p), t2.clone()))
            } else if is_value(t1) {
                if let Some(p) = step1(t2) {
                    Some(Tm::App(t1.clone(), Box::new(p)))
                } else if is_value(t2) {
                    if let Tm::Lam(_, _, body) = &**t1 {
                        Some(subst0(t2, body))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        Tm::With(t1, t2) => {
            if let Some(p) = step1(t1) {
                Some(Tm::With(Box::new(p), t2.clone()))
            } else if is_value(t1) {
                step1(t2).map(|p| Tm::With(t1.clone(), Box::new(p)))
            } else {
                None
            }
        }
        Tm::Fst(t0) => {
            if let Some(p) = step1(t0) {
                Some(Tm::Fst(Box::new(p)))
            } else if let Tm::With(v1, v2) = &**t0 {
                if is_value(v1) && is_value(v2) {
                    Some((**v1).clone())
                } else {
                    None
                }
            } else {
                None
            }
        }
        Tm::Snd(t0) => {
            if let Some(p) = step1(t0) {
                Some(Tm::Snd(Box::new(p)))
            } else if let Tm::With(v1, v2) = &**t0 {
                if is_value(v1) && is_value(v2) {
                    Some((**v2).clone())
                } else {
                    None
                }
            } else {
                None
            }
        }
        Tm::Tensor(t1, t2) => {
            if let Some(p) = step1(t1) {
                Some(Tm::Tensor(Box::new(p), t2.clone()))
            } else if is_value(t1) {
                step1(t2).map(|p| Tm::Tensor(t1.clone(), Box::new(p)))
            } else {
                None
            }
        }
        Tm::LetPair(t1, t2) => {
            if let Some(p) = step1(t1) {
                Some(Tm::LetPair(Box::new(p), t2.clone()))
            } else if let Tm::Tensor(v1, v2) = &**t1 {
                if is_value(v1) && is_value(v2) {
                    Some(subst2(v1, v2, t2))
                } else {
                    None
                }
            } else {
                None
            }
        }
        Tm::Inl(b, t0) => step1(t0).map(|p| Tm::Inl(b.clone(), Box::new(p))),
        Tm::Inr(a, t0) => step1(t0).map(|p| Tm::Inr(a.clone(), Box::new(p))),
        Tm::Case(s, l, r) => {
            if let Some(p) = step1(s) {
                Some(Tm::Case(Box::new(p), l.clone(), r.clone()))
            } else {
                match &**s {
                    Tm::Inl(_, v) if is_value(v) => Some(subst0(v, l)),
                    Tm::Inr(_, v) if is_value(v) => Some(subst0(v, r)),
                    _ => None,
                }
            }
        }
        Tm::Let(q, t1, t2) => {
            if let Some(p) = step1(t1) {
                Some(Tm::Let(*q, Box::new(p), t2.clone()))
            } else if is_value(t1) {
                Some(subst0(t1, t2))
            } else {
                None
            }
        }
        Tm::MkEcho(m, a, b, t0) => step1(t0).map(|p| Tm::MkEcho(*m, a.clone(), b.clone(), Box::new(p))),
        Tm::Weaken(t0) => {
            if let Some(p) = step1(t0) {
                Some(Tm::Weaken(Box::new(p)))
            } else if let Tm::MkEcho(Mode::Linear, a, b, v) = &**t0 {
                if is_value(v) {
                    Some(Tm::MkEcho(Mode::Affine, a.clone(), b.clone(), v.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Oracle tests: each mirrors a `reflexivity` `Example` in `SoloCore.v`, so
    //! a green run means this port computes what the machine-checked algorithm
    //! computes on the same closed terms.
    use super::*;
    use Q::*;

    fn b<T>(x: T) -> Box<T> {
        Box::new(x)
    }
    fn arr(q: Q, a: Ty, r: Ty) -> Ty {
        Ty::Arr(q, b(a), b(r))
    }
    fn lam(q: Q, a: Ty, body: Tm) -> Tm {
        Tm::Lam(q, a, b(body))
    }
    fn tensor(x: Tm, y: Tm) -> Tm {
        Tm::Tensor(b(x), b(y))
    }

    /// Coq `check_id_unit`: the `One`-binder is used exactly once -> accepted,
    /// synthesising the type AND the (empty, closed) usage.
    #[test]
    fn check_id_unit() {
        let t = lam(One, Ty::Unit, Tm::Var(0));
        assert_eq!(
            check(&[], &t),
            Some((arr(One, Ty::Unit, Ty::Unit), vec![]))
        );
    }

    /// Coq `check_drop_linear`: a `One`-binder left UNUSED -> rejected.
    #[test]
    fn check_drop_linear() {
        let t = lam(One, Ty::Unit, Tm::UnitT);
        assert_eq!(check(&[], &t), None);
    }

    /// Coq `check_dup_linear`: a `One`-binder used TWICE (usage `ω`) -> rejected.
    #[test]
    fn check_dup_linear() {
        let t = lam(One, Ty::Unit, tensor(Tm::Var(0), Tm::Var(0)));
        assert_eq!(check(&[], &t), None);
    }

    /// Coq `check_dup_omega`: the SAME body under an `ω`-binder -> accepted, so
    /// the rejection above is exactly the linearity check.
    #[test]
    fn check_dup_omega() {
        let t = lam(Omega, Ty::Unit, tensor(Tm::Var(0), Tm::Var(0)));
        assert_eq!(
            check(&[], &t),
            Some((arr(Omega, Ty::Unit, Ty::Tensor(b(Ty::Unit), b(Ty::Unit))), vec![]))
        );
    }

    /// Coq `aff_discard_ok` (R5b): with one `Unit` in scope at budget `One`,
    /// the affine judgement ACCEPTS dropping it (realises `Zero <= One`); the
    /// strict linear walk realises usage `[Zero]`, not `[One]`.
    #[test]
    fn aff_discard_ok() {
        let g = vec![Ty::Unit];
        assert_eq!(check(&g, &Tm::UnitT), Some((Ty::Unit, vec![Zero])));
        assert!(aff_check(&g, &Tm::UnitT, &Ty::Unit, &[One]));
    }

    /// The echo weakening rung: `Weaken (echo_L a b v) : echo_A a b`.
    #[test]
    fn weaken_linear_to_affine() {
        let inner = Tm::MkEcho(Mode::Linear, Ty::Unit, Ty::Unit, b(Tm::UnitT));
        let t = Tm::Weaken(b(inner));
        assert_eq!(
            check(&[], &t),
            Some((Ty::Echo(Mode::Affine, b(Ty::Unit), b(Ty::Unit)), vec![]))
        );
        // and the reverse weakening (affine -> ... ) has no source: an Affine
        // echo is not a `Weaken` redex, so `check` rejects weakening it again.
        let aff = Tm::MkEcho(Mode::Affine, Ty::Unit, Ty::Unit, b(Tm::UnitT));
        assert_eq!(check(&[], &Tm::Weaken(b(aff))), None);
    }

    /// Semiring spot-checks against the Coq tables.
    #[test]
    fn semiring_tables() {
        assert_eq!(qadd(One, One), Omega);
        assert_eq!(qmul(Omega, Zero), Zero);
        assert!(qle(Zero, One));
        assert!(!qle(One, Zero));
        assert!(qle(Omega, Omega));
    }

    /// Evaluator (Coq `step1`) spot-checks. The conformance harness
    /// (`conformance/run.sh`) is the systematic check vs the extracted verified
    /// `step1`; these pin the headline redexes for Coq-free CI.
    #[test]
    fn step_redexes() {
        // beta: (\x:Unit. x) star --> star
        let beta = Tm::App(b(lam(One, Ty::Unit, Tm::Var(0))), b(Tm::UnitT));
        assert_eq!(step1(&beta), Some(Tm::UnitT));
        // additive projection: fst <star, star> --> star
        let fst = Tm::Fst(b(Tm::With(b(Tm::UnitT), b(Tm::UnitT))));
        assert_eq!(step1(&fst), Some(Tm::UnitT));
        // echo weaken: weaken (echo_L star) --> echo_A star
        let wk = Tm::Weaken(b(Tm::MkEcho(Mode::Linear, Ty::Unit, Ty::Unit, b(Tm::UnitT))));
        assert_eq!(
            step1(&wk),
            Some(Tm::MkEcho(Mode::Affine, Ty::Unit, Ty::Unit, b(Tm::UnitT)))
        );
        // let-pair (exercises subst2): let (x,y) = (star,star) in x --> star
        let lp = Tm::LetPair(b(Tm::Tensor(b(Tm::UnitT), b(Tm::UnitT))), b(Tm::Var(1)));
        assert_eq!(step1(&lp), Some(Tm::UnitT));
        // values are normal
        assert_eq!(step1(&Tm::UnitT), None);
        assert_eq!(step1(&lam(One, Ty::Unit, Tm::UnitT)), None);
    }
}
