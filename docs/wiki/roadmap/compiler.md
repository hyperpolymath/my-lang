# Compiler Roadmap

This document outlines the development plan for the My Language compiler infrastructure.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Source Code (.ml)                         │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Lexer (Tokenization)                         │
│                        src/lexer.rs ✅                           │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Parser (AST Generation)                       │
│                       src/parser.rs ✅                           │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                Type Checker (Semantic Analysis)                  │
│                      src/checker.rs ✅                           │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    HIR (High-Level IR)                           │
│                         Planned 🔄                               │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    MIR (Mid-Level IR)                            │
│                         Planned 🔄                               │
└─────────────────────────────────────────────────────────────────┘
                                 │
                    ┌───────────┴───────────┐
                    ▼                       ▼
┌──────────────────────────┐   ┌──────────────────────────┐
│    Interpreter           │   │    Code Generation       │
│    (Tree-walking) 🔄     │   │    (LLVM/Cranelift) 🔄   │
└──────────────────────────┘   └──────────────────────────┘
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    ▼                      ▼                      ▼
            ┌─────────────┐        ┌─────────────┐        ┌─────────────┐
            │   Native    │        │    WASM     │        │    JIT      │
            │   Binary    │        │   Module    │        │  Execution  │
            └─────────────┘        └─────────────┘        └─────────────┘
```

## Current Implementation

### Lexer (✅ Complete)
**Location:** `src/lexer.rs`

Features:
- 87 token types including AI keywords
- Unicode identifier support
- String interpolation tokens
- Block and line comments
- Attribute syntax (`#[...]`)
- Span tracking for error messages

Performance targets:
- 100MB/s tokenization speed
- Zero-allocation token iteration

### Parser (✅ Complete)
**Location:** `src/parser.rs`

Features:
- Recursive descent parsing
- Full AST representation
- Error recovery (basic)
- Span preservation
- All syntax constructs supported

Architecture:
- Pratt parsing for expressions
- Lookahead for disambiguation
- Context-sensitive parsing for AI constructs

### Type Checker (✅ Complete)
**Location:** `src/checker.rs`, `src/types.rs`, `src/scope.rs`

Features:
- Two-pass analysis (collect, check)
- Symbol table with scopes
- Type environment for definitions
- Name resolution
- Basic type compatibility
- AI construct validation

## Phase 1: Interpreter

### Tree-Walking Interpreter
**Priority: High | Target: Q2 2025**

```rust
// Planned: src/interpreter.rs
pub struct Interpreter {
    env: Environment,
    ai_runtime: AIRuntime,
}

impl Interpreter {
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, RuntimeError>;
    pub fn exec(&mut self, stmt: &Stmt) -> Result<(), RuntimeError>;
}
```

Features:
- [ ] Expression evaluation
- [ ] Statement execution
- [ ] Function calls and closures
- [ ] Pattern matching runtime
- [ ] AI expression execution (mock + real)
- [ ] Effect handling

### REPL
**Priority: High | Target: Q2 2025**

```
$ ml repl
My Language v0.2.0 REPL
Type :help for help, :quit to exit

ml> let x = 5
x: Int = 5

ml> x * 2
Int = 10

ml> ai! { "What is 2 + 2?" }
AI<String> = "2 + 2 equals 4"

ml> :type ai query { prompt: "test" }
AI<String> with AI
```

Features:
- [ ] Expression evaluation
- [ ] Multi-line input
- [ ] History and completion
- [ ] Type queries (`:type`)
- [ ] Definition browsing (`:browse`)
- [ ] AI context persistence

## Phase 2: Intermediate Representations

### High-Level IR (HIR)
**Target: Q3 2025**

Purpose: Desugared, type-annotated representation

Transformations:
- Desugar match to decision trees
- Resolve all names to unique IDs
- Inline type information
- Lower string interpolation
- Normalize AI expressions

```rust
// Planned: src/hir/mod.rs
pub enum HirExpr {
    Literal(Literal, Type),
    Variable(DefId, Type),
    Call(DefId, Vec<HirExpr>, Type),
    AICall(AIKind, Vec<HirExpr>, Type),
    // ...
}
```

### Mid-Level IR (MIR)
**Target: Q3 2025**

Purpose: Control-flow graph for optimization and codegen

Features:
- Basic blocks
- SSA form
- Explicit drops
- Effect annotations
- AI call boundaries

```rust
// Planned: src/mir/mod.rs
pub struct MirBody {
    blocks: IndexVec<BasicBlock, BasicBlockData>,
    locals: IndexVec<Local, LocalDecl>,
}

pub struct BasicBlockData {
    statements: Vec<Statement>,
    terminator: Terminator,
}
```

## Phase 3: Optimization

### Optimization Passes
**Target: Q3-Q4 2025**

