# Parser Internals

The parser converts a stream of tokens into an Abstract Syntax Tree (AST).

## Overview

Location: `src/parser.rs`

The parser is responsible for:
1. Building hierarchical AST from flat token stream
2. Handling operator precedence
3. Reporting syntax errors with context
4. Supporting all language constructs

## Architecture

### Parser Structure

```rust
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    previous: Token,
    errors: Vec<ParseError>,
}
```

### Core Methods

```rust
impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self;

    pub fn parse(&mut self) -> Result<Program, Vec<ParseError>>;

    // Token management
    fn advance(&mut self) -> Token;
    fn check(&self, kind: TokenKind) -> bool;
    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<Token, ParseError>;

    // Lookahead
    fn peek(&self) -> TokenKind;
    fn peek_literal(&self) -> Option<&str>;
}
```

## Parsing Strategy

### Recursive Descent

Top-level parsing uses straightforward recursive descent:

```rust
pub fn parse(&mut self) -> Result<Program, Vec<ParseError>> {
    let mut items = Vec::new();

    while !self.at_end() {
        match self.parse_top_level() {
            Ok(item) => items.push(item),
            Err(e) => {
                self.errors.push(e);
                self.synchronize();
            }
        }
    }

    if self.errors.is_empty() {
        Ok(Program { items })
    } else {
        Err(std::mem::take(&mut self.errors))
    }
}

fn parse_top_level(&mut self) -> Result<TopLevel, ParseError> {
    // Handle attributes
    let attrs = self.parse_attributes()?;

    match self.peek() {
        TokenKind::Fn => self.parse_fn_decl(attrs),
        TokenKind::Struct => self.parse_struct_decl(attrs),
        TokenKind::Enum => self.parse_enum_decl(attrs),
        TokenKind::Trait => self.parse_trait_decl(attrs),
        TokenKind::Impl => self.parse_impl_block(attrs),
        TokenKind::Type => self.parse_type_alias(attrs),
        TokenKind::Const => self.parse_const_decl(attrs),
        TokenKind::Mod => self.parse_module(attrs),
        TokenKind::Use => self.parse_use_decl(attrs),
        TokenKind::AiModel => self.parse_ai_model(attrs),
        TokenKind::Prompt => self.parse_prompt(attrs),
        _ => Err(self.error("expected top-level item")),
    }
}
```

### Pratt Parsing for Expressions

Expressions use Pratt parsing for correct precedence:

```rust
fn parse_expression(&mut self) -> Result<Expr, ParseError> {
    self.parse_expr_bp(0)
}

fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
    // Parse prefix expression
    let mut lhs = self.parse_prefix()?;

    loop {
        // Check for infix operator
        let op = match self.infix_binding_power() {
            Some((op, l_bp, r_bp)) if l_bp >= min_bp => {
                self.advance();
                (op, r_bp)
            }
            _ => break,
        };

        let rhs = self.parse_expr_bp(op.1)?;
        lhs = Expr::Binary {
            lhs: Box::new(lhs),
            op: op.0,
            rhs: Box::new(rhs),
            span: self.span(),
        };
    }

    // Handle postfix operators
    lhs = self.parse_postfix(lhs)?;

    Ok(lhs)
}
```

### Binding Power Table

