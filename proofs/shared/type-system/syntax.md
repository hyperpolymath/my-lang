# Formal Syntax Definition

## Abstract Syntax

### Types

```
τ ::=
    | Int | Float | String | Bool | ()           -- Primitive types
    | τ → τ                                       -- Function type
    | τ × τ × ... × τ                            -- Tuple type
    | [τ]                                         -- Array type
    | {l₁: τ₁, ..., lₙ: τₙ}                      -- Record type
    | &τ | &mut τ                                 -- Reference types
    | AI⟨τ⟩                                       -- AI effect type
    | Effect⟨τ⟩                                   -- General effect type
    | ∀α. τ                                       -- Universal type
    | α                                           -- Type variable
    | T                                           -- Named type
    | τ where C                                   -- Constrained type
    | !                                           -- Never type
```

### Expressions

```
e ::=
    | x                                           -- Variable
    | n | f | s | true | false | ()              -- Literals
    | λx: τ. e                                   -- Lambda abstraction
    | e e                                         -- Application
    | let x: τ = e in e                          -- Let binding
    | let mut x: τ = e in e                      -- Mutable binding
    | if e then e else e                         -- Conditional
    | match e { p₁ => e₁, ..., pₙ => eₙ }       -- Pattern match
    | e.l                                         -- Field access
    | &e | &mut e                                -- Reference creation
    | *e                                          -- Dereference
    | e; e                                        -- Sequence
    | {l₁ = e₁, ..., lₙ = eₙ}                   -- Record literal
    | [e₁, ..., eₙ]                              -- Array literal
    | ai K { B }                                  -- AI expression
    | e!⟨e₁, ..., eₙ⟩                            -- Prompt invocation
```

### Patterns

```
p ::=
    | x                                           -- Variable pattern
    | _                                           -- Wildcard pattern
    | n | s | true | false                       -- Literal pattern
    | C(p₁, ..., pₙ)                             -- Constructor pattern
    | {l₁ = p₁, ..., lₙ = pₙ}                   -- Record pattern
```

### AI Keywords

```
K ::= query | verify | generate | embed | classify
    | optimize | test | infer | constrain | validate
```

### AI Body Items

```
B ::=
    | l: e                                        -- Field assignment
    | s                                           -- String literal
```

### Declarations

```
D ::=
    | fn f(x₁: τ₁, ..., xₙ: τₙ) → τ { e }       -- Function
    | struct T { l₁: τ₁, ..., lₙ: τₙ }          -- Struct
    | effect E { op₁: τ₁, ..., opₙ: τₙ }        -- Effect
    | ai_model M { A }                           -- AI model
    | prompt P { s }                             -- Prompt template
    | contract C { W }                           -- Contract
```

### Effects

```
ε ::=
    | ∅                                           -- Pure (no effects)
    | IO                                          -- I/O effect
    | AI                                          -- AI effect
    | Network                                     -- Network effect
    | State⟨τ⟩                                    -- State effect
    | Exception⟨τ⟩                                -- Exception effect
    | Async                                       -- Async effect
    | ε ∪ ε                                       -- Effect union
    | ρ                                           -- Effect variable
```

### Contracts

```
W ::=
    | pre: e                                      -- Precondition
    | post: e                                     -- Postcondition
    | invariant: e                               -- Loop/type invariant
    | ai_check: s                                 -- AI verification
    | ai_ensure: s                               -- AI guarantee
```

## Typing Environments

```
Γ ::= ∅ | Γ, x: τ                               -- Type environment
Δ ::= ∅ | Δ, T = τ                              -- Type definitions
Σ ::= ∅ | Σ, E: [op₁: τ₁, ..., opₙ: τₙ]        -- Effect signatures
Μ ::= ∅ | Μ, M: model_config                    -- AI model environment
Π ::= ∅ | Π, P: prompt_def                      -- Prompt environment
```

## Judgment Forms

| Judgment | Meaning |
|----------|---------|
| Γ ⊢ e : τ ! ε | Expression e has type τ with effects ε |
| Γ ⊢ e ⇒ τ ! ε | Expression e synthesizes type τ |
| Γ ⊢ e ⇐ τ ! ε | Expression e checks against type τ |
| Γ ⊢ p : τ ⊣ Γ' | Pattern p has type τ, binding variables in Γ' |
| Γ ⊢ D ok | Declaration D is well-formed |
| τ <: τ' | τ is a subtype of τ' |
| ε ≤ ε' | Effect ε is a sub-effect of ε' |
| τ ≡ τ' | Types τ and τ' are equivalent |

## Well-Formedness

### Type Well-Formedness

```
─────────────── (WF-Prim)
Δ ⊢ B wf           where B ∈ {Int, Float, String, Bool, ()}

Δ ⊢ τ₁ wf    Δ ⊢ τ₂ wf
──────────────────────── (WF-Fun)
Δ ⊢ τ₁ → τ₂ wf

Δ ⊢ τ wf
─────────────── (WF-Array)
Δ ⊢ [τ] wf

T ∈ dom(Δ)
─────────────── (WF-Named)
Δ ⊢ T wf

Δ ⊢ τ wf
─────────────── (WF-AI)
Δ ⊢ AI⟨τ⟩ wf

Δ, α wf ⊢ τ wf
─────────────── (WF-Forall)
Δ ⊢ ∀α. τ wf
```

## Notation Conventions

| Symbol | Meaning |
|--------|---------|
| τ, σ | Types |
| e | Expressions |
| x, y, z | Variables |
| α, β | Type variables |
| ρ | Effect variables |
| ε | Effect rows |
| Γ | Type environment |
| ⊢ | Turnstile (derives) |
| ! | Effect annotation separator |
| ⇒ | Synthesizes (inference direction) |
| ⇐ | Checks (checking direction) |
