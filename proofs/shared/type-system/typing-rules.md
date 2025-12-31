# Typing Rules

## Bidirectional Type Checking

My Language uses bidirectional type checking with two judgment forms:
- **Synthesis** (⇒): Infer the type from the term
- **Checking** (⇐): Check a term against an expected type

### Mode Switching

```
Γ ⊢ e ⇒ τ ! ε    τ ≡ σ
───────────────────────── (Sub)
Γ ⊢ e ⇐ σ ! ε

Γ ⊢ e ⇐ τ ! ε
─────────────── (Anno)
Γ ⊢ (e : τ) ⇒ τ ! ε
```

## Literal Rules

```
─────────────────── (T-Int)
Γ ⊢ n ⇒ Int ! ∅

─────────────────── (T-Float)
Γ ⊢ f ⇒ Float ! ∅

─────────────────── (T-String)
Γ ⊢ s ⇒ String ! ∅

─────────────────── (T-Bool-True)
Γ ⊢ true ⇒ Bool ! ∅

─────────────────── (T-Bool-False)
Γ ⊢ false ⇒ Bool ! ∅

─────────────────── (T-Unit)
Γ ⊢ () ⇒ () ! ∅
```

## Variable Rules

```
x: τ ∈ Γ
─────────────── (T-Var)
Γ ⊢ x ⇒ τ ! ∅
```

## Function Rules

```
Γ, x: τ₁ ⊢ e ⇐ τ₂ ! ε
──────────────────────────────── (T-Abs-Check)
Γ ⊢ λx. e ⇐ τ₁ → τ₂ ! ∅

Γ, x: τ ⊢ e ⇒ σ ! ε
────────────────────────────── (T-Abs-Synth)
Γ ⊢ λx: τ. e ⇒ τ → σ ! ∅

Γ ⊢ e₁ ⇒ τ₁ → τ₂ ! ε₁    Γ ⊢ e₂ ⇐ τ₁ ! ε₂
──────────────────────────────────────────── (T-App)
Γ ⊢ e₁ e₂ ⇒ τ₂ ! ε₁ ∪ ε₂
```

## Let Binding Rules

```
Γ ⊢ e₁ ⇒ τ₁ ! ε₁    Γ, x: τ₁ ⊢ e₂ ⇒ τ₂ ! ε₂
──────────────────────────────────────────────── (T-Let-Infer)
Γ ⊢ let x = e₁ in e₂ ⇒ τ₂ ! ε₁ ∪ ε₂

Γ ⊢ e₁ ⇐ τ ! ε₁    Γ, x: τ ⊢ e₂ ⇒ τ₂ ! ε₂
──────────────────────────────────────────────── (T-Let-Anno)
Γ ⊢ let x: τ = e₁ in e₂ ⇒ τ₂ ! ε₁ ∪ ε₂

Γ ⊢ e₁ ⇐ τ ! ε₁    Γ, x: τ (mut) ⊢ e₂ ⇒ τ₂ ! ε₂
──────────────────────────────────────────────────── (T-Let-Mut)
Γ ⊢ let mut x: τ = e₁ in e₂ ⇒ τ₂ ! ε₁ ∪ ε₂
```

## Conditional Rules

```
Γ ⊢ e₁ ⇐ Bool ! ε₁    Γ ⊢ e₂ ⇒ τ ! ε₂    Γ ⊢ e₃ ⇐ τ ! ε₃
────────────────────────────────────────────────────────────── (T-If)
Γ ⊢ if e₁ then e₂ else e₃ ⇒ τ ! ε₁ ∪ ε₂ ∪ ε₃
```

## Pattern Matching Rules

```
Γ ⊢ e ⇒ τ ! ε₀    ∀i. Γ ⊢ pᵢ : τ ⊣ Γᵢ    ∀i. Γ, Γᵢ ⊢ eᵢ ⇒ σ ! εᵢ
──────────────────────────────────────────────────────────────────── (T-Match)
Γ ⊢ match e { p₁ => e₁, ..., pₙ => eₙ } ⇒ σ ! ε₀ ∪ ⋃ᵢεᵢ
```

### Pattern Typing

