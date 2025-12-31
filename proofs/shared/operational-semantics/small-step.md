# Small-Step Operational Semantics

## Overview

Small-step semantics defines evaluation as a sequence of atomic reduction steps.
This style is well-suited for reasoning about:
- Evaluation order
- Stuck states (type errors at runtime)
- Non-termination
- Interleaving in concurrent settings

## Reduction Relation

```
e ⟶ e'    -- e reduces to e' in one step
e ⟶* e'   -- e reduces to e' in zero or more steps
```

## Values

Values are fully evaluated expressions that cannot reduce further:

```
v ::=
    | n                              -- Integer
    | f                              -- Float
    | s                              -- String
    | true | false                   -- Boolean
    | ()                             -- Unit
    | λx: τ. e                       -- Function
    | {l₁ = v₁, ..., lₙ = vₙ}       -- Record
    | [v₁, ..., vₙ]                  -- Array
    | loc                            -- Memory location
```

## Evaluation Contexts

Evaluation contexts define where reduction can occur (call-by-value):

```
E ::=
    | □                              -- Hole
    | E e                            -- Function position
    | v E                            -- Argument position
    | let x: τ = E in e              -- Let binding
    | if E then e₁ else e₂           -- Condition
    | E.l                            -- Record field access
    | {l₁=v₁,...,lᵢ=E,...}          -- Record construction
    | [v₁,...,vᵢ,E,...]             -- Array construction
    | E ⊕ e                          -- Left operand
    | v ⊕ E                          -- Right operand
    | &E | &mut E                    -- Reference creation
    | *E                             -- Dereference
    | match E { arms }               -- Scrutinee
    | ai K { ..., l: E, ... }        -- AI body fields
```

## Context Rule

```
e ⟶ e'
─────────────── (E-Context)
E[e] ⟶ E[e']
```

## Core Reduction Rules

### Function Application

```
───────────────────────────── (E-Beta)
(λx: τ. e) v ⟶ [v/x]e
```

### Let Binding

```
───────────────────────────── (E-Let)
let x: τ = v in e ⟶ [v/x]e
```

### Conditionals

```
───────────────────────────────── (E-If-True)
if true then e₁ else e₂ ⟶ e₁

───────────────────────────────── (E-If-False)
if false then e₁ else e₂ ⟶ e₂
```

### Pattern Matching

```
match(v, pᵢ) = σ
─────────────────────────────────────────── (E-Match)
match v { p₁ => e₁, ..., pₙ => eₙ } ⟶ σ(eᵢ)
```

where match(v, p) returns substitution σ if pattern p matches value v:

```
match(v, x) = [v/x]
match(v, _) = []
match(n, n) = []
match(C(v₁,...,vₙ), C(p₁,...,pₙ)) = match(v₁,p₁) ∪ ... ∪ match(vₙ,pₙ)
```

### Record Operations

```
─────────────────────────────────────── (E-Record-Field)
{..., l = v, ...}.l ⟶ v
```

### Array Operations

```
0 ≤ i < n
───────────────────────────────────── (E-Array-Index)
[v₀, ..., vₙ₋₁][i] ⟶ vᵢ
```

### Arithmetic Operations

```
n = n₁ + n₂
────────────────────── (E-Add-Int)
n₁ + n₂ ⟶ n

f = f₁ + f₂
────────────────────── (E-Add-Float)
f₁ + f₂ ⟶ f

n = n₁ - n₂
────────────────────── (E-Sub)
n₁ - n₂ ⟶ n

n = n₁ * n₂
────────────────────── (E-Mul)
n₁ * n₂ ⟶ n

n₂ ≠ 0    n = n₁ / n₂
──────────────────────── (E-Div)
n₁ / n₂ ⟶ n

s = s₁ ++ s₂
────────────────────── (E-Concat)
s₁ + s₂ ⟶ s
```

### Comparison Operations

