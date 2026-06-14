<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell <6759885+hyperpolymath@users.noreply.github.com> -->

# my-lang

`my-lang` is the *next-generation language* working project — a multi-dialect
surface for the hyperpolymath language stack. It is one of two flagship language
experiments in the estate (the other being
[`affinescript`](https://github.com/hyperpolymath/affinescript)). The two
projects are *siblings*, not a fork — they share design principles but diverge
in dialect scope and proof strategy.

For a gentler entry point see [EXPLAINME.adoc](EXPLAINME.adoc).

## What's in here

| Path | Contents |
|------|----------|
| `crates/` | Rust compiler crates. `crates/my-lang/src/types.rs` is the type checker, including `EchoMode`, `Ty::Echo`, and full affine-weakening semantics. |
| `dialects/` | Per-dialect surface-syntax definitions: *solo* (affine, single-agent), *duet* (session-typed), *ensemble* (multi-agent), *me* (visual/pedagogic). |
| `proofs/` | Formal verification assets — paper proofs, mechanised Coq + Idris2 solo-core. See [`proofs/STATUS.md`](proofs/STATUS.md) for the authoritative proof-status registry. |
| `conformance/` | Conformance tests — programs any compliant implementation must accept or reject identically. |
| `examples/` | Worked examples for each dialect. |
| `docs/` | Design notes, ADRs. Includes the [echo-types integration design note](docs/design/echo-types-integration.md). |
| `frontier-practices/` | Forward-looking research experiments (not shipped). |
| `contractiles/` | Project-level Mustfile / Dustfile invariant and recovery contracts. |

## Quickstart

```bash
git clone git@github.com:hyperpolymath/my-lang.git
cd my-lang

just build    # builds the workspace (Rust)
just test     # runs unit + conformance tests
```

To check the Coq solo-core:

```bash
cd proofs/verification/coq/solo-core
coq_makefile -f _CoqProject -o CoqMakefile && make -f CoqMakefile
```

To check the Idris2 solo-core:

```bash
cd proofs/verification/idris/solo-core
idris2 --build solo-core.ipkg
```

## Proof and verification status

`my-lang` follows a *statements-first, machine-checked later* proof methodology
mirroring `affinescript`'s solo-core approach.
[`proofs/STATUS.md`](proofs/STATUS.md) is the single authoritative source — no
proof is described as "proved" there until a proof assistant accepts it.

**Current state (2026-06-13):**

| Artefact | Status |
|----------|--------|
| QTT semiring + laws (Coq + Idris2) | *machine-checked* — all laws proved by exhaustive case analysis; run in CI (`proofs.yml`) |
| Solo syntax, contexts, typing (Coq + Idris2) | *locally-checked* |
| Operational semantics (CBV small-step) | *locally-checked* — Phase **F1.1 done** (both tracks) |
| **Progress** | *machine-checked (Coq)* — Phase **F1.3 done**: Coq `Theorem progress : Progress.` real `Qed.` (axiom-free), CI-guarded (`proofs.yml`); Idris `progress` hole-free/total (only `preservation` carries a `?todo`) |
| Preservation | *machine-checked (Coq)* — Phase **F1.4 done** on the Coq track: `Theorem preservation : Preservation.` and `affine_pres` are real `Qed.` (axiom-free, `Print Assumptions` closed), via the open-context QTT substitution lemma `ht_subst`. Product/elimination decision settled (additive `&` + multiplicative `⊗` both coherent). Idris twin pending (Phase F5 parity). |
| **Usage-walk checker** (`check`) | *machine-checked (Coq)* — Phase **F1.4 tail / R5 done**: `check : tctx → tm → option (ty × uvec)` synthesises type **and** usage in one bottom-up pass, with `check_correct : has_type G D t a ↔ check G t = Some (a, D)` a real `Qed.` (axiom-free, CI-guarded). This is the decidability/adequacy of QTT typing — the executable spec the Rust `dialects/solo` checker (still `TODO(#typeck)`) must meet, and it **overtakes AffineScript** (whose solo-core leaves the same equivalence as prose "future work"). **R5b** further decides the affine budget judgement (`aff_type_dec`). |
| **me → solo elaboration** (`elab`, `elab_data_check`, `me_wt_sound`) | *machine-checked (Coq)* — **M1.0 / M1.1 / M1.1b done**: the visual `me` surface ([`proofs/me/`](proofs/me/)) elaborates into the solo core. `elab : me_tm → tm` is a core-landing analogue of the paper `translate`. **M1.1** `elab_data_check` is the axiom-free Visual-Soundness theorem for the no-linear-use data fragment; **M1.1b** adds a me typing judgement `me_wt` with `me_wt_sound : me_wt G D e a → has_type G D (elab e) a` covering the linear-use constructs (token-consume / `MeLet` / pair-split / the faithful `MeIf → Case` conditional) **universally** — spanning the whole `me_tm` (incl. `MeSeq`, closed via the F1.4 `ht_shift0` weakening lemma). The first mechanised surface→core elaboration-correctness result in either sibling — AffineScript, as surveyed (`proofs/ALIGNMENT-PLAN.md` §1, AS@main 2026-06-02), is solo-only with no `me`-like dialect. |
| **Ensemble session-π core** (`SessionPi.v`: `proc`, `sty`/`dual`, `wt`, `step`) | *machine-checked (Coq), definitions + witnesses* — **S1.0 done (axis-2 STRUCTURE)**: a standalone Coq development for the ensemble session-typed π-calculus (binary, synchronous) after [`proofs/duet/session-types/`](proofs/duet/session-types/) + [`proofs/ensemble/agent-calculus/`](proofs/ensemble/agent-calculus/). Session types with a computed duality + `dual_involutive`, polarised endpoints, a *linear* channel-typing judgement with context splitting, and small-step reduction for the communication redex; executable witnesses (a well-typed ping-pong that reduces) are real `Qed`, **axiom-free** and CI-guarded. **Subject reduction is the S1.1 obligation** (not yet claimed). A greenfield overtake — AffineScript@main (surveyed 2026-06-02) has no concurrency/session/π metatheory in any form. |
| Echo in the type system (`TEcho`, `MkEcho`/`Weaken`, `THEcho`/`THWeaken`, `EchoMode`, `EchoResidue`) | *locally-checked* — Echo is a first-class type former in the formal kernel; `EchoResidue` backs the Rust `Ty::Echo` (5/5 unit tests are its laws) |
| Paper proofs (~6.3k lines) | *proved-on-paper* |
| Proof CI | *present* — `proofs.yml` machine-checks the Coq + Idris2 solo-cores (`coqc` + `idris2 --build`) on every PR touching `proofs/verification/**`, and asserts `progress`/`preservation`/`affine_pres` axiom-free |

See [`proofs/AXIS-ARCHITECTURE.md`](proofs/AXIS-ARCHITECTURE.md) for how the proof
effort factors (resource / structure / modality / surface axes, and the
resource↔echo non-identification invariant), and
[`proofs/ALIGNMENT-PLAN.md`](proofs/ALIGNMENT-PLAN.md) for the phased roadmap
toward AffineScript parity.

## Echo type integration

`my-lang` integrates the [`echo-types`](https://github.com/hyperpolymath/echo-types)
Agda library — a formal account of *loss that is not total erasure*.

```
Echo<A => B>   // "a proof-relevant residue of a lossy collapse from A to B"
```

`EchoMode { Linear, Affine }` mirrors the `EchoLinear.agda` two-point poset
`linear ⊑ affine`. A `Linear` echo may be weakened to `Affine` (one-way, no
section). Domain and codomain are invariant under subtyping.

**Claim boundary** (retraction R-2026-05-18):
> Echo is a *loss-graded reindexing modality over a thin poset*, not a graded
> comonad or free construction.

Full design rationale: [`docs/design/echo-types-integration.md`](docs/design/echo-types-integration.md).

## Architectural authority

- [`ANCHOR.scope-arrest.*`](ANCHOR.scope-arrest.2026-01-01.Jewell.scm) — the scope-arrest anchor enumerating what `my-lang` will and will not be.
- [`AUTHORITY_STACK.mustfile-nickel.scm`](AUTHORITY_STACK.mustfile-nickel.scm) — the cross-cutting authority stack.

Consult these before opening a feature request.

## Status

- **Licence**: MPL-2.0
- **Maturity**: design-iteration / early alpha. Working Rust compiler core exists (137+ passing tests); surface syntax and semantics still settling.
- **Proof phase**: F1.4 done on the Coq track — QTT semiring + `progress` + `preservation` machine-checked (axiom-free, CI-guarded); Idris twin pending (Phase F5 parity). **R2 done**: the Coq solo core is now one functor `SoloCoreF (M : SEMIRING)` parametric over a resource-algebra interface — `Include SoloCoreF Linear3` recovers the axiom-free result, and tropical/affine instances fall out (**R4 done**: `SoloCoreF Tropical` is verified axiom-free at an *infinite* min-plus carrier). **R3 done**: `affine_pres` is now a *distinct* theorem — affine budget-preservation over `ule` (pointwise `qle`), not an alias of `preservation` — with the functor parameter widened to `ORDERED_SEMIRING`. **R5 done (F1.4 tail)**: the declarative static context-splitting judgement is proved equivalent to an *executable* one-pass usage-walk checker `check` — `check_correct : has_type G D t a ↔ check G t = Some (a, D)` is axiom-free `Qed.`, the spec for the Rust `dialects/solo` checker; it inherits to the infinite tropical carrier for free and **overtakes AffineScript** (which leaves this equivalence as prose "future work"). **M1.0/M1.1/M1.1b done (2026-06-14)**: the visual `me` surface now elaborates into the solo core (`elab`, a core-landing analogue of the paper `translate`) with a machine-checked, axiom-free Visual-Soundness theorem (`elab_data_check`) for the **no-linear-use data fragment** — and **M1.1b** a me typing judgement (`me_wt`/`me_wt_sound`) making the linear-use constructs (token-consume, `MeLet`, pair-split, the faithful `MeIf → Case`) universal over the whole `me_tm` (incl. `MeSeq`) — M1 complete. For that fragment it is the first mechanised surface→core elaboration correctness in either sibling, as surveyed against AS@main 2026-06-02 (axis-4 SURFACE rung M1). **S1.0 commenced (2026-06-14)**: the *structure climb* (axis-2) opens with a new standalone Coq development `SessionPi.v` — a synchronous binary session-typed π-calculus core (session types + computed duality, polarised endpoints, a linear channel-typing judgement, small-step reduction) with axiom-free executable witnesses (a well-typed ping-pong that reduces). Subject reduction (S1.1) and session fidelity/congruence (S1.2) are the remaining obligations; this is a greenfield overtake — AS has no concurrency/session metatheory in any form (surveyed AS@main 2026-06-02).
- **Governance**: CI green on all shipped checks; proof CI **live** (`proofs.yml` machine-checks the solo-cores).

## Contributing

See [CONTRIBUTING.adoc](CONTRIBUTING.adoc). GPG-signed commits required.
Language policy, package management, and security requirements are enforced by
the estate governance workflow (`hyperpolymath/standards`). New contributors
should read [EXPLAINME.adoc](EXPLAINME.adoc) first.

## Companion repositories

| Repository | Relationship |
|------------|-------------|
| [`hyperpolymath/standards`](https://github.com/hyperpolymath/standards) | Estate-wide standards and governance |
| [`hyperpolymath/affinescript`](https://github.com/hyperpolymath/affinescript) | Sibling language; target parity state defined in `proofs/ALIGNMENT-PLAN.md` |
| [`hyperpolymath/echo-types`](https://github.com/hyperpolymath/echo-types) | Upstream Agda echo-types library integrated into the type checker |
| [`hyperpolymath/EchoTypes.jl`](https://github.com/hyperpolymath/EchoTypes.jl) | Julia companion; planned differential oracle for echo type testing |
| [`hyperpolymath/typed-wasm`](https://github.com/hyperpolymath/typed-wasm) | Shared typed-wasm compilation backend |
