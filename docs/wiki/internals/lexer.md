# Lexer Internals

The lexer (tokenizer) converts source text into a stream of tokens for the parser.

## Overview

Location: `src/lexer.rs`

The lexer is responsible for:
1. Breaking source code into tokens
2. Handling comments and whitespace
3. Recognizing keywords vs identifiers
4. Tracking source positions for error reporting

## Architecture

```rust
pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    position: usize,
    line: usize,
    column: usize,
}
```

### Key Methods

```rust
impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source
    pub fn new(source: &'a str) -> Self;

    /// Get the next token
    pub fn next_token(&mut self) -> Token;

    /// Peek at current character without consuming
    fn peek(&mut self) -> Option<char>;

    /// Advance and return current character
    fn advance(&mut self) -> Option<char>;

    /// Check if at end of input
    fn at_end(&self) -> bool;
}
```

## Token Structure

```rust
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub literal: Option<String>,
}

pub struct Span {
    pub start: usize,   // Byte offset in source
    pub end: usize,     // Byte offset (exclusive)
    pub line: usize,    // Line number (1-indexed)
    pub column: usize,  // Column number (1-indexed)
}
```

## Token Categories

### Literals

```rust
TokenKind::IntLiteral      // 42, 0xFF, 0b1010
TokenKind::FloatLiteral    // 3.14, 1e-10
TokenKind::StringLiteral   // "hello", """multiline"""
TokenKind::CharLiteral     // 'a', '\n'
```

### Keywords

```rust
// Declaration keywords
TokenKind::Fn, TokenKind::Let, TokenKind::Mut,
TokenKind::Const, TokenKind::Type, TokenKind::Struct,
TokenKind::Enum, TokenKind::Trait, TokenKind::Impl,

// Control flow
TokenKind::If, TokenKind::Else, TokenKind::Match,
TokenKind::For, TokenKind::While, TokenKind::Loop,
TokenKind::Break, TokenKind::Continue, TokenKind::Return,

// AI keywords
TokenKind::Ai, TokenKind::Query, TokenKind::Verify,
TokenKind::Generate, TokenKind::Embed, TokenKind::Classify,
```

### Operators

```rust
// Arithmetic
TokenKind::Plus, TokenKind::Minus, TokenKind::Star,
TokenKind::Slash, TokenKind::Percent, TokenKind::StarStar,

// Comparison
TokenKind::EqEq, TokenKind::BangEq,
TokenKind::Lt, TokenKind::Gt, TokenKind::LtEq, TokenKind::GtEq,

// Logical
TokenKind::AmpAmp, TokenKind::PipePipe, TokenKind::Bang,

// Assignment
TokenKind::Eq, TokenKind::PlusEq, TokenKind::MinusEq,
```

### Delimiters

```rust
TokenKind::LParen, TokenKind::RParen,   // ()
TokenKind::LBrace, TokenKind::RBrace,   // {}
TokenKind::LBracket, TokenKind::RBracket, // []
TokenKind::Comma, TokenKind::Semicolon,
TokenKind::Colon, TokenKind::ColonColon,
TokenKind::Arrow, TokenKind::FatArrow,  // -> =>
```

## Lexing Algorithm

### Main Loop

```rust
pub fn next_token(&mut self) -> Token {
    self.skip_whitespace_and_comments();

    if self.at_end() {
        return self.make_token(TokenKind::Eof);
    }

    let start_position = self.position;
    let start_line = self.line;
    let start_column = self.column;

    let c = self.advance().unwrap();

    let kind = match c {
        // Single-character tokens
        '(' => TokenKind::LParen,
        ')' => TokenKind::RParen,
        '{' => TokenKind::LBrace,
        '}' => TokenKind::RBrace,
        '[' => TokenKind::LBracket,
        ']' => TokenKind::RBracket,
        ',' => TokenKind::Comma,
        ';' => TokenKind::Semicolon,

        // Multi-character tokens
        ':' => self.colon_or_path(),
        '=' => self.equals_or_arrow(),
        '<' => self.less_than_variants(),
        '>' => self.greater_than_variants(),
        '&' => self.ampersand_variants(),
        '|' => self.pipe_variants(),

        // Literals
        '"' => self.read_string(),
        '\'' => self.read_char(),
        '0'..='9' => self.read_number(c),

        // Identifiers and keywords
        'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(c),

        _ => TokenKind::Error,
    };

    self.make_token_with_span(kind, start_position, start_line, start_column)
}
```

### Whitespace and Comments

```rust
fn skip_whitespace_and_comments(&mut self) {
    loop {
        match self.peek() {
            Some(' ') | Some('\t') | Some('\r') => {
                self.advance();
            }
            Some('\n') => {
                self.advance();
                self.line += 1;
                self.column = 1;
            }
            Some('/') => {
                if self.peek_next() == Some('/') {
                    // Line comment
                    while self.peek() != Some('\n') && !self.at_end() {
                        self.advance();
                    }
                } else if self.peek_next() == Some('*') {
                    // Block comment
                    self.skip_block_comment();
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
}
```

### Identifiers and Keywords

```rust
fn read_identifier(&mut self, first: char) -> TokenKind {
    let mut ident = String::new();
    ident.push(first);

    while let Some(c) = self.peek() {
        if c.is_alphanumeric() || c == '_' {
            ident.push(c);
            self.advance();
        } else {
            break;
        }
    }

    // Check for keywords
    match ident.as_str() {
        "fn" => TokenKind::Fn,
        "let" => TokenKind::Let,
        "mut" => TokenKind::Mut,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "match" => TokenKind::Match,
        "for" => TokenKind::For,
        "while" => TokenKind::While,
        "loop" => TokenKind::Loop,
        "return" => TokenKind::Return,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "ai" => TokenKind::Ai,
        "query" => TokenKind::Query,
        "verify" => TokenKind::Verify,
        "generate" => TokenKind::Generate,
        // ... more keywords
        _ => TokenKind::Ident,
    }
}
```

