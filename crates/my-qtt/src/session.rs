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
}
