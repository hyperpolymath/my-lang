# Contributing to My Language

Thank you for your interest in contributing to My Language! This guide covers the development workflow and standards.

## Quick Start

```bash
# Clone the repository
git clone https://github.com/hyperpolymath/my-lang.git
cd my-lang

# Build
cargo build --workspace

# Run tests
cargo test --workspace

# Run the compiler
cargo run -p my-lang -- run examples/hello.my
```

## Project Structure

```
my-lang/
├── crates/
│   ├── my-lang/        # Core language (parser, checker, interpreter)
│   ├── my-hir/         # High-level Intermediate Representation
│   ├── my-mir/         # Mid-level IR with CFG
│   ├── my-lsp/         # Language Server Protocol
│   ├── my-ai/          # AI runtime and providers
│   ├── my-pkg/         # Package manager
│   ├── my-fmt/         # Code formatter
│   ├── my-lint/        # Linter
│   └── my-test/        # Test runner
├── tests/              # Integration tests
├── docs/               # Documentation
│   └── wiki/           # Wiki documentation
└── examples/           # Example programs
```

## Compilation Pipeline

```
Source (.my)
    │
    ├─── Lexer ──────► Tokens
    │
    ├─── Parser ─────► AST (Abstract Syntax Tree)
    │
    ├─── Checker ────► Typed AST + Errors
    │
    ├─── HIR Lowering → HIR (simplified AST)
    │
    ├─── MIR Lowering → MIR (CFG with basic blocks)
    │
    └─── Interpreter ─► Execution
         or LLVM ─────► Native binary
```

## Development Standards

### Commit Messages

Use conventional commits:

```
type(scope): subject

Types: feat, fix, docs, test, refactor, chore
Scope: lang, hir, mir, lsp, ai, pkg, fmt, lint, test
```

Examples:
```
feat(lang): add pattern matching for tuples
fix(mir): handle empty blocks in interpreter
docs(wiki): add stdlib design patterns
test(lang): add integration tests for AI expressions
```

### Code Style

- Follow `rustfmt` formatting
- Use `clippy` for linting
- Prefer explicit types over inference in public APIs
- Document all public items

### Security

- No hardcoded credentials
- Use `SecureApiKey` for API key handling
- Validate inputs at system boundaries
- Never log sensitive data

### Error Handling

- Use `Result<T, E>` for fallible operations
- Avoid `unwrap()` in production code
- Use `expect()` with descriptive messages for invariants
- Include context in error messages

### Testing

- Unit tests in `#[cfg(test)]` modules
- Integration tests in `/tests/`
- Test edge cases and error conditions
- Use property-based testing where applicable

## Making Changes

### 1. Create a Branch

```bash
git checkout -b feat/my-feature
```

### 2. Make Changes

- Keep commits focused and atomic
- Update documentation as needed
- Add tests for new functionality

### 3. Run Checks

```bash
# Format code
cargo fmt --all

# Run lints
cargo clippy --workspace

# Run tests
cargo test --workspace

# Check compilation
cargo check --workspace
```

### 4. Submit Pull Request

- Provide clear description
- Reference any related issues
- Ensure CI passes

## Adding a New Feature

### Parser Changes

1. Add tokens to `lexer.rs` if needed
2. Add AST nodes to `ast.rs`
3. Add parsing rules to `parser.rs`
4. Update type checker in `checker.rs`
5. Add HIR/MIR lowering if needed

### Standard Library

1. Add to `/crates/my-lang/src/stdlib.rs`
2. Document in `/docs/wiki/reference/stdlib.md`
3. Add tests
4. Follow the three-component specification format

### LSP Features

1. Implement in `/crates/my-lsp/src/lib.rs`
2. Handle document changes
3. Return proper LSP types
4. Add error handling

## Getting Help

- Check existing documentation in `/docs/wiki/`
- Look at existing code for patterns
- Open an issue for discussion

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
