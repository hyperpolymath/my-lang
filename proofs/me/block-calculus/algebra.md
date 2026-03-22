# Block Calculus: Algebraic Theory

## Overview

The block calculus provides an algebraic foundation for reasoning about
visual program equivalence, composition, and transformation.

## Algebraic Structure

### Block Monoid

Visual programs form a **monoid** under vertical composition:

```
(VisualProgram, ∘, ε)

Identity:
V ∘ ε = V
ε ∘ V = V

Associativity:
(V₁ ∘ V₂) ∘ V₃ = V₁ ∘ (V₂ ∘ V₃)
```

### Block Category

Blocks with typed ports form a **category**:

- **Objects**: Color signatures [C₁, ..., Cₙ]
- **Morphisms**: Blocks B : [C₁, ..., Cₘ] → [D₁, ..., Dₙ]
- **Composition**: Sequential block connection
- **Identity**: Wire blocks (pass-through)

```
Block morphisms:
B : Σ₁ → Σ₂

Composition:
B₁ : Σ₁ → Σ₂
B₂ : Σ₂ → Σ₃
───────────────
B₁ ; B₂ : Σ₁ → Σ₃

Identity:
id_Σ : Σ → Σ
```

### Monoidal Structure

Parallel composition gives a **monoidal category**:

```
Tensor product:
B₁ : Σ₁ → Σ₂
B₂ : Σ₃ → Σ₄
─────────────────────────
B₁ ⊗ B₂ : Σ₁ ⊗ Σ₃ → Σ₂ ⊗ Σ₄

Unit:
I : [] → []

Associativity:
(B₁ ⊗ B₂) ⊗ B₃ ≅ B₁ ⊗ (B₂ ⊗ B₃)

Unit laws:
B ⊗ I ≅ B ≅ I ⊗ B
```

## Block Equivalence

### Semantic Equivalence

Two visual programs are equivalent if they have the same denotation:

```
V₁ ≡ V₂ ⟺ ∀ρ,θ. ⟦V₁⟧ρ,θ = ⟦V₂⟧ρ,θ
```

### Structural Equivalence

Equivalences derivable from the algebra:

```
Identity:
V ∘ ε ≡ V
ε ∘ V ≡ V

Associativity:
(V₁ ∘ V₂) ∘ V₃ ≡ V₁ ∘ (V₂ ∘ V₃)

Parallel independence:
(V₁ ⊗ V₂) ∘ (V₃ ⊗ V₄) ≡ (V₁ ∘ V₃) ⊗ (V₂ ∘ V₄)
    when connections are independent
```

### Beta Equivalence

Function block reduction:

```
FnCallBlock(f, [e₁,...,eₙ]) ≡ [e₁/x₁,...,eₙ/xₙ]V
    where FnDefBlock(f, [x₁,...,xₙ], V) is in scope
```

### Let Equivalence

Variable substitution:

```
LetBlock(x, LitBlock(v), V) ≡ [v/x]V
    when x is used exactly once in V
```

## Rewrite Rules

### Simplification Rules

```
PrintBlock(LitBlock(s)) ∘ PrintBlock(LitBlock(t))
    ≡ PrintBlock(LitBlock(s ++ t))           (Output fusion)

LetBlock(x, E, VarBlock(x))
    ≡ E                                       (Identity let)

IfBlock(LitBlock(true), V₁, V₂)
    ≡ V₁                                      (True branch)

IfBlock(LitBlock(false), V₁, V₂)
    ≡ V₂                                      (False branch)

RepeatBlock(LitBlock(0), V)
    ≡ ε                                       (Zero iteration)

RepeatBlock(LitBlock(1), V)
    ≡ V                                       (Single iteration)
```

### Dead Code Elimination

```
LetBlock(x, E, V) ≡ V
    when x ∉ FV(V) and E is pure
```

### Common Subexpression

```
LetBlock(x, E, LetBlock(y, E, V))
    ≡ LetBlock(x, E, [x/y]V)
    when E is pure
```

## Block Transformations

### Inlining

```
inline(FnCallBlock(f, args), FnDefBlock(f, params, body)) =
    substitute(zip(params, args), body)
```

### Loop Unrolling

```
unroll(RepeatBlock(LitBlock(n), V)) =
    V ∘ V ∘ ... ∘ V   (n times)
    when n is small
```

### Block Fusion

```
fuse(PrintBlock(E₁) ∘ PrintBlock(E₂)) =
    PrintBlock(ConcatBlock(E₁, E₂))
```

## Confluence

### Theorem (Local Confluence)

The rewrite system is locally confluent: if V → V₁ and V → V₂ by
different rules, then there exists V₃ with V₁ →* V₃ and V₂ →* V₃.

**Proof**: By analysis of critical pairs. Each overlapping rewrite can be joined. □

### Theorem (Termination)

The simplification rules terminate on well-formed programs.

**Proof**: Define a measure μ(V) = (size(V), depth(V)). Each rule strictly
decreases μ in the lexicographic order. □

### Corollary (Confluence)

By Newman's lemma, local confluence + termination implies confluence.

## Categorical Semantics

### String Diagrams

Visual programs correspond to **string diagrams** in monoidal categories:

```
Block   ↔ Box in diagram
Wire    ↔ String in diagram
∘       ↔ Vertical composition
⊗       ↔ Horizontal composition
```

### Traced Monoidal Category

Loops are modeled via the **trace** operator:

```
Tr : Hom(A ⊗ U, B ⊗ U) → Hom(A, B)

RepeatBlock corresponds to iteration:
Tr^n(f) = f ; f ; ... ; f   (n times)
```

### Completeness

**Theorem**: The block calculus is complete for the equational theory of
traced monoidal categories with coproducts (for conditionals).

## Implementation

The block equivalence checker:

```
fn equivalent(v1: &VisualProgram, v2: &VisualProgram) -> bool {
    let nf1 = normalize(v1);
    let nf2 = normalize(v2);
    structural_equal(&nf1, &nf2)
}

fn normalize(v: &VisualProgram) -> VisualProgram {
    let mut current = v.clone();
    loop {
        match apply_rewrite(&current) {
            Some(next) => current = next,
            None => return current,
        }
    }
}
```
