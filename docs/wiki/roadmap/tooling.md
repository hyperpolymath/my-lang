# Tooling Roadmap

This document outlines the development plan for My Language development tools.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      Developer Experience                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │   CLI    │  │  Editor  │  │ Debugger │  │ Package Manager  │ │
│  │   (ml)   │  │  (LSP)   │  │  (mldb)  │  │    (mlpkg)       │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │Formatter │  │  Linter  │  │   Test   │  │  Documentation   │ │
│  │ (mlfmt)  │  │(mllint)  │  │ (mltest) │  │     (mldoc)      │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Current State (v0.1.0)

### CLI Tool (✅ Basic)

```bash
$ ml --help
My Language Compiler v0.1.0

USAGE:
    ml <COMMAND>

COMMANDS:
    parse      Parse a source file and show AST
    typecheck  Type-check a source file
    compile    Parse and type-check a source file
    help       Print help information

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
```

## Phase 1: Core CLI Enhancement

### Extended CLI Commands
**Target: Q2 2025**

```bash
# Build commands
ml build                    # Build the project
ml build --release          # Build with optimizations
ml build --target wasm      # Build for WebAssembly

# Run commands
ml run                      # Build and run
ml run --example hello      # Run an example
ml run -- arg1 arg2         # Pass arguments

# Development
ml check                    # Type-check without building
ml repl                     # Start interactive REPL
ml watch                    # Watch mode with auto-rebuild

# Project management
ml new myproject            # Create new project
ml init                     # Initialize in current directory
ml clean                    # Clean build artifacts

# Package management
ml add package_name         # Add dependency
ml remove package_name      # Remove dependency
ml update                   # Update dependencies
ml publish                  # Publish to registry

# Documentation
ml doc                      # Generate documentation
ml doc --open               # Generate and open in browser

# Testing
ml test                     # Run tests
ml test --filter pattern    # Run filtered tests
ml bench                    # Run benchmarks

# Tooling
ml fmt                      # Format code
ml fmt --check              # Check formatting
ml lint                     # Run linter
ml fix                      # Auto-fix issues
```

### Project Structure
**Target: Q2 2025**

```
myproject/
├── ml.toml                 # Project configuration
├── ml.lock                 # Dependency lock file
├── src/
│   ├── main.ml             # Entry point
│   └── lib.ml              # Library root
├── tests/
│   └── integration_test.ml
├── examples/
│   └── basic.ml
├── benches/
│   └── perf_test.ml
└── target/
    ├── debug/
    └── release/
```

### Configuration (ml.toml)
```toml
[package]
name = "myproject"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2025"

[dependencies]
http = "1.0"
json = "2.0"

[dev-dependencies]
testing = "1.0"

[ai]
default_provider = "openai"
api_key_env = "OPENAI_API_KEY"

[ai.providers.openai]
model = "gpt-4"
max_tokens = 4096

[ai.providers.local]
path = "./models/llama.gguf"

[build]
target = "native"

[profile.release]
opt_level = 3
lto = true
```

## Phase 2: Language Server

### LSP Implementation
**Target: Q3 2025**

Features:
- [ ] Go to definition
- [ ] Find references
- [ ] Hover information
- [ ] Code completion
- [ ] Signature help
- [ ] Rename symbol
- [ ] Document symbols
- [ ] Workspace symbols
- [ ] Diagnostics
- [ ] Code actions
- [ ] Formatting

```typescript
// VS Code extension settings
{
  "mylang.server.path": "ml-lsp",
  "mylang.ai.enabled": true,
  "mylang.ai.inlineHints": true,
  "mylang.format.onSave": true
}
```

### AI-Enhanced IDE Features
**Target: Q4 2025**

```
┌─────────────────────────────────────────────────────────────────┐
│ fn process_data(input: String) -> Result<Data> {               │
│     let parsed = parse_json(input)?;                           │
│     ┌─────────────────────────────────────────────────────┐    │
│     │ 💡 AI Suggestion                                     │    │
│     │                                                      │    │
│     │ This function could benefit from input validation.  │    │
│     │ Consider adding:                                     │    │
│     │                                                      │    │
│     │ if input.is_empty() {                               │    │
│     │     return Err(Error::EmptyInput);                  │    │
│     │ }                                                    │    │
│     │                                                      │    │
│     │ [Apply] [Dismiss] [Configure AI hints]              │    │
│     └─────────────────────────────────────────────────────┘    │
│ }                                                               │
└─────────────────────────────────────────────────────────────────┘
```

