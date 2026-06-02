<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# my-lang Solo-core — Coq mechanisation

The Coq half of the dual-track mechanised QTT kernel for my-lang's
**solo** dialect. The Idris2 twin lives in
[`../../idris/solo-core/`](../../idris/solo-core/). Both tracks are
intentional (see `proofs/ALIGNMENT-PLAN.md`); the modules are kept
structurally parallel so a change to one signals the work needed in
the other.

## Modules

| File | Contents | Status |
|------|----------|--------|
| `Quantity.v` | Three-point QTT semiring + semiring/ordering laws | **proved** (`destruct; reflexivity`, real `Qed`) |
| `Syntax.v`   | Solo-kernel types and de Bruijn terms | definitions |
| `Context.v`  | QTT contexts: `ctx_add`, `ctx_scale`, `ctx_zero` | definitions |
| `Typing.v`   | `has_type` / `has_var` (context-splitting QTT rules) | rules |
| `Soundness.v`| `value`, `step` (declared), `Progress` / `Preservation` | **statement-only** (named `Prop`s) |

The soundness theorems are stated as named **propositions**
(`Definition Progress : Prop := ...`), *not* as incomplete theorems.
A bare `Prop` definition asserts nothing and introduces no proof hole
or unproved assumption into the trusted base — it just records the
obligation, which is discharged later as
`Theorem progress : Progress. Proof. ... Qed.` (Track F1.3 / F1.4).
The honest current state is recorded in `proofs/STATUS.md`; nothing
here is "proved" until those Theorems land.

## Building

```sh
# Generate a build driver from _CoqProject (output name CoqMakefile is
# generated, not committed) and build:
coq_makefile -f _CoqProject -o CoqMakefile && make -f CoqMakefile
# or check a single file directly:
coqc -R . SoloCore Quantity.v
```

Requires Coq 8.17+. No `coqc` is wired into CI yet — adding a
`coqc`/`dune` proof leg is a Track F1 deliverable (`proofs/STATUS.md`).

## Relationship to the existing Coq development

The pre-existing `../Syntax.v` / `../Typing.v` formalise the
*general* (non-quantitative) my-lang core and already prove a
substitution lemma. This `solo-core/` package adds the missing
**quantitative/affine** layer and the **operational-semantics +
soundness** scaffold that the general core lacks — the specific gap
versus AffineScript identified in `proofs/ALIGNMENT-PLAN.md`.
