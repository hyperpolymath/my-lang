<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# my-lang ⇄ AffineScript Alignment Plan (proofs-first)

**Status:** active · **Created:** 2026-06-02 · **Owner:** @hyperpolymath
**Scope of this document:** bring `my-lang` from its current maturity
to the position `hyperpolymath/affinescript` occupies, **starting with
proofs**. This is the master roadmap; per-phase status is tracked in
[`proofs/STATUS.md`](./STATUS.md).

> `my-lang` and `affinescript` are the two flagship language
> experiments in the estate (see the my-lang README). They are
> *siblings*, not a fork — alignment means matching **maturity and
> rigour**, not file-for-file layout.

---

## 1. Where AffineScript is (the target)

Measured 2026-06-02 against `hyperpolymath/affinescript@main`.

### Proof / formal-verification assets
- **Paper proofs** (`docs/academic/proofs/`, ~4.6k lines, marked
  *Complete*): type-soundness, quantitative-types, effect-soundness,
  ownership-soundness, row-polymorphism, dependent-types,
  coherence-parametricity, inference-algorithm, db-theory.
- **Semantics** (`docs/academic/formal-verification/`): operational,
  denotational, **axiomatic (Hoare)** — *Complete*.
- **Mathematical foundations**: categorical semantics, logic
  foundations, complexity analysis.
- **Mechanised solo-core** (`.../solo-core/`, Idris2, ~700 lines that
  typecheck): `Syntax`, `Typing`, `Context`, `Quantity` (QTT semiring,
  laws fully proved), `Soundness` (progress/preservation as
  **statements-first** typed holes). Coq/Agda/Lean dirs are stubs.

### Discipline / governance
- An **authoritative `CAPABILITY-MATRIX.adoc`** with a precise status
  vocabulary (*enforced / works / partial / declared-but-unwired /
  parse-only / absent*) and a "What AffineScript is NOT" anti-overclaim
  section, enforced by a mechanical doc-truthing guard.
- A compiler maturity well beyond proofs: 80 OCaml modules (~33k LOC),
  ~20 backends, borrow checker, LSP, stdlib AOT gate. *(Out of scope
  for the proofs-first phase; tracked in §4 as later phases.)*

## 2. Where my-lang is (current)

Measured 2026-06-02 on this branch.

### Proof assets (already strong on paper, thin on mechanisation)
- **Paper proofs** (`proofs/`, ~6.3k lines — *broader* than AS thanks
  to the multi-dialect split): `shared/` (type-system incl.
  syntax/typing-rules/soundness/subtyping/inference/metatheory,
  effect-system, memory-model, operational- & denotational-semantics,
  ai-semantics) + dialect proofs `solo/` (affine-types, contracts),
  `duet/` (session-types), `ensemble/` (agent calculus), `me/`
  (block/visual/pedagogy).
- **Mechanised**: one Coq development
  (`proofs/verification/coq/{Syntax,Typing}.v`, ~0.9k lines, **9 `Qed`,
  0 `Admitted`**) — typing relation, subtyping, weakening, and a real
  **substitution lemma**. It is **non-quantitative** (no affine/linear
  layer) and has **no operational semantics, progress or
  preservation**. No Idris2, no Lean.

### Implementation reality
- The solo dialect's affine type-checker is `TODO(#typeck)` in
  `dialects/solo/compiler/src/lib.rs`. So a mechanised solo core
  **leads** the implementation — it becomes the spec the checker must
  satisfy.

## 3. The gap (proofs-first framing)

my-lang has **more paper coverage** but **less mechanisation maturity**.
The specific deltas to AffineScript:

| # | Gap | Severity |
|---|-----|----------|
| G1 | No **mechanised quantitative/affine core** (AS has the Idris2 QTT solo-core). | **highest** — it is the defining feature of an affine language |
| G2 | No mechanised **operational semantics + progress/preservation** (the existing Coq stops at substitution). | high |
| G3 | No **proof-status registry** / capability-matrix discipline; `STATE.a2ml` is stale (last real update 2026-03-14). | high |
| G4 | Missing paper-proof topics AS has as dedicated docs: **quantitative-types**, **row-polymorphism**, **dependent/refinement types**, **axiomatic (Hoare) semantics**, **categorical semantics**, **complexity analysis**. | medium |
| G5 | No **mechanisation CI leg** (neither repo CI-checks proofs today; closing this overtakes AS). | medium |