### Numbers

```rust
fn read_number(&mut self, first: char) -> TokenKind {
    let mut number = String::new();
    number.push(first);

    // Check for hex, octal, binary
    if first == '0' {
        match self.peek() {
            Some('x') | Some('X') => return self.read_hex_number(),
            Some('o') | Some('O') => return self.read_octal_number(),
            Some('b') | Some('B') => return self.read_binary_number(),
            _ => {}
        }
    }

    // Read integer part
    while let Some(c) = self.peek() {
        if c.is_ascii_digit() || c == '_' {
            if c != '_' {
                number.push(c);
            }
            self.advance();
        } else {
            break;
        }
    }

    // Check for float
    if self.peek() == Some('.') && self.peek_next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        number.push('.');
        self.advance();

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                if c != '_' {
                    number.push(c);
                }
                self.advance();
            } else {
                break;
            }
        }

        // Exponent
        if self.peek() == Some('e') || self.peek() == Some('E') {
            number.push('e');
            self.advance();
            if self.peek() == Some('+') || self.peek() == Some('-') {
                number.push(self.advance().unwrap());
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    number.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        TokenKind::FloatLiteral
    } else {
        TokenKind::IntLiteral
    }
}
```

### Strings

```rust
fn read_string(&mut self) -> TokenKind {
    // Check for multi-line string
    if self.peek() == Some('"') && self.peek_next() == Some('"') {
        self.advance(); // consume second "
        self.advance(); // consume third "
        return self.read_multiline_string();
    }

    let mut string = String::new();

    loop {
        match self.advance() {
            None => return TokenKind::Error, // Unterminated string
            Some('"') => break,
            Some('\\') => {
                // Escape sequence
                match self.advance() {
                    Some('n') => string.push('\n'),
                    Some('r') => string.push('\r'),
                    Some('t') => string.push('\t'),
                    Some('\\') => string.push('\\'),
                    Some('"') => string.push('"'),
                    Some('{') => string.push('{'),
                    Some('0') => string.push('\0'),
                    Some('x') => {
                        // Hex escape \xNN
                        let hex = self.read_hex_escape();
                        string.push(hex);
                    }
                    _ => return TokenKind::Error,
                }
            }
            Some('{') => {
                // String interpolation marker
                // Parser handles this
                string.push('{');
            }
            Some(c) => string.push(c),
        }
    }

    TokenKind::StringLiteral
}
```

## AI-Specific Tokens

### ai! Macro

```rust
// ai! { "prompt" }
// Lexes as: Ai, Bang, LBrace, StringLiteral, RBrace

fn check_ai_bang(&mut self) -> TokenKind {
    if self.peek() == Some('!') {
        self.advance();
        TokenKind::AiBang  // Special token for ai!
    } else {
        TokenKind::Ai
    }
}
```

### Prompt Invocation

```rust
// my_prompt!(arg1, arg2)
// Identifier followed by ! is a macro/prompt invocation

fn read_identifier(&mut self, first: char) -> TokenKind {
    // ... read identifier ...

    if self.peek() == Some('!') {
        self.advance();
        TokenKind::MacroInvoke  // name!
    } else {
        // Regular identifier or keyword
    }
}
```

## Error Handling

The lexer reports errors via the `TokenKind::Error` variant:

```rust
pub fn next_token(&mut self) -> Token {
    // ...
    match c {
        // Unknown character
        _ => {
            let token = self.make_token(TokenKind::Error);
            // Error includes the problematic character in literal
            token.literal = Some(c.to_string());
            token
        }
    }
}
```

Parser can then report:
```
error: unexpected character '§' at line 5, column 12
```

## Performance Optimizations

### Zero-Copy Keywords

Keywords are identified without allocation:

```rust
fn is_keyword(s: &str) -> Option<TokenKind> {
    // Direct string comparison, no allocation
    match s {
        "fn" => Some(TokenKind::Fn),
        "let" => Some(TokenKind::Let),
        // ...
        _ => None,
    }
}
```

### Efficient Lookahead

Single-character lookahead without advancing:

```rust
fn peek(&mut self) -> Option<char> {
    self.chars.peek().map(|(_, c)| *c)
}

fn peek_next(&mut self) -> Option<char> {
    let mut iter = self.chars.clone();
    iter.next();
    iter.peek().map(|(_, c)| *c)
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("let x = 5;");
        assert_eq!(lexer.next_token().kind, TokenKind::Let);
        assert_eq!(lexer.next_token().kind, TokenKind::Ident);
        assert_eq!(lexer.next_token().kind, TokenKind::Eq);
        assert_eq!(lexer.next_token().kind, TokenKind::IntLiteral);
        assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
        assert_eq!(lexer.next_token().kind, TokenKind::Eof);
    }

    #[test]
    fn test_ai_keywords() {
        let mut lexer = Lexer::new("ai query { prompt: x }");
        assert_eq!(lexer.next_token().kind, TokenKind::Ai);
        assert_eq!(lexer.next_token().kind, TokenKind::Query);
        // ...
    }

    #[test]
    fn test_string_interpolation() {
        let mut lexer = Lexer::new(r#""hello {name}""#);
        // ...
    }
}
```

## Future Improvements

1. **Unicode Identifiers**: Full Unicode ID_Start/ID_Continue support
2. **Raw Strings**: `r#"raw string"#` syntax
3. **Streaming**: Iterator-based API for large files
4. **Error Recovery**: Continue lexing after errors
