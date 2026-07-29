// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Differential-conformance GENERATOR for the QTT checker coupling.
//
// Emits a corpus of random `(ctx, tm)` queries plus the result of the Rust
// `my_qtt::check` on each, in a canonical text form byte-identical to the one
// the OCaml ORACLE (extracted from the verified Coq `check`) prints. The
// harness `conformance/run.sh` then asserts the two streams are equal, i.e.
//     ∀ (g,t) in corpus.  my_qtt::check g t  ==  (extracted Coq check) g t
// which is the refinement evidence for "the Rust checker refines Coq `check`".
//
// Usage:  conformance_gen <count> <seed> <corpus_out> <rust_results_out>
//
// The generator is deterministic in `seed` (a tiny SplitMix64), depends only on
// std, and deliberately produces a MIX of well-typed and ill-typed terms (out-
// of-range vars, quantity/type mismatches) so both the `ok` and `none` branches
// of `check` are exercised on both sides.

use my_qtt::session::{cstep1, gstep1, nstep1, Config, Gty, Party, RoleAssignment, Val, Vty};
use my_qtt::{check, step1, Mode, Q, Tm, Ty};
use std::fs::File;
use std::io::{BufWriter, Write};

/// normal-form fuel cap — MUST equal the oracle's `eval_nf` cap so a divergent
/// term yields the same capped term on both sides.
const NF_FUEL: u32 = 64;

// ---------- deterministic PRNG (SplitMix64) ----------
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn upto(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

// ---------- random AST ----------
fn gen_q(r: &mut Rng) -> Q {
    match r.upto(3) {
        0 => Q::Zero,
        1 => Q::One,
        _ => Q::Omega,
    }
}
fn gen_m(r: &mut Rng) -> Mode {
    if r.upto(2) == 0 { Mode::Linear } else { Mode::Affine }
}

fn gen_ty(r: &mut Rng, fuel: u32) -> Ty {
    if fuel == 0 {
        return Ty::Unit;
    }
    match r.upto(6) {
        0 => Ty::Unit,
        1 => Ty::With(Box::new(gen_ty(r, fuel - 1)), Box::new(gen_ty(r, fuel - 1))),
        2 => Ty::Tensor(Box::new(gen_ty(r, fuel - 1)), Box::new(gen_ty(r, fuel - 1))),
        3 => Ty::Sum(Box::new(gen_ty(r, fuel - 1)), Box::new(gen_ty(r, fuel - 1))),
        4 => Ty::Arr(gen_q(r), Box::new(gen_ty(r, fuel - 1)), Box::new(gen_ty(r, fuel - 1))),
        _ => Ty::Echo(gen_m(r), Box::new(gen_ty(r, fuel - 1)), Box::new(gen_ty(r, fuel - 1))),
    }
}

// `scope` = number of binders in scope; Var picks 0..scope+1 so some indices
// fall out of range (→ `none`), exercising the ill-typed branch.
fn gen_tm(r: &mut Rng, fuel: u32, scope: usize) -> Tm {
    if fuel == 0 {
        return if r.upto(2) == 0 {
            Tm::Var((r.upto((scope + 1) as u64)) as usize)
        } else {
            Tm::UnitT
        };
    }
    match r.upto(15) {
        0 => Tm::Var((r.upto((scope + 2) as u64)) as usize),
        1 => Tm::UnitT,
        2 => Tm::Lam(gen_q(r), gen_ty(r, fuel - 1), Box::new(gen_tm(r, fuel - 1, scope + 1))),
        3 => Tm::App(Box::new(gen_tm(r, fuel - 1, scope)), Box::new(gen_tm(r, fuel - 1, scope))),
        4 => Tm::With(Box::new(gen_tm(r, fuel - 1, scope)), Box::new(gen_tm(r, fuel - 1, scope))),
        5 => Tm::Fst(Box::new(gen_tm(r, fuel - 1, scope))),
        6 => Tm::Snd(Box::new(gen_tm(r, fuel - 1, scope))),
        7 => Tm::Tensor(Box::new(gen_tm(r, fuel - 1, scope)), Box::new(gen_tm(r, fuel - 1, scope))),
        8 => Tm::LetPair(Box::new(gen_tm(r, fuel - 1, scope)), Box::new(gen_tm(r, fuel - 1, scope + 2))),
        9 => Tm::Inl(gen_ty(r, fuel - 1), Box::new(gen_tm(r, fuel - 1, scope))),
        10 => Tm::Inr(gen_ty(r, fuel - 1), Box::new(gen_tm(r, fuel - 1, scope))),
        11 => Tm::Case(
            Box::new(gen_tm(r, fuel - 1, scope)),
            Box::new(gen_tm(r, fuel - 1, scope + 1)),
            Box::new(gen_tm(r, fuel - 1, scope + 1)),
        ),
        12 => Tm::Let(gen_q(r), Box::new(gen_tm(r, fuel - 1, scope)), Box::new(gen_tm(r, fuel - 1, scope + 1))),
        13 => Tm::MkEcho(gen_m(r), gen_ty(r, fuel - 1), gen_ty(r, fuel - 1), Box::new(gen_tm(r, fuel - 1, scope))),
        _ => Tm::Weaken(Box::new(gen_tm(r, fuel - 1, scope))),
    }
}

// ---------- canonical printers (MUST match conformance/oracle.ml) ----------
fn show_q(q: Q) -> &'static str {
    match q {
        Q::Zero => "0",
        Q::One => "1",
        Q::Omega => "w",
    }
}
fn show_m(m: Mode) -> &'static str {
    match m {
        Mode::Linear => "lin",
        Mode::Affine => "aff",
    }
}
fn show_ty(t: &Ty) -> String {
    match t {
        Ty::Unit => "unit".into(),
        Ty::With(a, b) => format!("(with {} {})", show_ty(a), show_ty(b)),
        Ty::Tensor(a, b) => format!("(tensor {} {})", show_ty(a), show_ty(b)),
        Ty::Sum(a, b) => format!("(sum {} {})", show_ty(a), show_ty(b)),
        Ty::Arr(q, a, b) => format!("(arr {} {} {})", show_q(*q), show_ty(a), show_ty(b)),
        Ty::Echo(m, a, b) => format!("(echo {} {} {})", show_m(*m), show_ty(a), show_ty(b)),
    }
}
fn show_uvec(d: &[Q]) -> String {
    let parts: Vec<&str> = d.iter().map(|&q| show_q(q)).collect();
    format!("[{}]", parts.join(" "))
}
fn show_result(r: &Option<(Ty, Vec<Q>)>) -> String {
    match r {
        None => "none".into(),
        Some((a, d)) => format!("(ok {} {})", show_ty(a), show_uvec(d)),
    }
}