| Pass | Priority | Description |
|------|----------|-------------|
| Dead Code Elimination | High | Remove unused code |
| Constant Folding | High | Evaluate constant expressions |
| Inlining | High | Inline small functions |
| Common Subexpression | Medium | Eliminate redundant computations |
| Loop Optimizations | Medium | LICM, unrolling, vectorization |
| AI Call Batching | Medium | Combine multiple AI calls |
| Escape Analysis | Low | Stack allocation for non-escaping values |

### AI-Specific Optimizations

```ml
// Before optimization
let a = ai query { prompt: "Question 1" };
let b = ai query { prompt: "Question 2" };
let c = ai query { prompt: "Question 3" };

// After AI call batching
let [a, b, c] = ai batch_query([
    { prompt: "Question 1" },
    { prompt: "Question 2" },
    { prompt: "Question 3" },
]);
```

Optimizations:
- [ ] AI call batching
- [ ] Prompt template caching
- [ ] Embedding cache reuse
- [ ] Model switching optimization

## Phase 4: Code Generation

### LLVM Backend
**Priority: High | Target: Q4 2025**

Features:
- [ ] LLVM IR generation
- [ ] Debug info (DWARF)
- [ ] Platform-specific optimizations
- [ ] Link-time optimization (LTO)

Targets:
- x86_64-linux
- x86_64-darwin
- aarch64-linux
- aarch64-darwin

### Cranelift Backend
**Priority: Medium | Target: Q4 2025**

Purpose: Fast compilation for development

Features:
- [ ] Cranelift IR generation
- [ ] Fast compile times
- [ ] JIT execution
- [ ] Debug support

### WebAssembly Backend
**Priority: High | Target: 2026**

Features:
- [ ] WASM binary generation
- [ ] WASI support
- [ ] Browser runtime
- [ ] Component model support

```bash
$ ml build --target wasm32-wasi
$ wasmtime ./target/app.wasm
```

## Phase 5: Runtime

### Core Runtime
**Target: Q3 2025**

Components:
- Memory allocator (jemalloc or custom)
- Panic/unwinding support
- Stack traces
- Async executor

### AI Runtime
**Target: Q3 2025**

```rust
// Planned: src/runtime/ai.rs
pub trait AIProvider: Send + Sync {
    fn query(&self, request: QueryRequest) -> BoxFuture<QueryResponse>;
    fn embed(&self, text: &str) -> BoxFuture<Vec<f32>>;
    fn stream(&self, request: QueryRequest) -> BoxStream<Token>;
}

pub struct AIRuntime {
    providers: HashMap<String, Box<dyn AIProvider>>,
    cache: PromptCache,
    rate_limiter: RateLimiter,
}
```

Providers:
- [ ] OpenAI
- [ ] Anthropic
- [ ] Local models (llama.cpp)
- [ ] Mock provider for testing

## Compiler Flags

### Planned CLI Options

```bash
# Optimization levels
ml build --release        # Full optimizations
ml build --debug          # No optimizations, debug info
ml build -O2              # Specific level

# Targets
ml build --target wasm32-wasi
ml build --target x86_64-linux

# Features
ml build --features "openai,anthropic"

# Codegen options
ml build --emit=llvm-ir   # Emit LLVM IR
ml build --emit=mir       # Emit MIR
ml build --emit=asm       # Emit assembly

# Diagnostics
ml build --explain=E0001  # Explain error code
ml build -W all           # Enable all warnings
```

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Lexing | 100 MB/s | Streaming tokenization |
| Parsing | 50 MB/s | Full AST construction |
| Type checking | 20 MB/s | Including inference |
| Debug compile | 1s for 10K LOC | Development iteration |
| Release compile | 10s for 10K LOC | Full optimizations |
| Binary size | <1MB base | Minimal runtime |

## Error Messages

### Design Principles

1. **Precise locations**: Point to exact span of error
2. **Clear explanation**: What went wrong and why
3. **Suggestions**: How to fix the issue
4. **Context**: Show relevant code

```
error[E0001]: undefined variable 'user_name'
  --> src/main.ml:15:12
   |
15 |     greet(user_name)
   |            ^^^^^^^^^ not found in this scope
   |
   = help: did you mean 'username'?

note: 'username' is defined here
  --> src/main.ml:10:9
   |
10 |     let username = "Alice";
   |         ^^^^^^^^
```

## Incremental Compilation

**Target: 2026**

Features:
- [ ] Query-based architecture
- [ ] Fine-grained dependency tracking
- [ ] Parallel compilation
- [ ] Persistent cache

```
$ ml build
   Compiling my-app v0.1.0
   Unchanged: 45 modules
   Recompiling: 3 modules (src/api.ml changed)
   Finished in 0.3s
```

## Testing Strategy

### Compiler Tests
- Unit tests for each pass
- Integration tests for full pipeline
- Fuzzing for parser robustness
- Snapshot tests for error messages
- Performance benchmarks

### Test Commands
```bash
cargo test                    # All tests
cargo test --lib              # Library tests only
cargo test parser             # Parser tests only
cargo bench                   # Benchmarks
```

## Contributing

See [Internals Documentation](../internals/architecture.md) for details on compiler architecture and how to contribute.
