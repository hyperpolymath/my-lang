/// Token types for Solo dialect lexer
///
/// Solo syntax is Rust-inspired with affine types and arena allocation.

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    Return,
    Struct,
    Enum,
    Impl,
    Trait,
    Type,
    Arena,
    Belief,  // Epistemic extension
    Where,   // Type constraints

    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    True,
    False,

    // Identifiers
    Identifier(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Arrow,      // ->
    FatArrow,   // =>
    Ampersand,  // &
    Pipe,       // |
    Tilde,      // ~ (for belief distributions)

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    DoubleColon,  // ::
    Dot,

    // Special
    Eof,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
        }
    }

    pub fn is_keyword(ident: &str) -> Option<TokenKind> {
        match ident {
            "fn" => Some(TokenKind::Fn),
            "let" => Some(TokenKind::Let),
            "mut" => Some(TokenKind::Mut),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "while" => Some(TokenKind::While),
            "return" => Some(TokenKind::Return),
            "struct" => Some(TokenKind::Struct),
            "enum" => Some(TokenKind::Enum),
            "impl" => Some(TokenKind::Impl),
            "trait" => Some(TokenKind::Trait),
            "type" => Some(TokenKind::Type),
            "arena" => Some(TokenKind::Arena),
            "belief" => Some(TokenKind::Belief),
            "where" => Some(TokenKind::Where),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            _ => None,
        }
    }
}
