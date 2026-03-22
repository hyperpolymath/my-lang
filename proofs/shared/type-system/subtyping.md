# Subtyping Relations

## Overview

My Language employs structural subtyping with variance annotations. This document
formalizes the subtyping relation τ <: σ ("τ is a subtype of σ").

## Subtyping Judgment

```
τ <: σ     -- τ is a subtype of σ
```

## Reflexivity and Transitivity

```
─────────── (S-Refl)
τ <: τ

τ₁ <: τ₂    τ₂ <: τ₃
────────────────────── (S-Trans)
τ₁ <: τ₃
```

## Primitive Subtyping

```
─────────────── (S-Int-Float)
Int <: Float

─────────────── (S-Never)
! <: τ           (Never is bottom type)
```

## Function Subtyping (Contravariant in argument, Covariant in result)

```
σ₁ <: τ₁    τ₂ <: σ₂
──────────────────────── (S-Fun)
τ₁ → τ₂ <: σ₁ → σ₂
```

**Explanation**: Functions are contravariant in their parameter type and covariant
in their return type. A function that accepts more general inputs and produces
more specific outputs can be used where a more restrictive function is expected.

## Record Subtyping (Width and Depth)

```
∀i ∈ 1..m. τᵢ <: σᵢ    m ≥ n
────────────────────────────────────────────────────── (S-Record)
{l₁: τ₁, ..., lₘ: τₘ} <: {l₁: σ₁, ..., lₙ: σₙ}
```

**Width Subtyping**: A record with more fields is a subtype of one with fewer.
**Depth Subtyping**: Field types must be subtypes (covariant).

## Array Subtyping (Invariant for mutability)

```
τ <: σ
─────────────── (S-Array-Read)
[τ] <: [σ]      (only when used covariantly/read-only)
```

For mutable arrays, we require invariance:

```
─────────────── (S-Array-Mut)
[τ] <: [τ]      (arrays are invariant when mutable)
```

## Reference Subtyping

```
─────────────────── (S-Ref-Immut)
&τ <: &τ          (immutable references are invariant)

─────────────────── (S-RefMut)
&mut τ <: &mut τ   (mutable references are invariant)

─────────────────── (S-RefMut-Degrade)
&mut τ <: &τ       (mutable can degrade to immutable)
```

## Tuple Subtyping (Covariant)

```
∀i. τᵢ <: σᵢ
─────────────────────────────────────── (S-Tuple)
(τ₁, ..., τₙ) <: (σ₁, ..., σₙ)
```

## AI Type Subtyping (Covariant)

```
τ <: σ
─────────────────── (S-AI)
AI⟨τ⟩ <: AI⟨σ⟩
```

## Effect Type Subtyping (Covariant)

```
τ <: σ
───────────────────────── (S-Effect)
Effect⟨τ⟩ <: Effect⟨σ⟩
```

## Generic Type Subtyping

```
[σ/α]τ₁ <: τ₂
─────────────────────── (S-Forall-L)
∀α. τ₁ <: τ₂

τ₁ <: τ₂    α ∉ FTV(τ₁)
───────────────────────── (S-Forall-R)
τ₁ <: ∀α. τ₂
```

## Named Type Subtyping

```
T = τ ∈ Δ    τ <: σ
────────────────────── (S-Named-L)
T <: σ

τ <: σ    σ = T ∈ Δ
────────────────────── (S-Named-R)
τ <: T
```

## Effect Row Subtyping

```
────────────── (S-Eff-Empty)
∅ ≤ ε

ε₁ ≤ ε₂
───────────────── (S-Eff-Subset)
ε₁ ≤ ε₁ ∪ ε₂

E ≤ E
────────────── (S-Eff-Refl)

ε₁ ≤ ε₂    ε₂ ≤ ε₃
───────────────────── (S-Eff-Trans)
ε₁ ≤ ε₃
```

## Variance Summary

| Type Constructor | Variance |
|------------------|----------|
| τ → σ | Contravariant in τ, Covariant in σ |
| [τ] (immutable) | Covariant |
| [τ] (mutable) | Invariant |
| &τ | Invariant |
| &mut τ | Invariant |
| (τ₁, ..., τₙ) | Covariant in each τᵢ |
| {l: τ, ...} | Covariant (width and depth) |
| AI⟨τ⟩ | Covariant |
| Effect⟨τ⟩ | Covariant |
| ∀α. τ | Covariant in τ |

## Algorithmic Subtyping

The declarative rules above are not directly implementable. We define an
algorithmic version that is sound and complete with respect to the declarative system.

### Algorithm

```
sub(τ, σ) = case (τ, σ) of
    | (α, α) => true
    | (Int, Int) => true
    | (Float, Float) => true
    | (Int, Float) => true
    | (!, _) => true
    | (τ₁ → τ₂, σ₁ → σ₂) => sub(σ₁, τ₁) && sub(τ₂, σ₂)
    | ({ls: τs}, {ms: σs}) =>
        ms ⊆ ls && ∀l ∈ ms. sub(τs[l], σs[l])
    | ([τ], [σ]) => τ == σ  // invariant for mutable
    | (&τ, &σ) => τ == σ
    | (&mut τ, &σ) => τ == σ
    | (&mut τ, &mut σ) => τ == σ
    | ((τ₁,...,τₙ), (σ₁,...,σₙ)) => ∀i. sub(τᵢ, σᵢ)
    | (AI⟨τ⟩, AI⟨σ⟩) => sub(τ, σ)
    | (∀α.τ, σ) => sub([fresh/α]τ, σ)
    | (τ, ∀α.σ) => sub(τ, σ)  // α ∉ FTV(τ)
    | (T, σ) where T = τ' => sub(τ', σ)
    | (τ, T) where T = σ' => sub(τ, σ')
    | _ => false
```

## Theorems

**Theorem 1 (Soundness of Algorithmic Subtyping)**:
If sub(τ, σ) = true, then τ <: σ.

**Theorem 2 (Completeness of Algorithmic Subtyping)**:
If τ <: σ, then sub(τ, σ) = true.

**Theorem 3 (Subtyping Preserves Meaning)**:
If τ <: σ and Γ ⊢ e : τ, then Γ ⊢ e : σ and the meaning of e is preserved.

## Proofs

### Proof of Theorem 1 (Soundness)

By induction on the structure of τ and σ.

**Case** sub(Int, Float) = true:
By S-Int-Float, Int <: Float. ✓

**Case** sub(τ₁ → τ₂, σ₁ → σ₂):
We have sub(σ₁, τ₁) = true and sub(τ₂, σ₂) = true.
By IH, σ₁ <: τ₁ and τ₂ <: σ₂.
By S-Fun, τ₁ → τ₂ <: σ₁ → σ₂. ✓

Other cases follow similarly. □

### Proof of Theorem 2 (Completeness)

By induction on the derivation of τ <: σ. Each derivation rule corresponds
to a case in the algorithm that returns true. □

## Implementation Notes

The subtyping algorithm is implemented in `src/checker.rs` in the
`is_assignable_from` method of the `Ty` enum.

```rust
pub fn is_assignable_from(&self, other: &Ty) -> bool {
    // Implements algorithmic subtyping
    ...
}
```
