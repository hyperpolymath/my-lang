# Effect Algebra

## Effect Syntax

```
ε ::=
    | ∅                    -- Pure (empty effect)
    | E                    -- Named effect
    | ε ∪ ε                -- Effect union
    | ρ                    -- Effect variable
```

### Named Effects

```
E ::=
    | IO                   -- Input/Output
    | AI                   -- AI operations
    | Network              -- Network I/O
    | State⟨τ⟩             -- Mutable state
    | Exception⟨τ⟩         -- Exceptions
    | Async                -- Asynchrony
```

## Effect Algebra Properties

The effect system forms a **bounded join-semilattice** with the following properties:

### Identity

```
ε ∪ ∅ = ε
∅ ∪ ε = ε
```

### Commutativity

```
ε₁ ∪ ε₂ = ε₂ ∪ ε₁
```

### Associativity

```
(ε₁ ∪ ε₂) ∪ ε₃ = ε₁ ∪ (ε₂ ∪ ε₃)
```

### Idempotence

```
ε ∪ ε = ε
```

### Bottom Element

```
∅ ≤ ε    for all ε
```

## Effect Ordering

The effect ordering ε₁ ≤ ε₂ ("ε₁ is at most ε₂") is defined as:

```
ε₁ ≤ ε₂  ⟺  ε₁ ∪ ε₂ = ε₂
```

### Ordering Rules

```
─────────── (E-Ord-Refl)
ε ≤ ε

ε₁ ≤ ε₂    ε₂ ≤ ε₃
────────────────────── (E-Ord-Trans)
ε₁ ≤ ε₃

─────────── (E-Ord-Bot)
∅ ≤ ε

─────────────────── (E-Ord-Union-L)
ε₁ ≤ ε₁ ∪ ε₂

─────────────────── (E-Ord-Union-R)
ε₂ ≤ ε₁ ∪ ε₂

ε₁ ≤ ε    ε₂ ≤ ε
─────────────────── (E-Ord-Union-Lub)
ε₁ ∪ ε₂ ≤ ε
```

## Effect Row Representation

Effects are represented as **effect rows** - unordered sets of effect labels:

```
Row ::= { E₁, E₂, ..., Eₙ }
      | { E₁, E₂, ..., Eₙ | ρ }    -- Open row with tail variable
```

### Row Operations

**Row Union**:
```
{ E₁, ..., Eₘ } ∪ { F₁, ..., Fₙ } = { E₁, ..., Eₘ, F₁, ..., Fₙ }
```
(duplicates removed)

**Row Containment**:
```
E ∈ { E₁, ..., Eₙ }  ⟺  ∃i. E = Eᵢ
```

**Row Difference**:
```
{ E₁, ..., Eₘ } \ E = { Eᵢ | Eᵢ ∈ {E₁,...,Eₘ} ∧ Eᵢ ≠ E }
```

## Effect Polymorphism

Effect variables enable effect-polymorphic functions:

```
∀ρ. (τ₁ → τ₂ ! ρ) → [τ₁] → [τ₂] ! ρ
```

This type describes `map`: a function that applies an effectful operation
to each element, producing the union of all effects.

### Effect Abstraction

```
Γ ⊢ e : τ ! ε    ρ ∉ FEV(Γ)
───────────────────────────── (E-Gen)
Γ ⊢ e : ∀ρ. τ ! ε
```

### Effect Instantiation

```
Γ ⊢ e : ∀ρ. τ ! ε
────────────────────── (E-Inst)
Γ ⊢ e : [ε'/ρ]τ ! [ε'/ρ]ε
```

## Effect Composition Laws

### Sequential Composition

```
Γ ⊢ e₁ : τ₁ ! ε₁    Γ ⊢ e₂ : τ₂ ! ε₂
───────────────────────────────────────── (E-Seq)
Γ ⊢ e₁; e₂ : τ₂ ! ε₁ ∪ ε₂
```

### Parallel Composition (for independent effects)

```
Γ ⊢ e₁ : τ₁ ! ε₁    Γ ⊢ e₂ : τ₂ ! ε₂    ε₁ ⊥ ε₂
──────────────────────────────────────────────────── (E-Par)
Γ ⊢ e₁ ∥ e₂ : τ₁ × τ₂ ! ε₁ ∪ ε₂
```

where ε₁ ⊥ ε₂ means the effects are non-interfering.

### Effect Masking

When an effect handler fully handles an effect:

```
Γ ⊢ e : τ ! ε ∪ E    Γ ⊢ H handles E
───────────────────────────────────────── (E-Mask)
Γ ⊢ handle e with H : τ ! ε
```

## Theorems

**Theorem 1 (Effect Lattice)**: The structure (Effects, ∪, ∅) forms a bounded
join-semilattice.

**Proof**:
1. ∪ is associative: by definition
2. ∪ is commutative: by set union
3. ∪ is idempotent: ε ∪ ε = ε by set properties
4. ∅ is the identity: ε ∪ ∅ = ε by set properties
□

**Theorem 2 (Effect Ordering is a Partial Order)**: The relation ≤ is reflexive,
antisymmetric, and transitive.

**Proof**:
1. Reflexive: ε ≤ ε by E-Ord-Refl
2. Antisymmetric: If ε₁ ≤ ε₂ and ε₂ ≤ ε₁, then ε₁ ∪ ε₂ = ε₂ and ε₂ ∪ ε₁ = ε₁,
   so ε₁ = ε₂ by commutativity
3. Transitive: By E-Ord-Trans
□

**Theorem 3 (Effect Row Equivalence)**: Effect rows with the same elements
(modulo ordering) are equivalent.

**Proof**: By the commutativity of set union. □

## Implementation Notes

Effects are represented in the implementation as sets of effect labels:

```rust
type Effects = HashSet<Effect>;

fn union(e1: &Effects, e2: &Effects) -> Effects {
    e1.union(e2).cloned().collect()
}

fn is_subeffect(e1: &Effects, e2: &Effects) -> bool {
    e1.is_subset(e2)
}
```
