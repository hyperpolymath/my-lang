# Effect System Formal Specification

This directory contains the formal specification and proofs for My Language's effect system.

## Contents

1. [Effect Algebra](algebra.md) - Effect composition and ordering
2. [Effect Typing](typing.md) - Effect-aware type rules
3. [Effect Handlers](handlers.md) - Handler semantics and safety
4. [Effect Soundness](soundness.md) - Effect safety proofs
5. [Effect Inference](inference.md) - Effect inference algorithm

## Overview

My Language tracks computational effects in the type system, enabling:

- **Effect Polymorphism**: Functions generic over effects
- **Effect Handlers**: First-class effect interpretation
- **Effect Inference**: Automatic effect derivation
- **Effect Subtyping**: Safe effect weakening

## Core Effects

| Effect | Description | Operations |
|--------|-------------|------------|
| IO | Input/Output | print, read, write |
| AI | AI Operations | query, verify, generate, embed, classify |
| Network | Network I/O | http_get, http_post, connect |
| State⟨S⟩ | Mutable State | get, put, modify |
| Exception⟨E⟩ | Exceptions | throw, catch |
| Async | Asynchrony | await, spawn, select |

## Key Theorems

- **Theorem 1 (Effect Soundness)**: Declared effects bound actual effects
- **Theorem 2 (Handler Correctness)**: Handlers correctly discharge effects
- **Theorem 3 (Effect Inference Soundness)**: Inferred effects are sound
- **Theorem 4 (Effect Polymorphism Safety)**: Effect abstraction is safe
