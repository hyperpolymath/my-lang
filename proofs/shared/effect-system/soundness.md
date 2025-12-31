# Effect Soundness Proofs

## Overview

Effect soundness ensures that the statically declared effects accurately describe
the effects that may occur during runtime execution.

## Effect Safety Theorem

**Theorem (Effect Soundness)**: If Γ ⊢ e : τ ! ε, then execution of e can only
perform effects in ε.

More formally: If Γ ⊢ e : τ ! ε and e ⟶* e' where e' performs effect E,
then E ∈ ε.

## Definitions

### Effect Observations

We extend the operational semantics to track effect occurrences:

```
Configuration: ⟨e, σ, μ⟩
where:
  e = expression
  σ = effect trace (list of performed effects)
  μ = memory/state
```

### Effectful Reduction

```
⟨e, σ, μ⟩ →ₑ ⟨e', σ', μ'⟩
```

means e reduces to e' with effects σ' and state μ'.

### Effect Observation Rules

```
─────────────────────────────────────────── (Eff-Pure)
⟨v, σ, μ⟩ →ₑ ⟨v, σ, μ⟩                       (values are stuck, no new effects)

───────────────────────────────────────────── (Eff-IO-Print)
⟨print(s), σ, μ⟩ →ₑ ⟨(), σ ++ [IO], μ⟩

─────────────────────────────────────────────────── (Eff-AI-Query)
⟨ai query {...}, σ, μ⟩ →ₑ ⟨result, σ ++ [AI], μ⟩

────────────────────────────────────────────────── (Eff-State-Get)
⟨get(), σ, μ⟩ →ₑ ⟨μ.state, σ ++ [State], μ⟩

──────────────────────────────────────────────────── (Eff-State-Put)
⟨put(v), σ, μ⟩ →ₑ ⟨(), σ ++ [State], μ[state ↦ v]⟩

e →ₑ e'    fresh_eff = ε
───────────────────────────────────────── (Eff-Context)
⟨E[e], σ, μ⟩ →ₑ ⟨E[e'], σ ++ ε, μ'⟩
```

## Effect Preservation Lemma

**Lemma (Effect Preservation)**: If Γ ⊢ e : τ ! ε and ⟨e, [], μ⟩ →ₑ* ⟨e', σ, μ'⟩,
then every effect in σ is contained in ε.

**Proof**: By induction on the length of the reduction sequence.

**Base case**: Zero steps, σ = [], and [] ⊆ ε trivially.

**Inductive case**: Suppose the claim holds for n steps and we have:
⟨e, [], μ⟩ →ₑⁿ ⟨eₙ, σₙ, μₙ⟩ →ₑ ⟨eₙ₊₁, σₙ₊₁, μₙ₊₁⟩

By IH, every effect in σₙ is in ε.

For the (n+1)th step, we case-analyze the rule used:

**Case Eff-IO-Print**: eₙ = print(s) for some s
- The reduction adds IO to σ
- By typing, Γ ⊢ print(s) : () ! IO
- Since this appears in e, IO ∈ ε by effect composition rules
- Thus σₙ₊₁ ⊆ ε ✓

**Case Eff-AI-Query**: eₙ = ai query {...}
- The reduction adds AI to σ
- By typing, AI expressions have type AI⟨τ⟩ ! AI
- Since this appears in e, AI ∈ ε
- Thus σₙ₊₁ ⊆ ε ✓

**Case Eff-State-Get, Eff-State-Put**: Similar reasoning for State effect.

**Case Eff-Context**: The effect comes from a subexpression, which is
part of e, so its effect is included in ε.

**Case Eff-Pure**: No effect added, σₙ₊₁ = σₙ ⊆ ε. ✓

□

## Effect Handler Soundness

**Theorem (Handler Correctness)**: If Γ ⊢ handle e with H : τ ! ε' and
Γ ⊢ e : τ ! ε ∪ E and H handles E, then ε' = ε (the handled effect E is removed).

**Proof**: By the semantics of effect handlers.

When an effect operation E.op is invoked:
1. Execution suspends at the operation
2. The handler H receives the operation and continuation
3. H may resume the continuation with a value
4. The effect E is not propagated past the handler

Thus the handled effect E is not in the resulting effect ε'. □

## Effect Inference Soundness

**Theorem (Inference Soundness)**: If the effect inference algorithm infers
effects ε for expression e under context Γ, then Γ ⊢ e : τ ! ε for some τ.

**Proof**: By induction on the structure of e.

**Case Literal**: Inferred effects = ∅, and literals have no effects. ✓

**Case Variable**: Inferred effects = ∅, and variables have no effects. ✓

**Case Application e₁ e₂**:
- Infer ε₁ for e₁ and ε₂ for e₂
- Combined effect = ε₁ ∪ ε₂
- By typing rule T-App, effects compose as union ✓

**Case Let binding**:
- Infer ε₁ for binding, ε₂ for body
- Combined = ε₁ ∪ ε₂ by T-Let ✓

**Case Effect operation E.op(...)**:
- Infer E as the effect
- This matches the effect signature ✓

**Case Handle**:
- Infer ε for body, E for handler
- Result = ε \ E (handled effect removed)
- Matches T-Handle rule ✓

□

## Effect Polymorphism Soundness

**Theorem (Effect Abstraction Safety)**: Effect-polymorphic functions preserve
effect soundness under any instantiation.

**Proof**: Let f : ∀ρ. τ₁ → τ₂ ! ρ and suppose we apply f to an argument
with effects ε₁. The instantiation [ε₁/ρ] gives f : τ₁ → τ₂ ! ε₁.

By preservation, the call f(x) has effects ε₁, which is exactly what was
declared by the instantiated type. □

## Corollaries

**Corollary 1 (Effect Determinism)**: If Γ ⊢ e : τ ! ∅ (pure), then e performs
no observable effects.

**Corollary 2 (Effect Compositionality)**: Effects of compound expressions
are exactly the union of component effects.

**Corollary 3 (Handler Completeness)**: A handled effect cannot escape its handler.

## TODO: Extensions

- [ ] Proof for concurrent effect interactions
- [ ] Formalization of effect handler resumption
- [ ] Effect row polymorphism proofs
- [ ] Mechanization in Agda