```
─────────────────────── (P-Var)
Γ ⊢ x : τ ⊣ {x: τ}

─────────────────────── (P-Wild)
Γ ⊢ _ : τ ⊣ ∅

─────────────────────── (P-Lit-Int)
Γ ⊢ n : Int ⊣ ∅

C : (τ₁, ..., τₙ) → T    ∀i. Γ ⊢ pᵢ : τᵢ ⊣ Γᵢ
──────────────────────────────────────────────── (P-Ctor)
Γ ⊢ C(p₁, ..., pₙ) : T ⊣ Γ₁ ∪ ... ∪ Γₙ
```

## Record and Field Rules

```
∀i. Γ ⊢ eᵢ ⇒ τᵢ ! εᵢ
────────────────────────────────────────────── (T-Record)
Γ ⊢ {l₁ = e₁, ..., lₙ = eₙ} ⇒ {l₁: τ₁, ..., lₙ: τₙ} ! ⋃ᵢεᵢ

Γ ⊢ e ⇒ {l₁: τ₁, ..., lₙ: τₙ} ! ε    lᵢ ∈ {l₁, ..., lₙ}
────────────────────────────────────────────────────────── (T-Field)
Γ ⊢ e.lᵢ ⇒ τᵢ ! ε
```

## Array Rules

```
∀i. Γ ⊢ eᵢ ⇐ τ ! εᵢ
────────────────────────────────── (T-Array)
Γ ⊢ [e₁, ..., eₙ] ⇒ [τ] ! ⋃ᵢεᵢ

─────────────────────── (T-Array-Empty)
Γ ⊢ [] ⇒ [α] ! ∅          (α fresh)
```

## Reference Rules

```
Γ ⊢ e ⇒ τ ! ε
─────────────────── (T-Ref)
Γ ⊢ &e ⇒ &τ ! ε

Γ ⊢ e ⇒ τ ! ε    e is lvalue
─────────────────────────────── (T-RefMut)
Γ ⊢ &mut e ⇒ &mut τ ! ε

Γ ⊢ e ⇒ &τ ! ε
─────────────────── (T-Deref)
Γ ⊢ *e ⇒ τ ! ε

Γ ⊢ e ⇒ &mut τ ! ε
─────────────────── (T-DerefMut)
Γ ⊢ *e ⇒ τ ! ε
```

## Binary Operation Rules

```
Γ ⊢ e₁ ⇒ Int ! ε₁    Γ ⊢ e₂ ⇒ Int ! ε₂
─────────────────────────────────────────── (T-BinOp-Int)
Γ ⊢ e₁ ⊕ e₂ ⇒ Int ! ε₁ ∪ ε₂               where ⊕ ∈ {+, -, *, /}

Γ ⊢ e₁ ⇒ Float ! ε₁    Γ ⊢ e₂ ⇒ Float ! ε₂
───────────────────────────────────────────── (T-BinOp-Float)
Γ ⊢ e₁ ⊕ e₂ ⇒ Float ! ε₁ ∪ ε₂              where ⊕ ∈ {+, -, *, /}

Γ ⊢ e₁ ⇒ τ ! ε₁    Γ ⊢ e₂ ⇒ τ ! ε₂    τ numeric
──────────────────────────────────────────────────── (T-Compare)
Γ ⊢ e₁ ⋈ e₂ ⇒ Bool ! ε₁ ∪ ε₂                        where ⋈ ∈ {<, >, <=, >=}

Γ ⊢ e₁ ⇒ τ ! ε₁    Γ ⊢ e₂ ⇒ τ ! ε₂
──────────────────────────────────────── (T-Eq)
Γ ⊢ e₁ == e₂ ⇒ Bool ! ε₁ ∪ ε₂

Γ ⊢ e₁ ⇒ Bool ! ε₁    Γ ⊢ e₂ ⇒ Bool ! ε₂
──────────────────────────────────────────── (T-Logic)
Γ ⊢ e₁ ⊛ e₂ ⇒ Bool ! ε₁ ∪ ε₂                where ⊛ ∈ {&&, ||}

Γ ⊢ e₁ ⇒ String ! ε₁    Γ ⊢ e₂ ⇒ String ! ε₂
────────────────────────────────────────────── (T-Concat)
Γ ⊢ e₁ + e₂ ⇒ String ! ε₁ ∪ ε₂
```

## Unary Operation Rules

```
Γ ⊢ e ⇒ τ ! ε    τ numeric
─────────────────────────── (T-Neg)
Γ ⊢ -e ⇒ τ ! ε

Γ ⊢ e ⇒ Bool ! ε
─────────────────── (T-Not)
Γ ⊢ !e ⇒ Bool ! ε
```