## 4. The plan

Phases are ordered **proofs-first**. Compiler-parity phases (P4+) are
listed for completeness but are explicitly *after* the proof phases.

### Phase F1 — Mechanised affine/QTT solo-core  ← **commenced in this PR**
Closes **G1** and scaffolds **G2**. Dual-track by decision: **both Coq
and Idris2** (sibling-parity with AS *and* reuse of the existing Coq
investment).

- **F1.0** — Scaffold both tracks with the QTT semiring, syntax,
  contexts, typing judgement, and soundness *statements*. **DONE** —
  `proofs/verification/idris/solo-core/` and
  `proofs/verification/coq/solo-core/`. Semiring laws are *fully
  proved* on both tracks; progress/preservation are statements-first
  (Coq named `Prop`s; Idris `?todo_*` holes — no `Admitted`/`Axiom`
  in the Coq trusted base).
- **F1.1** — Commit the small-step operational semantics (`Step` /
  `step` constructors): call-by-value, left-to-right beta / projection
  / case. Mirror across both tracks.
- **F1.2** — Extend the three-point semiring to the **four-point
  affine** domain (`0 ⊏ ? ⊏ 1 ⊏ ω`, `?` = at-most-once) matching
  `proofs/solo/affine-types/formal-system.md`; re-prove the semiring
  laws.
- **F1.3** — Prove **progress** (canonical-forms lemmas) on both tracks.
- **F1.4** — Prove **preservation** via a QTT substitution lemma that
  respects context splitting; prove the **affine-accounting**
  corollary. Then state & prove the equivalence between static
  context-splitting and the post-hoc usage-walk the future
  `dialects/solo` checker will use.

### Phase F2 — Effects metatheory (mechanised)
Closes part of **G2/G4**. Lift `proofs/shared/effect-system/` to a
mechanised effect-row calculus with an effect-soundness statement, then
proof. Pairs with the duet/ensemble dialect calculi.

### Phase F3 — Memory model (mechanised)
Ownership / borrowing soundness, mirroring AS's ownership-soundness.
Builds on F1's quantities (references as quantity-`ω` views).

### Phase F4 — Paper-proof topic parity
Closes **G4**. Add the dedicated docs my-lang lacks: quantitative-types
(extract from solo affine doc), row-polymorphism, dependent/refinement
types, axiomatic (Hoare) semantics, categorical semantics, complexity
analysis. Keep them under `proofs/shared/` / `proofs/<dialect>/`.

### Phase F5 — Proof CI + status discipline
Closes **G3/G5**. (a) A `proofs/STATUS.md` registry with AS-style
precise vocabulary — **started in this PR**. (b) A CI leg that runs
`idris2 --check` and `coqc` over the mechanised cores so "proved" is
machine-verified, not asserted. (c) Refresh `.machine_readable` STATE
so it mirrors reality.

### Phases P-later — compiler/runtime parity (out of proofs scope)
Backends, borrow checker in the Rust `crates/`, LSP, stdlib AOT gate,
capability matrix for the *implementation*. Sequenced after the proof
phases per the user's "starting with proofs" directive.

## 5. Decisions on record

- **D1 — Dual mechanisation (Coq + Idris2).** Reuse my-lang's Coq
  investment *and* mirror AS's Idris2 solo-core for cross-repo parity.
  The two tracks are kept structurally parallel.
- **D2 — Statements-first.** Soundness theorems are committed as
  statements before proof, matching AS's methodology — as named `Prop`s
  on the Coq track (no `Admitted`/`Axiom`, to keep the trusted base
  clean per the standards Trusted-Base Reduction Policy) and as typed
  holes on the Idris2 track. `proofs/STATUS.md` is the single truth on what is
  actually proved.
- **D3 — Three-point semiring first, affine `?` next (F1.2).** Land a
  fully-proved canonical QTT core, then refine to four-point affine —
  avoids shipping unverified semiring tables.
- **D4 — No big restructure.** Siblings, not a fork: enrich `proofs/`
  in place rather than re-layout to mirror AS's `docs/academic/`.

## 6. References
- AffineScript proofs: `hyperpolymath/affinescript`
  `docs/academic/` and `.../solo-core/`.
- my-lang paper proofs: `proofs/shared/`, `proofs/solo/`.
- Mechanised cores: `proofs/verification/{coq,idris}/solo-core/`.
- Status: `proofs/STATUS.md`.
