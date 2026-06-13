<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# my-lang Solo-core — Coq mechanisation

The Coq half of the dual-track mechanised QTT kernel for my-lang's
**solo** dialect. The Idris2 twin lives in
[`../../idris/solo-core/`](../../idris/solo-core/). Both tracks are
intentional (see `proofs/ALIGNMENT-PLAN.md`). **As of 2026-06-13 (D1,
option A) the tracks are NOT kept at full parity:** this Coq track is the
canonical mechanisation of record (axiom-free soundness + the `SEMIRING`
functor / affine layer / tropical instance); the Idris2 twin is a cheaper
definitions + `progress` cross-check whose `?todo_preservation` stays open
by choice.

## Modules

| File | Contents | Status |
|------|----------|--------|
| `Quantity.v` | Three-point QTT semiring + semiring/ordering laws | **machine-checked** (`destruct; reflexivity`, real `Qed`) |
| `EchoMode.v` | Two-point linearity poset (`Linear ⊑ Affine`) + no-section | **machine-checked** |
| `ResourceAlgebra.v` | `Module Type SEMIRING` (10-law resource-algebra boundary) + the live `ORDERED_SEMIRING` (preorder `qle`, R3) with decidable carrier equality `Q_eq_dec` (R5); the `Linear3` instance and the sealed boundary-check `Linear3_Sealed` | **machine-checked** |
| `SoloCore.v` | **Consolidated functor** `SoloCoreF (M : ORDERED_SEMIRING)`: syntax + de Bruijn terms, usage vectors, context-splitting QTT typing, CBV operational semantics, `progress` / `preservation` / `affine_pres`, **and the R5 usage-walk checker** `check` with `check_correct : has_type ↔ check` (sound + complete — decidability/adequacy of QTT typing). `Include SoloCoreF Linear3` recovers the concrete development under bare names. | **machine-checked, axiom-free** |
| `Context.v` / `ContextProps.v` | Alternative conflated-`ctx` algebra (a parallel presentation; *not* on the soundness path) | **machine-checked** |
| `EchoResidue.v` | Echo residue object + the subtyping facts the Rust checker relies on | **machine-checked** |

> **R2 (2026-06-13):** the former per-layer files `Syntax.v` / `Usage.v` /
> `Typing.v` / `Soundness.v` are merged into the single functor `SoloCore.v`.
> Coq functors are generative for inductives, so a per-file functor chain
> could not share the carrier-bearing `ty` / `tm` / `has_type` — hence one
> functor, instantiated once. See [`../../../AXIS-ARCHITECTURE.md`](../../../AXIS-ARCHITECTURE.md) §4.1.

The soundness theorems are **proved**: `Theorem progress : Progress.`,
`Theorem preservation : Preservation.` and `affine_pres` are real `Qed`
(Tracks F1.3 / F1.4), and `Print Assumptions` is closed for all three —
*no* `Admitted`/`Axiom` in their dependency cone. As of R2 they are
proved **parametrically** over `Module Type SEMIRING` inside `SoloCoreF`;
`Include SoloCoreF Linear3` recovers them axiom-free for the concrete
three-point carrier. **R5 (F1.4 tail)** adds the *executable* usage-walk
checker `check` and proves it sound + complete against `has_type`
(`check_correct`, axiom-free), with usage/type determinacy as the
corollary `typing_unique` — the decidability/adequacy of QTT typing, and
the spec the Rust `dialects/solo` checker must meet. The authoritative
state is `proofs/STATUS.md`.

## Building

```sh
# Generate a build driver from _CoqProject (output name CoqMakefile is
# generated, not committed) and build:
coq_makefile -f _CoqProject -o CoqMakefile && make -f CoqMakefile
# or check a single file directly:
coqc -R . SoloCore Quantity.v
```

Requires Coq 8.17+ (CI uses the Ubuntu `coq` package, 8.18). The `coqc`
proof leg **is** wired into CI: `.github/workflows/proofs.yml` builds this
package via `_CoqProject` and asserts `progress` / `preservation` /
`affine_pres` are axiom-free on every PR touching `proofs/verification/**`.

## Relationship to the existing Coq development

The pre-existing `../Syntax.v` / `../Typing.v` formalise the
*general* (non-quantitative) my-lang core and already prove a
substitution lemma. This `solo-core/` package adds the missing
**quantitative/affine** layer and the **operational-semantics +
soundness** scaffold that the general core lacks — the specific gap
versus AffineScript identified in `proofs/ALIGNMENT-PLAN.md`.
