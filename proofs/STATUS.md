<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# my-lang Proof Status Registry

**This file is the single authoritative source for the state of
my-lang's proofs.** Where any other document (a proof file's own
prose, `proofs/verification/README.md`, `.machine_readable/STATE`)
disagrees, *this registry wins*. Modelled on AffineScript's
`CAPABILITY-MATRIX.adoc` discipline.

Last verified: 2026-06-14.

## Status vocabulary (used precisely)

| Term | Meaning |
|------|---------|
| **machine-checked** | A proof assistant accepts it today (`Qed` / no holes) **and** it is run in CI. |
| **locally-checked** | Accepts under the proof assistant locally; **not** yet in CI. |
| **proved-on-paper** | A written proof exists in `proofs/**.md`; not mechanised. |
| **statement-only** | The theorem is *stated* mechanically (typed hole / `Admitted`); the proof is a tracked obligation, not done. |
| **definitions-only** | Syntax/rules/operations defined; no theorems yet. |
| **absent** | Does not exist, regardless of prose elsewhere. |

> **No proof hole is ever described as "proved".** A statement-only
> theorem is an obligation, not a result. On the Coq track the
> obligations are recorded as named `Prop`s (no `Admitted`/`Axiom` in
> the trusted base); on the Idris2 track as typed holes (`?todo_*`).

## Mechanised cores

