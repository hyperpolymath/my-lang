# Type Inference Algorithm

## Overview

My Language employs **bidirectional type inference** with **constraint-based unification**,
extending the classical Algorithm W (Damas-Milner) to handle effects, AI types, and subtyping.

## Algorithm Structure

The inference algorithm has four phases:

1. **Constraint Generation**: Traverse the AST, generating type constraints
2. **Constraint Solving**: Unify constraints to find most general solution
3. **Effect Inference**: Infer effect annotations from operations
4. **Generalization**: Generalize let-bound expressions

## Type Variables and Substitutions

### Type Variables

```
Type Variable: α, β, γ, ...
Effect Variable: ρ, σ, ...

Fresh variable generation:
fresh() → α where α ∉ current context
```

### Substitutions

```
Substitution: θ = [τ₁/α₁, ..., τₙ/αₙ]

Application:
θ(α) = τ if [τ/α] ∈ θ, else α
θ(Int) = Int
θ(τ₁ → τ₂) = θ(τ₁) → θ(τ₂)
θ(AI⟨τ⟩) = AI⟨θ(τ)⟩
θ(∀α. τ) = ∀α. θ(τ) where α ∉ dom(θ)

Composition:
(θ₁ ∘ θ₂)(τ) = θ₁(θ₂(τ))
```

## Constraint Language

```
Constraint: C ::=
    | τ₁ ≐ τ₂                 -- Equality constraint
    | τ₁ ≤ τ₂                 -- Subtype constraint
    | ε₁ ⊆ ε₂                 -- Effect subset
    | C₁ ∧ C₂                 -- Conjunction
    | ∃α. C                   -- Existential
    | true                    -- Trivial
```

## Constraint Generation

### Judgment Form

```
Γ ⊢ e : τ ⇝ C ! ε
```

Reads: Under environment Γ, expression e has type τ, generating constraints C
with effects ε.

### Rules

```
────────────────────────────────── (CG-Lit-Int)
Γ ⊢ n : Int ⇝ true ! ∅

────────────────────────────────── (CG-Lit-String)
Γ ⊢ s : String ⇝ true ! ∅

x: τ ∈ Γ
────────────────────────────────── (CG-Var)
Γ ⊢ x : inst(τ) ⇝ true ! ∅

where inst(∀α₁...αₙ. τ) = [β₁/α₁,...,βₙ/αₙ]τ with fresh βᵢ

Γ, x: α ⊢ e : τ ⇝ C ! ε    α fresh
────────────────────────────────────────────── (CG-Abs)
Γ ⊢ λx. e : α → τ ⇝ C ! ∅

Γ ⊢ e₁ : τ₁ ⇝ C₁ ! ε₁
Γ ⊢ e₂ : τ₂ ⇝ C₂ ! ε₂
α fresh
───────────────────────────────────────────────────────────── (CG-App)
Γ ⊢ e₁ e₂ : α ⇝ C₁ ∧ C₂ ∧ (τ₁ ≐ τ₂ → α) ! ε₁ ∪ ε₂

Γ ⊢ e₁ : τ₁ ⇝ C₁ ! ε₁
Γ, x: gen(τ₁, Γ) ⊢ e₂ : τ₂ ⇝ C₂ ! ε₂
────────────────────────────────────────────────── (CG-Let)
Γ ⊢ let x = e₁ in e₂ : τ₂ ⇝ C₁ ∧ C₂ ! ε₁ ∪ ε₂

where gen(τ, Γ) = ∀(FTV(τ) \ FTV(Γ)). τ

Γ ⊢ e₁ : τ₁ ⇝ C₁ ! ε₁
Γ ⊢ e₂ : τ₂ ⇝ C₂ ! ε₂
Γ ⊢ e₃ : τ₃ ⇝ C₃ ! ε₃
────────────────────────────────────────────────────────────────────── (CG-If)
Γ ⊢ if e₁ then e₂ else e₃ : τ₂ ⇝ C₁ ∧ C₂ ∧ C₃ ∧ (τ₁ ≐ Bool) ∧ (τ₂ ≐ τ₃) ! ε₁ ∪ ε₂ ∪ ε₃

Γ ⊢ e : τ ⇝ C ! ε
l: τₗ ∈ fields(τ) or α fresh
────────────────────────────────────────────────── (CG-Field)
Γ ⊢ e.l : τₗ ⇝ C ∧ (τ has field l: τₗ) ! ε

Γ ⊢ prompt : τ ⇝ C ! ε
────────────────────────────────────────────────────────────── (CG-AI-Query)
Γ ⊢ ai query { prompt: prompt } : AI⟨String⟩ ⇝ C ∧ (τ ≐ String) ! ε ∪ AI
```

## Constraint Solving (Unification)

### Unification Algorithm

