# Type Soundness Proof

## Overview

Type soundness establishes that well-typed programs do not "go wrong" at runtime. We prove this via the standard technique of **progress** and **preservation** (also known as "subject reduction").

## Definitions

### Values

```
v ::=
    | n | f | s | true | false | ()      -- Literals
    | λx: τ. e                           -- Functions
    | {l₁ = v₁, ..., lₙ = vₙ}           -- Records
    | [v₁, ..., vₙ]                      -- Arrays
    | ref v                              -- References
    | loc                                -- Locations (runtime only)
```

### Evaluation Contexts

```
E ::=
    | □
    | E e | v E
    | let x: τ = E in e
    | if E then e else e
    | E.l
    | &E | &mut E | *E
    | E ⊕ e | v ⊕ E
    | {l₁ = v₁, ..., lᵢ = E, ...}
    | [v₁, ..., vᵢ, E, ...]
    | match E { arms }
    | ai K { ..., l: E, ... }
```

### Small-Step Operational Semantics

```
───────────────────────────────── (E-App)
(λx: τ. e) v ⟶ [v/x]e

───────────────────────────────── (E-Let)
let x: τ = v in e ⟶ [v/x]e

e₁ ⟶ e₁'
───────────────────────────────── (E-Context)
E[e₁] ⟶ E[e₁']

───────────────────────────────── (E-If-True)
if true then e₁ else e₂ ⟶ e₁

───────────────────────────────── (E-If-False)
if false then e₁ else e₂ ⟶ e₂

pᵢ matches v with σ
────────────────────────────────────────────── (E-Match)
match v { ..., pᵢ => eᵢ, ... } ⟶ σ(eᵢ)

───────────────────────────────── (E-Field)
{..., l = v, ...}.l ⟶ v

───────────────────────────────── (E-Arith)
n₁ ⊕ n₂ ⟶ n₃                     where n₃ = n₁ ⊕ᵢₙₜ n₂
```

## Canonical Forms Lemma

**Lemma 1 (Canonical Forms)**: If ∅ ⊢ v : τ ! ∅ and v is a value, then:

1. If τ = Int, then v = n for some integer n
2. If τ = Float, then v = f for some float f
3. If τ = String, then v = s for some string s
4. If τ = Bool, then v = true or v = false
5. If τ = (), then v = ()
6. If τ = τ₁ → τ₂, then v = λx: τ₁. e for some x, e
7. If τ = {l₁: τ₁, ..., lₙ: τₙ}, then v = {l₁ = v₁, ..., lₙ = vₙ}
8. If τ = [σ], then v = [v₁, ..., vₙ] where each vᵢ : σ
9. If τ = &σ, then v = ref v' for some v' : σ
10. If τ = AI⟨σ⟩, then v is an AI computation returning σ

**Proof**: By induction on the typing derivation. Each case follows directly from the typing rules, as only specific syntactic forms can have each type. □

## Progress Theorem

**Theorem 2 (Progress)**: If ∅ ⊢ e : τ ! ε, then either:
1. e is a value, or
2. There exists e' such that e ⟶ e'

**Proof**: By induction on the typing derivation.

**Case T-Var**: Cannot occur since Γ = ∅.

**Case T-Int, T-Float, T-String, T-Bool-True, T-Bool-False, T-Unit**:
e is a literal, hence a value.

**Case T-Abs-Check, T-Abs-Synth**: e = λx: τ. e', which is a value.

**Case T-App**: e = e₁ e₂ where:
- ∅ ⊢ e₁ : τ₁ → τ₂ ! ε₁
- ∅ ⊢ e₂ : τ₁ ! ε₂

By IH on e₁:
- If e₁ is not a value, then e₁ ⟶ e₁' and e₁ e₂ ⟶ e₁' e₂ by E-Context
- If e₁ is a value, by Canonical Forms, e₁ = λx: τ₁. e'