```rust
fn infix_binding_power(&self) -> Option<(BinOp, u8, u8)> {
    let kind = self.peek();
    let (op, l_bp, r_bp) = match kind {
        // Assignment (right associative)
        TokenKind::Eq => (BinOp::Assign, 2, 1),

        // Logical OR
        TokenKind::PipePipe => (BinOp::Or, 4, 5),

        // Logical AND
        TokenKind::AmpAmp => (BinOp::And, 6, 7),

        // Comparison
        TokenKind::EqEq => (BinOp::Eq, 8, 9),
        TokenKind::BangEq => (BinOp::Neq, 8, 9),
        TokenKind::Lt => (BinOp::Lt, 10, 11),
        TokenKind::Gt => (BinOp::Gt, 10, 11),
        TokenKind::LtEq => (BinOp::Leq, 10, 11),
        TokenKind::GtEq => (BinOp::Geq, 10, 11),

        // Bitwise OR
        TokenKind::Pipe => (BinOp::BitOr, 12, 13),

        // Bitwise XOR
        TokenKind::Caret => (BinOp::BitXor, 14, 15),

        // Bitwise AND
        TokenKind::Amp => (BinOp::BitAnd, 16, 17),

        // Shift
        TokenKind::LtLt => (BinOp::Shl, 18, 19),
        TokenKind::GtGt => (BinOp::Shr, 18, 19),

        // Additive
        TokenKind::Plus => (BinOp::Add, 20, 21),
        TokenKind::Minus => (BinOp::Sub, 20, 21),

        // Multiplicative
        TokenKind::Star => (BinOp::Mul, 22, 23),
        TokenKind::Slash => (BinOp::Div, 22, 23),
        TokenKind::Percent => (BinOp::Mod, 22, 23),

        // Exponentiation (right associative)
        TokenKind::StarStar => (BinOp::Pow, 25, 24),

        _ => return None,
    };
    Some((op, l_bp, r_bp))
}
```

## AST Node Types

### Expressions

```rust
pub enum Expr {
    // Literals
    Literal { value: Literal, span: Span },

    // Names
    Ident { name: Ident },
    Path { segments: Vec<Ident>, span: Span },

    // Operations
    Binary { lhs: Box<Expr>, op: BinOp, rhs: Box<Expr>, span: Span },
    Unary { op: UnaryOp, expr: Box<Expr>, span: Span },

    // Function-related
    Call { callee: Box<Expr>, args: Vec<Expr>, span: Span },
    MethodCall { receiver: Box<Expr>, method: Ident, args: Vec<Expr>, span: Span },
    Lambda { params: Vec<Param>, body: Box<Expr>, span: Span },

    // Control flow
    If { cond: Box<Expr>, then_branch: Block, else_branch: Option<Block>, span: Span },
    Match { expr: Box<Expr>, arms: Vec<MatchArm>, span: Span },
    Loop { body: Block, span: Span },
    While { cond: Box<Expr>, body: Block, span: Span },
    For { pattern: Pattern, iter: Box<Expr>, body: Block, span: Span },

    // Data construction
    Tuple { elements: Vec<Expr>, span: Span },
    Array { elements: Vec<Expr>, span: Span },
    Struct { name: Path, fields: Vec<(Ident, Expr)>, span: Span },

    // References
    Ref { mutable: bool, expr: Box<Expr>, span: Span },
    Deref { expr: Box<Expr>, span: Span },

    // Field access
    Field { expr: Box<Expr>, field: Ident, span: Span },
    Index { expr: Box<Expr>, index: Box<Expr>, span: Span },

    // AI expressions
    AI { expr: AiExpr, span: Span },

    // Async
    Await { expr: Box<Expr>, span: Span },

    // Block
    Block { block: Block, span: Span },

    // Error recovery
    Error { span: Span },
}
```

### Statements

```rust
pub enum Stmt {
    Let {
        name: Ident,
        ty: Option<Type>,
        value: Expr,
        mutable: bool,
        span: Span,
    },
    Expr {
        expr: Expr,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Break {
        value: Option<Expr>,
        span: Span,
    },
    Continue {
        span: Span,
    },
    AI {
        stmt: AiStmt,
        span: Span,
    },
}
```

### Declarations

```rust
pub struct FnDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Option<Generics>,
    pub params: Vec<Param>,
    pub return_ty: Option<Type>,
    pub where_clause: Option<WhereClause>,
    pub contracts: Vec<Contract>,
    pub body: Block,
    pub is_async: bool,
    pub span: Span,
}

pub struct StructDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Option<Generics>,
    pub fields: Vec<StructField>,
    pub invariants: Vec<Expr>,
    pub span: Span,
}

pub struct EnumDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Option<Generics>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}
```

## Parsing Specific Constructs

### Functions