```
unify : (τ, τ) → Option⟨Substitution⟩

unify(α, τ) =
    if α = τ then Some([])
    else if α ∈ FTV(τ) then None  -- Occurs check
    else Some([τ/α])

unify(τ, α) = unify(α, τ)

unify(Int, Int) = Some([])
unify(Float, Float) = Some([])
unify(String, String) = Some([])
unify(Bool, Bool) = Some([])

unify(τ₁ → τ₂, σ₁ → σ₂) =
    θ₁ ← unify(τ₁, σ₁)?
    θ₂ ← unify(θ₁(τ₂), θ₁(σ₂))?
    Some(θ₂ ∘ θ₁)

unify([τ], [σ]) = unify(τ, σ)

unify({l₁: τ₁, ..., lₙ: τₙ}, {l₁: σ₁, ..., lₙ: σₙ}) =
    θ ← foldM unify' [] [(τ₁,σ₁), ..., (τₙ,σₙ)]
    Some(θ)

unify(AI⟨τ⟩, AI⟨σ⟩) = unify(τ, σ)

unify(_, _) = None  -- Types don't match
```

### Constraint Solving

```
solve : Constraint → Option⟨Substitution⟩

solve(true) = Some([])

solve(τ₁ ≐ τ₂) = unify(τ₁, τ₂)

solve(C₁ ∧ C₂) =
    θ₁ ← solve(C₁)?
    θ₂ ← solve(θ₁(C₂))?
    Some(θ₂ ∘ θ₁)

solve(∃α. C) = solve(C)  -- α is implicitly existential

solve(ε₁ ⊆ ε₂) =
    if ε₁ ⊆ ε₂ then Some([])
    else None

solve(τ₁ ≤ τ₂) = -- Subtyping constraint
    if τ₁ <: τ₂ then Some([])
    else unify(τ₁, τ₂)  -- Fall back to equality
```

## Principal Types

### Definition

A type τ is **principal** for expression e under Γ if:
1. Γ ⊢ e : τ
2. For all σ such that Γ ⊢ e : σ, there exists θ such that σ = θ(τ)

### Theorem (Principal Types)

If constraint generation succeeds and solving produces θ, then θ(τ) is a
principal type for e.

**Proof**: By showing that:
1. θ(τ) is a valid type for e (soundness)
2. θ(τ) is at least as general as any other valid type (completeness)

The proof follows the structure of the Damas-Milner proof, extended for
our additional type constructors. □

## Generalization

```
gen(τ, Γ) = ∀α₁...αₙ. τ
where {α₁, ..., αₙ} = FTV(τ) \ FTV(Γ)
```

### Value Restriction

To ensure type safety with mutable references, we only generalize
let-bindings of **syntactic values**:

```
isValue(λx. e) = true
isValue(n) = true
isValue(s) = true
isValue({...}) = true
isValue([...]) = true
isValue(_) = false

gen(τ, Γ, e) =
    if isValue(e) then ∀(FTV(τ) \ FTV(Γ)). τ
    else τ
```

## Effect Inference

Effects are inferred bottom-up during constraint generation:

```
inferEffects(e) = case e of
    | n, s, true, false, () => ∅
    | x => ∅
    | λx. e => ∅  -- Effects captured in function body, not creation
    | e₁ e₂ => inferEffects(e₁) ∪ inferEffects(e₂)
    | let x = e₁ in e₂ => inferEffects(e₁) ∪ inferEffects(e₂)
    | if e₁ then e₂ else e₃ => inferEffects(e₁) ∪ inferEffects(e₂) ∪ inferEffects(e₃)
    | ai query {...} => AI
    | ai verify {...} => AI
    | ai generate {...} => AI
    | ai embed(e) => AI ∪ inferEffects(e)
    | print(e) => IO ∪ inferEffects(e)
    | read_file(e) => IO ∪ inferEffects(e)
    | get() => State
    | put(e) => State ∪ inferEffects(e)
    | throw(e) => Exception ∪ inferEffects(e)
    | await(e) => Async ∪ inferEffects(e)
    | handle e with H => inferEffects(e) \ handledEffect(H)
```

## Complexity Analysis

### Time Complexity

- Constraint generation: O(n) where n = |AST|
- Unification: O(n · α(n)) with path compression (nearly linear)
- Solving: O(n · m) where m = number of constraints
- Overall: O(n²) in worst case, O(n · log n) typical

### Space Complexity

- O(n) for AST and constraint storage
- O(m) for substitution

## Decidability

**Theorem (Decidability)**: Type inference for My Language is decidable.

**Proof**: The constraint language is equivalent to first-order unification,
which is decidable. Effect inference adds only finite set operations. □

## Soundness and Completeness

**Theorem (Inference Soundness)**: If the algorithm produces τ for expression e,
then Γ ⊢ e : τ according to the typing rules.

**Theorem (Inference Completeness)**: If Γ ⊢ e : σ according to the typing rules,
then the algorithm produces some τ such that σ is an instance of τ.

## TODO: Implementation Details

- [ ] Extend for higher-rank polymorphism
- [ ] Add constraint simplification optimizations
- [ ] Implement error message generation from failed constraints
