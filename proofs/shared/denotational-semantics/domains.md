# Domain Theory Foundations

## Complete Partial Orders (CPOs)

### Definition

A **complete partial order** (CPO) is a partially ordered set (D, ⊑) such that:

1. D has a least element ⊥ (bottom)
2. Every ω-chain d₀ ⊑ d₁ ⊑ d₂ ⊑ ... has a least upper bound ⊔ᵢdᵢ

### Notation

```
d ⊑ e       -- d approximates e (d is less defined than e)
⊔S          -- Least upper bound of set S
⊥           -- Bottom (undefined/divergence)
```

## Basic Domains

### Flat Domains

A **flat domain** D⊥ lifts a set D by adding bottom:

```
D⊥ = D ∪ {⊥}

Ordering:
⊥ ⊑ d for all d ∈ D
d ⊑ d for all d ∈ D
d ⊑ e iff d = ⊥ or d = e
```

**Examples**:
- Z⊥ = {⊥, ..., -2, -1, 0, 1, 2, ...}
- B⊥ = {⊥, true, false}
- String⊥ = {⊥} ∪ String

### Product Domains

For domains D₁ and D₂:

```
D₁ × D₂ = {(d₁, d₂) | d₁ ∈ D₁, d₂ ∈ D₂}

(d₁, d₂) ⊑ (e₁, e₂) iff d₁ ⊑ e₁ and d₂ ⊑ e₂

⊥ = (⊥, ⊥)
⊔ᵢ(dᵢ, eᵢ) = (⊔ᵢdᵢ, ⊔ᵢeᵢ)
```

### Sum Domains

For domains D₁ and D₂:

```
D₁ + D₂ = {inl(d₁) | d₁ ∈ D₁} ∪ {inr(d₂) | d₂ ∈ D₂}

Ordering (coalesced sum):
inl(d) ⊑ inl(e) iff d ⊑ e
inr(d) ⊑ inr(e) iff d ⊑ e
⊥ = inl(⊥) = inr(⊥)  -- Identified bottoms
```

### Function Domains

For domains D and E:

```
[D → E] = {f : D → E | f is continuous}

f ⊑ g iff ∀d ∈ D. f(d) ⊑ g(d)

⊥ = λd. ⊥
(⊔ᵢfᵢ)(d) = ⊔ᵢ(fᵢ(d))
```

### Lifted Domains

For any domain D:

```
D⊥ = D + {⊥}

With strict ordering:
⊥ ⊑ d for all d
d ⊑ e iff d = ⊥ or d ⊑_D e
```

## Continuity

### Scott Continuity

A function f : D → E is **continuous** if:

1. **Monotonic**: d ⊑ e implies f(d) ⊑ f(e)
2. **Preserves lubs**: For every ω-chain {dᵢ}, f(⊔ᵢdᵢ) = ⊔ᵢf(dᵢ)

### Strict Functions

A function f : D → E is **strict** if f(⊥) = ⊥.

### Theorem: Composition Preserves Continuity

If f : D → E and g : E → F are continuous, then g ∘ f : D → F is continuous.

**Proof**: For any chain {dᵢ}:
```
(g ∘ f)(⊔ᵢdᵢ) = g(f(⊔ᵢdᵢ))       -- Definition of composition
              = g(⊔ᵢf(dᵢ))       -- f continuous
              = ⊔ᵢg(f(dᵢ))       -- g continuous
              = ⊔ᵢ(g ∘ f)(dᵢ)    -- Definition of composition
```
□

## Fixed Points

### Fixed Point Theorem (Kleene)

For any continuous f : D → D, the least fixed point exists and equals:

```
fix(f) = ⊔ₙ fⁿ(⊥)
       = ⊥ ⊔ f(⊥) ⊔ f(f(⊥)) ⊔ ...
```

**Proof**:
1. The sequence {fⁿ(⊥)} is a chain: fⁿ(⊥) ⊑ fⁿ⁺¹(⊥) by monotonicity
2. Let d = ⊔ₙ fⁿ(⊥)
3. f(d) = f(⊔ₙ fⁿ(⊥)) = ⊔ₙ f(fⁿ(⊥)) = ⊔ₙ fⁿ⁺¹(⊥) = d
4. For any fixed point e, ⊥ ⊑ e, so fⁿ(⊥) ⊑ fⁿ(e) = e for all n, hence d ⊑ e
□

### Application to Recursion

Recursive function definition:

```
let rec f = λx. body
```

is interpreted as:

```
⟦f⟧ = fix(λf. λx. ⟦body⟧)
```

## Domain Equations

### Solving Recursive Domain Equations

For equations like D = F(D), we use:

1. **Bilimit construction**: Find D as the bilimit of an approximating sequence
2. **Category theory**: D is an initial algebra of functor F

**Example**: Lists of integers

```
List(Z) ≅ 1 + (Z × List(Z))
```

Solved by:
```
D₀ = {⊥}
D₁ = 1 + (Z × D₀) = {⊥, nil} ∪ {cons(n, ⊥) | n ∈ Z}
D₂ = 1 + (Z × D₁) = ...
List(Z) = ⊔ₙ Dₙ
```

## Powerdomains

For modeling nondeterminism and effects:

### Hoare Powerdomain (Lower)

```
H(D) = {X ⊆ D | X is downward closed and has a lub}

X ⊑_H Y iff X ⊆ Y
```

Models angelic nondeterminism (success if any branch succeeds).

### Smyth Powerdomain (Upper)

```
S(D) = {X ⊆ D | X is upward closed and Scott-closed}

X ⊑_S Y iff Y ⊆ X
```

Models demonic nondeterminism (success only if all branches succeed).

### Plotkin Powerdomain (Convex)

```
P(D) = convex combinations of H(D) and S(D)
```

Models general nondeterminism.

## Domain Constructions for My Language

### Record Domains

```
⟦{l₁: τ₁, ..., lₙ: τₙ}⟧ = ⟦τ₁⟧ × ... × ⟦τₙ⟧
```

### Array Domains

```
⟦[τ]⟧ = [N → ⟦τ⟧⊥] × N⊥
```

(Functions from indices to elements, paired with length)

### AI Domains

```
AI(D) = Reader(Config) ∘ State(Cache) ∘ IO(D)
      = Config → Cache → IO(D × Cache)
```

Where:
- Config = AI model configuration
- Cache = Response cache
- IO = External computation monad

### Effect Domains

```
Eff(ε, D) = case ε of
    | ∅ => D                           -- Pure
    | IO => IO(D)                      -- I/O monad
    | State(S) => S → D × S            -- State monad
    | Exception(E) => D + E            -- Exception monad
    | ε₁ ∪ ε₂ => Eff(ε₁, Eff(ε₂, D))  -- Composed effects
```

## Theorems

### Theorem: CPO Category is Cartesian Closed

The category **CPO** of CPOs and continuous functions is Cartesian closed:
- Terminal object: 1 = {⊥, ()}
- Products: D × E
- Exponentials: [D → E]

This ensures that types can be interpreted compositionally.

### Theorem: Fixed Point Operators are Continuous

The operator fix : [D → D] → D is itself continuous.

### Theorem: Domain Isomorphism

If D and E are domains and f : D → E, g : E → D are continuous with
g ∘ f = id_D and f ∘ g = id_E, then D ≅ E.
