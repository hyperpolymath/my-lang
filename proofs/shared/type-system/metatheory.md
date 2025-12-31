# Metatheory

## Overview

This document establishes fundamental metatheoretic properties of My Language's
type system: decidability, complexity bounds, and relationships to other systems.

## Decidability Results

### Theorem 1 (Type Checking Decidability)

Given Γ, e, and τ, it is decidable whether Γ ⊢ e : τ.

**Proof**: The typing rules are syntax-directed (up to subsumption). Each rule
has finitely many premises, and we can enumerate all applicable rules for any
expression-type pair. Since expressions are finite and types in judgments are
either given or synthesized from subexpressions, the process terminates. □

### Theorem 2 (Type Inference Decidability)

Given Γ and e, it is decidable whether there exists τ such that Γ ⊢ e : τ.

**Proof**: By the constraint-based inference algorithm (see [inference.md](inference.md)).
Constraint generation terminates (finite AST), unification is decidable (Robinson 1965),
and effect inference over finite effect sets is decidable. □

### Theorem 3 (Subtyping Decidability)

Given τ and σ, it is decidable whether τ <: σ.

**Proof**: The subtyping rules are structural and syntax-directed. Recursive
subtyping checks strictly decrease type size. Named types can be unfolded
at most once (preventing infinite unfolding). □

## Complexity Analysis

### Type Checking Complexity

**Theorem 4**: Type checking is O(n · d) where n is expression size and d is
maximum type depth.

**Proof**: Each subexpression is visited once. At each node, we perform at most
O(d) subtyping checks. Subtyping is O(d) for structural types. □

### Type Inference Complexity

**Theorem 5**: Type inference is O(n² · α(n)) in the worst case, where α is
the inverse Ackermann function.

**Proof**:
- Constraint generation: O(n) — one pass over AST
- Constraint size: O(n) — at most one constraint per node
- Unification with union-find: O(n · α(n)) per constraint
- Total solving: O(n² · α(n))

In practice, most programs are O(n log n) due to structure sharing. □

### Effect Inference Complexity

**Theorem 6**: Effect inference is O(n) where n is expression size.

**Proof**: Effects are collected in a single bottom-up pass. Effect union
is O(|Effects|) which is bounded by a constant. □

## Relationship to Other Type Systems

### Hindley-Milner (System F₁)

My Language's core is an extension of Hindley-Milner:

| Feature | HM | My Language |
|---------|-----|-------------|
| Let-polymorphism | ✓ | ✓ |
| Type inference | ✓ | ✓ |
| Principal types | ✓ | ✓ |
| Higher-rank types | ✗ | Partial |
| Subtyping | ✗ | ✓ |
| Effects | ✗ | ✓ |
| AI types | ✗ | ✓ |

### System F (Second-order λ-calculus)

For explicit type annotations, we support System F features:

- Universal quantification (∀α. τ)
- Type application (e[τ])
- Impredicative instantiation (limited)

### System Fω (Higher-kinded types)

Planned extension for type-level computation:

- Type constructors (F : * → *)
- Higher-kinded polymorphism (∀F: * → *. ...)

### Linear/Affine Types

The Solo dialect extends with affine types:

- Affine type: τᵃ (use at most once)
- Linear type: τˡ (use exactly once)

See [../../../solo/affine-types/](../../../solo/affine-types/).

### Session Types

The Duet dialect incorporates session types:

- Session type: S ::= !τ.S | ?τ.S | S ⊕ S | S & S | end

See [../../../duet/session-types/](../../../duet/session-types/).

## Normalization

### Strong Normalization (Simply-Typed Fragment)

**Theorem 7**: All well-typed, pure, non-recursive terms are strongly normalizing.

**Proof**: By logical relations. Define a family of predicates Rτ such that:
- Rᵢₙₜ(e) iff e ⟶* n for some n
- Rₛₜᵣᵢₙ(e) iff e ⟶* s for some s
- Rτ₁→τ₂(e) iff for all v ∈ Rτ₁, e v ∈ Rτ₂

Show by induction that Γ ⊢ e : τ implies e ∈ Rτ. Since Rτ implies termination,
all well-typed terms terminate. □

### Weak Normalization (with Recursion)

**Theorem 8**: Well-typed terms with recursion are weakly normalizing if
all recursive calls are well-founded.

**Proof**: With a termination measure based on structural recursion or
explicit termination proofs. □

## Confluence

**Theorem 9 (Confluence)**: If e ⟶* e₁ and e ⟶* e₂, then there exists e₃
such that e₁ ⟶* e₃ and e₂ ⟶* e₃.

**Proof**: The reduction relation satisfies the diamond property for
non-overlapping redexes. For overlapping redexes (critical pairs), we show
they can be joined. □

## Subject Expansion

**Theorem 10 (Subject Expansion for Values)**: If e ⟶ v and Γ ⊢ v : τ,
then Γ ⊢ e : τ.

**Proof**: By case analysis on the reduction rule. Each rule's conclusion
type equals or is a supertype of the input type. □

Note: Subject expansion does not hold in general due to effects—a stuck
effectful term may not have the same type as a value.

## Parametricity

**Theorem 11 (Parametricity)**: Polymorphic functions satisfy the abstraction theorem.

For f : ∀α. τ[α] and any types A, B with relation R ⊆ A × B:
If (a, b) ∈ R, then (f[A](a), f[B](b)) ∈ τ[R]

This enables "free theorems" derivation.

**Example**: For f : ∀α. [α] → [α], parametricity implies f must be a
permutation—it cannot inspect or create elements.

## Effect Parametricity

**Theorem 12 (Effect Parametricity)**: Effect-polymorphic functions satisfy
an effect-level abstraction theorem.

For f : ∀ρ. (τ₁ → τ₂ ! ρ) → (τ₃ → τ₄ ! ρ):
The function f cannot observe which specific effects are performed, only that
effects matching ρ occur.

## Coherence

**Theorem 13 (Coherence)**: Type inference produces coherent typing derivations.

If the inference algorithm produces Γ ⊢ e : τ via derivation D, then any
other valid derivation D' for the same judgment is semantically equivalent.

**Proof**: By the uniqueness of principal types and the Church-Rosser property
of our type equivalence relation. □

## Type Safety Summary

Combining the theorems:

1. **Progress + Preservation** = Type Soundness
2. **Decidability** = Algorithmic type checking
3. **Principal Types** = Optimal inference
4. **Normalization** = Termination guarantees
5. **Parametricity** = Abstraction safety
6. **Coherence** = Deterministic semantics

## Open Problems

1. **Full higher-rank inference**: Complete inference for System F types
2. **Dependent type integration**: Gradual introduction of dependent types
3. **Effect inference optimality**: Minimizing inferred effect sets
4. **AI type refinement**: Learning tighter AI return types from data

## References

1. Damas, L., Milner, R. (1982). Principal type-schemes for functional programs.
2. Pierce, B. C. (2002). Types and Programming Languages.
3. Wadler, P. (1989). Theorems for free!
4. Wright, A. K., Felleisen, M. (1994). A syntactic approach to type soundness.
5. Plotkin, G., Pretnar, M. (2009). Handlers of algebraic effects.
