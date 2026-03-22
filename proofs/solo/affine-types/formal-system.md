# Affine Type System

## Overview

Solo's affine type system ensures resources are used at most once,
preventing use-after-free, double-free, and resource leaks.

## Type Syntax

### Usage Qualifiers

```
Usage Qualifier:
q ::= ω        -- Unrestricted (can use any number of times)
    | 1        -- Linear (must use exactly once)
    | ?        -- Affine (can use at most once)
    | 0        -- Irrelevant (cannot use)

Qualified Type:
τᵠ ::= τ^ω    -- Unrestricted type
     | τ¹     -- Linear type
     | τ?     -- Affine type
     | τ⁰     -- Irrelevant type
```

### Type Grammar

```
τ ::= B                     -- Base type
    | τ₁ ⊸ τ₂               -- Linear function
    | τ₁ → τ₂               -- Unrestricted function
    | τ₁ ⊗ τ₂               -- Tensor (linear pair)
    | τ₁ × τ₂               -- Cartesian product
    | τ₁ ⊕ τ₂               -- Linear sum
    | τ₁ + τ₂               -- Sum
    | !τ                    -- Bang (unrestricted box)
    | ?τ                    -- Whynot (affine box)
    | ∀α. τ                 -- Polymorphism
    | α                     -- Type variable
```

## Typing Rules

### Context Splitting

Linear resources must be split between subexpressions:

```
Context Split:
Γ = Γ₁ ⊎ Γ₂

Split Rules:
∅ = ∅ ⊎ ∅

Γ = Γ₁ ⊎ Γ₂
────────────────────────── (Split-Lin)
Γ, x:¹τ = (Γ₁, x:¹τ) ⊎ Γ₂
         or Γ₁ ⊎ (Γ₂, x:¹τ)

Γ = Γ₁ ⊎ Γ₂
────────────────────────────────── (Split-Aff)
Γ, x:?τ = (Γ₁, x:?τ) ⊎ Γ₂
         or Γ₁ ⊎ (Γ₂, x:?τ)
         or Γ₁ ⊎ Γ₂            -- Can be dropped

Γ = Γ₁ ⊎ Γ₂
────────────────────────────────── (Split-Unr)
Γ, x:ωτ = (Γ₁, x:ωτ) ⊎ (Γ₂, x:ωτ)
```

### Variable Rules

```
─────────────────────── (T-Var-Lin)
x:¹τ ⊢ x : τ

x:?τ ∈ Γ    Γ affine-ok
─────────────────────────── (T-Var-Aff)
Γ ⊢ x : τ

x:ωτ ∈ Γ
─────────────────────── (T-Var-Unr)
Γ ⊢ x : τ
```

### Function Rules

```
Γ, x:¹τ₁ ⊢ e : τ₂
────────────────────────── (T-Lam-Lin)
Γ ⊢ λx. e : τ₁ ⊸ τ₂

Γ, x:ωτ₁ ⊢ e : τ₂
────────────────────────── (T-Lam-Unr)
Γ ⊢ λx. e : τ₁ → τ₂

Γ₁ ⊢ e₁ : τ₁ ⊸ τ₂    Γ₂ ⊢ e₂ : τ₁
Γ = Γ₁ ⊎ Γ₂
───────────────────────────────────── (T-App-Lin)
Γ ⊢ e₁ e₂ : τ₂
```

### Tensor Rules

```
Γ₁ ⊢ e₁ : τ₁    Γ₂ ⊢ e₂ : τ₂
Γ = Γ₁ ⊎ Γ₂
──────────────────────────────── (T-Tensor-Intro)
Γ ⊢ (e₁, e₂) : τ₁ ⊗ τ₂

Γ₁ ⊢ e₁ : τ₁ ⊗ τ₂    Γ₂, x:¹τ₁, y:¹τ₂ ⊢ e₂ : τ
Γ = Γ₁ ⊎ Γ₂
───────────────────────────────────────────────── (T-Tensor-Elim)
Γ ⊢ let (x, y) = e₁ in e₂ : τ
```

### Bang (!) Rules

```
Γ unrestricted
Γ ⊢ e : τ
──────────────── (T-Bang-Intro)
Γ ⊢ !e : !τ

Γ₁ ⊢ e₁ : !τ    Γ₂, x:ωτ ⊢ e₂ : σ
Γ = Γ₁ ⊎ Γ₂
──────────────────────────────────── (T-Bang-Elim)
Γ ⊢ let !x = e₁ in e₂ : σ
```

### Affine (?) Rules