// ---------- S-expression query emitters (MUST match the oracle grammar) ----------
fn sexp_ty(t: &Ty) -> String {
    show_ty(t) // identical grammar
}
fn sexp_tm(t: &Tm) -> String {
    match t {
        Tm::Var(n) => format!("(var {})", n),
        Tm::UnitT => "star".into(),
        Tm::Lam(q, a, b) => format!("(lam {} {} {})", show_q(*q), sexp_ty(a), sexp_tm(b)),
        Tm::App(f, x) => format!("(app {} {})", sexp_tm(f), sexp_tm(x)),
        Tm::With(a, b) => format!("(with {} {})", sexp_tm(a), sexp_tm(b)),
        Tm::Fst(t) => format!("(fst {})", sexp_tm(t)),
        Tm::Snd(t) => format!("(snd {})", sexp_tm(t)),
        Tm::Tensor(a, b) => format!("(tensor {} {})", sexp_tm(a), sexp_tm(b)),
        Tm::LetPair(a, b) => format!("(letpair {} {})", sexp_tm(a), sexp_tm(b)),
        Tm::Inl(ty, t) => format!("(inl {} {})", sexp_ty(ty), sexp_tm(t)),
        Tm::Inr(ty, t) => format!("(inr {} {})", sexp_ty(ty), sexp_tm(t)),
        Tm::Case(s, l, r) => format!("(case {} {} {})", sexp_tm(s), sexp_tm(l), sexp_tm(r)),
        Tm::Let(q, a, b) => format!("(let {} {} {})", show_q(*q), sexp_tm(a), sexp_tm(b)),
        Tm::MkEcho(m, a, b, t) => {
            format!("(mkecho {} {} {} {})", show_m(*m), sexp_ty(a), sexp_ty(b), sexp_tm(t))
        }
        Tm::Weaken(t) => format!("(weaken {})", sexp_tm(t)),
    }
}
fn sexp_ctx(g: &[Ty]) -> String {
    let mut s = String::from("(ctx");
    for ty in g {
        s.push(' ');
        s.push_str(&sexp_ty(ty));
    }
    s.push(')');
    s
}

