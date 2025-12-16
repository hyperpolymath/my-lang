//! Lexer for My Language with AI integration

use crate::token::{Span, Token, TokenKind};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.pos;
        let start_line = self.line;
        let start_column = self.column;

        let Some(ch) = self.advance() else {
            return Token::new(
                TokenKind::Eof,
                Span::new(start, start, start_line, start_column),
                "",
            );
        };

        let kind = match ch {
            // Single-character tokens
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            '.' => TokenKind::Dot,
            '@' => TokenKind::At,
            '+' => TokenKind::Plus,
            '*' => TokenKind::Star,
            '?' => TokenKind::Question,

            // Multi-character tokens
            '-' => {
                if self.peek() == Some(&'>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '/' => TokenKind::Slash,
            '=' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::EqEq
                } else if self.peek() == Some(&'>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::BangEq
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                if self.peek() == Some(&'&') {
                    self.advance();
                    TokenKind::AndAnd
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                if self.peek() == Some(&'|') {
                    self.advance();
                    TokenKind::OrOr
                } else {
                    TokenKind::Pipe
                }
            }
            ':' => {
                if self.peek() == Some(&':') {
                    self.advance();
                    TokenKind::ColonColon
                } else {
                    TokenKind::Colon
                }
            }
            '#' => {
                if self.peek() == Some(&'[') {
                    self.advance();
                    TokenKind::HashBracket
                } else {
                    TokenKind::Error
                }
            }

            // String literals
            '"' => return self.scan_string(start, start_line, start_column),

            // Numbers
            c if c.is_ascii_digit() => {
                return self.scan_number(c, start, start_line, start_column)
            }

            // Identifiers and keywords
            c if c.is_alphabetic() || c == '_' => {
                return self.scan_identifier(c, start, start_line, start_column)
            }

            _ => TokenKind::Error,
        };

        let literal = &self.input[start..self.pos];
        Token::new(kind, Span::new(start, self.pos, start_line, start_column), literal)
    }

    fn scan_string(&mut self, start: usize, start_line: usize, start_column: usize) -> Token {
        let content_start = self.pos;
        while let Some(&ch) = self.peek() {
            if ch == '"' {
                break;
            }
            if ch == '\\' {
                self.advance();
                self.advance(); // Skip escaped character
            } else {
                self.advance();
            }
        }
        let content_end = self.pos;

        // Consume closing quote
        if self.peek() == Some(&'"') {
            self.advance();
        }

        let content = &self.input[content_start..content_end];
        Token::new(
            TokenKind::StringLit,
            Span::new(start, self.pos, start_line, start_column),
            content,
        )
    }

    fn scan_number(&mut self, _first: char, start: usize, start_line: usize, start_column: usize) -> Token {
        let mut is_float = false;

        while let Some(&ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !is_float {
                // Look ahead to see if this is a float
                let mut chars = self.input[self.pos + 1..].chars();
                if chars.next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    is_float = true;
                    self.advance(); // consume '.'
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let literal = &self.input[start..self.pos];
        let kind = if is_float {
            TokenKind::FloatLit
        } else {
            TokenKind::IntLit
        };

        Token::new(kind, Span::new(start, self.pos, start_line, start_column), literal)
    }

    fn scan_identifier(&mut self, _first: char, start: usize, start_line: usize, start_column: usize) -> Token {
        while let Some(&ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let literal = &self.input[start..self.pos];

        // Check for ai! special token
        if literal == "ai" && self.peek() == Some(&'!') {
            self.advance();
            return Token::new(
                TokenKind::AiBang,
                Span::new(start, self.pos, start_line, start_column),
                "ai!",
            );
        }

        let kind = self.keyword_or_ident(literal);
        Token::new(kind, Span::new(start, self.pos, start_line, start_column), literal)
    }

    fn keyword_or_ident(&self, s: &str) -> TokenKind {
        match s {
            // Standard keywords
            "fn" => TokenKind::Fn,
            "struct" => TokenKind::Struct,
            "effect" => TokenKind::Effect,
            "where" => TokenKind::Where,
            "pre" => TokenKind::Pre,
            "post" => TokenKind::Post,
            "invariant" => TokenKind::Invariant,
            "comptime" => TokenKind::Comptime,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "go" => TokenKind::Go,
            "return" => TokenKind::Return,
            "await" => TokenKind::Await,
            "try" => TokenKind::Try,
            "restrict" => TokenKind::Restrict,
            "match" => TokenKind::Match,
            "use" => TokenKind::Use,
            "op" => TokenKind::Op,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "async" => TokenKind::Ident, // Handled as modifier

            // AI keywords
            "ai" => TokenKind::Ai,
            "query" => TokenKind::Query,
            "verify" => TokenKind::Verify,
            "generate" => TokenKind::Generate,
            "embed" => TokenKind::Embed,
            "classify" => TokenKind::Classify,
            "optimize" => TokenKind::Optimize,
            "test" => TokenKind::Test,
            "infer" => TokenKind::Infer,
            "constrain" => TokenKind::Constrain,
            "validate" => TokenKind::Validate,
            "prompt" => TokenKind::Prompt,
            "ai_model" => TokenKind::AiModel,
            "ai_check" => TokenKind::AiCheck,
            "ai_valid" => TokenKind::AiValid,
            "ai_format" => TokenKind::AiFormat,
            "ai_infer" => TokenKind::AiInfer,
            "ai_ensure" => TokenKind::AiEnsure,

            // Type keywords
            "Int" => TokenKind::Int,
            "String" => TokenKind::String,
            "Bool" => TokenKind::Bool,
            "Float" => TokenKind::Float,
            "AI" => TokenKind::AI,
            "Effect" => TokenKind::Ident, // Treated as type identifier

            _ => TokenKind::Ident,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(&ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some(&'/') => {
                    // Look ahead for comment
                    let remaining = &self.input[self.pos..];
                    if remaining.starts_with("//") {
                        // Line comment
                        while let Some(&ch) = self.peek() {
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if remaining.starts_with("/*") {
                        // Block comment
                        self.advance(); // consume /
                        self.advance(); // consume *
                        while let Some(ch) = self.advance() {
                            if ch == '*' && self.peek() == Some(&'/') {
                                self.advance();
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
                Some(&'(') => {
                    // Check for EBNF-style comment (* ... *)
                    let remaining = &self.input[self.pos..];
                    if remaining.starts_with("(*") {
                        self.advance(); // consume (
                        self.advance(); // consume *
                        while let Some(ch) = self.advance() {
                            if ch == '*' && self.peek() == Some(&')') {
                                self.advance();
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("fn main() { }");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].literal, "main");
        assert_eq!(tokens[2].kind, TokenKind::LParen);
        assert_eq!(tokens[3].kind, TokenKind::RParen);
        assert_eq!(tokens[4].kind, TokenKind::LBrace);
        assert_eq!(tokens[5].kind, TokenKind::RBrace);
        assert_eq!(tokens[6].kind, TokenKind::Eof);
    }

    #[test]
    fn test_ai_keywords() {
        let mut lexer = Lexer::new("ai query verify generate embed classify");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Ai);
        assert_eq!(tokens[1].kind, TokenKind::Query);
        assert_eq!(tokens[2].kind, TokenKind::Verify);
        assert_eq!(tokens[3].kind, TokenKind::Generate);
        assert_eq!(tokens[4].kind, TokenKind::Embed);
        assert_eq!(tokens[5].kind, TokenKind::Classify);
    }

    #[test]
    fn test_ai_bang() {
        let mut lexer = Lexer::new("ai! { \"hello\" }");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::AiBang);
        assert_eq!(tokens[0].literal, "ai!");
    }

    #[test]
    fn test_ai_model_decl() {
        let mut lexer = Lexer::new("ai_model gpt4 { }");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::AiModel);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].literal, "gpt4");
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 100");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::IntLit);
        assert_eq!(tokens[0].literal, "42");
        assert_eq!(tokens[1].kind, TokenKind::FloatLit);
        assert_eq!(tokens[1].literal, "3.14");
        assert_eq!(tokens[2].kind, TokenKind::IntLit);
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("\"hello world\"");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::StringLit);
        assert_eq!(tokens[0].literal, "hello world");
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("-> => :: == != <= >= && ||");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Arrow);
        assert_eq!(tokens[1].kind, TokenKind::FatArrow);
        assert_eq!(tokens[2].kind, TokenKind::ColonColon);
        assert_eq!(tokens[3].kind, TokenKind::EqEq);
        assert_eq!(tokens[4].kind, TokenKind::BangEq);
        assert_eq!(tokens[5].kind, TokenKind::LtEq);
        assert_eq!(tokens[6].kind, TokenKind::GtEq);
        assert_eq!(tokens[7].kind, TokenKind::AndAnd);
        assert_eq!(tokens[8].kind, TokenKind::OrOr);
    }

    #[test]
    fn test_attributes() {
        let mut lexer = Lexer::new("#[ai_optimize]");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::HashBracket);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
    }

    #[test]
    fn test_type_constraints() {
        let mut lexer = Lexer::new("where ai_check: \"valid email\"");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Where);
        assert_eq!(tokens[1].kind, TokenKind::AiCheck);
        assert_eq!(tokens[2].kind, TokenKind::Colon);
        assert_eq!(tokens[3].kind, TokenKind::StringLit);
    }

    #[test]
    fn test_line_comments() {
        let mut lexer = Lexer::new("fn // comment\nmain");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].literal, "main");
    }

    #[test]
    fn test_block_comments() {
        let mut lexer = Lexer::new("fn /* block */ main");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].literal, "main");
    }
}
