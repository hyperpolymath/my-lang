# Denotational Semantics

This directory contains the denotational semantics for My Language, providing
a mathematical model that assigns meaning to programs independent of evaluation strategy.

## Contents

1. [Domain Theory](domains.md) - CPO and domain constructions
2. [Type Interpretation](type-interpretation.md) - Types as domains
3. [Expression Semantics](expression-semantics.md) - Semantic function
4. [Effect Semantics](effect-denotation.md) - Monadic effect interpretation
5. [Adequacy](adequacy.md) - Correspondence to operational semantics

## Overview

Denotational semantics interprets programs as mathematical objects:

- **Types** → Domains (complete partial orders)
- **Expressions** → Continuous functions
- **Effects** → Monads
- **Programs** → Fixed points

## Semantic Function

The interpretation function:

```
⟦−⟧ : Expr → Env → Domain
```

maps expressions to their meanings in an environment.

## Key Properties

1. **Compositionality**: ⟦e₁ op e₂⟧ = ⟦op⟧(⟦e₁⟧, ⟦e₂⟧)
2. **Adequacy**: ⟦e⟧ = ⟦e'⟧ iff e ≃ e' (observational equivalence)
3. **Full Abstraction**: ⟦e⟧ = ⟦e'⟧ iff e ≅ e' (contextual equivalence)

## Semantic Domains

| Type | Domain |
|------|--------|
| Int | Z⊥ (integers with bottom) |
| Float | R⊥ (reals with bottom) |
| Bool | B⊥ = {⊥, tt, ff} |
| τ → σ | [⟦τ⟧ → ⟦σ⟧] (continuous functions) |
| AI⟨τ⟩ | AI(⟦τ⟧) (AI monad) |

## Theorems

- **Theorem 1 (Soundness)**: Operational evaluation matches denotation
- **Theorem 2 (Adequacy)**: Denotational equality implies observational equality
- **Theorem 3 (Continuity)**: All definable functions are Scott-continuous
- **Theorem 4 (Fixed Point)**: Recursive definitions have least fixed points