fn show_step(r: &Option<Tm>) -> String {
    match r {
        None => "none".into(),
        Some(t) => format!("(some {})", sexp_tm(t)),
    }
}

fn eval_nf(t: &Tm, fuel: u32) -> Tm {
    let mut cur = t.clone();
    let mut f = fuel;
    while f > 0 {
        match step1(&cur) {
            None => break,
            Some(n) => {
                cur = n;
                f -= 1;
            }
        }
    }
    cur
}

// ---------- closed, redex-rich generators (deep substitution coverage) ----------
fn gen_value(r: &mut Rng, fuel: u32) -> Tm {
    if fuel == 0 {
        return Tm::UnitT;
    }
    match r.upto(6) {
        0 => Tm::UnitT,
        1 => Tm::Lam(gen_q(r), gen_ty(r, 1), Box::new(gen_tm(r, fuel - 1, 1))),
        2 => Tm::With(Box::new(gen_value(r, fuel - 1)), Box::new(gen_value(r, fuel - 1))),
        3 => Tm::Tensor(Box::new(gen_value(r, fuel - 1)), Box::new(gen_value(r, fuel - 1))),
        4 => Tm::Inl(gen_ty(r, 1), Box::new(gen_value(r, fuel - 1))),
        _ => Tm::MkEcho(gen_m(r), gen_ty(r, 1), gen_ty(r, 1), Box::new(gen_value(r, fuel - 1))),
    }
}

// closed (scope 0) terms built around redexes so reduction proceeds and
// exercises subst0/subst2/shift in chains.
fn gen_closed(r: &mut Rng, fuel: u32) -> Tm {
    if fuel == 0 {
        return gen_value(r, 1);
    }
    match r.upto(8) {
        0 => Tm::App(
            Box::new(Tm::Lam(gen_q(r), gen_ty(r, 1), Box::new(gen_tm(r, fuel - 1, 1)))),
            Box::new(gen_value(r, fuel - 1)),
        ),
        1 => Tm::Let(gen_q(r), Box::new(gen_value(r, fuel - 1)), Box::new(gen_tm(r, fuel - 1, 1))),
        2 => Tm::Fst(Box::new(Tm::With(Box::new(gen_value(r, fuel - 1)), Box::new(gen_value(r, fuel - 1))))),
        3 => Tm::Snd(Box::new(Tm::With(Box::new(gen_value(r, fuel - 1)), Box::new(gen_value(r, fuel - 1))))),
        4 => Tm::LetPair(
            Box::new(Tm::Tensor(Box::new(gen_value(r, fuel - 1)), Box::new(gen_value(r, fuel - 1)))),
            Box::new(gen_tm(r, fuel - 1, 2)),
        ),
        5 => Tm::Case(
            Box::new(Tm::Inl(gen_ty(r, 1), Box::new(gen_value(r, fuel - 1)))),
            Box::new(gen_tm(r, fuel - 1, 1)),
            Box::new(gen_tm(r, fuel - 1, 1)),
        ),
        6 => Tm::Weaken(Box::new(Tm::MkEcho(
            Mode::Linear,
            gen_ty(r, 1),
            gen_ty(r, 1),
            Box::new(gen_value(r, fuel - 1)),
        ))),
        _ => gen_value(r, fuel - 1),
    }
}

