# Design-by-Contract Verification

## Overview

Solo supports design-by-contract (DbC) programming with preconditions,
postconditions, and invariants, extended with AI-assisted verification.

## Contract Syntax

```
Contract Clause:
W ::= pre: φ                    -- Precondition
    | post: φ                   -- Postcondition
    | invariant: φ              -- Loop/type invariant
    | ai_check: s               -- AI verification
    | ai_ensure: s              -- AI guarantee
    | modifies: [x]             -- Frame condition

Contract:
C ::= where W, W, ...

Function with Contract:
fn f(x₁: τ₁, ..., xₙ: τₙ) -> τ where C { e }
```

## Assertion Language

### Syntax

```
Assertion φ:
φ ::= true | false              -- Constants
    | e                          -- Boolean expression
    | φ ∧ φ | φ ∨ φ | ¬φ        -- Logical connectives
    | φ → φ | φ ↔ φ             -- Implication, equivalence
    | ∀x: τ. φ | ∃x: τ. φ      -- Quantifiers
    | old(e)                     -- Pre-state value
    | result                     -- Return value
    | e ≈ e                      -- Approximate equality (for floats)
    | valid(e)                   -- Pointer validity
    | fresh(e)                   -- Newly allocated
    | unchanged(e)               -- Not modified
```

### Semantics

```
⟦true⟧ρ,ρ' = ⊤
⟦false⟧ρ,ρ' = ⊥
⟦e⟧ρ,ρ' = eval(e, ρ')
⟦φ₁ ∧ φ₂⟧ρ,ρ' = ⟦φ₁⟧ρ,ρ' ∧ ⟦φ₂⟧ρ,ρ'
⟦old(e)⟧ρ,ρ' = eval(e, ρ)       -- Pre-state evaluation
⟦result⟧ρ,ρ' = ρ'(result)
⟦∀x: τ. φ⟧ρ,ρ' = ∀v ∈ ⟦τ⟧. ⟦φ⟧ρ[x↦v],ρ'[x↦v]
```

## Hoare Logic

### Hoare Triple

```
{P} S {Q}
```

Reads: If P holds before S, and S terminates, then Q holds after.

### Axioms and Rules

```
───────────────── (Skip)
{P} skip {P}

{P[e/x]} x := e {P}
────────────────────── (Assignment)

{P} S₁ {Q}    {Q} S₂ {R}
─────────────────────────── (Sequence)
{P} S₁; S₂ {R}

{P ∧ B} S₁ {Q}    {P ∧ ¬B} S₂ {Q}
───────────────────────────────────── (Conditional)
{P} if B then S₁ else S₂ {Q}

{P ∧ B} S {P}
──────────────────────── (While)
{P} while B do S {P ∧ ¬B}

P' → P    {P} S {Q}    Q → Q'
──────────────────────────────── (Consequence)
{P'} S {Q'}
```

### Function Call Rule

```
fn f(x₁: τ₁, ..., xₙ: τₙ) -> τ
where pre: P(x₁, ..., xₙ), post: Q(x₁, ..., xₙ, result)

P[e₁/x₁, ..., eₙ/xₙ]
─────────────────────────────────────────────────────────── (Call)
{P[e₁/x₁, ..., eₙ/xₙ]} y := f(e₁, ..., eₙ) {Q[e₁/x₁, ..., eₙ/xₙ, y/result]}
```

## Verification Condition Generation

### Weakest Precondition

```
wp(skip, Q) = Q

wp(x := e, Q) = Q[e/x]

wp(S₁; S₂, Q) = wp(S₁, wp(S₂, Q))

wp(if B then S₁ else S₂, Q) =
    (B → wp(S₁, Q)) ∧ (¬B → wp(S₂, Q))

wp(while B do S, Q) = I ∧ ∀x. (I ∧ B → wp(S, I)) ∧ (I ∧ ¬B → Q)
    where I is loop invariant

wp(f(e₁,...,eₙ), Q) =
    P[eᵢ/xᵢ] ∧ ∀r. Q_post[eᵢ/xᵢ, r/result] → Q[r/result]
```