| Artefact | Track | Status | Notes |
|----------|-------|--------|-------|
| QTT semiring + laws | Idris2 `idris/solo-core/Quantity.idr` | **machine-checked** | Semiring + ordering laws by exhaustive `Refl`, `%default total`. CI: `proofs.yml` Idris job (`idris2 --build`). |
| QTT semiring + laws | Coq `coq/solo-core/Quantity.v` | **machine-checked** | Laws by `destruct; reflexivity`, real `Qed`. CI: `proofs.yml` Coq job (`coqc`). |
| Resource-algebra boundary + **parametric soundness** | Coq `solo-core/{ResourceAlgebra,SoloCore}.v` | **machine-checked** | **R2 (2026-06-13).** `Module Type SEMIRING` (10 laws). The whole Coq solo core (syntax/usage/typing/soundness) is one functor `SoloCoreF (M : SEMIRING)`; `Include SoloCoreF Linear3` recovers `progress`/`preservation`/`affine_pres` **axiom-free** (`Print Assumptions` closed, CI-guarded). Tropical/affine instances reuse the functor unchanged (R4). The four former chain files (`Syntax`/`Usage`/`Typing`/`Soundness.v`) are merged into `SoloCore.v`. |
| Affine layer (`aff_type`, `affine_pres`) | Coq `solo-core/SoloCore.v` | **machine-checked** | **R3 (2026-06-13).** `ORDERED_SEMIRING` is now live and the functor is `SoloCoreF (M : ORDERED_SEMIRING)`. `ule` = pointwise `qle`; `aff_type G D t a := ∃ D0, has_type G D0 t a ∧ ule D0 D` ("uses at most budget `D`"); `has_type_aff` (linear ⊆ affine), `aff_weaken` (discard surplus), and `affine_pres` (budget-preservation) are real `Qed`, axiom-free. Makes `affine_pres` **DISTINCT** from `preservation` (no longer `:= Preservation`). The linear proofs use only `SEMIRING`. |
| Tropical (min-plus) instance | Coq `solo-core/Tropical.v` | **machine-checked** | **R4 (2026-06-13).** `Module Tropical <: ORDERED_SEMIRING` (carrier ℕ∪{∞}, ⊕=min, ⊗=+, 0=∞, 1=Fin 0); `Module SoloTropical := SoloCoreF Tropical` yields `progress`/`preservation`/`affine_pres` **axiom-free at an INFINITE carrier** with no new soundness proof — the R2 functor's acceptance test (it provably does not rely on the finite carrier). |
| Usage-walk checker (`check`) + adequacy | Coq `solo-core/SoloCore.v` | **machine-checked** | **R5 (2026-06-13).** Executable one-pass synthesiser `check : tctx → tm → option (ty × uvec)` proved equivalent to the declarative `has_type`: `check_sound` (only well-typed terms accepted), `check_complete` (every derivation recovered with **exactly** its usage), and `check_correct : has_type G D t a ↔ check G t = Some (a, D)` are real `Qed`, **axiom-free** (`Print Assumptions` closed, CI-guarded). Internalises usage/type determinacy (corollary `typing_unique`). `SoloTropical` inherits the verified checker at the **infinite** carrier for free. Needs `Q_eq_dec` (added to `ORDERED_SEMIRING`); the soundness proofs do not. The F1.4 tail; **overtakes AffineScript** (which states the same static-split≡usage-walk equivalence only as prose — "an explicit equivalence lemma is future work"). Several `Compute`-backed examples pin non-vacuity. |
| Affine judgement decidability (`aff_type_dec`) | Coq `solo-core/SoloCore.v` | **machine-checked** | **R5b (2026-06-13).** `check` decides the strictly-**linear** `has_type`; the **affine** judgement `aff_type` (R3, the affine *discard*) is decided by `aff_type_dec`, via `aff_type_iff : aff_type G D t a ↔ ∃ D0, check G t = Some (a, D0) ∧ ule D0 D` (the affine analogue of `check_correct`) and `ule_dec` (the budget order is decidable as `qle` is `bool`). All axiom-free, CI-guarded, inherited free at the tropical carrier. The discard is the exact separator: the same `UnitT` at budget `One` is `aff_type`-accepted but `has_type`-rejected. |
| `me → solo` elaboration (`elab`) + concrete adequacy | Coq `solo-core/SoloCore.v` | **machine-checked** | **M1.0 (2026-06-14).** The visual/block `me` surface (no settled AST in code — only the paper block grammar in `proofs/me/visual-semantics/formal-model.md`) is pinned as `me_tm` (the affine/token fragment) with `elab : me_tm → tm` landing in the de Bruijn solo core (the mechanised `translate`). `Example`s EXECUTE `elab` + the R5 `check` on the concrete carrier: a linear token create-and-consume is accepted; the dropped token is rejected linearly but accepted via sequencing and via the R3 `aff_type` layer; the echo linear→affine bridge end-to-end. Real `Qed`/`reflexivity`. |
| `me → solo` universal adequacy (`elab_data_check`) | Coq `solo-core/SoloCore.v` | **machine-checked** | **M1.1 (2026-06-14).** `elab_data_check : ∀ e G, me_data e = true → ∃ a, check G (elab e) = Some (a, uzero G)` — formal-model.md Theorem 1 (Visual Soundness) for the no-linear-use fragment (data + discard-sequencing + sum injections), real `Qed`, **axiom-free** (`Print Assumptions` closed, CI-guarded). Corollaries `elab_data_typed` (`has_type` via `check_correct` — the cleanest Visual-Soundness statement) and `elab_data_aff_budget` (fits any affine budget ≥ `uzero G`, the R3 discard). The first mechanised surface→core elaboration-correctness result in either sibling; **overtakes AffineScript** (solo-only, no `me`-like dialect). The linear-USE constructs are M1.0 `Example`s here; their UNIVERSAL adequacy is **M1.1b** (below). |
| `me → solo` universal linear-use adequacy (`me_wt` / `me_wt_sound`) | Coq `solo-core/SoloCore.v` | **machine-checked** | **M1.1b (2026-06-14).** A me-level typing/usage judgement `me_wt : tctx → uvec → me_tm → ty → Prop` (the visual linear token discipline, colours/ports abstracted) with `me_wt_sound : me_wt G D e a → has_type G D (elab e) a` — elaboration is type- AND usage-preserving — real `Qed`, **axiom-free** (`Print Assumptions` closed, CI-guarded). Covers the resource-INTERESTING constructs UNIVERSALLY: token consumption (`MeVar`), `MeLet` (consumed once), `MeUsePair`, and the new faithful conditional `MeIf → Case` (the paper IfBlock = sum ELIM; `MeInl`/`MeInr` are sum INTRO). Corollaries `me_wt_check` (accepted by the verified `check`) and `me_wt_aff` (any affine budget ≥ realised usage). It spans the WHOLE `me_tm`: `MeSeq` (sequencing/discard) is closed too via `MW_Seq`, reusing the existing F1.4 `ht_shift0` weakening lemma (e1 erased at multiplicity Zero, e2 weakened under the discarded binder) — so the elaboration is universally type- and usage-preserving over the entire pinned surface. **M1 (axis-4 SURFACE) is complete.** |
| Solo syntax / contexts / typing | Idris2 + Coq `solo-core/` | **definitions-only** | de Bruijn terms, QTT context split/scale/zero, context-splitting typing judgement. |
| `progress` | Coq `solo-core/SoloCore` | **machine-checked** | `Theorem progress : Progress.` real `Qed`, axiom-free. Phase F1.3. CI: `proofs.yml` compiles it + asserts `Print Assumptions` closed. |
| `progress` | Idris2 `solo-core/Soundness` | **locally-checked** | Hole-free total function (no `?todo_progress`); accepted by `idris2`, package built in CI (`proofs.yml`). Phase F1.3. |
| `preservation` | Coq `solo-core/SoloCore` | **machine-checked** | `Theorem preservation : Preservation.`, real `Qed`, axiom-free, via the open-context QTT substitution lemma `ht_subst`. Phase F1.4. CI: `proofs.yml` compiles it + asserts `Print Assumptions` closed (and likewise `affine_pres`). |
| `preservation` | Idris2 `solo-core/Soundness` | **statement-only** | `?todo_preservation`. Proof = Phase F1.4 (Idris track). |
| Small-step `step` | Coq `solo-core/SoloCore` | **definitions-only** | CBV left-to-right relation, all redex + congruence constructors. Committed in F1.1. |
| Small-step `Step` | Idris2 `solo-core/Soundness` | **statement-only** | Idris twin (unverified on this track). |
| General core typing + substitution | Coq `coq/Typing.v` | **locally-checked** | Pre-existing: 9 `Qed`, 0 `Admitted`. Non-quantitative; substitution lemma proved. |

