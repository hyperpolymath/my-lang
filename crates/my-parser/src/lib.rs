//! Parser scaffolding for the My Language toolchain (Solo → Duet → Ensemble).
//! This module exposes `Parser` helpers used by the upcoming Menhir/Tokio-driven
//! parser and serves as the reference point for the Solo v1 grammar and Duet/Ensemble
//! extensions.
#![forbid(unsafe_code)]
use anyhow::Result;
use thiserror::Error;

/// Basic parse error that carries span information.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token `{token}` at {span}")]
    Unexpected { token: String, span: String },
    #[error("expected {expected} but found {found} at {span}")]
    Expected { expected: String, found: String, span: String },
}

/// Parser state over a token stream.
pub struct Parser<'src> {
    source: &'src str,
    position: usize,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, position: 0 }
    }

    pub fn parse_program(&mut self) -> Result<(), ParseError> {
        // TODO: Hook the Solo v1.0 grammar into this method.
        Ok(())
    }

    pub fn parse_top_level(&mut self) -> Result<(), ParseError> {
        Ok(())
    }

    pub fn parse_fn_decl(&mut self) -> Result<(), ParseError> {
        Ok(())
    }
}
