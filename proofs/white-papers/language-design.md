# My Language: A Progressive-Disclosure Language with First-Class AI Integration

**White Paper v1.0**

## Abstract

We present My Language (ML), a novel programming language designed around
progressive disclosure and first-class AI integration. ML features four
dialects—Me, Solo, Duet, and Ensemble—that progressively reveal language
complexity while maintaining semantic consistency. Key contributions include:
a unified type system spanning visual blocks to textual code, an effect system
that treats AI as a first-class computational effect, and the Newtonian spectrum
of seven specialized AI agents for orchestration.

## 1. Introduction

### 1.1 Motivation

Contemporary programming languages face a tension between accessibility and
expressiveness. Educational languages (Scratch, Logo) sacrifice power for
simplicity. Professional languages (Rust, Haskell) sacrifice accessibility
for expressiveness. We propose a third way: progressive disclosure within
a single coherent language.

Additionally, existing languages treat AI as an external service rather than
an integrated capability. We argue that modern languages should embrace AI
as a first-class citizen, with proper typing, effect tracking, and composition.

### 1.2 Contributions

1. **Progressive disclosure** via four dialects with consistent semantics
2. **Visual affine types** introducing resource management to young learners
3. **AI effect system** treating AI operations as tracked effects
4. **Session types** for human-AI collaboration protocols
5. **Agent calculus** for multi-agent AI orchestration
6. **Newtonian spectrum** partitioning AI concerns into seven agents

### 1.3 Paper Organization

Section 2 presents the four dialects. Section 3 covers the type system.
Section 4 describes the effect system. Section 5 details AI integration.
Section 6 discusses implementation. Section 7 evaluates the approach.
Section 8 surveys related work. Section 9 concludes.

## 2. The Four Dialects

### 2.1 Dialect Overview

| Dialect | Interface | Target | Core Concepts |
|---------|-----------|--------|---------------|
| Me | Visual blocks | Ages 8-12 | Sequence, variables, loops |
| Solo | Text | Ages 13+ | Affine types, contracts |
| Duet | Collaborative | — | Session types, pair programming |
| Ensemble | Orchestration | — | Agent calculus, spectrum |

### 2.2 Semantic Consistency

All dialects share a common semantic core. A well-typed Me program corresponds
to a well-typed Solo program:

```
∀ M : MeProgram. wellTyped(M) ⟹ wellTyped(lift(M))
```

### 2.3 Me: Visual Programming with Affine Intuition

Me introduces resource management through colored tokens:

- Tokens represent resources
- Using a resource consumes a token
- Running out of tokens prevents further use

This provides intuition for affine types without formal machinery.

### 2.4 Solo: Explicit Affine Types

Solo makes affine types explicit:

```ml
fn process(handle: FileHandle¹) -> Data {
    let content = read(handle);  // handle consumed
    parse(content)
}
```

The superscript ¹ indicates a linear resource.

### 2.5 Duet: Collaborative Programming

Duet introduces session-typed protocols for human-AI interaction:

```ml
session type CodeAssist = {
    request: recv Context, send Suggestion, loop,
    done: end
}
```

### 2.6 Ensemble: Multi-Agent Orchestration

Ensemble provides the Newtonian spectrum of seven agents:

```ml
task |> Red       // Optimize
     |> Yellow    // Verify
     |> Blue      // Audit
     |> output
```

## 3. Type System

### 3.1 Base Types

Standard primitive types with AI extensions:

```
τ ::= Int | Float | String | Bool | ()
    | τ → τ | [τ] | {l: τ, ...}
    | AI⟨τ⟩ | Effect⟨τ⟩
    | ∀α. τ | α
```

### 3.2 Affine Types

Resources that can be used at most once:

```
τᵃ ::= τ¹        -- Linear (exactly once)
     | τ?        -- Affine (at most once)
     | τω        -- Unrestricted
```

### 3.3 AI Types

AI operations return wrapped results:

```
ai query { prompt: s } : AI⟨String⟩
ai embed(text) : AI⟨[Float]⟩
```

### 3.4 Session Types

Typed communication channels:

```
S ::= !τ.S | ?τ.S | S ⊕ S | S & S | end
```

### 3.5 Type Soundness

We prove type preservation and progress:

**Theorem (Type Safety)**: Well-typed programs don't go wrong.

See [proofs/shared/type-system/soundness.md](../shared/type-system/soundness.md).

## 4. Effect System

### 4.1 Effect Algebra

Effects form a bounded join-semilattice:

