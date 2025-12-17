# Compiler Architecture

This document describes the internal architecture of the My Language compiler.

## Overview

The compiler follows a traditional multi-stage pipeline:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Source Code                               │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                     1. Lexer (Tokenization)                      │
│                        src/lexer.rs                              │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                     2. Parser (AST Generation)                   │
│                        src/parser.rs                             │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                 3. Type Checker (Semantic Analysis)              │
│                        src/checker.rs                            │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    4. HIR (High-Level IR)                        │
│                         (Planned)                                │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    5. MIR (Mid-Level IR)                         │
│                         (Planned)                                │
└─────────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    ▼                       ▼
        ┌───────────────────┐   ┌───────────────────┐
        │   6. Interpreter  │   │  7. Code Gen      │
        │     (Planned)     │   │    (Planned)      │
        └───────────────────┘   └───────────────────┘
```

## Current Implementation

### Source Files

| File | Purpose | Lines | Status |
|------|---------|-------|--------|
| `src/token.rs` | Token definitions | ~150 | ✅ Complete |
| `src/lexer.rs` | Tokenization | ~400 | ✅ Complete |
| `src/ast.rs` | AST node definitions | ~500 | ✅ Complete |
| `src/parser.rs` | Recursive descent parser | ~1800 | ✅ Complete |
| `src/types.rs` | Internal type representation | ~150 | ✅ Complete |
| `src/scope.rs` | Symbol table and scopes | ~200 | ✅ Complete |
| `src/checker.rs` | Type checking | ~1000 | ✅ Complete |
| `src/lib.rs` | Library exports | ~50 | ✅ Complete |
| `src/main.rs` | CLI interface | ~100 | ✅ Complete |

### Module Dependency Graph

```
main.rs
  │
  └─── lib.rs
         │
         ├─── lexer.rs ─── token.rs
         │
         ├─── parser.rs ─── ast.rs
         │                    │
         │                    └─── token.rs
         │
         └─── checker.rs ─── types.rs
                              │
                              └─── scope.rs
```

## Stage 1: Lexer

The lexer (`src/lexer.rs`) converts source text into a stream of tokens.

### Token Types

```rust
pub enum TokenKind {
    // Literals
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    CharLiteral,

    // Identifiers and Keywords
    Ident,
    Fn, Let, Mut, If, Else, Match, ...

    // AI Keywords
    Ai, Query, Verify, Generate, Embed, Classify, ...

    // Operators
    Plus, Minus, Star, Slash, ...

    // Delimiters
    LParen, RParen, LBrace, RBrace, ...

    // Special
    Eof, Error,
}
```

### Lexer Implementation

```rust
pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self { ... }

    pub fn next_token(&mut self) -> Token { ... }

    fn skip_whitespace(&mut self) { ... }
    fn read_identifier(&mut self) -> Token { ... }
    fn read_number(&mut self) -> Token { ... }
    fn read_string(&mut self) -> Token { ... }
}
```

### Span Tracking

Every token includes span information for error reporting:

```rust
pub struct Span {
    pub start: usize,   // Byte offset
    pub end: usize,     // Byte offset
    pub line: usize,    // 1-indexed
    pub column: usize,  // 1-indexed
}

pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub literal: Option<String>,
}
```

## Stage 2: Parser

The parser (`src/parser.rs`) builds an Abstract Syntax Tree from tokens.

### Parser Design

We use a recursive descent parser with Pratt parsing for expressions:

```rust
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut items = Vec::new();
        while !self.at_end() {
            items.push(self.parse_top_level()?);
        }
        Ok(Program { items })
    }
}
```

### Expression Parsing (Pratt)

```rust
fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
    let mut lhs = self.parse_prefix()?;

    loop {
        let op = match self.current.kind {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            // ...
            _ => break,
        };

        let (l_bp, r_bp) = op.binding_power();
        if l_bp < min_bp {
            break;
        }

        self.advance();
        let rhs = self.parse_expression(r_bp)?;
        lhs = Expr::Binary { lhs, op, rhs };
    }

    Ok(lhs)
}
```

### AST Nodes

```rust
pub enum Expr {
    Literal { value: Literal, span: Span },
    Ident { name: Ident },
    Binary { lhs: Box<Expr>, op: BinOp, rhs: Box<Expr>, span: Span },
    Unary { op: UnaryOp, expr: Box<Expr>, span: Span },
    Call { callee: Box<Expr>, args: Vec<Expr>, span: Span },
    If { cond: Box<Expr>, then_branch: Block, else_branch: Option<Block>, span: Span },
    Match { expr: Box<Expr>, arms: Vec<MatchArm>, span: Span },
    Lambda { params: Vec<Param>, body: Box<Expr>, span: Span },
    AI { expr: AiExpr, span: Span },
    // ...
}