### Strongest Postcondition

```
sp(skip, P) = P

sp(x := e, P) = ∃v. P[v/x] ∧ x = e[v/x]

sp(S₁; S₂, P) = sp(S₂, sp(S₁, P))
```

## AI-Assisted Verification

### AI Check Clauses

```ml
fn validate_email(s: String) -> Bool
where ai_check: "string is a valid email format"
{
    // Implementation
}
```

### Semantics of AI Checks

```
⟦ai_check: s⟧ρ,ρ' = AI_VERIFY(s, ρ, ρ')

AI_VERIFY invokes the AI model to check if the natural language
condition s holds given the pre-state ρ and post-state ρ'.
```

### AI Guarantee Clauses

```ml
fn summarize(text: String) -> String
where ai_ensure: "output is a coherent summary of input"
{
    ai generate { prompt: "Summarize: {text}" }
}
```

### Verification Strategy

1. **Static verification**: Check standard contracts with SMT solver
2. **AI verification**: Check AI clauses via model invocation
3. **Hybrid**: Combine static and AI results

```
verify(f) =
    let vc = generate_vc(f)
    let smt_result = smt_check(vc.static_conditions)
    let ai_result = ai_check(vc.ai_conditions)
    combine(smt_result, ai_result)
```

## Soundness

### Theorem (Contract Soundness)

If all verification conditions are valid, then the function satisfies
its contract.

**Proof**: By the soundness of Hoare logic. The VCGen produces conditions
whose validity implies the Hoare triple {pre} body {post}. □

### Theorem (AI Check Soundness)

AI checks provide high-probability guarantees when:
1. The AI model is reliable for the domain
2. The natural language specification is unambiguous
3. The context is sufficient for verification

Note: AI checks are probabilistic, not absolute.

## Runtime Checking

### Assertion Compilation

```ml
fn checked_div(a: Int, b: Int) -> Int
where pre: b != 0, post: result * b == a
{
    a / b
}
```

Compiles to:

```rust
fn checked_div(a: i64, b: i64) -> i64 {
    assert!(b != 0, "precondition failed: b != 0");
    let result = a / b;
    assert!(result * b == a, "postcondition failed: result * b == a");
    result
}
```

### AI Runtime Checks

```rust
fn validated_summary(text: &str) -> String {
    let result = ai_generate(format!("Summarize: {}", text));
    let valid = ai_verify(&format!(
        "Is '{}' a coherent summary of '{}'?",
        result, text
    ));
    assert!(valid, "AI postcondition failed");
    result
}
```

## Frame Conditions

### Modifies Clause

```ml
fn update_balance(account: &mut Account, delta: Int)
where
    modifies: [account.balance],
    post: account.balance == old(account.balance) + delta
{
    account.balance = account.balance + delta;
}
```

### Frame Rule

```
{P} S {Q}    modifies(S) ∩ FV(R) = ∅
──────────────────────────────────────── (Frame)
{P * R} S {Q * R}
```

## Invariants

### Type Invariants

```ml
struct PositiveInt
where invariant: self.value > 0
{
    value: Int
}
```

### Loop Invariants

```ml
fn sum(arr: [Int]) -> Int {
    let mut total = 0;
    let mut i = 0;
    while i < len(arr)
    where invariant: total == sum(arr[0..i])
    {
        total = total + arr[i];
        i = i + 1;
    }
    total
}
```

## Examples

### Binary Search with Contract

```ml
fn binary_search(arr: [Int], target: Int) -> Option<Int>
where
    pre: is_sorted(arr),
    post: match result {
        Some(i) => arr[i] == target,
        None => !contains(arr, target)
    }
{
    let mut lo = 0;
    let mut hi = len(arr);

    while lo < hi
    where invariant:
        0 <= lo <= hi <= len(arr) &&
        !contains(arr[0..lo], target) &&
        !contains(arr[hi..len(arr)], target)
    {
        let mid = lo + (hi - lo) / 2;
        if arr[mid] == target {
            return Some(mid);
        } else if arr[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    None
}
```
