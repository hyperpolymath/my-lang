<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
<!-- Handoff prompt — paste to the next Claude working on my-lang. Dated 2026-06-19. -->

# Next-session prompt — my-lang (paste this to the next Claude)

You are continuing work on **my-lang** (the QTT multi-dialect language with a
mechanised Coq + Idris2 solo-core and an AI-first Rust compiler). Develop on
branch `claude/dreamy-hypatia-O8XHo`; commit green slices; push and open a
**draft** PR when a slice is complete. Read `docs/STATUS.adoc` (musts/intends/
wishes) and `proofs/STATUS.md` (authoritative proof registry) first.

## The one open MUST: close Idris `preservation` (#108)

The Coq metatheory is complete and axiom-free. The Idris twin's `progress` is
done; **`preservation` is the last normative obligation** (`?todo_preservation`
in `proofs/verification/idris/solo-core/Soundness.idr`). It rests on the QTT
substitution lemma `htSubst`, which is now **structurally landed** (commit
`acd29d6` on the dev branch, build green 8/8):

- **10/15 `htSubst` cases proved**: Var, Unit, Lam, With, Fst, Snd, Inl, Inr,
  MkEcho, Weaken.
- **5 usage-splitting holes remain** in `Substitution.idr`:
  `?htSubst_app`, `?htSubst_tensor`, `?htSubst_letpair`, `?htSubst_case`,
  `?htSubst_let`.

### Do this, in order
1. Fill the 5 split holes. Each mirrors the Coq `ht_subst` case in
   `proofs/verification/coq/solo-core/SoloCore.v` (≈ lines 1618–1889) and
   **consumes already-landed lemmas**: `usplit3`, `uaddSplitBoundary2`,
   `substReassocAdd` (additive, for App/Let), `substReassocMult` (for
   Tensor/LetPair/Case), `uaddUappend`, `uscaleUappend`, `ushiftUscale`.
   App splits `d1` and the `q`-scaled `d2`; Tensor/LetPair/Case split `d1`,`d2`;
   Let scales `d1`. The binder cases (LetPair/Case/Let) recurse into bodies with
   the prefix `I` extended (mirror the `hvSubst`/`htShift` binder pattern).
2. Add `substLemma0` (the `I = TEmpty` corollary) and `subst2Lemma` (two-var,
   for `LetPair`), mirroring Coq `subst_lemma0` / `subst2_lemma` (≈ 1893–1937).
3. Fill `?todo_preservation` in `Soundness.idr` by induction on `Step`
   (mirror Coq `preservation`, ≈ 1941–2036): reduction cases use
   `substLemma0`/`subst2Lemma`; congruence cases recurse.
4. Move the `preservation` row in `proofs/STATUS.md` to *machine-checked* and
   update `docs/STATUS.adoc`.

### Load-bearing context (do NOT relitigate)
- **`b`-erasure is solved.** The result type `b` is a genuine type index, so
  `hvSubst` and `htSubst` both carry it as an **erased implicit** (`{0 b}` /
  auto-bound). Passing an *unrestricted* `b` (or an erased `b` to an
  unrestricted param) gives "`{b}` is not accessible" — keep it erased on both.
- **Let-clause quantity clash:** match `THLet`'s quantity field with `_` (the
  term's `q'` forces it; two named patterns "unify with" each other otherwise).
- **`THLet`/`THCase`/`THLetPair` carry explicit bound-type fields** (Option A,
  already merged) — that is how the binder cases recover the body context under
  erasure. Don't revert it.
- Toolchain: Idris2 0.7.0 isn't preinstalled — bootstrap from source
  (`make bootstrap SCHEME=chezscheme && make install`, ~10 min), then
  `cd proofs/verification/idris/solo-core && idris2 --build solo-core.ipkg`.
  Discipline: every edit ends with a compile; `--safe`/`%default total`; no
  postulates; pin headline theorems where the repo convention does.

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
