# My Language: Technical Implementation Roadmap

## Overview

This document outlines the complete implementation plan for My Language,
covering all components needed for a production-ready language ecosystem.

## Technology Stack

### Core (Rust)

| Component | Crate | Dependencies |
|-----------|-------|--------------|
| Compiler | `my-lang` (existing) | — |
| LLVM Codegen | `my-llvm` | `inkwell`, `llvm-sys` |
| Language Server | `my-lsp` | `tower-lsp`, `lsp-types` |
| Package Manager | `my-pkg` | `toml`, `semver`, `petgraph` |
| AI Runtime | `my-ai` | `reqwest`, `rocketcache`, `tokio` |
| Formatter | `my-fmt` | `my-lang` |
| Linter | `my-lint` | `my-lang` |
| Test Runner | `my-test` | `my-lang` |
| Debugger | `my-dbg` | `dap` |

### Frontend (ReScript)

| Component | Package | Purpose |
|-----------|---------|---------|
| VS Code Extension | `my-vscode` | IDE integration |
| Block Editor | `my-blocks` | Me dialect visual editor |
| Web Playground | `my-playground` | Browser-based REPL |
| Documentation | `my-docs` | Generated API docs |

### Backend (Gleam)

| Component | Package | Purpose |
|-----------|---------|---------|
| Package Registry | `my-registry` | Central package hosting |
| Telemetry | `my-telemetry` | Usage analytics (opt-in) |

## Phase 1: Compiler Backend (Months 1-3)

### 1.1 Intermediate Representations

```
Source → AST → HIR → MIR → LLVM IR → Native
              ↓
          Interpreter (existing)
```

**HIR (High-level IR)**:
- Desugar match expressions
- Expand macros and prompts
- Resolve imports
- Normalize control flow

**MIR (Mid-level IR)**:
- SSA form
- Basic blocks
- Explicit drops (ownership)
- Monomorphized generics
- Inlined small functions

### 1.2 LLVM Integration

```toml
# crates/my-llvm/Cargo.toml
[dependencies]
inkwell = { version = "0.4", features = ["llvm17-0"] }
```

Key codegen components:
- Type lowering (My types → LLVM types)
- Function codegen
- Effect handling (compile-time erasure)
- AI stub generation (runtime dispatch)
- Ownership/drop insertion

### 1.3 Targets

| Target | Priority | Notes |
|--------|----------|-------|
| x86_64-linux | P0 | Primary dev platform |
| x86_64-darwin | P0 | macOS Intel |
| aarch64-darwin | P0 | macOS Apple Silicon |
| x86_64-windows | P1 | Windows |
| wasm32-unknown | P1 | Browser/edge |
| aarch64-linux | P2 | Linux ARM |

## Phase 2: Standard Library (Months 2-4)