```rust
fn parse_fn_decl(&mut self, attrs: Vec<Attribute>) -> Result<FnDecl, ParseError> {
    let is_async = self.check(TokenKind::Async);
    if is_async {
        self.advance();
    }

    self.consume(TokenKind::Fn, "expected 'fn'")?;

    let name = self.parse_ident()?;
    let generics = self.parse_optional_generics()?;

    self.consume(TokenKind::LParen, "expected '('")?;
    let params = self.parse_param_list()?;
    self.consume(TokenKind::RParen, "expected ')'")?;

    let return_ty = if self.check(TokenKind::Arrow) {
        self.advance();
        Some(self.parse_type()?)
    } else {
        None
    };

    let where_clause = self.parse_optional_where_clause()?;
    let contracts = self.parse_contracts()?;

    let body = self.parse_block()?;

    Ok(FnDecl {
        attrs,
        visibility: Visibility::Private,
        name,
        generics,
        params,
        return_ty,
        where_clause,
        contracts,
        body,
        is_async,
        span: self.span(),
    })
}
```

### Match Expressions

```rust
fn parse_match_expr(&mut self) -> Result<Expr, ParseError> {
    self.consume(TokenKind::Match, "expected 'match'")?;
    let expr = self.parse_expression()?;

    self.consume(TokenKind::LBrace, "expected '{'")?;

    let mut arms = Vec::new();
    while !self.check(TokenKind::RBrace) {
        arms.push(self.parse_match_arm()?);

        if !self.check(TokenKind::RBrace) {
            self.consume(TokenKind::Comma, "expected ',' or '}'")?;
        }
    }

    self.consume(TokenKind::RBrace, "expected '}'")?;

    Ok(Expr::Match {
        expr: Box::new(expr),
        arms,
        span: self.span(),
    })
}

fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
    let pattern = self.parse_pattern()?;

    let guard = if self.check(TokenKind::If) {
        self.advance();
        Some(self.parse_expression()?)
    } else {
        None
    };

    self.consume(TokenKind::FatArrow, "expected '=>'")?;

    let body = if self.check(TokenKind::LBrace) {
        Expr::Block { block: self.parse_block()?, span: self.span() }
    } else {
        self.parse_expression()?
    };

    Ok(MatchArm { pattern, guard, body })
}
```

### AI Expressions

```rust
fn parse_ai_expr(&mut self) -> Result<AiExpr, ParseError> {
    self.consume(TokenKind::Ai, "expected 'ai'")?;

    // ai! { "quick query" }
    if self.check(TokenKind::Bang) {
        self.advance();
        self.consume(TokenKind::LBrace, "expected '{'")?;
        let query = self.parse_expression()?;
        self.consume(TokenKind::RBrace, "expected '}'")?;
        return Ok(AiExpr::Quick { query: Box::new(query), span: self.span() });
    }

    // ai query { ... }
    let keyword = self.parse_ai_keyword()?;

    if self.check(TokenKind::LBrace) {
        // Block form
        self.advance();
        let fields = self.parse_ai_fields()?;
        self.consume(TokenKind::RBrace, "expected '}'")?;
        Ok(AiExpr::Block { keyword, fields, span: self.span() })
    } else if self.check(TokenKind::LParen) {
        // Call form: ai embed(text)
        self.advance();
        let args = self.parse_expr_list()?;
        self.consume(TokenKind::RParen, "expected ')'")?;
        Ok(AiExpr::Call { keyword, args, span: self.span() })
    } else {
        Err(self.error("expected '{' or '('"))
    }
}

fn parse_ai_keyword(&mut self) -> Result<AiKeyword, ParseError> {
    match self.peek() {
        TokenKind::Query => { self.advance(); Ok(AiKeyword::Query) }
        TokenKind::Verify => { self.advance(); Ok(AiKeyword::Verify) }
        TokenKind::Generate => { self.advance(); Ok(AiKeyword::Generate) }
        TokenKind::Embed => { self.advance(); Ok(AiKeyword::Embed) }
        TokenKind::Classify => { self.advance(); Ok(AiKeyword::Classify) }
        TokenKind::Optimize => { self.advance(); Ok(AiKeyword::Optimize) }
        TokenKind::Test => { self.advance(); Ok(AiKeyword::Test) }
        TokenKind::Infer => { self.advance(); Ok(AiKeyword::Infer) }
        TokenKind::Constrain => { self.advance(); Ok(AiKeyword::Constrain) }
        TokenKind::Validate => { self.advance(); Ok(AiKeyword::Validate) }
        _ => Err(self.error("expected AI keyword")),
    }
}
```

