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
investment). *(Parity relaxed 2026-06-13 — see D1: Coq is now the
canonical track; Idris2 is a definitions + `progress` cross-check only.)*

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
  `dialects/solo` checker will use. **DONE (Coq, 2026-06-13).**
  Preservation + `affine_pres` are axiom-free `Qed`; the **F1.4 tail
  (R5)** is closed by an *executable* one-pass checker `check : tctx →
  tm → option (ty × uvec)` with `check_sound` / `check_complete` /
  `check_correct : has_type G D t a ↔ check G t = Some (a, D)` — real
  `Qed`, axiom-free, CI-guarded, and the corollary `typing_unique`
  (usage/type determinacy). It inherits to the infinite tropical
  carrier for free. **This overtakes AffineScript**, whose solo-core
  states the same static-split≡usage-walk equivalence only as prose
  ("an explicit equivalence lemma is future work"). The checker decides
  the strictly-**linear** `has_type`; **R5b (2026-06-13) DONE** decides
  the affine *discard* too: `aff_type_dec` (decidability of the affine
  judgement `aff_type`/`ule`), via `aff_type_iff` (its `check`-character-
  isation, the affine analogue of `check_correct`) and `ule_dec` — all
  axiom-free, CI-guarded, inherited free at the tropical carrier.

  **Surface (M1, axis-4) — DONE (partial), 2026-06-14.** Beyond F1, the
  visual/block `me` surface (`proofs/me/`, axis-4 in
  `AXIS-ARCHITECTURE.md`) now elaborates into the solo core: a Coq
  `me_tm` + `elab : me_tm → tm` (the mechanised `translate`), with
  **M1.0** `check`-executed `Example` witnesses and **M1.1** the
  universal axiom-free Visual-Soundness theorem `elab_data_check`
  (formal-model.md Theorem 1) for the no-linear-use fragment, plus
  `elab_data_typed` / `elab_data_aff_budget`. This is the first
  mechanised surface→core elaboration-correctness result in either
  sibling and a clean **overtake** — AffineScript is solo-only with no
  `me`-like dialect and nothing analogous as surveyed (AS@main 2026-06-02). **M1.1b** (done,
  2026-06-14) adds a me-level typing judgement `me_wt` with `me_wt_sound : me_wt G D e a →
  has_type G D (elab e) a` (axiom-free), making the linear-USE constructs (`MeVar`/`MeLet`/
  `MeUsePair`) and the faithful conditional `MeIf → Case` UNIVERSAL — including `MeSeq`
  (closed via the F1.4 `ht_shift0` weakening lemma), so `me_wt` spans the whole `me_tm`. **M1
  (axis-4 SURFACE) is complete.**

  **Structure (S1 + S2, axis-2) — S1.0–S1.3 + S2.0–S2.2 done 2026-06-14.** The
  structure climb (`AXIS-ARCHITECTURE.md` axis 2: solo ⊂ duet ⊂ ensemble, done
  *ensemble-first* on the process side since `solo ⊄ ensemble`). A standalone Coq
  development `proofs/verification/coq/solo-core/SessionPi.v` (module
  `SoloCore.SessionPi`) mechanises a synchronous binary session-typed
  π-calculus core after `proofs/duet/session-types/` and
  `proofs/ensemble/agent-calculus/`. **S1.0:** definitions
  (payloads, session types `sty` with a computed duality + `dual_involutive`,
  polarised endpoints, processes, a *linear* channel-typing judgement `wt`
  with context splitting, small-step `step`) plus executable witnesses.
  **S1.1a/S1.1b:** value substitution (`wt_subst`) + communication-redex
  subject reduction (`sr_comm`), and full closed-system subject reduction
  (`config_subject_reduction`) via the fused two-party `(νc)(P∣Q)` form.
  **S1.2:** session fidelity (`session_fidelity`) + progress / deadlock-freedom
  (`config_progress`) — the duet paper's safety + liveness theorems for the
  binary fragment. **S2.0/S2.1 — duet by projection:** the multiparty-*shaped*
  global-type layer `gty`, the three-case projection `proj G r`, the two-party
  restriction `two_party`, `projection_duality` (a two-party choreography
  projects to DUAL local types, `p≠q` load-bearing), and `projected_config_wf`
  + corollaries that transport the whole S1.1b/S1.2 guarantee across projection
  — a projected choreography is deadlock-free BY CONSTRUCTION. All axiom-free
  (`Print Assumptions` closed) and CI-guarded (`proofs.yml`). This is a
  greenfield **overtake**: AffineScript@main (surveyed 2026-06-02) has *no*
  concurrency / session-types / π-calculus / multiparty metatheory in any form
  — a category AS does not enter. **Honest fence:** S2 mechanises message-passing
  + end only and *instantiates* the duet thesis on two-party choreographies — it
  does not prove a general n-party result. **S1.3 done (2026-06-14):** n-ary
  labelled **choice** (S1.3a — the three fused theorems extended via dedicated
  mutual inductives `sbranch`/`pbranch`), structural **congruence** preserves
  typing on the open `proc` (S1.3c — `wt_congr`, par laws), and the equi-recursive
  **μ type-layer** (S1.3b-core — `unfold_mu`/`dual_unfold`/`guarded`). **S2.2 done
  (2026-06-14):** global-type labelled **choice** (`GBra p→q:{lᵢ:Gᵢ}`) + equi-recursive
  (`GMu`/`GVar`) projection — projection becomes **partial** (`proj : gty → role →
  option sty`, three-way mutual) with a plain **`merge`** operator (keystone `merge_idem`)
  combining the branches for an uninvolved role; `projection_duality` reproved in
  **option-map form** over choice + μ (mutual `two_party_mut` scheme), the whole S2.0/S2.1
  bridge rethreaded through `proj … = Some _`. Design-panel-validated (the panel
  empirically compiled the fixpoints on Coq 8.18 and caught a real None-erasure soundness
  bug before implementation). Fences: **plain** (not full label-union) merge, **unpruned**
  μ projection (a non-participating role's `μX.X` is non-theorematic, never shown as a
  type), **no** global-level metatheory. **Remaining:** μ typing/SR up-to-unfolding
  (**S1.3b-meta**, deferred — needs a `PT_Unfold` rule + soundness); n≥3 coherence /
  full-union merge / projection-existence (**S3** — where "duet" stops and "ensemble"
  begins). Echo-types: NOT-RELEVANT (axis-2 STRUCTURE vs axis-3 MODALITY).

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
  **Amended 2026-06-13 (owner, option A):** the tracks are NOT kept at
  full structural parity. **Coq is the canonical mechanisation of
  record** — axiom-free `progress`/`preservation`/`affine_pres` plus the
  R2 functor (`SoloCoreF`), R3 affine layer, and R4 tropical instance.
  **Idris2 is a cheap definitions + `progress` second-source** whose
  `?todo_preservation` is deliberately left open. Coq-module-functor work
  (R2/R4) is *not* mirrored — Idris2 has no functor system, so it would be
  a different design, not a translation. The Idris CI leg stays (it only
  rebuilds the package). Do not auto-create matching Idris work when the
  Coq core advances.
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