> **Architecture note (superseded sketch removed).** An earlier draft of this
> section showed a `stdlib/std/*.my` source-module tree and an `@ffi(...)`
> binding strategy. **That was never built and was actively misleading.** The
> Solo standard library is delivered as **native Rust builtins** registered in
> `crates/my-lang/src/stdlib.rs` (`register_stdlib` / `stdlib_functions()`) and
> exposed to programs as ordinary callable identifiers — there are no
> `stdlib/std/*.my` files and no `@ffi` surface. This section now describes the
> real architecture and the *capability* roadmap only.
>
> **Live status is the GitHub Phase-2 stdlib tracker
> ([`#45`](https://github.com/hyperpolymath/my-lang/issues/45)) and its
> sub-issues — not this document.** This file states the plan; the tracker
> states what has landed.

### 2.1 Architecture

Builtins are `NativeFunction` values registered into the interpreter
environment and the checker symbol table. Adding a primitive is a Rust change
in `crates/my-lang/src/stdlib.rs`:

1. Implement the `NativeFunction` in the relevant `register_*` block.
2. Add its name to `stdlib_functions()` (checker visibility).
3. The checker uses an `Unknown`-type fallback for these names unless a
   signature is added in `checker.rs::stdlib_function_type`.

Naming convention: flat snake-case with an area prefix (`fs_read_file`,
`map_get`, `json_parse`), not dotted module paths.

### 2.2 Capability Roadmap

Each capability area is delivered as a group of native builtins. Status
columns are intentionally omitted here — see tracker `#45`.

| Area | Representative builtins |
|---|---|
| I/O & terminal | `print`, `println`, `input`, `input_prompt` |
| Strings | `str_concat`, `str_split`, `format`, … |
| Math | `abs`, `sqrt`, `pow`, constants |
| Arrays | `push`, `slice`, `range`, … |
| Filesystem | `fs_read_file`, `fs_write_file`, `fs_create_dir_all`, `fs_exists` |
| Environment | `env`, `env_args` |
| Maps / dict | `map_new`, `map_set`, `map_get`, `map_has`, `map_keys`, `map_remove`, `map_len` |
| JSON | `json_parse`, `json_stringify` |
| Date / time | `time`, `date_today` |
| Networking, regex, crypto, subprocess, async | future capability areas |

Implementation strategy: native Rust for everything that needs host access or
performance; effect annotations are a language-level concern, not a stdlib
delivery mechanism.

## Phase 3: AI Runtime (Months 2-4)

### 3.1 Architecture

```
┌─────────────────────────────────────────────────┐
│                 My Language Code                │
├─────────────────────────────────────────────────┤
│                   AI Effect Layer               │
│    (type checking, effect tracking)             │
├─────────────────────────────────────────────────┤
│                  AI Runtime (Rust)              │
│  ┌───────────┬───────────┬───────────┐         │
│  │ Providers │   Cache   │  Agents   │         │
│  │           │(rocketcache)          │         │
│  ├───────────┼───────────┼───────────┤         │
│  │ Anthropic │ OpenAI    │ Ollama    │         │
│  └───────────┴───────────┴───────────┘         │
└─────────────────────────────────────────────────┘
```

### 3.2 Provider Abstraction

```rust
// crates/my-ai/src/provider.rs
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn stream(&self, request: CompletionRequest) -> Result<impl Stream<Item = Token>>;
}

pub struct AnthropicProvider { /* ... */ }
pub struct OpenAIProvider { /* ... */ }
pub struct OllamaProvider { /* ... */ }
```

### 3.3 Caching (rocketcache)

```rust
// crates/my-ai/src/cache.rs
use rocketcache::Cache;

pub struct AICache {
    cache: Cache<CacheKey, CachedResponse>,
    config: CacheConfig,
}

impl AICache {
    pub async fn get_or_compute<F, Fut>(&self, key: CacheKey, compute: F) -> Result<Response>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Response>>,
    {
        if let Some(cached) = self.cache.get(&key).await? {
            return Ok(cached);
        }
        let response = compute().await?;
        self.cache.set(&key, &response).await?;
        Ok(response)
    }
}
```

### 3.4 Newtonian Agents

```rust
// crates/my-ai/src/agents/mod.rs
pub mod red;      // Performance
pub mod orange;   // Concurrency
pub mod yellow;   // Contracts
pub mod green;    // Config
pub mod blue;     // Audit
pub mod indigo;   // Comptime
pub mod violet;   // Governance

pub trait Agent: Send + Sync {
    fn spectrum(&self) -> Spectrum;
    async fn execute(&self, task: Task, ctx: &Context) -> Result<Output>;
}

pub struct AgentOrchestrator {
    agents: HashMap<Spectrum, Box<dyn Agent>>,
    planner: Planner,
}
```

## Phase 4: Developer Tools (Months 3-5)

### 4.1 Language Server (LSP)

```rust
// crates/my-lsp/src/main.rs
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| MyLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

Features:
- Diagnostics (errors, warnings)
- Completions (context-aware)
- Hover information
- Go to definition
- Find references
- Rename symbol
- Code actions (quick fixes)
- Formatting
- Signature help

### 4.2 Formatter

```rust
// crates/my-fmt/src/lib.rs
pub struct Formatter {
    config: FormatConfig,
}

impl Formatter {
    pub fn format(&self, source: &str) -> Result<String> {
        let ast = parse(source)?;
        let doc = self.to_doc(&ast);
        Ok(doc.pretty(self.config.max_width))
    }
}
```

### 4.3 Linter

```rust
// crates/my-lint/src/lib.rs
pub trait LintRule: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, ast: &Program) -> Vec<Diagnostic>;
}

