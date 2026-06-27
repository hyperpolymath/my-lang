// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! # `session` — the binary session-config evaluator (Coq `SessionEval.cstep1`)
//!
//! Coupling **#4**: a faithful Rust port of the Coq functional session stepper
//! `cstep1`, which `SessionEval.v` proves SOUND and COMPLETE vs the reference
//! `cstep` RELATION of `SessionPi.v` (the binary fused `(νc)(P∣Q)` form, S1.1b).
//! `conformance/run.sh` differentially tests this `cstep1` against the extracted
//! Coq `cstep1` on a random corpus of configurations.
//!
//! The reference semantics is a relation (not runnable); the verified `cstep1`
//! is its executable mirror, so a green conformance run means this runtime makes
//! exactly the communication step the machine-checked semantics sanctions.

/// Payload values (Coq `val`): a de Bruijn receive-binder variable + base data.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Val {
    Var(usize),
    Unit,
    Bool(bool),
    Nat(u64),
}

/// An endpoint process (Coq `party`); `Bra` is the labelled-branch list (Coq
/// `pbranch`, an assoc list — `pget` takes the first matching label).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Party {
    End,
    Send(Val, Box<Party>),
    Recv(Box<Party>),
    Sel(usize, Box<Party>),
    Bra(Vec<(usize, Party)>),
}

/// The fused two-party configuration (Coq `config = Conf party party`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Config(pub Box<Party>, pub Box<Party>);

/// value shift (Coq `vlift`).
fn vlift(c: usize, v: &Val) -> Val {
    match v {
        Val::Var(k) => {
            if *k < c {
                Val::Var(*k)
            } else {
                Val::Var(k + 1)
            }
        }
        _ => v.clone(),
    }
}

/// value substitution (Coq `vsubst`).
fn vsubst(c: usize, u: &Val, v: &Val) -> Val {
    match v {
        Val::Var(k) => {
            if *k < c {
                Val::Var(*k)
            } else if *k == c {
                u.clone()
            } else {
                Val::Var(k - 1)
            }
        }
        _ => v.clone(),
    }
}

/// payload substitution into a party body (Coq `psubst_party`).
fn psubst_party(c: usize, u: &Val, p: &Party) -> Party {
    match p {
        Party::End => Party::End,
        Party::Send(v, q) => Party::Send(vsubst(c, u, v), Box::new(psubst_party(c, u, q))),
        Party::Recv(q) => Party::Recv(Box::new(psubst_party(c + 1, &vlift(0, u), q))),
        Party::Sel(l, q) => Party::Sel(*l, Box::new(psubst_party(c, u, q))),
        Party::Bra(bs) => {
            Party::Bra(bs.iter().map(|(l, q)| (*l, psubst_party(c, u, q))).collect())
        }
    }
}

/// receive the value `u` into the index-0 binder (Coq `open_party`).
pub fn open_party(u: &Val, p: &Party) -> Party {
    psubst_party(0, u, p)
}

/// first-match label lookup (Coq `pget`).
fn pget<'a>(l: usize, bs: &'a [(usize, Party)]) -> Option<&'a Party> {
    bs.iter().find(|(k, _)| *k == l).map(|(_, q)| q)
}

/// one synchronous communication step of the fused config (Coq `cstep1`,
/// proved sound + complete vs the `cstep` relation in SessionEval.v).
pub fn cstep1(c: &Config) -> Option<Config> {
    match (&*c.0, &*c.1) {
        (Party::Send(v, p), Party::Recv(q)) => {
            Some(Config(p.clone(), Box::new(open_party(v, q))))
        }
        (Party::Recv(q), Party::Send(v, p)) => {
            Some(Config(Box::new(open_party(v, q)), p.clone()))
        }
        (Party::Sel(l, p), Party::Bra(bs)) => {
            pget(*l, bs).map(|q| Config(p.clone(), Box::new(q.clone())))
        }
        (Party::Bra(bs), Party::Sel(l, p)) => {
            pget(*l, bs).map(|q| Config(Box::new(q.clone()), p.clone()))
        }
        _ => None,
    }
}

// ===== global-type stepper (Coq `SessionEval.gstep1`) =====

/// Payload value types (Coq `vty`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Vty {
    Unit,
    Bool,
    Nat,
}

/// Global session types (Coq `gty`); `Bra` is the labelled-branch list
/// (Coq `gbranch`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Gty {
    End,
    Msg(usize, usize, Vty, Box<Gty>),
    Bra(usize, usize, Vec<(usize, Gty)>),
    Mu(Box<Gty>),
    Var(usize),
}

/// One global-type reduction step (Coq `gstep1`; sound + progress vs the
/// `gstep` relation). Commits to the head branch of a `Bra` — adequacy is
/// soundness + progress, not functional determinism (any branch may be chosen).
pub fn gstep1(g: &Gty) -> Option<Gty> {
    match g {
        Gty::Msg(_, _, _, cont) => Some((**cont).clone()),
        Gty::Bra(_, _, bs) => bs.first().map(|(_, g)| g.clone()),
        _ => None,
    }
}

// ===== n-ary located stepper (Coq `SessionEval.nstep1`) =====

/// A located role → endpoint assignment (Coq `role_assignment`).
pub type RoleAssignment = Vec<(usize, Party)>;

/// First binding for `r` (Coq `ra_get`).
fn ra_get(ra: &[(usize, Party)], r: usize) -> Option<&Party> {
    ra.iter().find(|(r2, _)| *r2 == r).map(|(_, p)| p)
}