```
Γ affine    Γ ⊢ e : τ
──────────────────────── (T-Affine-Intro)
Γ ⊢ ?e : ?τ

Γ₁ ⊢ e₁ : ?τ    Γ₂, x:?τ ⊢ e₂ : σ
Γ = Γ₁ ⊎ Γ₂
────────────────────────────────── (T-Affine-Elim)
Γ ⊢ let ?x = e₁ in e₂ : σ
```

## Resource Tracking

### Resource State

```
Resource State:
ρ : Var → {Available, Consumed, Dropped}

Transition Rules:
Available → Consumed   (on use)
Available → Dropped    (on scope exit, affine only)
```

### Resource Invariants

**Invariant 1 (Linear Consumption)**:
Linear resources transition from Available to Consumed exactly once.

**Invariant 2 (Affine Usage)**:
Affine resources transition to Consumed or Dropped, never both.

**Invariant 3 (No Resurrection)**:
Once Consumed or Dropped, a resource cannot become Available.

## Checkpoint/Rollback

### Syntax

```
checkpoint x in e           -- Create checkpoint
rollback to x               -- Restore to checkpoint
commit x                    -- Finalize (discard checkpoint)
```

### Typing

```
Γ₁ ⊢ e : τ    x fresh
───────────────────────────────────── (T-Checkpoint)
Γ ⊢ checkpoint x in e : τ ⊣ Checkpoint(x, Γ)

Checkpoint(x, Γ') in scope
───────────────────────────────────── (T-Rollback)
Γ ⊢ rollback to x : ⊥ ⊣ Γ'

Checkpoint(x, Γ') in scope
───────────────────────────────────── (T-Commit)
Γ ⊢ commit x : () ⊣ Γ \ Checkpoint(x)
```

### Semantics

```
⟦checkpoint x in e⟧ρ,σ =
    save_state(x, (ρ, σ))
    ⟦e⟧ρ,σ

⟦rollback to x⟧ρ,σ =
    let (ρ', σ') = restore_state(x)
    return with (ρ', σ')

⟦commit x⟧ρ,σ =
    discard_checkpoint(x)
    ((), ρ, σ)
```

## Soundness Theorems

### Theorem 1 (Affine Type Safety)

If Γ ⊢ e : τ and e evaluates without error, then each linear resource
in Γ is consumed exactly once, and each affine resource is consumed at
most once.

**Proof**: By induction on typing derivation.

**Case T-Var-Lin**: Resource x:¹τ is consumed by this use. No other
rule can use x (context splitting ensures uniqueness). ✓

**Case T-Tensor-Elim**: Resources in e₁ are consumed; x and y are
introduced for e₂ and must be consumed there. ✓

**Case T-App-Lin**: Context splits ensure argument resources go to e₂,
function resources to e₁. ✓

Other cases follow similarly. □

### Theorem 2 (No Use-After-Free)

Well-typed programs never access consumed resources.

**Proof**: Context splitting ensures each linear variable appears in
exactly one branch. Once consumed, it's removed from the context and
cannot be referenced. □

### Theorem 3 (No Double-Free)

Well-typed programs never consume a resource twice.

**Proof**: Linear context splitting is disjoint—each variable goes to
exactly one sub-derivation. □

### Theorem 4 (Checkpoint Consistency)

Rollback restores a consistent state where all resources match their
checkpoint state.

**Proof**: Checkpoint saves the complete resource state. Rollback
atomically restores it. No intermediate states are observable. □

## Translation to Core

### Erasure

Affine types can be erased for runtime:

```
erase(τ¹) = erase(τ)
erase(τ?) = erase(τ)
erase(τ ⊸ σ) = erase(τ) → erase(σ)
erase(τ ⊗ σ) = (erase(τ), erase(σ))
erase(!τ) = erase(τ)
```

### Runtime Checks (Debug Mode)

Optional runtime resource tracking for debugging:

```rust
struct Resource<T> {
    value: T,
    consumed: Cell<bool>,
}

impl<T> Resource<T> {
    fn consume(self) -> T {
        assert!(!self.consumed.get(), "double use of linear resource");
        self.consumed.set(true);
        self.value
    }
}
```

## Examples

### File Handling

```ml
fn safe_read(path: String) -> Result<String, Error>¹ {
    let handle¹ = open(path)?;     // Linear file handle
    let content = read(handle);    // handle consumed
    // handle no longer usable
    close_implicit();              // Implicit close via consumption
    Ok(content)
}
```

### Database Transaction

```ml
fn transfer(from: Account, to: Account, amount: Money)
where pre: from.balance >= amount
{
    checkpoint tx in {
        let from' = debit(from, amount)?;   // May fail
        let to' = credit(to, amount)?;      // May fail
        commit tx;
        (from', to')
    } handle Error => {
        rollback to tx;
        Err(TransferFailed)
    }
}
```
