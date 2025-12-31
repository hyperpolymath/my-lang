//! Token definitions for My Language with AI integration

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub literal: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Literals
    IntLit,
    FloatLit,
    StringLit,
    True,
    False,

    // Identifiers
    Ident,

    // Keywords
    Fn,
    Struct,
    Effect,
    Where,
    Pre,
    Post,
    Invariant,
    Comptime,
    Let,
    Mut,
    If,
    Else,
    Go,
    Return,
    Await,
    Try,
    Restrict,
    Match,
    Use,
    Op,

    // AI Keywords
    Ai,
    AiBang,      // ai!
    Query,
    Verify,
    Generate,
    Embed,
    Classify,
    Optimize,
    Test,
    Infer,
    Constrain,
    Validate,
    Prompt,
    AiModel,
    AiCheck,
    AiValid,
    AiFormat,
    AiInfer,
    AiEnsure,

    // Type Keywords
    Int,
    String,
    Bool,
    Float,
    AI,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    EqEq,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    AndAnd,
    OrOr,
    Bang,
    Question,
    Arrow,       // ->
    FatArrow,    // =>
    ColonColon,  // ::
    Ampersand,   // &
    Pipe,        // |

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
    At,          // @

    // Attributes
    HashBracket, // #[

    // Special
    Eof,
    Error,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::IntLit => write!(f, "integer"),
            TokenKind::FloatLit => write!(f, "float"),
            TokenKind::StringLit => write!(f, "string"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Ident => write!(f, "identifier"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Effect => write!(f, "effect"),
            TokenKind::Where => write!(f, "where"),
            TokenKind::Pre => write!(f, "pre"),
            TokenKind::Post => write!(f, "post"),
            TokenKind::Invariant => write!(f, "invariant"),
            TokenKind::Comptime => write!(f, "comptime"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Go => write!(f, "go"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Await => write!(f, "await"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Restrict => write!(f, "restrict"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Op => write!(f, "op"),
            TokenKind::Ai => write!(f, "ai"),
            TokenKind::AiBang => write!(f, "ai!"),
            TokenKind::Query => write!(f, "query"),
            TokenKind::Verify => write!(f, "verify"),
            TokenKind::Generate => write!(f, "generate"),
            TokenKind::Embed => write!(f, "embed"),
            TokenKind::Classify => write!(f, "classify"),
            TokenKind::Optimize => write!(f, "optimize"),
            TokenKind::Test => write!(f, "test"),
            TokenKind::Infer => write!(f, "infer"),
            TokenKind::Constrain => write!(f, "constrain"),
            TokenKind::Validate => write!(f, "validate"),
            TokenKind::Prompt => write!(f, "prompt"),
            TokenKind::AiModel => write!(f, "ai_model"),
            TokenKind::AiCheck => write!(f, "ai_check"),
            TokenKind::AiValid => write!(f, "ai_valid"),
            TokenKind::AiFormat => write!(f, "ai_format"),
            TokenKind::AiInfer => write!(f, "ai_infer"),
            TokenKind::AiEnsure => write!(f, "ai_ensure"),
            TokenKind::Int => write!(f, "Int"),
            TokenKind::String => write!(f, "String"),
            TokenKind::Bool => write!(f, "Bool"),
            TokenKind::Float => write!(f, "Float"),
            TokenKind::AI => write!(f, "AI"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::BangEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::AndAnd => write!(f, "&&"),
            TokenKind::OrOr => write!(f, "||"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::At => write!(f, "@"),
            TokenKind::HashBracket => write!(f, "#["),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Error => write!(f, "ERROR"),
        }
    }
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, literal: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            literal: literal.into(),
        }
    }
}
