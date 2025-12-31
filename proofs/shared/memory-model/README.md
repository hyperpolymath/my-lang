# Memory Model and Ownership System

This directory formalizes My Language's memory safety system based on ownership,
borrowing, and lifetimes, drawing from affine type theory and region-based memory management.

## Contents

1. [Ownership Semantics](ownership.md) - Formal ownership rules
2. [Borrowing Rules](borrowing.md) - Reference creation and validity
3. [Lifetime Calculus](lifetimes.md) - Lifetime inference and checking
4. [Memory Safety Proofs](safety-proofs.md) - Freedom from use-after-free
5. [Affine Types](affine.md) - Linear/affine resource management

## Overview

My Language guarantees memory safety without garbage collection through:

1. **Ownership**: Each value has exactly one owner
2. **Move Semantics**: Ownership transfers on assignment
3. **Borrowing**: Temporary access via references
4. **Lifetimes**: Static tracking of reference validity

## Core Principles

### Single Ownership

```
let x = value;      // x owns value
let y = x;          // ownership moves to y
// x is no longer valid
```

### Borrowing Rules

At any given time, you can have either:
- Any number of immutable references (&T), OR
- Exactly one mutable reference (&mut T)

Never both simultaneously.

### Lifetime Invariant

References never outlive their referents.

## Key Theorems

- **Theorem 1 (Memory Safety)**: Well-typed programs never access freed memory
- **Theorem 2 (Data Race Freedom)**: No data races under the borrowing discipline
- **Theorem 3 (Ownership Uniqueness)**: Each value has at most one owner at any time
- **Theorem 4 (Lifetime Soundness)**: References are valid for their declared lifetime

## Relationship to Other Work

- **Rust**: Similar ownership model
- **Linear Types**: Foundation in linear logic
- **Region-Based Memory**: Tofte-Talpin regions
- **Alias Types**: Walker-Morrisett calculus
