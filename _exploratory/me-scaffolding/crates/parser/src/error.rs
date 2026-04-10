// Parser error types

use my_lang_lexer::Token;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
    },

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Invalid literal: {0}")]
    InvalidLiteral(String),

    #[error("Invalid operator: {0}")]
    InvalidOperator(String),

    #[error("Invalid type: {0}")]
    InvalidType(String),

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("Missing identifier")]
    MissingIdentifier,

    #[error("Missing type annotation")]
    MissingType,

    #[error("Invalid expression")]
    InvalidExpression,

    #[error("Invalid statement")]
    InvalidStatement,

    #[error("Invalid item")]
    InvalidItem,

    #[error("Invalid contract clause: {0}")]
    InvalidContract(String),

    #[error("Invalid attribute: {0}")]
    InvalidAttribute(String),
}

pub type ParseResult<T> = Result<T, ParseError>;

impl ParseError {
    pub fn unexpected(expected: impl Into<String>, found: &Token) -> Self {
        ParseError::UnexpectedToken {
            expected: expected.into(),
            found: format!("{:?}", found),
        }
    }
}
