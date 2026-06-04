<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Formal Verification Framework

This directory contains formal verification artifacts for My Language,
including mechanized proofs in Coq and Lean, and property specifications
for testing.

## Contents

1. [Coq](coq/) - Mechanized proofs in Coq
2. [Lean](lean/) - Mechanized proofs in Lean 4
3. [Properties](properties/) - Property-based test specifications

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
