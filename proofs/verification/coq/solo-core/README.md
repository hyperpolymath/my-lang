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
| `ResourceAlgebra.v` | `Module Type SEMIRING` (10-law resource-algebra boundary) + the live `ORDERED_SEMIRING` (preorder `qle`, R3) with decidable carrier equality `Q_eq_dec` (R5); the `Linear3` instance and the sealed boundary-check `Linear3_Sealed`; **the E4 seam interface `Module Type RESIDUE_MEASURE (S : SEMIRING)`** (the residue→carrier measure homomorphism, mirrored from upstream `EchoBridge.lean` — witness in `EchoMeasure.v`) | **machine-checked** |
| `SoloCore.v` | **Consolidated functor** `SoloCoreF (M : ORDERED_SEMIRING)`: syntax + de Bruijn terms, usage vectors, context-splitting QTT typing, CBV operational semantics, `progress` / `preservation` / `affine_pres`, **and the R5 usage-walk checker** `check` with `check_correct : has_type ↔ check` (sound + complete — decidability/adequacy of QTT typing). `Include SoloCoreF Linear3` recovers the concrete development under bare names. It also carries the **M1 me→solo elaboration** (`me_tm`, `elab`, and the axiom-free Visual-Soundness theorem `elab_data_check` for the no-linear-use data fragment). | **machine-checked, axiom-free** |
| `Context.v` / `ContextProps.v` | Alternative conflated-`ctx` algebra (a parallel presentation; *not* on the soundness path) | **machine-checked** |
| `EchoResidue.v` | Echo residue object + the subtyping facts the Rust checker relies on | **machine-checked** |
| `EchoMeasure.v` | **E4 SEAM (Coq mirror, axis-3 ECHO):** inhabits the `RESIDUE_MEASURE` interface at the R4 Tropical cost carrier (`EchoTraceTropical` — residue = echo reindexing trace, `measure` = accumulated `Affine`-collapse cost), with the monoid-homomorphism laws (`measure_empty`/`measure_combine`) and a measure-independence witness (`echo_measure_not_injective`). Mirrors upstream `EchoBridge.lean` / `Echo.Measure.Interface`; `measure` is a one-directional `E→R` decoration (Echo is **not** a resource instance); full modality-level measure-independence is upstream-cited. | **machine-checked, axiom-free** |
| `SessionPi.v` | **Ensemble session-π core + duet by projection + n-party projectability + static n-party config (axis-2 STRUCTURE, S1.0–S1.3 + S2.0–S2.2 + S3a + S3b).** A *standalone* development (NOT an extension of `SoloCore.v` — processes are a different term language, nothing carrier-bearing imported). **S1:** base-value payloads, session types `sty` with computed duality + `dual_involutive`, polarised endpoints, processes, a linear channel-typing judgement `wt` with context splitting, small-step `step`; value substitution (`wt_subst`) + communication-redex subject reduction (`sr_comm`); full closed-system subject reduction (`config_subject_reduction`) via the fused two-party `(νc)(P∣Q)` form; session fidelity (`session_fidelity`) + progress/deadlock-freedom (`config_progress`); **S1.3** n-ary choice (`SSelect`/`SBranch`), structural congruence (`wt_congr`), and the equi-recursive μ type-layer (`SMu`/`unfold_mu`/`dual_unfold`/`guarded`). **S2:** multiparty-*shaped* global types `gty` (message + **choice `GBra`** + **recursion `GMu`/`GVar`**), the now-**partial** projection `proj : gty → role → option sty` (three-way mutual; sender→`SSelect`, receiver→`SBranch`, uninvolved→**`merge`**, keystone `merge_idem`), two-party restriction `two_party`, `projection_duality` in **option-map form** (projection yields DUAL local types, mutual `two_party_mut` scheme), and `projected_config_wf` + corollaries transporting the whole S1 guarantee across projection where both projections are `Some` (deadlock-free by construction). **S3a:** the first n≥3 theorem — `projection_total : projectable_wf G → ∀ r:role, ∃ s, proj G r = Some s` (every role of a well-branched global type projects), via the mutual `projectable_wf_mut` scheme; non-vacuity at n=3 (3-party ring + agreeing 3-party choice merged), plain-merge boundary witnessed. **S3b:** the first n-party CONFIGURATION form — `role_assignment := list (role*party)` + the In-based **`wf_assignment G ra`** (every listed endpoint typed at its role's projection; a plain `Definition`, NOT nth/length, NOT a Fixpoint/Inductive), with the binary `Conf`/`wf_config` recovered as the **n=2 slice** (`conf_is_role_assignment2`, reusing `projected_config_wf`) + the converse embed `role_assignment2_of_conf` + the lookup bridge `ra_get`/`ra_get_in`, an n=3 non-`Conf` static witness over `g_ring` (`wf_ra_ring`), and honesty witnesses (duplicate-key tolerance, `projectable_but_uncoverable`). Fences: **plain** (not label-union) merge, **unpruned** μ projection, **no** global-level metatheory; `projectable_wf` = projection EXISTENCE not safety; `two_party → projectable_wf` not proved (false); `wf_assignment` = typed-at-projection (STATIC) not n-party safety, `role_assignment` an association list (duplicate-tolerant, no coverage claim); full-union merge + n-party SR + n-party operational semantics (S3c) / μ typing up-to-unfolding (S1.3b-meta) are OUT. | **machine-checked (Coq), axiom-free** |

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
the spec the Rust `dialects/solo` checker must meet. **R5b** decides the
affine layer too: `aff_type_dec` (the affine judgement `aff_type` is
decidable), via `aff_type_iff` (its `check`-characterisation) and
`ule_dec` (the budget order). **M1 (me→solo elaboration)** pins the
visual `me` surface (`proofs/me/`) as a Coq `me_tm` with
`elab : me_tm → tm` into this core: M1.0 gives `check`-executed
`Example` witnesses and M1.1 the universal, axiom-free Visual-Soundness
theorem `elab_data_check` (with `elab_data_typed` / `elab_data_aff_budget`)
for the no-linear-use fragment, and M1.1b a me typing judgement `me_wt`
with `me_wt_sound : me_wt G D e a → has_type G D (elab e) a` (axiom-free)
covering the linear-use constructs (token-consume / `MeLet` / pair-split)
and the faithful conditional `MeIf → Case` universally — the first
mechanised surface→core elaboration in either sibling. The authoritative
state is
`proofs/STATUS.md`.

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