```
ε ::= ∅ | IO | AI | Network | State⟨τ⟩ | Exception⟨τ⟩ | Async | ε ∪ ε
```

### 4.2 Effect Annotations

```ml
fn read_file(path: String) -> String with IO {
    // I/O operations
}

fn analyze(text: String) -> Analysis with AI {
    ai query { prompt: text }
}
```

### 4.3 Effect Polymorphism

```ml
fn map<T, U, E>(list: [T], f: fn(T) -> U with E) -> [U] with E
```

### 4.4 Effect Handlers

```ml
handle computation() {
    throw(e) => recover(e),
    return(v) => v
}
```

## 5. AI Integration

### 5.1 AI as Effect

AI operations are tracked in the type system:

```ml
fn summarize(text: String) -> String with AI {
    ai generate { prompt: "Summarize: {text}" }
}
```

### 5.2 AI Model Declarations

```ml
ai_model claude {
    provider: "anthropic"
    model: "claude-3-opus"
    temperature: 0.7
}
```

### 5.3 Prompt Templates

```ml
prompt analyze(code: String) -> Analysis {
    "Analyze this code for potential issues: {code}"
}
```

### 5.4 AI Contracts

```ml
fn validate(input: String) -> Bool
where ai_check: "input is a valid email address"
{
    // implementation
}
```

## 6. The Newtonian Spectrum

### 6.1 Seven Agents

| Agent | Responsibility |
|-------|----------------|
| Red | Performance optimization |
| Orange | Concurrency management |
| Yellow | Type and contract verification |
| Green | Configuration management |
| Blue | Audit and tracing |
| Indigo | Compile-time computation |
| Violet | Governance and policy |

### 6.2 Agent Composition

Agents compose via typed pipelines:

```ml
code |> Yellow.verify
     |> Red.optimize
     |> Blue.audit
```

### 6.3 Orchestration

```ml
orchestrate(goal: "Deploy feature X") {
    Yellow.check_types(code);
    Green.generate_config(schema);
    Red.optimize(code);
    Blue.log_deployment();
    Violet.verify_policy(deployment);
}
```

## 7. Implementation

### 7.1 Compiler Architecture

1. **Lexer** → Tokens
2. **Parser** → AST
3. **Type Checker** → Typed AST
4. **Effect Inference** → Effected AST
5. **Interpreter/Codegen** → Execution

### 7.2 AI Runtime

AI operations are dispatched to configured providers:

```rust
fn ai_query(prompt: &str, model: &Model) -> Result<String> {
    match model.provider {
        "anthropic" => anthropic::complete(prompt, model),
        "openai" => openai::complete(prompt, model),
        _ => Err(UnsupportedProvider)
    }
}
```

### 7.3 Status

- ✅ Lexer, Parser, Type Checker
- ✅ Interpreter
- ✅ Standard Library (60+ functions)
- 🔄 Compilation to native code
- 🔄 Full AI runtime

## 8. Related Work

### 8.1 Educational Languages

- **Scratch** (Resnick et al.): Visual blocks, no types
- **Blockly**: Block library, no semantic foundation
- **Pyret**: Gradual typing, no AI integration

### 8.2 Effect Systems

- **Koka** (Leijen): Algebraic effects, no AI
- **Frank** (Lindley et al.): Effect handlers
- **Effekt**: Effect polymorphism

### 8.3 Linear/Affine Types

- **Rust**: Ownership and borrowing
- **Linear Haskell**: Linear arrows
- **Granule**: Graded modal types

### 8.4 Session Types

- **Links** (Lindley & Morris): Session types for web
- **Scribble**: Multiparty session types

### 8.5 AI Integration

- **LangChain**: AI orchestration library (no types)
- **Semantic Kernel**: AI plugins (no effect tracking)

## 9. Conclusion

My Language demonstrates that progressive disclosure and AI integration
can be achieved within a single, type-safe language. The key insights are:

1. Visual and textual programming share semantics
2. AI should be a first-class, typed effect
3. Human-AI collaboration can be protocol-typed
4. Multi-agent systems benefit from spectrum decomposition

### 9.1 Future Work

- Dependent types for stronger guarantees
- Distributed agent execution
- Formal verification in Coq/Lean
- Large-scale evaluation studies

## References

1. Pierce, B. C. (2002). Types and Programming Languages.
2. Leijen, D. (2014). Koka: Programming with row polymorphic effect types.
3. Resnick, M. et al. (2009). Scratch: Programming for all.
4. Honda, K. et al. (2008). Multiparty session types.
5. Walker, D. (2005). Substructural type systems.