```
────────────────────────── (E-Eq-True)
v == v ⟶ true

v₁ ≠ v₂
────────────────────────── (E-Eq-False)
v₁ == v₂ ⟶ false

n₁ < n₂
────────────────────────── (E-Lt-True)
n₁ < n₂ ⟶ true

n₁ ≥ n₂
────────────────────────── (E-Lt-False)
n₁ < n₂ ⟶ false
```

### Logical Operations

```
────────────────────────── (E-And-True)
true && e ⟶ e

────────────────────────── (E-And-False)
false && e ⟶ false

────────────────────────── (E-Or-True)
true || e ⟶ true

────────────────────────── (E-Or-False)
false || e ⟶ e

────────────────────────── (E-Not-True)
!true ⟶ false

────────────────────────── (E-Not-False)
!false ⟶ true
```

### Unary Operations

```
────────────────────── (E-Neg-Int)
-n ⟶ -ₘₐₜₕn

────────────────────── (E-Neg-Float)
-f ⟶ -ₘₐₜₕf
```

## Stateful Reduction

For stateful operations, we extend configurations with a store:

```
⟨e, σ⟩ ⟶ ⟨e', σ'⟩
```

### Reference Operations

```
loc fresh
─────────────────────────────── (E-Ref)
⟨ref v, σ⟩ ⟶ ⟨loc, σ[loc ↦ v]⟩

σ(loc) = v
─────────────────────────────── (E-Deref)
⟨!loc, σ⟩ ⟶ ⟨v, σ⟩

──────────────────────────────────── (E-Assign)
⟨loc := v, σ⟩ ⟶ ⟨(), σ[loc ↦ v]⟩
```

## AI Operation Reduction

AI operations are handled by the AI runtime:

```
result = AI_RUNTIME.query(prompt, model)
───────────────────────────────────────────────────── (E-AI-Query)
ai query { prompt: s, model: M } ⟶ result

result = AI_RUNTIME.verify(input, constraint)
────────────────────────────────────────────────────── (E-AI-Verify)
ai verify { input: v, constraint: s } ⟶ result

result = AI_RUNTIME.generate(prompt)
────────────────────────────────────────────────────── (E-AI-Generate)
ai generate { prompt: s } ⟶ result

result = AI_RUNTIME.embed(text)
────────────────────────────────────── (E-AI-Embed)
ai embed(s) ⟶ result

result = AI_RUNTIME.classify(input, categories)
───────────────────────────────────────────────────────────────── (E-AI-Classify)
ai classify { input: v, categories: [...] } ⟶ result
```

## Sequence Reduction

```
────────────────────── (E-Seq)
v; e ⟶ e
```

## Block Reduction

```
────────────────────────────────────────── (E-Block)
{ let x₁ = v₁; ...; let xₙ = vₙ; e } ⟶ [v₁/x₁,...,vₙ/xₙ]e
```

## Lambda Evaluation

Lambdas are already values:

```
───────────────────────────── (Value)
λx: τ. e is a value
```

## Determinism Theorem

**Theorem (Determinism)**: For pure expressions (no effects), if e ⟶ e₁ and e ⟶ e₂,
then e₁ = e₂.

**Proof**: By case analysis on the reduction rules. Each rule has non-overlapping
premises, and evaluation contexts uniquely decompose expressions. □

## Progress and Preservation

See [Type Soundness](../type-system/soundness.md) for proofs that:
1. Well-typed terms make progress (don't get stuck)
2. Reduction preserves types

## Normalization

**Definition**: An expression e is **normalizing** if there exists v such that e ⟶* v.

**Theorem (Strong Normalization for Pure Terms)**: All pure, well-typed terms
without recursion are strongly normalizing.

**Proof Sketch**: By logical relations argument, using the types as measure. □

## Implementation Correspondence

The small-step semantics corresponds to the `step` function that would be
used in a step-by-step evaluator. The current tree-walking interpreter in
`src/interpreter.rs` implements the equivalent big-step semantics for efficiency.
