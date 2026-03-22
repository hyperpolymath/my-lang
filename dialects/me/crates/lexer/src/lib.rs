// Solo Language Lexer
// Tokenizes source code into a stream of tokens

use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]  // Skip whitespace
#[logos(skip r"//[^\n]*")]     // Skip line comments
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]  // Skip block comments
pub enum Token {
    // Keywords
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("match")]
    Match,
    #[token("loop")]
    Loop,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
    #[token("mod")]
    Mod,
    #[token("import")]
    Import,
    #[token("use")]
    Use,
    #[token("pub")]
    Pub,
    #[token("as")]
    As,
    #[token("where")]
    Where,
    #[token("type")]
    Type,
    #[token("const")]
    Const,
    #[token("static")]
    Static,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("move")]
    Move,
    #[token("ref")]
    Ref,
    #[token("unsafe")]
    Unsafe,
    #[token("true")]
    True,
    #[token("false")]
    False,

    // Affine types
    #[token("affine")]
    Affine,

    // Comptime
    #[token("comptime")]
    Comptime,

    // Contracts
    #[token("pre")]
    Pre,
    #[token("post")]
    Post,
    #[token("invariant")]
    Invariant,
    #[token("requires")]
    Requires,
    #[token("ensures")]
    Ensures,

    // Duet-specific keywords
    #[token("intent")]
    Intent,
    #[token("hybrid")]
    Hybrid,

    // Ensemble-specific keywords (Variant A)
    #[token("agent")]
    Agent,
    #[token("workflow")]
    Workflow,
    #[token("spawn")]
    Spawn,
    #[token("send")]
    Send,
    #[token("receive")]
    Receive,
    #[token("broadcast")]
    Broadcast,
    #[token("state")]
    State,
    #[token("capabilities")]
    Capabilities,
    #[token("goals")]
    Goals,
    #[token("constraints")]
    Constraints,
    #[token("communication")]
    Communication,

    // Primitive types
    #[token("i8")]
    I8,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,
    #[token("i128")]
    I128,
    #[token("isize")]
    Isize,
    #[token("u8")]
    U8,
    #[token("u16")]
    U16,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,
    #[token("u128")]
    U128,
    #[token("usize")]
    Usize,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("bool")]
    Bool,
    #[token("char")]
    Char,
    #[token("str")]
    Str,

    // Identifiers and literals
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"-?[0-9]+", priority = 2)]
    IntLiteral,

    #[regex(r"-?[0-9]+\.[0-9]+")]
    FloatLiteral,

    #[regex(r#""([^"\\]|\\["\\bnfrt]|\\u\{[a-fA-F0-9]+\})*""#)]
    StringLiteral,

    #[regex(r"'([^'\\]|\\['\\bnfrt]|\\u\{[a-fA-F0-9]+\})'")]
    CharLiteral,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("==")]
    EqualEqual,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("<<")]
    LeftShift,
    #[token(">>")]
    RightShift,
    #[token("=")]
    Equal,
    #[token("+=")]
    PlusEqual,
    #[token("-=")]
    MinusEqual,
    #[token("*=")]
    StarEqual,
    #[token("/=")]
    SlashEqual,
    #[token("%=")]
    PercentEqual,

    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("::")]
    ColonColon,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,
    #[token("..=")]
    DotDotEqual,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("#")]
    Hash,
    #[token("@")]
    At,
    #[token("?")]
    Question,
    #[token("_")]
    Underscore,
    #[token("'")]
    Lifetime,

    // Special
    Error,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Lexer<'source> {
    lexer: logos::Lexer<'source, Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
        }
    }

    pub fn next_token(&mut self) -> (Token, &'source str) {
        match self.lexer.next() {
            Some(Ok(token)) => (token, self.lexer.slice()),
            Some(Err(_)) => (Token::Error, self.lexer.slice()),
            None => (Token::Eof, ""),
        }
    }

    pub fn tokenize_all(&mut self) -> Vec<(Token, String)> {
        let mut tokens = Vec::new();
        loop {
            let (token, slice) = self.next_token();
            if token == Token::Eof {
                tokens.push((token, slice.to_string()));
                break;
            }
            tokens.push((token, slice.to_string()));
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "fn let mut if else";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all();

        assert_eq!(tokens[0].0, Token::Fn);
        assert_eq!(tokens[1].0, Token::Let);
        assert_eq!(tokens[2].0, Token::Mut);
        assert_eq!(tokens[3].0, Token::If);
        assert_eq!(tokens[4].0, Token::Else);
    }

    #[test]
    fn test_identifiers() {
        let source = "foo bar _baz foo123";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all();

        assert_eq!(tokens[0].0, Token::Identifier);
        assert_eq!(tokens[0].1, "foo");
        assert_eq!(tokens[1].0, Token::Identifier);
        assert_eq!(tokens[1].1, "bar");
    }

    #[test]
    fn test_literals() {
        let source = r#"42 3.14 "hello" 'c' true false"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all();

        assert_eq!(tokens[0].0, Token::IntLiteral);
        assert_eq!(tokens[1].0, Token::FloatLiteral);
        assert_eq!(tokens[2].0, Token::StringLiteral);
        assert_eq!(tokens[3].0, Token::CharLiteral);
        assert_eq!(tokens[4].0, Token::True);
        assert_eq!(tokens[5].0, Token::False);
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / == != < > && ||";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all();

        assert_eq!(tokens[0].0, Token::Plus);
        assert_eq!(tokens[1].0, Token::Minus);
        assert_eq!(tokens[2].0, Token::Star);
        assert_eq!(tokens[3].0, Token::Slash);
        assert_eq!(tokens[4].0, Token::EqualEqual);
        assert_eq!(tokens[5].0, Token::NotEqual);
    }

    #[test]
    fn test_function() {
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all();

        assert_eq!(tokens[0].0, Token::Fn);
        assert_eq!(tokens[1].0, Token::Identifier);
        assert_eq!(tokens[2].0, Token::LeftParen);
    }

    #[test]
    fn test_comments() {
        let source = "fn foo() { // comment\n  42 }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all();

        // Comment should be skipped
        assert_eq!(tokens[4].0, Token::IntLiteral);
    }
}