pub struct Linter {
    rules: Vec<Box<dyn LintRule>>,
}

// Built-in rules
mod rules {
    pub struct UnusedVariable;
    pub struct UnnecessaryMut;
    pub struct MissingEffectAnnotation;
    pub struct DeprecatedAIModel;
    pub struct ContractViolation;
}
```

### 4.4 Test Runner

```rust
// crates/my-test/src/lib.rs
pub struct TestRunner {
    config: TestConfig,
}

impl TestRunner {
    pub async fn run(&self, tests: Vec<TestCase>) -> TestResults {
        let mut results = Vec::new();
        for test in tests {
            let result = self.run_single(test).await;
            results.push(result);
        }
        TestResults::new(results)
    }
}
```

## Phase 5: Package Manager (Months 4-5)

### 5.1 Manifest Format

```toml
# my.toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
license = "MIT"

[dependencies]
std = "0.1"
http = "0.2"
json = { version = "0.1", features = ["streaming"] }

[dev-dependencies]
test = "0.1"

[ai]
default-model = "claude-3-opus"
cache = true

[dialects]
enabled = ["solo", "duet"]  # Which dialects this package uses
```

### 5.2 CLI Commands

```bash
my new <name>           # Create new project
my init                 # Initialize in current directory
my build                # Compile project
my run                  # Run project
my test                 # Run tests
my bench                # Run benchmarks
my fmt                  # Format code
my lint                 # Lint code
my add <pkg>            # Add dependency
my remove <pkg>         # Remove dependency
my update               # Update dependencies
my publish              # Publish to registry
my doc                  # Generate documentation
my repl                 # Interactive REPL
```

### 5.3 Dependency Resolution

```rust
// crates/my-pkg/src/resolver.rs
use petgraph::Graph;
use semver::VersionReq;

pub struct Resolver {
    registry: Registry,
    cache: PackageCache,
}

impl Resolver {
    pub async fn resolve(&self, manifest: &Manifest) -> Result<LockFile> {
        let mut graph = Graph::new();
        // Build dependency graph
        // Apply version constraints
        // Detect conflicts
        // Return resolved versions
    }
}
```

## Phase 6: Dialect Implementations (Months 5-8)

### 6.1 Me Dialect (Visual)

**Block Editor Architecture**:
```
┌─────────────────────────────────────────┐
│         Block Editor (ReScript)         │
├─────────────────────────────────────────┤
│  Canvas  │  Palette  │  Properties     │
├──────────┴───────────┴─────────────────┤
│           Block Engine                  │
│  (connection validation, execution)     │
├─────────────────────────────────────────┤
│         My Language Runtime (WASM)      │
└─────────────────────────────────────────┘
```

**Block Categories**:
- Control (if, loop, function)
- Variables (let, assign, read)
- I/O (print, input)
- Math (operators)
- Logic (and, or, not)
- AI (query, verify)
- Tokens (create, consume)

### 6.2 Solo Dialect

Already largely implemented. Additions:
- Contract verification with Z3
- Better affine type error messages
- Checkpoint/rollback runtime

### 6.3 Duet Dialect

**Session Runtime**:
```rust
// In my-lang runtime
pub struct SessionChannel<S: SessionType> {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    _phantom: PhantomData<S>,
}

