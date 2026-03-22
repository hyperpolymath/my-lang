// Pratt parser precedence handling

use my_lang_lexer::Token;
use my_lang_ast::BinaryOp;

/// Operator precedence levels for Pratt parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None,
    Assignment,  // =, +=, -=, etc.
    Or,          // ||
    And,         // &&
    BitOr,       // |
    BitXor,      // ^
    BitAnd,      // &
    Equality,    // ==, !=
    Comparison,  // <, >, <=, >=
    Shift,       // <<, >>
    Term,        // +, -
    Factor,      // *, /, %
    Unary,       // !, -, &, *
    Call,        // (), [], .
    Primary,
}

impl Precedence {
    /// Get the precedence of a binary operator
    pub fn of_binary(token: &Token) -> Self {
        match token {
            Token::Equal | Token::PlusEqual | Token::MinusEqual
            | Token::StarEqual | Token::SlashEqual | Token::PercentEqual => Precedence::Assignment,
            Token::Or => Precedence::Or,
            Token::And => Precedence::And,
            Token::Pipe => Precedence::BitOr,
            Token::Caret => Precedence::BitXor,
            Token::Ampersand => Precedence::BitAnd,
            Token::EqualEqual | Token::NotEqual => Precedence::Equality,
            Token::Less | Token::Greater | Token::LessEqual | Token::GreaterEqual => Precedence::Comparison,
            Token::LeftShift | Token::RightShift => Precedence::Shift,
            Token::Plus | Token::Minus => Precedence::Term,
            Token::Star | Token::Slash | Token::Percent => Precedence::Factor,
            Token::LeftParen | Token::LeftBracket | Token::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }

    /// Get the next higher precedence level
    pub fn next(self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::BitOr,
            Precedence::BitOr => Precedence::BitXor,
            Precedence::BitXor => Precedence::BitAnd,
            Precedence::BitAnd => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Shift,
            Precedence::Shift => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

/// Convert a token to a BinaryOp
pub fn token_to_binary_op(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::Plus => Some(BinaryOp::Add),
        Token::Minus => Some(BinaryOp::Sub),
        Token::Star => Some(BinaryOp::Mul),
        Token::Slash => Some(BinaryOp::Div),
        Token::Percent => Some(BinaryOp::Mod),
        Token::EqualEqual => Some(BinaryOp::Eq),
        Token::NotEqual => Some(BinaryOp::Ne),
        Token::Less => Some(BinaryOp::Lt),
        Token::Greater => Some(BinaryOp::Gt),
        Token::LessEqual => Some(BinaryOp::Le),
        Token::GreaterEqual => Some(BinaryOp::Ge),
        Token::And => Some(BinaryOp::And),
        Token::Or => Some(BinaryOp::Or),
        Token::Ampersand => Some(BinaryOp::BitAnd),
        Token::Pipe => Some(BinaryOp::BitOr),
        Token::Caret => Some(BinaryOp::BitXor),
        Token::LeftShift => Some(BinaryOp::Shl),
        Token::RightShift => Some(BinaryOp::Shr),
        Token::Equal => Some(BinaryOp::Assign),
        _ => None,
    }
}

/// Check if the token is a right-associative operator
pub fn is_right_associative(token: &Token) -> bool {
    matches!(
        token,
        Token::Equal | Token::PlusEqual | Token::MinusEqual
        | Token::StarEqual | Token::SlashEqual | Token::PercentEqual
    )
}
