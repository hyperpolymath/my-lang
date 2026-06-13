<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# my-lang Proof Status Registry

**This file is the single authoritative source for the state of
my-lang's proofs.** Where any other document (a proof file's own
prose, `proofs/verification/README.md`, `.machine_readable/STATE`)
disagrees, *this registry wins*. Modelled on AffineScript's
`CAPABILITY-MATRIX.adoc` discipline.

Last verified: 2026-06-02.

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
| QTT semiring + laws | Idris2 `idris/solo-core/Quantity.idr` | **locally-checked** | Semiring + ordering laws by exhaustive `Refl`, `%default total`. CI leg pending (Phase F5). |
| QTT semiring + laws | Coq `coq/solo-core/Quantity.v` | **locally-checked** | Laws by `destruct; reflexivity`, real `Qed`. CI leg pending (F5). |
| Solo syntax / contexts / typing | Idris2 + Coq `solo-core/` | **definitions-only** | de Bruijn terms, QTT context split/scale/zero, context-splitting typing judgement. |
| `progress` | Coq `solo-core/Soundness` | **locally-checked** | `Theorem progress : Progress.` real `Qed`, axiom-free (`Print Assumptions` closed). Phase F1.3. CI leg pending (F5). |
| `progress` | Idris2 `solo-core/Soundness` | **statement-only** | `?todo_progress`. Proof = Phase F1.3 (Idris track). |
| `preservation` / `affine_pres` | Coq `solo-core/Soundness` | **locally-checked** | `Theorem preservation : Preservation.` and `affine_pres : AffinePreservation.`, real `Qed`, axiom-free, via the open-context QTT substitution lemma `ht_subst`. Phase F1.4. CI leg pending (F5). |
| `preservation` | Idris2 `solo-core/Soundness` | **statement-only** | `?todo_preservation`. Proof = Phase F1.4 (Idris track). |
| Small-step `step` | Coq `solo-core/Soundness` | **definitions-only** | CBV left-to-right relation, all redex + congruence constructors. Committed in F1.1. |
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
| Mechanised progress/preservation | Coq **locally-checked** (axiom-free `Qed`); Idris twin pending | F1.3 / F1.4 |
| Quantitative-types (dedicated doc) | folded into solo affine doc | F4 |
| Row-polymorphism proof | absent | F4 |
| Dependent/refinement types proof | absent | F4 |
| Axiomatic (Hoare) semantics | absent | F4 |
| Categorical semantics | absent | F4 |
| Complexity analysis | absent | F4 |
| Proof CI leg | absent (AS also absent) | F5 |

## How to update this file

Change this registry in the **same PR** as any proof change. When an
`Admitted`/`?todo` becomes a real proof, move the row from
*statement-only* to *locally-checked*, and to *machine-checked* once
the F5 CI leg verifies it.