Features:
- [ ] AI-powered code completion
- [ ] Inline AI explanations
- [ ] Prompt template editing with preview
- [ ] AI model selection in IDE
- [ ] Token usage tracking

### Editor Extensions
**Target: Q3-Q4 2025**

| Editor | Priority | Status |
|--------|----------|--------|
| VS Code | High | Planned |
| Neovim | Medium | Planned |
| Zed | Medium | Planned |
| JetBrains | Low | Planned |
| Helix | Low | Planned |

## Phase 3: Formatter

### mlfmt
**Target: Q3 2025**

```bash
# Format all files
ml fmt

# Check formatting (CI mode)
ml fmt --check

# Format specific file
ml fmt src/main.ml

# Format stdin
echo "let x=1" | ml fmt --stdin
```

Configuration (ml.toml):
```toml
[format]
max_width = 100
indent_style = "space"
indent_size = 4
trailing_comma = true
single_line_if_threshold = 50
```

### Formatting Rules

```ml
// Before formatting
fn example(  a:Int,b:String   ,c:Bool)->Result<String>{
let x=if a>0{"positive"}else{"negative"};
let y=match c{true=>"yes",false=>"no"};x}

// After formatting
fn example(a: Int, b: String, c: Bool) -> Result<String> {
    let x = if a > 0 {
        "positive"
    } else {
        "negative"
    };
    let y = match c {
        true => "yes",
        false => "no",
    };
    x
}
```

## Phase 4: Linter

### mllint
**Target: Q4 2025**

```bash
# Run linter
ml lint

# With specific rules
ml lint --deny unused_variables

# Auto-fix
ml lint --fix

# Configuration file
ml lint --config .mllint.toml
```

### Lint Rules

| Category | Rules |
|----------|-------|
| **Correctness** | unused_variables, unreachable_code, infinite_loop |
| **Style** | naming_conventions, max_line_length, complexity |
| **Performance** | unnecessary_clone, redundant_allocation |
| **AI** | unbounded_ai_call, missing_ai_timeout, prompt_injection_risk |
| **Security** | sql_injection, command_injection, sensitive_data_exposure |

Configuration (.mllint.toml):
```toml
[rules]
unused_variables = "warn"
unreachable_code = "error"
naming_conventions = "warn"

[rules.complexity]
max_function_length = 50
max_cyclomatic_complexity = 10

[rules.ai]
require_timeout = true
max_tokens_warning = 10000

[ignore]
paths = ["tests/", "examples/"]
```

### AI-Specific Lints

```ml
// Warning: AI call without timeout
let result = ai query { prompt: "..." };
//           ^^^^^^^^^^^^^^^^^^^^^^^^^
// warning[ai-timeout]: AI call should have a timeout
// help: add `timeout: 30` to prevent hanging

// Warning: Potential prompt injection
let result = ai query { prompt: user_input };
//                              ^^^^^^^^^^
// warning[prompt-injection]: User input directly in prompt
// help: use sanitize_prompt(user_input) or a template

// Warning: Large token usage
let result = ai generate {
    prompt: very_long_text,  // 50,000 tokens
//          ^^^^^^^^^^^^^^
// warning[token-limit]: Prompt exceeds recommended 10,000 tokens
};
```

## Phase 5: Package Manager

### mlpkg
**Target: Q4 2025**

```bash
# Initialize new project
mlpkg init

# Add dependencies
mlpkg add http
mlpkg add json@2.0
mlpkg add --dev testing

# Remove dependencies
mlpkg remove http

# Update dependencies
mlpkg update
mlpkg update http

# Search registry
mlpkg search json

# Publish package
mlpkg publish

# View package info
mlpkg info http
```

### Package Registry
**Target: 2026**

Features:
- [ ] Central package registry (packages.mylang.org)
- [ ] Package versioning (semver)
- [ ] Dependency resolution
- [ ] Security scanning
- [ ] Download statistics
- [ ] Documentation hosting

```
┌─────────────────────────────────────────────────────────────────┐
│  packages.mylang.org                                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  🔍 Search packages...                                          │
│                                                                  │
│  Popular Packages                                                │
│  ├── http (v1.2.0) - HTTP client and server library             │
│  ├── json (v2.0.1) - JSON parsing and serialization            │
│  ├── async (v1.0.0) - Async runtime and utilities              │
│  ├── neuron (v0.5.0) - AI framework for ML applications        │
│  └── spark (v0.3.0) - Web framework                            │
│                                                                  │
│  Recently Updated                                                │
│  ├── database v0.8.0 (2 hours ago)                              │
│  ├── testing v1.1.0 (5 hours ago)                               │
│  └── crypto v0.9.0 (1 day ago)                                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Phase 6: Debugger

### mldb
**Target: 2026**

```bash
# Start debugging
ml debug ./target/debug/myapp