// ---------- session configs (coupling #4) ----------
fn sval(v: &Val) -> String {
    match v {
        Val::Var(n) => format!("(vvar {})", n),
        Val::Unit => "vunit".into(),
        Val::Bool(true) => "(vbool t)".into(),
        Val::Bool(false) => "(vbool f)".into(),
        Val::Nat(n) => format!("(vnat {})", n),
    }
}
fn sparty(p: &Party) -> String {
    match p {
        Party::End => "qend".into(),
        Party::Send(v, q) => format!("(qsend {} {})", sval(v), sparty(q)),
        Party::Recv(q) => format!("(qrecv {})", sparty(q)),
        Party::Sel(l, q) => format!("(qsel {} {})", l, sparty(q)),
        Party::Bra(bs) => {
            let mut s = String::from("(qbra");
            for (l, q) in bs {
                s.push_str(&format!(" (br {} {})", l, sparty(q)));
            }
            s.push(')');
            s
        }
    }
}
fn sconfig(c: &Config) -> String {
    format!("(conf {} {})", sparty(&c.0), sparty(&c.1))
}
fn show_cstep(r: &Option<Config>) -> String {
    match r {
        None => "none".into(),
        Some(c) => format!("(some {})", sconfig(c)),
    }
}

fn gen_val(r: &mut Rng) -> Val {
    match r.upto(4) {
        0 => Val::Var(r.upto(3) as usize),
        1 => Val::Unit,
        2 => Val::Bool(r.upto(2) == 0),
        _ => Val::Nat(r.upto(5)),
    }
}
fn gen_party(r: &mut Rng, fuel: u32) -> Party {
    if fuel == 0 {
        return Party::End;
    }
    match r.upto(6) {
        0 | 5 => Party::End,
        1 => Party::Send(gen_val(r), Box::new(gen_party(r, fuel - 1))),
        2 => Party::Recv(Box::new(gen_party(r, fuel - 1))),
        3 => Party::Sel(r.upto(3) as usize, Box::new(gen_party(r, fuel - 1))),
        _ => {
            let n = r.upto(3) as usize;
            Party::Bra((0..n).map(|i| (i, gen_party(r, fuel - 1))).collect())
        }
    }
}
// bias toward dual pairs so the comm/select steps fire (alongside stuck configs)
fn gen_config(r: &mut Rng) -> Config {
    let f = 2 + r.upto(2) as u32;
    match r.upto(6) {
        0 => Config(Box::new(Party::Send(gen_val(r), Box::new(gen_party(r, f)))), Box::new(Party::Recv(Box::new(gen_party(r, f))))),
        1 => Config(Box::new(Party::Recv(Box::new(gen_party(r, f)))), Box::new(Party::Send(gen_val(r), Box::new(gen_party(r, f))))),
        2 => Config(
            Box::new(Party::Sel(r.upto(4) as usize, Box::new(gen_party(r, f)))),
            Box::new(gen_party(r, f + 1)),
        ),
        3 => Config(
            Box::new(gen_party(r, f + 1)),
            Box::new(Party::Sel(r.upto(4) as usize, Box::new(gen_party(r, f)))),
        ),
        _ => Config(Box::new(gen_party(r, f)), Box::new(gen_party(r, f))),
    }
}

// ---------- global types ----------
fn svty(t: &Vty) -> &'static str {
    match t {
        Vty::Unit => "vtunit",
        Vty::Bool => "vtbool",
        Vty::Nat => "vtnat",
    }
}
fn sgty(g: &Gty) -> String {
    match g {
        Gty::End => "gend".into(),
        Gty::Msg(p, q, t, c) => format!("(gmsg {} {} {} {})", p, q, svty(t), sgty(c)),
        Gty::Bra(p, q, bs) => {
            let mut s = format!("(gbra {} {}", p, q);
            for (l, g) in bs {
                s.push_str(&format!(" (gbr {} {})", l, sgty(g)));
            }
            s.push(')');
            s
        }
        Gty::Mu(c) => format!("(gmu {})", sgty(c)),
        Gty::Var(n) => format!("(gvar {})", n),
    }
}
fn show_gstep(r: &Option<Gty>) -> String {
    match r {
        None => "none".into(),
        Some(g) => format!("(some {})", sgty(g)),
    }
}
fn gen_vty(r: &mut Rng) -> Vty {
    match r.upto(3) {
        0 => Vty::Unit,
        1 => Vty::Bool,
        _ => Vty::Nat,
    }
}
fn gen_gty(r: &mut Rng, fuel: u32) -> Gty {
    if fuel == 0 {
        return Gty::End;
    }
    match r.upto(6) {
        0 | 5 => Gty::End,
        1 => Gty::Msg(r.upto(3) as usize, r.upto(3) as usize, gen_vty(r), Box::new(gen_gty(r, fuel - 1))),
        2 => {
            let nb = r.upto(3) as usize;
            Gty::Bra(r.upto(3) as usize, r.upto(3) as usize, (0..nb).map(|i| (i, gen_gty(r, fuel - 1))).collect())
        }
        3 => Gty::Mu(Box::new(gen_gty(r, fuel - 1))),
        _ => Gty::Var(r.upto(3) as usize),
    }
}