## Paper proofs (`proofs/**.md`)

| Area | Status | Location |
|------|--------|----------|
| Type-system soundness | proved-on-paper | `proofs/shared/type-system/soundness.md` |
| Type-system metatheory / inference / subtyping | proved-on-paper | `proofs/shared/type-system/` |
| Effect-system soundness | proved-on-paper | `proofs/shared/effect-system/soundness.md` |
| Memory model / ownership | proved-on-paper | `proofs/shared/memory-model/ownership.md` |
| Operational & denotational semantics | proved-on-paper | `proofs/shared/{operational,denotational}-semantics/` |
| Solo affine types | proved-on-paper | `proofs/solo/affine-types/formal-system.md` |
| Duet session types | proved-on-paper | `proofs/duet/session-types/formal-system.md` |
| Ensemble agent calculus | proved-on-paper | `proofs/ensemble/agent-calculus/` |
| AI semantics | proved-on-paper | `proofs/shared/ai-semantics/` |

## Gaps vs AffineScript (see ALIGNMENT-PLAN.md §3)

| Topic AS has | my-lang status | Plan phase |
|--------------|----------------|-----------|
| Mechanised QTT solo-core | **commenced** (this PR) | F1 |
| Mechanised progress/preservation | Coq **machine-checked** (axiom-free `Qed`, CI-guarded); Idris twin pending | F1.3 / F1.4 |
| Algorithmic typing / decidable typecheck (static-split ≡ usage-walk) | Coq **machine-checked** (`check_correct`, axiom-free) — **overtakes AS** (states it as prose "future work" only) | F1.4 |
| Surface→core elaboration correctness (`me` → `solo`) | Coq **machine-checked** (`me_wt_sound` over the WHOLE `me_tm` + `elab_data_check`, axiom-free, M1.1/M1.1b — M1 complete) — **overtakes AS** (solo-only; no `me`-like dialect, nothing analogous as surveyed (AS@main 2026-06-02)) | M1 |
| Quantitative-types (dedicated doc) | folded into solo affine doc | F4 |
| Row-polymorphism proof | absent | F4 |
| Dependent/refinement types proof | absent | F4 |
| Axiomatic (Hoare) semantics | absent | F4 |
| Categorical semantics | absent | F4 |
| Complexity analysis | absent | F4 |
| Proof CI leg | **present** — `proofs.yml` machine-checks both tracks (`coqc` + `idris2 --build`); overtakes AS | F5 |

## How to update this file

Change this registry in the **same PR** as any proof change. When an
`Admitted`/`?todo` becomes a real proof, move the row from
*statement-only* to *locally-checked*, and to *machine-checked* once
the F5 CI leg verifies it.
