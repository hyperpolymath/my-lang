# First-Class AI Integration in Programming Languages

**Technical Report**

## Abstract

This paper presents a formal treatment of AI as a first-class computational
effect in programming languages. We introduce the AI effect type, typed prompt
templates, AI-aware contracts, and the Newtonian spectrum of specialized agents.
Our approach ensures type safety, effect tracking, and composable AI operations.

## 1. Introduction

Traditional integration of AI into programming languages treats AI services
as black-box external APIs. This approach suffers from:

1. **Untyped returns**: AI outputs are typically strings without structure
2. **Implicit effects**: AI calls are not tracked in the type system
3. **No composition**: Combining AI operations lacks formal semantics
4. **Missing contracts**: AI behavior is unconstrained

We address these issues by treating AI as a first-class effect with proper
typing, effect tracking, and algebraic composition.

## 2. The AI Effect

### 2.1 AI Effect Type

```
AI : Effect
AI ⊆ Effects

Typing rule:
Γ ⊢ ai query { prompt: e } : AI⟨String⟩ ! AI
```

### 2.2 AI Operation Types

| Operation | Type |
|-----------|------|
| ai query | String → AI⟨String⟩ |
| ai verify | (τ, String) → AI⟨Bool⟩ |
| ai generate | String → AI⟨String⟩ |
| ai embed | String → AI⟨[Float]⟩ |
| ai classify | (τ, [String]) → AI⟨String⟩ |

### 2.3 Effect Composition

AI effects compose with other effects:

```ml
fn process(file: String) -> Analysis with IO, AI {
    let content = read_file(file);  // IO
    ai query { prompt: content }    // AI
}
```

## 3. Typed Prompts

### 3.1 Prompt Templates

```ml
prompt summarize(text: String, max_words: Int = 100) -> String {
    "Summarize in at most {max_words} words: {text}"
}
```

### 3.2 Prompt Type Safety

Prompts are functions from parameters to AI results:

```
prompt P(x₁: τ₁, ..., xₙ: τₙ) -> τ
⟹ P : (τ₁, ..., τₙ) → AI⟨τ⟩
```

### 3.3 Structured Returns

```ml
prompt extract_entities(text: String) -> List<Entity> {
    """
    Extract named entities from: {text}
    Return as JSON array of {name, type} objects.
    """
}

// Type-safe invocation
let entities: AI<List<Entity>> = extract_entities!(text);
```

## 4. AI Contracts

### 4.1 AI Check Clauses

```ml
fn validate_email(s: String) -> Bool
where ai_check: "s is a syntactically valid email address"
{
    // Implementation
}
```

### 4.2 Semantics

```
⟦ai_check: description⟧ = AI_VERIFY(description, context)

AI_VERIFY invokes the AI model to verify the natural language property.
```

### 4.3 Soundness

AI checks provide probabilistic guarantees:

```
P(property holds | ai_check passes) ≥ 1 - ε
```

where ε depends on model reliability and specification clarity.

## 5. AI Model Configuration

### 5.1 Model Declarations

```ml
ai_model claude {
    provider: "anthropic"
    model: "claude-3-opus"
    temperature: 0.7
    max_tokens: 4096
    cache: true
}
```

### 5.2 Model Selection

```ml
fn creative_writing(prompt: String) -> String with AI {
    ai query {
        model: claude,
        prompt: prompt
    }
}
```

### 5.3 Model Abstraction

Models are first-class values:

```ml
fn with_model<T>(model: AIModel, f: fn() -> T with AI) -> T with AI {
    use_model(model, f)
}
```

## 6. The Newtonian Spectrum

### 6.1 Agent Decomposition

We decompose AI concerns into seven orthogonal agents:

1. **Red** (Performance): Optimization, hot paths
2. **Orange** (Concurrency): Async, parallelism
3. **Yellow** (Contracts): Types, verification
4. **Green** (Config): Configuration, schemas
5. **Blue** (Audit): Logging, tracing
6. **Indigo** (Comptime): Static evaluation
7. **Violet** (Governance): Policy, modes

### 6.2 Agent Types

```
Agent⟨σ, τ_in, τ_out⟩
```

where σ is the spectrum color.

### 6.3 Orchestration

```ml
orchestrate {
    code
    |> Yellow.verify      // Type check
    |> Red.optimize       // Performance
    |> Blue.audit         // Logging
}
```

## 7. Formal Properties

### 7.1 AI Type Safety

**Theorem**: Well-typed AI operations respect their declared return types.

Assuming the AI runtime maintains its contracts, if
`Γ ⊢ ai op { ... } : AI⟨τ⟩`, then the runtime result can be safely
interpreted as type τ.

### 7.2 Effect Soundness

**Theorem**: AI effects are precisely tracked.

If `Γ ⊢ e : τ ! ε` and AI ∉ ε, then e performs no AI operations.

### 7.3 Prompt Type Safety

**Theorem**: Typed prompts produce correctly-typed results.

If prompt P has type `(τ₁, ..., τₙ) → AI⟨τ⟩` and the AI model correctly
follows the prompt format, then `P!(v₁, ..., vₙ)` produces a value of type τ.

## 8. Implementation

### 8.1 AI Runtime Architecture

```
┌─────────────────────────────────────┐
│         Application Code            │
├─────────────────────────────────────┤
│          Type Checker               │
│    (validates AI type usage)        │
├─────────────────────────────────────┤
│          Effect System              │
│    (tracks AI effects)              │
├─────────────────────────────────────┤
│          AI Runtime                 │
│    ┌─────────┬─────────┐           │
│    │ Router  │ Cache   │           │
│    ├─────────┼─────────┤           │
│    │Anthropic│ OpenAI  │ ...       │
│    └─────────┴─────────┘           │
└─────────────────────────────────────┘
```

### 8.2 Caching Strategy

AI responses are cached based on:
- Prompt content (hashed)
- Model configuration
- Temperature (exact match for temperature=0)

### 8.3 Error Handling

```ml
fn safe_query(prompt: String) -> Result<String, AIError> with AI {
    try {
        Ok(ai query { prompt: prompt })
    } catch AIError(e) {
        Err(e)
    }
}
```

## 9. Related Work

### 9.1 Language-Level AI

- **Wolfram Language**: Built-in ML functions (untyped)
- **Julia**: Flux.jl for ML (library, not language)

### 9.2 AI Orchestration

- **LangChain**: Python library for LLM chains
- **Semantic Kernel**: Microsoft's AI SDK
- **AutoGPT**: Autonomous agents

### 9.3 Effect Systems for External Services

- **Bow** (Haskell): Effects for network calls
- **Koka**: Algebraic effects for async

## 10. Conclusion

First-class AI integration requires:
1. Typed AI operations
2. Effect tracking
3. Structured prompts
4. AI-aware contracts
5. Composable agents

Our approach provides all five within a coherent type-theoretic framework.