By IH on e₂:
- If e₂ is not a value, then e₂ ⟶ e₂' and (λx. e') e₂ ⟶ (λx. e') e₂'
- If e₂ is a value v, then (λx. e') v ⟶ [v/x]e' by E-App

**Case T-Let-Infer**: e = let x = e₁ in e₂ where:
- ∅ ⊢ e₁ : τ₁ ! ε₁

By IH on e₁:
- If e₁ is not a value, then e₁ ⟶ e₁' and let x = e₁ in e₂ ⟶ let x = e₁' in e₂
- If e₁ is a value v, then let x = v in e₂ ⟶ [v/x]e₂ by E-Let

**Case T-If**: e = if e₁ then e₂ else e₃ where:
- ∅ ⊢ e₁ : Bool ! ε₁

By IH on e₁:
- If e₁ is not a value, we can step e₁
- If e₁ is a value, by Canonical Forms, e₁ ∈ {true, false}
  - If e₁ = true, e ⟶ e₂ by E-If-True
  - If e₁ = false, e ⟶ e₃ by E-If-False

**Case T-Match**: e = match e₀ { p₁ => e₁, ..., pₙ => eₙ } where:
- ∅ ⊢ e₀ : τ ! ε₀

By IH on e₀:
- If e₀ is not a value, we can step it
- If e₀ is a value v, exhaustiveness checking ensures some pᵢ matches v

**Case T-Field**: e = e₀.l where:
- ∅ ⊢ e₀ : {l₁: τ₁, ..., lₙ: τₙ} ! ε

By IH on e₀:
- If e₀ is not a value, we can step it
- If e₀ is a value, by Canonical Forms, e₀ = {l₁ = v₁, ..., lₙ = vₙ}
  - Since l ∈ {l₁, ..., lₙ}, e ⟶ vᵢ where lᵢ = l

**Case T-BinOp-Int**: e = e₁ ⊕ e₂ where both have type Int.
By IH, either subexpressions step, or both are values (integers),
and we can compute the result.

**Case T-AI-Query, T-AI-Verify, T-AI-Generate, etc.**:
AI expressions are handled by the AI runtime, which provides progress
through external evaluation. See [AI Semantics](../ai-semantics/README.md).

Other cases follow similarly. □

## Preservation Theorem

**Theorem 3 (Preservation / Subject Reduction)**:
If Γ ⊢ e : τ ! ε and e ⟶ e', then Γ ⊢ e' : τ ! ε' where ε' ≤ ε.

**Proof**: By induction on the typing derivation, with case analysis on the reduction rule used.

**Case E-App**: (λx: τ. e) v ⟶ [v/x]e

Given:
- Γ ⊢ (λx: τ. e) v : τ₂ ! ε₁ ∪ ε₂
- Γ ⊢ λx: τ. e : τ → τ₂ ! ε₁  (by inversion on T-App)
- Γ ⊢ v : τ ! ε₂

From T-Abs:
- Γ, x: τ ⊢ e : τ₂ ! ε₁

By the Substitution Lemma (Lemma 4):
- Γ ⊢ [v/x]e : τ₂ ! ε₁ ∪ ε₂ ✓

**Case E-Let**: let x: τ = v in e ⟶ [v/x]e

Given:
- Γ ⊢ let x: τ = v in e : τ₂ ! ε₁ ∪ ε₂
- Γ ⊢ v : τ ! ε₁
- Γ, x: τ ⊢ e : τ₂ ! ε₂

By Substitution Lemma:
- Γ ⊢ [v/x]e : τ₂ ! ε₂ ⊆ ε₁ ∪ ε₂ ✓

**Case E-If-True**: if true then e₁ else e₂ ⟶ e₁

Given:
- Γ ⊢ if true then e₁ else e₂ : τ ! ε₁ ∪ ε₂ ∪ ε₃
- Γ ⊢ e₁ : τ ! ε₂

Since ε₂ ≤ ε₁ ∪ ε₂ ∪ ε₃, preservation holds. ✓

**Case E-Match**: match v { ..., pᵢ => eᵢ, ... } ⟶ σ(eᵢ)

Given:
- Γ ⊢ v : τ ! ε₀
- Γ ⊢ pᵢ : τ ⊣ Γᵢ  (pattern matching introduces bindings)
- Γ, Γᵢ ⊢ eᵢ : σ ! εᵢ
- Match produces substitution σ corresponding to Γᵢ

By Substitution Lemma applied to each binding in σ:
- Γ ⊢ σ(eᵢ) : σ ! εᵢ ✓

**Case E-Context**: E[e] ⟶ E[e'] where e ⟶ e'

By IH, if Γ ⊢ e : τ ! ε and e ⟶ e', then Γ ⊢ e' : τ ! ε'.
Context typing preserves types through evaluation contexts. ✓

**Case E-Field**: {..., l = v, ...}.l ⟶ v

Given:
- Γ ⊢ {l₁ = v₁, ..., lₙ = vₙ} : {l₁: τ₁, ..., lₙ: τₙ} ! ⋃ᵢεᵢ
- By T-Record inversion: Γ ⊢ vᵢ : τᵢ ! εᵢ for each i
- Field l corresponds to some lᵢ with type τᵢ

Thus Γ ⊢ v : τᵢ, which is the expected type of e.l ✓

Other cases follow similarly. □

## Substitution Lemma

**Lemma 4 (Substitution)**: If Γ, x: τ ⊢ e : σ ! ε and Γ ⊢ v : τ ! ε',
then Γ ⊢ [v/x]e : σ ! ε ∪ ε'.

**Proof**: By induction on the derivation of Γ, x: τ ⊢ e : σ ! ε.

**Case T-Var**: e = y

If y = x:
- [v/x]x = v
- Need: Γ ⊢ v : τ ! ε'
- Given: Γ ⊢ v : τ ! ε' ✓

If y ≠ x:
- [v/x]y = y
- y: σ ∈ Γ (since y ≠ x)
- Γ ⊢ y : σ ! ∅ by T-Var ✓

**Case T-Abs**: e = λy: τ'. e'
(assuming y ≠ x and y ∉ FV(v), renaming if necessary)

- Γ, x: τ, y: τ' ⊢ e' : σ' ! ε₁
- [v/x](λy: τ'. e') = λy: τ'. [v/x]e'
- By IH: Γ, y: τ' ⊢ [v/x]e' : σ' ! ε₁ ∪ ε'
- By T-Abs: Γ ⊢ λy: τ'. [v/x]e' : τ' → σ' ! ∅ ✓

**Case T-App**: e = e₁ e₂
- [v/x](e₁ e₂) = ([v/x]e₁) ([v/x]e₂)
- By IH: Γ ⊢ [v/x]e₁ : τ₁ → τ₂ and Γ ⊢ [v/x]e₂ : τ₁
- By T-App: Γ ⊢ ([v/x]e₁) ([v/x]e₂) : τ₂ ✓

Other cases follow similarly. □

## Effect Preservation

**Lemma 5 (Effect Preservation)**: If Γ ⊢ e : τ ! ε and e ⟶ e',
then the effects performed by the reduction step are bounded by ε.

**Proof**: By case analysis on the reduction relation. Most pure reductions (E-App, E-Let, E-If, etc.) perform no effects. Effect-producing reductions only occur within effect handlers, which discharge the effect from the type. □

## Type Soundness

**Theorem 6 (Type Soundness)**: If ∅ ⊢ e : τ ! ε, then either:
1. e diverges, or
2. e ⟶* v where ∅ ⊢ v : τ ! ∅

**Proof**: By Progress and Preservation, repeatedly applied.
- Progress ensures we can always take a step if not a value
- Preservation ensures the type is maintained
- The effect weakens (or stays same) with each step
- If evaluation terminates, we have a well-typed value □

## Corollaries

**Corollary 7 (No Stuck States)**: A well-typed closed term never gets stuck.

**Corollary 8 (Effect Safety)**: Effects declared in the type are an upper bound
on the effects that can actually occur.

**Corollary 9 (AI Type Safety)**: AI operations respect their declared return types
assuming the AI runtime maintains its contracts.

## TODO: Extensions

- [ ] Proof extension for affine types (Solo dialect)
- [ ] Proof extension for session types (Duet dialect)
- [ ] Proof extension for agent composition (Ensemble dialect)
- [ ] Mechanization in Coq or Lean
