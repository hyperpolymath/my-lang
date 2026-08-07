<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

<!-- Handoff prompt — paste to the next Claude working on my-lang. Dated 2026-06-19. -->

# Next-session prompt — my-lang (paste this to the next Claude)

You are continuing work on **my-lang** (the QTT multi-dialect language with a
mechanised Coq + Idris2 solo-core and an AI-first Rust compiler). Develop on
branch `claude/dreamy-hypatia-O8XHo`; commit green slices; push and open a
**draft** PR when a slice is complete. Read `docs/STATUS.adoc` (musts/intends/
wishes) and `proofs/STATUS.md` (authoritative proof registry) first.

## The one open MUST is now CLOSED: Idris `preservation` (#108) ✓

The Coq metatheory is complete and axiom-free, and **the Idris twin's
`preservation` is now DISCHARGED** (2026-06-21): a total, hole-free function
by induction on `Step` in `proofs/verification/idris/solo-core/Soundness.idr`,
`:total preservation` confirmed, `%default total`, no postulates. The whole
solo core is HOLE-FREE on both tracks. `htSubst` (all 15 cases), `substLemma0`,
`subst2Lemma`, and `uaddAssoc` all land in `Substitution.idr` (the module
dependency was inverted: `Soundness` now imports `Substitution`).

### Where the normative work goes next (the compilation axis)
The sound core is done and coupled. The open items are *intends*/*wishes* in
`docs/STATUS.adoc`:
1. Make the QTT resource axis the **default** in `crates/my-lang/src/checker.rs`
   (the bridge is opt-in today via `qtt_bridge::check_expr`).
2. Add surface **quantity syntax** so the bridge enforces real per-binder
   linearity (`Param` has no quantity field; the bridge defaults to `One`).
3. **wasm32** via the existing LLVM backend (`TargetSpec::wasm32` + `wasm-ld`),
   then **RISC-V** (`riscv64gc-unknown-linux-gnu`), then the **typed-wasm**
   WasmGC verified leg (blocked until `typed-wasm` / AffineScript / Ephapax are
   added to the working session).

### Load-bearing context (do NOT relitigate)
- **`preservation` takes the reduct term `t` explicitly** (relevant), exactly
  as `progress` does. Idris erases the derivation's term/type indices, but the
  substitution lemmas compute on the bound body + substituted value, so those
  must come from a matched (relevant) term; `{g}`/`{d}` are relevant for the
  same reason. `subst2Lemma`'s result type `c` is an erased implicit.
- **Per-clause pattern variables that unify** (the Lam/arrow quantity, the
  Inl/Inr annotation vs the sum's summand, the Let quantity) must share ONE
  name in the LHS — Idris rejects two names that provably unify.
- **`let`-block rule**: don't mix type-annotated bindings with dependent-pair
  pattern bindings in one block (parse error). Use single-binding nested
  `let … in let …`, or inline.
- **`b`-erasure is solved.** The result type `b` is a genuine type index, so
  `hvSubst` and `htSubst` both carry it as an **erased implicit** (`{0 b}` /
  auto-bound). Passing an *unrestricted* `b` (or an erased `b` to an
  unrestricted param) gives "`{b}` is not accessible" — keep it erased on both.
- **Let-clause quantity clash:** match `THLet`'s quantity field with `_` (the
  term's `q'` forces it; two named patterns "unify with" each other otherwise).
- **`THLet`/`THCase`/`THLetPair` carry explicit bound-type fields** (Option A,
  already merged) — that is how the binder cases recover the body context under
  erasure. Don't revert it.
- Toolchain: if Idris2 0.7.0 isn't preinstalled, bootstrap from source
  (`make bootstrap SCHEME=chezscheme && make install`, ~10 min). Then
  `cd proofs/verification/idris/solo-core && idris2 --build solo-core.ipkg`
  (exit 0), and check totality with `:total <name>` in the REPL. Discipline:
  every edit ends with a compile; `%default total`; no postulates; pin
  headline theorems where the repo convention does.

## The other axis: fundamentals -> compilation (see docs/STATUS.adoc)

The verified usage-checker is now in the compiler (`crates/my-qtt` =
faithful R5/R5b port; `crates/my-lang/src/qtt_bridge.rs` lowers real
`ast::Expr` and runs it). Next intends: make the QTT check the *default* in
`checker.rs`; add surface **quantity syntax** so linearity is enforced for
real; then **wasm32 via the existing LLVM backend** (`TargetSpec::wasm32` is
defined but unwired — add the `wasm-ld` link path), then **RISC-V**, then the
**typed-wasm** WasmGC verified leg (blocked until the `typed-wasm` /
AffineScript / Ephapax repos are added to the session).

## Don't reopen
The merged `my-qtt` and coupling (PRs #116/#117); the `b`-erasure design; the
Option-A bound-type fields; the Coq core (complete, axiom-free).
