# Type System Formal Specification

This directory contains the formal specification and proofs for My Language's type system.

## Contents

1. [Syntax](syntax.md) - Formal syntax definitions
2. [Typing Rules](typing-rules.md) - Inference rules for type judgments
3. [Type Soundness](soundness.md) - Type preservation and progress proofs
4. [Inference Algorithm](inference.md) - Bidirectional type inference
5. [Subtyping](subtyping.md) - Subtyping relations and variance

## Overview

My Language employs a static type system with the following features:

- **Hindley-Milner foundation** with extensions
- **Bidirectional type inference**
- **Effect-aware typing** with row polymorphism
- **AI-aware types** for first-class AI integration
- **Affine types** for resource management
- **Gradual typing** for dialect progression

## Key Theorems

- **Theorem 1 (Type Preservation)**: Well-typed terms reduce to well-typed terms
- **Theorem 2 (Progress)**: Well-typed terms are either values or can take a step
- **Theorem 3 (Effect Soundness)**: Declared effects bound actual effects
- **Theorem 4 (AI Type Safety)**: AI operations maintain type invariants