pub enum Stmt {
    Let { name: Ident, ty: Option<Type>, value: Expr, mutable: bool, span: Span },
    Expr { expr: Expr, span: Span },
    Return { value: Option<Expr>, span: Span },
    // ...
}

pub struct FnDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_ty: Option<Type>,
    pub body: Block,
    pub is_async: bool,
    pub effects: Vec<Ident>,
    pub contracts: Vec<Contract>,
    pub span: Span,
}
```

## Stage 3: Type Checker

The type checker (`src/checker.rs`) performs semantic analysis.

### Two-Pass Analysis

1. **Collection Pass**: Gather all definitions (structs, functions, AI models)
2. **Checking Pass**: Type-check expressions and statements

```rust
pub struct Checker {
    symbols: SymbolTable,
    types: TypeEnv,
    errors: Vec<CheckError>,
}

impl Checker {
    pub fn check(&mut self, program: &Program) -> Vec<CheckError> {
        // Pass 1: Collect definitions
        for item in &program.items {
            self.collect_definition(item);
        }

        // Pass 2: Type check
        for item in &program.items {
            self.check_item(item);
        }

        std::mem::take(&mut self.errors)
    }
}
```

### Type Representation

```rust
pub enum Ty {
    // Primitives
    Int, Float, String, Bool, Unit,

    // Compound
    Named(String),
    Function { params: Vec<Ty>, result: Box<Ty> },
    Array(Box<Ty>),
    Tuple(Vec<Ty>),
    Ref { mutable: bool, inner: Box<Ty> },

    // AI
    AI(Box<Ty>),
    Effect(Box<Ty>),

    // Inference
    Var(usize),
    Error,
    Unknown,
}
```

### Symbol Table

```rust
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

pub struct Scope {
    symbols: HashMap<String, Symbol>,
}

pub struct Symbol {
    pub name: String,
    pub ty: Ty,
    pub kind: SymbolKind,
    pub mutable: bool,
}

impl SymbolTable {
    pub fn enter_scope(&mut self) { ... }
    pub fn exit_scope(&mut self) { ... }
    pub fn define(&mut self, name: String, symbol: Symbol) { ... }
    pub fn lookup(&self, name: &str) -> Option<&Symbol> { ... }
}
```

### Error Types

```rust
pub enum CheckError {
    UndefinedVariable { name: String, line: usize, column: usize },
    TypeMismatch { expected: Ty, found: Ty, line: usize, column: usize },
    UndefinedAiModel { name: String, line: usize, column: usize },
    UndefinedPrompt { name: String, line: usize, column: usize },
    WrongArgumentCount { expected: usize, found: usize, line: usize, column: usize },
    // ...
}
```

## Planned Stages

### HIR (High-Level IR)

Purpose: Desugared, fully typed representation

Transformations:
- Name resolution (all identifiers become unique IDs)
- Type annotations on all expressions
- Desugar: for loops, if-let, method calls
- Lower string interpolation

### MIR (Mid-Level IR)

Purpose: Control-flow graph for analysis and optimization

Structure:
- Basic blocks
- SSA form
- Explicit drops
- Borrow checking

### Code Generation

Planned backends:
- LLVM (native binaries)
- Cranelift (fast JIT)
- WebAssembly

## Testing Strategy

### Unit Tests

Each module has inline tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let mut lexer = Lexer::new("let x = 5;");
        assert_eq!(lexer.next_token().kind, TokenKind::Let);
        // ...
    }
}
```

### Integration Tests

In `tests/`:
- Parser tests with example programs
- Type checker tests
- Error message tests

### Running Tests

```bash
cargo test              # All tests
cargo test lexer        # Lexer tests only
cargo test --lib        # Library tests only
```

## Performance Considerations

### Lexer
- Single-pass, streaming
- No allocations for keywords
- Span calculation is O(1)

### Parser
- Recursive descent (simple, maintainable)
- Pratt parsing for expressions (efficient precedence handling)
- Arena allocation for AST (future)

### Type Checker
- Two-pass for forward references
- Hash-based symbol lookup
- Incremental checking (future)

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for:
- Code style guidelines
- Testing requirements
- PR process
- Architecture decisions