/// Replace the FIRST binding for `r` (Coq `ra_set` — only the first occurrence).
fn ra_set(ra: &[(usize, Party)], r: usize, p: &Party) -> RoleAssignment {
    match ra.split_first() {
        None => vec![],
        Some(((r2, q), rest)) => {
            if *r2 == r {
                let mut v = vec![(*r2, p.clone())];
                v.extend_from_slice(rest);
                v
            } else {
                let mut v = vec![(*r2, q.clone())];
                v.extend(ra_set(rest, r, p));
                v
            }
        }
    }
}

/// First `q != r` whose canonical party is `Recv`; yields `(q, open_party v Qc)`.
fn find_recv(ra: &[(usize, Party)], r: usize, v: &Val) -> Option<(usize, Party)> {
    for (q, _) in ra {
        if *q == r {
            continue;
        }
        if let Some(Party::Recv(qc)) = ra_get(ra, *q) {
            return Some((*q, open_party(v, qc)));
        }
    }
    None
}

/// First `q != r` whose canonical party is `Bra` offering label `l`; yields `(q, Q)`.
fn find_bra(ra: &[(usize, Party)], r: usize, l: usize) -> Option<(usize, Party)> {
    for (q, _) in ra {
        if *q == r {
            continue;
        }
        if let Some(Party::Bra(bs)) = ra_get(ra, *q) {
            if let Some(qq) = bs.iter().find(|(k, _)| *k == l).map(|(_, p)| p.clone()) {
                return Some((*q, qq));
            }
        }
    }
    None
}

/// Try to fire role `r` (canonical party `p`) against a partner (Coq `try_role`).
fn try_role(ra: &[(usize, Party)], r: usize, p: &Party) -> Option<RoleAssignment> {
    match p {
        Party::Send(v, cont) => {
            find_recv(ra, r, v).map(|(q, qres)| ra_set(&ra_set(ra, r, cont), q, &qres))
        }
        Party::Sel(l, cont) => {
            find_bra(ra, r, *l).map(|(q, qres)| ra_set(&ra_set(ra, r, cont), q, &qres))
        }
        _ => None,
    }
}

/// One synchronous n-ary step: the first sender/selector role (left-to-right)
/// paired with its first dual partner (Coq `nstep1`; sound + progress vs the
/// `nstep` relation).
pub fn nstep1(ra: &[(usize, Party)]) -> Option<RoleAssignment> {
    for (r, _) in ra {
        if let Some(p) = ra_get(ra, *r) {
            let p = p.clone();
            if let Some(ra2) = try_role(ra, *r, &p) {
                return Some(ra2);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b<T>(x: T) -> Box<T> {
        Box::new(x)
    }

    /// ping-pong: `Conf (send 42; end) (recv; end)` steps to `Conf end end`
    /// (the received value is substituted into the — here closed — continuation).
    #[test]
    fn comm_step() {
        let c = Config(
            b(Party::Send(Val::Nat(42), b(Party::End))),
            b(Party::Recv(b(Party::End))),
        );
        assert_eq!(
            cstep1(&c),
            Some(Config(b(Party::End), b(Party::End)))
        );
    }

    /// select/branch: `Conf (sel 1) (bra {0:end, 1:send; end})` picks label 1.
    #[test]
    fn select_step() {
        let c = Config(
            b(Party::Sel(1, b(Party::End))),
            b(Party::Bra(vec![
                (0, Party::End),
                (1, Party::Send(Val::Unit, b(Party::End))),
            ])),
        );
        assert_eq!(
            cstep1(&c),
            Some(Config(b(Party::End), b(Party::Send(Val::Unit, b(Party::End)))))
        );
    }

    /// a select on an UNOFFERED label is stuck (no step) — `pget` returns None.
    #[test]
    fn select_missing_label_stuck() {
        let c = Config(
            b(Party::Sel(7, b(Party::End))),
            b(Party::Bra(vec![(0, Party::End)])),
        );
        assert_eq!(cstep1(&c), None);
    }

    /// two ended parties are a normal form.
    #[test]
    fn ended_is_normal() {
        assert_eq!(cstep1(&Config(b(Party::End), b(Party::End))), None);
    }

    /// gstep1: a message prefix steps to its continuation; a branch to its head;
    /// `End` and an empty offer are normal.
    #[test]
    fn global_steps() {
        assert_eq!(gstep1(&Gty::Msg(0, 1, Vty::Nat, b(Gty::End))), Some(Gty::End));
        let br = Gty::Bra(0, 1, vec![(0, Gty::End), (1, Gty::Var(0))]);
        assert_eq!(gstep1(&br), Some(Gty::End)); // head branch
        assert_eq!(gstep1(&Gty::End), None);
        assert_eq!(gstep1(&Gty::Bra(0, 1, vec![])), None);
    }

    /// nstep1: among n parties, the sender meets the dual receiver and both
    /// advance; the idle third party is untouched.
    #[test]
    fn nary_comm_step() {
        let ra: RoleAssignment = vec![
            (0, Party::Send(Val::Nat(7), b(Party::End))),
            (1, Party::Recv(b(Party::End))),
            (2, Party::End),
        ];
        let stepped = nstep1(&ra).expect("should step");
        assert_eq!(ra_get(&stepped, 0), Some(&Party::End));
        assert_eq!(ra_get(&stepped, 1), Some(&Party::End));
        assert_eq!(ra_get(&stepped, 2), Some(&Party::End));
    }

    /// nstep1: two senders and no receiver → no communicating pair → stuck.
    #[test]
    fn nary_stuck() {
        let ra: RoleAssignment = vec![
            (0, Party::Send(Val::Unit, b(Party::End))),
            (1, Party::Send(Val::Unit, b(Party::End))),
        ];
        assert_eq!(nstep1(&ra), None);
    }
}