# Commands
(mldb) break src/main.ml:25      # Set breakpoint
(mldb) run                        # Start execution
(mldb) next                       # Step over
(mldb) step                       # Step into
(mldb) continue                   # Continue execution
(mldb) print x                    # Print variable
(mldb) backtrace                  # Show call stack
(mldb) watch x                    # Watch variable
(mldb) ai-trace                   # Show AI call history
```

### AI Debugging Features

```bash
(mldb) ai-calls                   # List all AI calls
ID  Time      Model    Tokens  Status
1   0.234s    gpt-4    150     Success
2   0.567s    gpt-4    300     Success
3   1.234s    gpt-4    --      In Progress

(mldb) ai-inspect 2               # Inspect AI call
Call #2:
  Model: gpt-4
  Prompt: "Summarize the following..."
  Response: "The document discusses..."
  Tokens: 300 (input: 200, output: 100)
  Latency: 567ms
  Cost: $0.012

(mldb) ai-replay 2                # Replay AI call
```

## Phase 7: Documentation Generator

### mldoc
**Target: Q4 2025**

```bash
# Generate documentation
ml doc

# Generate and open
ml doc --open

# Include private items
ml doc --document-private-items

# Generate for specific package
ml doc --package mylib
```

### Documentation Syntax

```ml
/// Summarizes the given text using AI.
///
/// # Arguments
/// * `text` - The text to summarize
/// * `max_words` - Maximum words in summary (default: 100)
///
/// # Returns
/// A summarized version of the input text
///
/// # Examples
/// ```ml
/// let summary = summarize("Long article...", max_words: 50);
/// assert(summary.word_count() <= 50);
/// ```
///
/// # AI Usage
/// This function makes an AI call with approximately 100-500 tokens.
fn summarize(text: String, max_words: Int = 100) -> AI<String> {
    ai generate {
        prompt: "Summarize in {max_words} words: {text}"
    }
}
```

## Phase 8: Testing Framework

### mltest
**Target: Q3 2025**

```bash
# Run all tests
ml test

# Run specific test
ml test test_name

# Run with filter
ml test --filter "integration"

# Run with coverage
ml test --coverage

# Watch mode
ml test --watch
```

### Test Syntax

```ml
#[test]
fn test_basic_addition() {
    assert_eq(2 + 2, 4);
}

#[test]
#[should_panic(expected: "division by zero")]
fn test_division_by_zero() {
    let _ = 1 / 0;
}

#[test]
async fn test_ai_query() {
    let result = ai query { prompt: "Say hello" };
    assert(result.contains("hello") || result.contains("Hello"));
}

#[test]
#[ai_mock(responses: ["mocked response"])]
fn test_with_mock_ai() {
    let result = ai query { prompt: "anything" };
    assert_eq(result, "mocked response");
}

#[bench]
fn bench_parsing(b: &mut Bencher) {
    let input = read_file("large_input.ml");
    b.iter(|| parse(input));
}
```

## Tool Integration

### CI/CD Integration

```yaml
# GitHub Actions example
name: CI
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: mylang/setup-ml@v1
        with:
          version: '0.5.0'

      - name: Check formatting
        run: ml fmt --check

      - name: Lint
        run: ml lint

      - name: Build
        run: ml build --release

      - name: Test
        run: ml test --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### Pre-commit Hooks

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/mylang/pre-commit-hooks
    rev: v0.1.0
    hooks:
      - id: ml-fmt
      - id: ml-lint
      - id: ml-check
```

## Performance Targets

| Tool | Target | Notes |
|------|--------|-------|
| `ml check` | <1s for 10K LOC | Fast feedback loop |
| `ml fmt` | <100ms for single file | Near-instant |
| `ml lint` | <2s for 10K LOC | Parallel analysis |
| LSP response | <50ms | Smooth editing |
| `ml test` startup | <500ms | Quick iteration |

## Installation

### Planned Installation Methods

```bash
# Official installer (Unix)
curl -sSf https://mylang.org/install.sh | sh

# Official installer (Windows)
irm https://mylang.org/install.ps1 | iex

# Homebrew (macOS/Linux)
brew install mylang

# Cargo (for Rust users)
cargo install mylang

# npm (for Node.js users)
npm install -g @mylang/cli

# Docker
docker pull mylang/mylang
```

## Feedback

Tool development is driven by developer experience. Please share feedback:
- GitHub Issues for bugs
- Discussions for feature requests
- Discord for real-time help
