<!-- SPDX-License-Identifier: MPL-2.0 -->
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
idris2 --check solo-core.ipkg
```

## Proof and verification status

`my-lang` follows a *statements-first, machine-checked later* proof methodology
mirroring `affinescript`'s solo-core approach.
[`proofs/STATUS.md`](proofs/STATUS.md) is the single authoritative source — no
proof is described as "proved" there until a proof assistant accepts it.

**Current state (2026-06-02):**

| Artefact | Status |
|----------|--------|
| QTT semiring + laws (Coq + Idris2) | *locally-checked* — all laws proved by exhaustive case analysis |
| Solo syntax, contexts, typing (Coq + Idris2) | *definitions-only* |
| Progress / Preservation | *statement-only* — Phase F1.3/F1.4 |
| Echo type former (`EchoMode`, `Ty::Echo`, weakening) | *locally-checked* — 5 Agda-mirroring unit tests |
| Paper proofs (~6.3k lines) | *proved-on-paper* |
| Proof CI | *absent* — Phase F5 |

See [`proofs/ALIGNMENT-PLAN.md`](proofs/ALIGNMENT-PLAN.md) for the phased
roadmap toward AffineScript parity.

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
- **Proof phase**: F1.0 complete — QTT semiring proved on dual Coq + Idris2 tracks; soundness statements committed.
- **Governance**: CI green on all shipped checks; proof CI (Phase F5) pending.

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