impl<T, S> SessionChannel<Send<T, S>> {
    pub fn send(self, value: T) -> SessionChannel<S> { /* ... */ }
}
```

### 6.4 Ensemble Dialect

**Orchestration Runtime**:
```rust
pub struct Orchestrator {
    agents: AgentRegistry,
    scheduler: AgentScheduler,
    monitor: AgentMonitor,
}

impl Orchestrator {
    pub async fn execute(&self, goal: Goal) -> Result<Output> {
        let plan = self.planner.plan(goal)?;
        for step in plan {
            let agent = self.agents.get(step.spectrum);
            let result = agent.execute(step.task).await?;
            self.monitor.log(step, &result);
        }
        Ok(self.collect_output())
    }
}
```

## Phase 7: IDE Integration (Months 5-6)

### 7.1 VS Code Extension

```
editors/vscode/
├── package.json
├── src/
│   ├── extension.res      # ReScript
│   ├── client.res         # LSP client
│   ├── commands.res       # Editor commands
│   └── debugger.res       # DAP integration
├── syntaxes/
│   └── my.tmLanguage.json # Syntax highlighting
└── snippets/
    └── my.json            # Code snippets
```

### 7.2 Features

- Syntax highlighting (TextMate grammar)
- LSP integration (completions, diagnostics, etc.)
- Debugger (DAP)
- Block editor webview (Me dialect)
- AI assistant integration
- Test explorer
- REPL panel

## Phase 8: Documentation & Community (Ongoing)

### 8.1 Documentation Site

Using existing My SSG:
- Language reference
- Standard library API
- Tutorials (per dialect)
- Examples
- Migration guides

### 8.2 Playground

Browser-based environment:
- Code editor
- Block editor (Me)
- Live execution (WASM)
- Share functionality

## Implementation Timeline

```
Month 1-2:  HIR + MIR design and implementation
Month 2-3:  LLVM codegen (basic)
Month 2-4:  Standard library (core modules)
Month 2-4:  AI runtime with rocketcache
Month 3-5:  LSP + formatter + linter
Month 4-5:  Package manager
Month 5-6:  VS Code extension
Month 5-7:  Me dialect block editor
Month 6-8:  Duet + Ensemble runtimes
Month 7+:   Polish, documentation, community
```

## File Structure (Final)

```
my-lang/
├── Cargo.toml              # Workspace
├── crates/
│   ├── my-lang/            # Core (existing)
│   ├── my-hir/             # High-level IR
│   ├── my-mir/             # Mid-level IR
│   ├── my-llvm/            # LLVM codegen
│   ├── my-lsp/             # Language server
│   ├── my-pkg/             # Package manager
│   ├── my-ai/              # AI runtime
│   ├── my-fmt/             # Formatter
│   ├── my-lint/            # Linter
│   ├── my-test/            # Test runner
│   └── my-dbg/             # Debugger
├── stdlib/                  # Standard library (My)
├── editors/
│   ├── vscode/             # VS Code (ReScript)
│   └── neovim/             # Neovim (Lua)
├── web/
│   ├── playground/         # Web playground (ReScript)
│   └── blocks/             # Block editor (ReScript)
├── registry/               # Package registry (Gleam)
├── docs/                   # Documentation
├── examples/               # Example projects
├── proofs/                 # Formal proofs (existing)
└── tests/                  # Integration tests
```

## Success Metrics

### v0.3.0 (Compiler)
- [ ] Native binaries compile and run
- [ ] 90% of test suite passes with codegen
- [ ] < 5s compile time for medium projects

### v0.4.0 (Tooling)
- [ ] LSP provides completions and diagnostics
- [ ] Formatter handles all syntax
- [ ] Package manager resolves dependencies

### v0.5.0 (AI)
- [ ] AI operations work with Claude/OpenAI
- [ ] Caching reduces API calls by 50%+
- [ ] Agents execute orchestrated workflows

### v1.0.0 (Production)
- [ ] All four dialects functional
- [ ] VS Code extension published
- [ ] Package registry live
- [ ] Documentation complete
- [ ] 10+ community packages
