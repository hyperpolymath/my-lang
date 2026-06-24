<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# My Language Roadmap Overview

*Last Updated: 2025-12-17*

This document outlines the complete development roadmap for My Language, covering the language itself, compiler infrastructure, tooling, and ecosystem.

## Vision

My Language aims to be the premier language for AI-native application development, combining:
- **Safety**: Memory safety, type safety, and effect tracking
- **Expressiveness**: Functional programming with pragmatic imperative features
- **AI Integration**: First-class AI primitives that feel native to the language
- **Performance**: Zero-cost abstractions and efficient compilation

## Development Phases

### Phase 1: Foundation ✅
**Status: Complete**

| Component | Status | Description |
|-----------|--------|-------------|
| Lexer | ✅ Complete | Full tokenization with AI keywords |
| Parser | ✅ Complete | Recursive descent parser, full AST |
| Type Checker | ✅ Complete | Semantic analysis with AI type support |
| CLI | ✅ Complete | Parse, typecheck, compile, run commands |
| Grammar Spec | ✅ Complete | EBNF specification |

### Phase 2: Execution ✅
**Status: Complete**

| Component | Status | Description |
|-----------|--------|-------------|
| Interpreter | ✅ Complete | Tree-walking interpreter with full expression support |
| REPL | ✅ Complete | Interactive environment with multiline support |
| AI Runtime | ✅ Complete | Mock AI operations (real integration planned) |
| Standard Library | ✅ Complete | I/O, strings, math, arrays, types, utilities |

### Phase 2.5: Ecosystem Tools 🔄
**Status: In Progress**

| Component | Status | Description |
|-----------|--------|-------------|
| My SSG | ✅ Complete | Static site generator with My Language templates |
| Documentation | ✅ Complete | Wiki with guides, tutorials, reference |
| Examples | ✅ Complete | Demo programs and hello world |

### Phase 3: Compilation
**Target: Q1 2025**

| Component | Status | Description |
|-----------|--------|-------------|
| IR Generation | 🔄 Planned | Intermediate representation |
| Optimizer | 🔄 Planned | Basic optimizations |
| LLVM Backend | 🔄 Planned | Native code generation |
| WASM Backend | 🔄 Planned | Web Assembly target |

### Phase 4: Tooling
**Target: Q2 2025**

| Component | Status | Description |
|-----------|--------|-------------|
| Package Manager | 🔄 Planned | mlpkg for dependencies |
| Language Server | 🔄 Planned | LSP implementation |
| Formatter | 🔄 Planned | mlfmt code formatter |
| Linter | 🔄 Planned | mllint static analysis |

### Phase 5: Ecosystem
**Target: Q3 2025**

| Component | Status | Description |
|-----------|--------|-------------|
| Web Framework | 🔄 Planned | Spark web framework |
| AI Framework | 🔄 Planned | Neuron AI toolkit |
| Database Libraries | 🔄 Planned | SQL, NoSQL bindings |
| Package Registry | 🔄 Planned | Central package repository |

## Detailed Roadmaps

- [Language Roadmap](language.md) - Core language features and evolution
- [Compiler Roadmap](compiler.md) - Compiler architecture and backends
- [Tooling Roadmap](tooling.md) - Developer tools and IDE support
- [Ecosystem Roadmap](ecosystem.md) - Frameworks, libraries, community

## Milestones

### v0.1.0 - Parser & Type Checker ✅
- [x] Complete lexer with all token types
- [x] Full recursive descent parser
- [x] AST representation
- [x] Basic type checking
- [x] CLI interface

### v0.2.0 - Interpreter & Runtime ✅
- [x] Tree-walking interpreter
- [x] REPL environment
- [x] Mock AI runtime
- [x] Core standard library (60+ functions)
- [x] Static Site Generator (my-ssg)

### v0.3.0 - Full Type System (Next)
- [ ] Generic types
- [ ] Trait system
- [ ] Effect inference
- [ ] Type constraint solving

### v0.4.0 - Compiler MVP
- [ ] IR design and generation
- [ ] Basic optimizations
- [ ] Single backend (LLVM or Cranelift)
- [ ] Executable generation

### v0.5.0 - Tooling MVP
- [ ] Package manager basics
- [ ] Language server protocol
- [ ] VS Code extension

### v1.0.0 - Production Ready
- [ ] Stable language specification
- [ ] Optimizing compiler
- [ ] Complete standard library
- [ ] Production AI runtime
- [ ] Comprehensive tooling

## Current Components

### Core Language (`my-lang`)
```
src/
├── ast.rs          # Abstract syntax tree
├── checker.rs      # Type checker
├── interpreter.rs  # Tree-walking interpreter
├── lexer.rs        # Tokenizer
├── lib.rs          # Library entry point
├── main.rs         # CLI entry point
├── parser.rs       # Recursive descent parser
├── scope.rs        # Symbol table
├── stdlib.rs       # Standard library (60+ functions)
├── token.rs        # Token definitions
└── types.rs        # Type system
```

### Standard Library
- **I/O**: print, println, debug, input
- **Strings**: len, concat, split, join, trim, upper, lower, contains, replace
- **Math**: abs, min, max, floor, ceil, round, sqrt, pow, sin, cos, tan, log
- **Arrays**: push, pop, first, last, get, set, concat, slice, reverse, range
- **Types**: type_of, to_string, to_int, to_float, to_bool, is_*
- **Utilities**: assert, panic, time, sleep, random, env

### My SSG (`my-ssg`)
Static site generator powered by My Language:
```
my-ssg/
├── src/
│   ├── main.rs      # CLI entry point
│   ├── config.rs    # Configuration parsing
│   ├── generator.rs # Site generation
│   ├── markdown.rs  # Markdown processing
│   └── template.rs  # Template engine with My Language
└── Cargo.toml
```

Features:
- Markdown with YAML frontmatter
- Template system with My Language expressions
- Control flow (if/else, for loops)
- Filters (uppercase, lowercase, truncate, etc.)
- Static file copying
- Blog post support

## Security Status

Last Review: 2025-12-17

| Check | Status |
|-------|--------|
| No shell execution | ✅ Pass |
| No unsafe code | ✅ Pass |
| Sandboxed eval | ✅ Pass |
| File I/O scoped | ✅ Pass |

## Contributing to the Roadmap

We welcome community input on prioritization. Please open issues or discussions on GitHub to propose changes or additions to the roadmap.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Complete |
| 🔄 | Planned/In Progress |
| ❌ | Blocked/Deferred |
| 🧪 | Experimental |
