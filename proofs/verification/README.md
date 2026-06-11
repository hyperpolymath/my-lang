<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Formal Verification Framework

This directory contains formal verification artifacts for My Language,
including mechanized proofs in Coq and Idris2, and property
specifications for testing.

> **Authoritative status:** [`../STATUS.md`](../STATUS.md) is the single
> source of truth for what is actually proved. The roadmap to
> AffineScript parity is [`../ALIGNMENT-PLAN.md`](../ALIGNMENT-PLAN.md).
> The status table further down this file is a convenience summary and
> defers to `STATUS.md` on any disagreement.

## Contents

1. [`coq/`](coq/) — General Coq core (`Syntax.v`, `Typing.v`:
   non-quantitative, substitution lemma proved).
2. [`coq/solo-core/`](coq/solo-core/) — **Mechanised QTT affine
   solo-core** (Coq track): semiring laws proved; progress/preservation
   stated.
3. [`idris/solo-core/`](idris/solo-core/) — **Mechanised QTT affine
   solo-core** (Idris2 track), mirroring AffineScript's solo-core.
4. `properties/` — Property-based test specifications.

## Verification Goals

### Type System

- [ ] **Type Preservation**: `∀ e e' τ. Γ ⊢ e : τ ∧ e ⟶ e' ⟹ Γ ⊢ e' : τ`
- [ ] **Progress**: `∀ e τ. ∅ ⊢ e : τ ⟹ value(e) ∨ ∃e'. e ⟶ e'`
- [ ] **Principal Types**: Inference produces most general types

### Effect System

- [ ] **Effect Soundness**: Declared effects bound actual effects
- [ ] **Handler Correctness**: Handlers correctly discharge effects
- [ ] **Effect Inference**: Inferred effects are sound and minimal

### Memory Safety

- [ ] **Ownership Uniqueness**: Each value has one owner
- [ ] **Borrow Safety**: No aliasing of mutable references
- [ ] **Lifetime Soundness**: References don't outlive referents

### Session Types (Duet)

- [ ] **Session Fidelity**: Communication follows protocol
- [ ] **Deadlock Freedom**: Well-typed sessions don't deadlock

### Agent Calculus (Ensemble)

- [ ] **Agent Safety**: Agents respect type interfaces
- [ ] **Orchestration Termination**: Goals are achieved

## Verification Status

| Property | Paper Proof | Coq | Lean | Tests |
|----------|-------------|-----|------|-------|
| Type Preservation | ✅ | 🔲 | 🔲 | ✅ |
| Progress | ✅ | 🔲 | 🔲 | ✅ |
| Effect Soundness | ✅ | 🔲 | 🔲 | 🔲 |
| Ownership Safety | ✅ | 🔲 | 🔲 | ✅ |
| Session Fidelity | ✅ | 🔲 | 🔲 | 🔲 |
| Agent Safety | ✅ | 🔲 | 🔲 | 🔲 |

Legend: ✅ Complete, 🔲 TODO

## Contribution Guide

To contribute mechanized proofs:

1. Choose a theorem from the paper proofs
2. Implement in Coq or Lean
3. Ensure compilation with CI
4. Submit PR with proof walkthrough