// ---------- role assignments (n-ary located) ----------
fn sra(ra: &[(usize, Party)]) -> String {
    let mut s = String::from("(ra");
    for (r, p) in ra {
        s.push_str(&format!(" (rp {} {})", r, sparty(p)));
    }
    s.push(')');
    s
}
fn show_nstep(r: &Option<RoleAssignment>) -> String {
    match r {
        None => "none".into(),
        Some(ra) => format!("(some {})", sra(ra)),
    }
}
// bias toward a fireable send/recv (or select/branch) pair among n roles
fn gen_ra(r: &mut Rng) -> RoleAssignment {
    let n = 2 + r.upto(3) as usize; // 2..=4 roles
    let mut ra: RoleAssignment = Vec::new();
    match r.upto(4) {
        0 => {
            ra.push((0, Party::Send(gen_val(r), Box::new(gen_party(r, 2)))));
            ra.push((1, Party::Recv(Box::new(gen_party(r, 2)))));
        }
        1 => {
            ra.push((0, Party::Sel(r.upto(3) as usize, Box::new(gen_party(r, 2)))));
            ra.push((1, gen_party(r, 2)));
        }
        _ => {}
    }
    while ra.len() < n {
        let i = ra.len();
        ra.push((i, gen_party(r, 2)));
    }
    ra
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("usage: conformance_gen <count> <seed> <corpus_out> <rust_results_out>");
        std::process::exit(2);
    }
    let count: u64 = args[1].parse().expect("count");
    let seed: u64 = args[2].parse().expect("seed");
    let mut corpus = BufWriter::new(File::create(&args[3]).expect("corpus_out"));
    let mut results = BufWriter::new(File::create(&args[4]).expect("rust_results_out"));

    let mut r = Rng(seed.wrapping_add(0xD1B5_4A32_D192_ED03));
    for _ in 0..count {
        // (a) checker (#1) + one-step (#2) conformance on a random term that
        //     may be open and may be ill-typed (exercises both decision branches)
        let k = r.upto(4) as usize;
        let g: Vec<Ty> = (0..k).map(|_| gen_ty(&mut r, 2)).collect();
        let fuel = 2 + (r.upto(3) as u32); // depth 2..=4
        let t = gen_tm(&mut r, fuel, k);
        writeln!(corpus, "(q {} {})", sexp_ctx(&g), sexp_tm(&t)).unwrap();
        writeln!(results, "{}", show_result(&check(&g, &t))).unwrap();
        writeln!(corpus, "(s {})", sexp_tm(&t)).unwrap();
        writeln!(results, "{}", show_step(&step1(&t))).unwrap();

        // (b) normal-form (#2, deep) conformance on a CLOSED redex-rich term,
        //     cross-checking subst0/subst2/shift against Coq's extracted subst
        let ct = gen_closed(&mut r, 3);
        writeln!(corpus, "(nf {})", sexp_tm(&ct)).unwrap();
        writeln!(results, "{}", sexp_tm(&eval_nf(&ct, NF_FUEL))).unwrap();

        // (c) session steppers (#4): Rust my_qtt::session vs the extracted Coq
        //     cstep1 / gstep1 / nstep1 (binary / global / n-ary located layers)
        let cfg = gen_config(&mut r);
        writeln!(corpus, "(cstep {})", sconfig(&cfg)).unwrap();
        writeln!(results, "{}", show_cstep(&cstep1(&cfg))).unwrap();

        let g = gen_gty(&mut r, 3);
        writeln!(corpus, "(gstep {})", sgty(&g)).unwrap();
        writeln!(results, "{}", show_gstep(&gstep1(&g))).unwrap();

        let ra = gen_ra(&mut r);
        writeln!(corpus, "(nstep {})", sra(&ra)).unwrap();
        writeln!(results, "{}", show_nstep(&nstep1(&ra))).unwrap();
    }
}
