# Operational Semantics

This directory contains the formal operational semantics for My Language.

## Contents

1. [Small-Step Semantics](small-step.md) - Reduction semantics
2. [Big-Step Semantics](big-step.md) - Natural semantics
3. [Abstract Machine](abstract-machine.md) - CEK machine specification
4. [Concurrent Semantics](concurrent.md) - Concurrency primitives
5. [AI Runtime Semantics](ai-runtime.md) - AI operation semantics

## Overview

My Language's operational semantics is defined using multiple complementary styles:

- **Small-step (structural)**: For reasoning about evaluation order and stuck states
- **Big-step (natural)**: For reasoning about termination and results
- **Abstract machine**: For implementation guidance

## Semantic Domains

```
Values:
v ::= n | f | s | true | false | ()
    | λx:τ.e
    | {l₁ = v₁, ..., lₙ = vₙ}
    | [v₁, ..., vₙ]
    | ref(v)
    | loc

Environments:
ρ ::= ∅ | ρ[x ↦ v]

Stores:
σ ::= ∅ | σ[loc ↦ v]

Continuations:
κ ::= halt
    | arg(e, ρ, κ)
    | fun(v, κ)
    | let(x, e, ρ, κ)
    | if(e₁, e₂, ρ, κ)
    | field(l, κ)
    | ...

Configurations:
C ::= ⟨e, ρ, σ, κ⟩
```

## Key Properties

1. **Determinism**: Pure expressions evaluate deterministically
2. **Type Safety**: Well-typed programs don't get stuck
3. **Effect Tracking**: Effects are precisely characterized
4. **Termination**: Pure total functions terminate

## Evaluation Strategies

My Language uses **call-by-value** evaluation:
- Arguments are evaluated before function application
- Left-to-right evaluation order for subexpressions
- Effects occur in program order

## Relation to Implementation

The abstract machine semantics directly corresponds to the tree-walking
interpreter in `src/interpreter.rs`.