## AI Expression Rules

```
Γ ⊢ prompt ⇒ String ! ε    M ∈ dom(Μ)
────────────────────────────────────────────── (T-AI-Query)
Γ ⊢ ai query { prompt: prompt, model: M } ⇒ AI⟨String⟩ ! ε ∪ AI

Γ ⊢ e ⇒ τ ! ε    Γ ⊢ constraint ⇒ String ! ε'
─────────────────────────────────────────────────── (T-AI-Verify)
Γ ⊢ ai verify { input: e, constraint: constraint } ⇒ AI⟨Bool⟩ ! ε ∪ ε' ∪ AI

Γ ⊢ prompt ⇒ String ! ε
─────────────────────────────────────── (T-AI-Generate)
Γ ⊢ ai generate { prompt: prompt } ⇒ AI⟨String⟩ ! ε ∪ AI

Γ ⊢ e ⇒ String ! ε
───────────────────────────────── (T-AI-Embed)
Γ ⊢ ai embed(e) ⇒ AI⟨[Float]⟩ ! ε ∪ AI

Γ ⊢ e ⇒ τ ! ε    ∀i. Γ ⊢ cᵢ ⇒ String ! ε'ᵢ
────────────────────────────────────────────────────────────────── (T-AI-Classify)
Γ ⊢ ai classify { input: e, categories: [c₁, ..., cₙ] } ⇒ AI⟨String⟩ ! ε ∪ AI ∪ ⋃ᵢε'ᵢ

P ∈ dom(Π)    ∀i. Γ ⊢ eᵢ ⇒ τᵢ ! εᵢ
────────────────────────────────────────── (T-Prompt-Invoke)
Γ ⊢ P!(e₁, ..., eₙ) ⇒ AI⟨String⟩ ! AI ∪ ⋃ᵢεᵢ

Γ ⊢ s ⇒ String ! ε
──────────────────────────────── (T-AI-Quick)
Γ ⊢ ai! { s } ⇒ AI⟨String⟩ ! ε ∪ AI
```

## Polymorphism Rules

```
Γ ⊢ e ⇒ ∀α. τ ! ε
────────────────────── (T-TApp)
Γ ⊢ e ⇒ [σ/α]τ ! ε

Γ ⊢ e ⇒ τ ! ε    α ∉ FV(Γ)
───────────────────────────── (T-TAbs)
Γ ⊢ e ⇒ ∀α. τ ! ε
```

## Effect Rules

```
Γ ⊢ e ⇒ τ ! ε    ε ≤ ε'
───────────────────────── (T-EffSub)
Γ ⊢ e ⇒ τ ! ε'

Γ ⊢ e ⇒ τ ! ε    E ∈ Σ
─────────────────────────── (T-EffOp)
Γ ⊢ op(e) ⇒ τ' ! ε ∪ E       where op: τ → τ' ∈ E

Γ ⊢ e ⇒ τ ! ε ∪ E    Γ ⊢ H handles E
─────────────────────────────────────── (T-Handle)
Γ ⊢ handle e with H ⇒ τ ! ε
```

## Subtyping Rules

See [subtyping.md](subtyping.md) for complete subtyping relation.

## Auxiliary Definitions

### Free Variables

```
FV(x) = {x}
FV(λx: τ. e) = FV(e) \ {x}
FV(e₁ e₂) = FV(e₁) ∪ FV(e₂)
FV(let x = e₁ in e₂) = FV(e₁) ∪ (FV(e₂) \ {x})
```

### Free Type Variables

```
FTV(Int) = FTV(Float) = FTV(String) = FTV(Bool) = ∅
FTV(α) = {α}
FTV(τ₁ → τ₂) = FTV(τ₁) ∪ FTV(τ₂)
FTV(∀α. τ) = FTV(τ) \ {α}
FTV(AI⟨τ⟩) = FTV(τ)
```

### Type Substitution

```
[σ/α]α = σ
[σ/α]β = β                    (α ≠ β)
[σ/α](τ₁ → τ₂) = [σ/α]τ₁ → [σ/α]τ₂
[σ/α](∀β. τ) = ∀β. [σ/α]τ    (β ∉ FTV(σ))
[σ/α]AI⟨τ⟩ = AI⟨[σ/α]τ⟩
```