## Pattern Parsing

```rust
fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
    self.parse_or_pattern()
}

fn parse_or_pattern(&mut self) -> Result<Pattern, ParseError> {
    let mut pattern = self.parse_pattern_no_or()?;

    while self.check(TokenKind::Pipe) {
        self.advance();
        let right = self.parse_pattern_no_or()?;
        pattern = Pattern::Or {
            left: Box::new(pattern),
            right: Box::new(right),
            span: self.span(),
        };
    }

    Ok(pattern)
}

fn parse_pattern_no_or(&mut self) -> Result<Pattern, ParseError> {
    match self.peek() {
        TokenKind::Underscore => {
            self.advance();
            Ok(Pattern::Wildcard { span: self.span() })
        }
        TokenKind::IntLiteral | TokenKind::StringLiteral |
        TokenKind::True | TokenKind::False => {
            let lit = self.parse_literal()?;
            Ok(Pattern::Literal { value: lit, span: self.span() })
        }
        TokenKind::Ident => {
            let name = self.parse_ident()?;

            // Check for struct pattern
            if self.check(TokenKind::LBrace) {
                self.parse_struct_pattern(name)
            }
            // Check for tuple struct pattern
            else if self.check(TokenKind::LParen) {
                self.parse_tuple_struct_pattern(name)
            }
            // Check for @ binding
            else if self.check(TokenKind::At) {
                self.advance();
                let pattern = self.parse_pattern()?;
                Ok(Pattern::Binding {
                    name,
                    pattern: Some(Box::new(pattern)),
                    span: self.span(),
                })
            }
            // Simple identifier pattern
            else {
                Ok(Pattern::Ident { name, span: self.span() })
            }
        }
        TokenKind::LParen => self.parse_tuple_pattern(),
        TokenKind::LBracket => self.parse_slice_pattern(),
        _ => Err(self.error("expected pattern")),
    }
}
```

## Error Handling

### Error Recovery

```rust
fn synchronize(&mut self) {
    self.advance();

    while !self.at_end() {
        // Synchronize at statement boundaries
        if self.previous.kind == TokenKind::Semicolon {
            return;
        }

        // Synchronize at keyword boundaries
        match self.peek() {
            TokenKind::Fn | TokenKind::Struct | TokenKind::Enum |
            TokenKind::Trait | TokenKind::Impl | TokenKind::Let |
            TokenKind::If | TokenKind::For | TokenKind::While |
            TokenKind::Return => return,
            _ => {}
        }

        self.advance();
    }
}
```

### Error Messages

```rust
fn error(&self, message: &str) -> ParseError {
    ParseError {
        message: message.to_string(),
        span: self.current.span.clone(),
        expected: None,
        found: Some(self.current.kind),
    }
}

fn expected(&self, expected: &str) -> ParseError {
    ParseError {
        message: format!("expected {}, found {:?}", expected, self.current.kind),
        span: self.current.span.clone(),
        expected: Some(expected.to_string()),
        found: Some(self.current.kind),
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Result<Program, Vec<ParseError>> {
        Parser::new(source).parse()
    }

    #[test]
    fn test_simple_function() {
        let result = parse("fn main() { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ai_expression() {
        let result = parse(r#"
            fn test() {
                let x = ai! { "What is 2+2?" };
            }
        "#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_expression() {
        let result = parse(r#"
            fn test(x: Int) {
                match x {
                    0 => "zero",
                    1 | 2 => "small",
                    n if n < 0 => "negative",
                    _ => "other",
                }
            }
        "#);
        assert!(result.is_ok());
    }
}
```
