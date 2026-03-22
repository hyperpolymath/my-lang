# Visual Semantics Formal Model

## Abstract Syntax of Visual Programs

### Block Grammar

```
Visual Program:
V ::= ε                          -- Empty program
    | B                          -- Single block
    | V ∘ V                      -- Vertical composition (sequence)
    | V ⊗ V                      -- Horizontal composition (parallel)

Block:
B ::= PrintBlock(E)              -- Output block
    | InputBlock(x)              -- Input block
    | LetBlock(x, E, V)          -- Variable definition
    | IfBlock(E, V, V)           -- Conditional
    | RepeatBlock(E, V)          -- Loop
    | FnDefBlock(f, [x], V)      -- Function definition
    | FnCallBlock(f, [E])        -- Function call
    | TokenBlock(c, n)           -- Resource token (color c, count n)
    | ConsumeBlock(x)            -- Use resource (affine)

Expression Block:
E ::= LitBlock(v)                -- Literal value
    | VarBlock(x)                -- Variable reference
    | OpBlock(E, ⊕, E)           -- Binary operation
    | TokenRefBlock(x)           -- Token reference

Visual Type (Color):
C ::= Red | Orange | Yellow | Green | Blue | Purple | Gray

Connection:
K ::= Flow(p₁, p₂)               -- Data flow connection
    | Sequence(B₁, B₂)           -- Control flow connection
```

### Port System

Each block has **ports** for connections:

```
Port:
P ::= (block_id, port_type, index)

Port Type:
port_type ::= input | output | top | bottom

Port Signature:
Σ(B) = (inputs: [C], outputs: [C], top: Bool, bottom: Bool)
```

### Connection Validity

```
valid_connection(Flow(p₁, p₂)) =
    port_type(p₁) = output ∧
    port_type(p₂) = input ∧
    color(p₁) = color(p₂)

valid_connection(Sequence(B₁, B₂)) =
    has_bottom(B₁) ∧ has_top(B₂)
```

## Denotational Semantics

### Semantic Domains

```
Val = Int ∪ Float ∪ String ∪ Bool ∪ Token
Env = Var → Val
TokenEnv = Var → (Color × Nat × Status)
Status = Available | Consumed
```

### Semantic Function

```
⟦−⟧ : VisualProgram → Env → TokenEnv → (Env × TokenEnv × Output)

⟦ε⟧ρ,θ = (ρ, θ, [])

⟦PrintBlock(E)⟧ρ,θ =
    let v = ⟦E⟧ρ,θ in
    (ρ, θ, [v])

⟦LetBlock(x, E, V)⟧ρ,θ =
    let v = ⟦E⟧ρ,θ in
    ⟦V⟧(ρ[x ↦ v]),θ

⟦IfBlock(E, V₁, V₂)⟧ρ,θ =
    let b = ⟦E⟧ρ,θ in
    if b then ⟦V₁⟧ρ,θ else ⟦V₂⟧ρ,θ

⟦RepeatBlock(E, V)⟧ρ,θ =
    let n = ⟦E⟧ρ,θ in
    iterate(n, λ(ρ',θ'). ⟦V⟧ρ',θ', (ρ, θ))

⟦TokenBlock(c, n)⟧ρ,θ =
    let x = fresh_var() in
    (ρ[x ↦ Token(c, n)], θ[x ↦ (c, n, Available)], [])

⟦ConsumeBlock(x)⟧ρ,θ =
    require θ(x) = (c, n, Available)
    (ρ, θ[x ↦ (c, n-1, if n=1 then Consumed else Available)], [])

⟦V₁ ∘ V₂⟧ρ,θ =
    let (ρ₁, θ₁, o₁) = ⟦V₁⟧ρ,θ in
    let (ρ₂, θ₂, o₂) = ⟦V₂⟧ρ₁,θ₁ in
    (ρ₂, θ₂, o₁ ++ o₂)
```

### Expression Semantics

```
⟦LitBlock(n)⟧ρ,θ = n
⟦LitBlock(s)⟧ρ,θ = s
⟦VarBlock(x)⟧ρ,θ = ρ(x)
⟦OpBlock(E₁, +, E₂)⟧ρ,θ = ⟦E₁⟧ρ,θ + ⟦E₂⟧ρ,θ
⟦TokenRefBlock(x)⟧ρ,θ =
    require θ(x).status = Available
    ρ(x)
```

## Visual Type System

### Block Types as Colors

Each block has input/output colors:

```
typeof : Block → ([Color], [Color])

typeof(PrintBlock(E)) = ([typeof_expr(E)], [])
typeof(InputBlock(x)) = ([], [Gray])
typeof(LetBlock(x, E, V)) = ([typeof_expr(E)] ++ inputs(V), outputs(V))
typeof(OpBlock(E₁, +, E₂)) = ([typeof_expr(E₁), typeof_expr(E₂)], [Blue])
typeof(TokenBlock(c, n)) = ([], [c])
typeof(ConsumeBlock(x)) = ([color_of(x)], [])
```

### Color Compatibility

```
Red     : Error states, exceptions
Orange  : Numbers (Int, Float)
Yellow  : Strings
Green   : Booleans
Blue    : Computed values
Purple  : Functions
Gray    : Polymorphic (any type)
```

### Type Checking as Connection Validity

A visual program is well-typed iff all connections are valid:

```
well_typed(V) = ∀(Flow(p₁, p₂) ∈ connections(V)).
    color(p₁) ⊑ color(p₂)
```

where ⊑ is color compatibility:
```
c ⊑ c       -- Same color
Gray ⊑ c    -- Gray is polymorphic
c ⊑ Gray
```

## Token System (Visual Affine Types)

### Token Semantics

Tokens are visual representations of affine resources:

```
Token:
- Has a color (resource type)
- Has a count (usage budget)
- Tracks consumption status

Token Rules:
1. Creation: TokenBlock creates n tokens of color c
2. Reference: TokenRefBlock reads without consuming
3. Consumption: ConsumeBlock decrements count
4. Exhaustion: When count reaches 0, token becomes Consumed
5. Error: Referencing Consumed token is an error
```

### Visual Token Tracking

```
┌─────────────────┐
│ Token: [🔵🔵🔵] │  ← 3 blue tokens available
└────────┬────────┘
         │
    ┌────▼────┐
    │  Use    │  ← Consume one token
    └────┬────┘
         │
┌────────▼────────┐
│ Token: [🔵🔵]   │  ← 2 blue tokens remaining
└─────────────────┘
```

### Token Invariants

**Invariant 1 (Token Conservation)**: Tokens are neither created nor destroyed
except through TokenBlock and ConsumeBlock.

**Invariant 2 (Single Consumption)**: Each token unit is consumed at most once.

**Invariant 3 (Visual Tracking)**: The visual display accurately reflects token state.

## Theorems

### Theorem 1 (Visual Soundness)

If a visual program V is well-formed (all connections valid), then its
translation to Solo is well-typed.

```
well_formed(V) ⟹ Γ ⊢ translate(V) : τ
```

**Proof**: By structural induction on V. Each block type corresponds to a
well-typed Solo construct, and connection validity ensures type compatibility. □

### Theorem 2 (Token Correctness)

The visual token system correctly implements affine types.

```
⟦ConsumeBlock(x)⟧ρ,θ is defined ⟺ θ(x).status = Available ∧ θ(x).count > 0
```

**Proof**: The semantics explicitly checks token availability before consumption. □

### Theorem 3 (Visual-Textual Correspondence)

For any visual program V:

```
⟦V⟧ = ⟦translate(V)⟧
```

The visual semantics matches the semantics of the translated Solo program.

**Proof**: By showing that translate preserves the denotational semantics
for each block constructor. □

## Translation to Solo

```
translate : VisualProgram → SoloProgram

translate(PrintBlock(E)) = println(translate_expr(E))
translate(InputBlock(x)) = let x = input()
translate(LetBlock(x, E, V)) = let x = translate_expr(E); translate(V)
translate(IfBlock(E, V₁, V₂)) =
    if translate_expr(E) { translate(V₁) } else { translate(V₂) }
translate(RepeatBlock(E, V)) =
    for _ in 0..translate_expr(E) { translate(V) }
translate(TokenBlock(c, n)) = let x = Resource::new(n)
translate(ConsumeBlock(x)) = x.consume()
translate(V₁ ∘ V₂) = translate(V₁); translate(V₂)
```
