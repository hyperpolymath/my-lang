/// Lexer for Solo dialect
///
/// Converts source code into a stream of tokens.

use crate::token::{Token, TokenKind};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof, "");
        }

        let ch = self.current_char();
        let start_column = self.column;

        match ch {
            // Single-character tokens
            '(' => self.single_char_token(TokenKind::LParen),
            ')' => self.single_char_token(TokenKind::RParen),
            '{' => self.single_char_token(TokenKind::LBrace),
            '}' => self.single_char_token(TokenKind::RBrace),
            '[' => self.single_char_token(TokenKind::LBracket),
            ']' => self.single_char_token(TokenKind::RBracket),
            ',' => self.single_char_token(TokenKind::Comma),
            ';' => self.single_char_token(TokenKind::Semicolon),
            '.' => self.single_char_token(TokenKind::Dot),
            '+' => self.single_char_token(TokenKind::Plus),
            '*' => self.single_char_token(TokenKind::Star),
            '%' => self.single_char_token(TokenKind::Percent),
            '~' => self.single_char_token(TokenKind::Tilde),
            '&' => self.single_char_token(TokenKind::Ampersand),
            '|' => self.single_char_token(TokenKind::Pipe),

            // Multi-character tokens
            '-' => {
                if self.peek() == '>' {
                    self.advance();
                    self.advance();
                    self.make_token(TokenKind::Arrow, "->")
                } else {
                    self.single_char_token(TokenKind::Minus)
                }
            }
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    self.advance();
                    self.make_token(TokenKind::EqEq, "==")
                } else if self.peek() == '>' {
                    self.advance();
                    self.advance();
                    self.make_token(TokenKind::FatArrow, "=>")
                } else {
                    self.single_char_token(TokenKind::Eq)
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    self.advance();
                    self.make_token(TokenKind::Ne, "!=")
                } else {
                    self.single_char_token(TokenKind::Not)
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    self.advance();
                    self.make_token(TokenKind::Le, "<=")
                } else {
                    self.single_char_token(TokenKind::Lt)
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    self.advance();
                    self.make_token(TokenKind::Ge, ">=")
                } else {
                    self.single_char_token(TokenKind::Gt)
                }
            }
            ':' => {
                if self.peek() == ':' {
                    self.advance();
                    self.advance();
                    self.make_token(TokenKind::DoubleColon, "::")
                } else {
                    self.single_char_token(TokenKind::Colon)
                }
            }

            // Comments
            '/' => {
                if self.peek() == '/' {
                    self.skip_line_comment();
                    return self.next_token();
                } else if self.peek() == '*' {
                    self.skip_block_comment();
                    return self.next_token();
                } else {
                    self.single_char_token(TokenKind::Slash)
                }
            }

            // String literals
            '"' => self.string_literal(),

            // Numbers
            '0'..='9' => self.number_literal(),

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),

            _ => {
                let msg = format!("Unexpected character: '{}'", ch);
                self.advance();
                Token::new(
                    TokenKind::Error(msg.clone()),
                    msg,
                    self.line,
                    start_column,
                )
            }
        }
    }

    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let ch = self.current_char();
        self.advance();
        self.make_token(kind, &ch.to_string())
    }

    fn identifier(&mut self) -> Token {
        let start = self.position;
        let start_column = self.column;

        while !self.is_at_end() && self.is_identifier_char(self.current_char()) {
            self.advance();
        }

        let lexeme: String = self.input[start..self.position].iter().collect();

        let kind = Token::is_keyword(&lexeme).unwrap_or_else(|| TokenKind::Identifier(lexeme.clone()));

        Token::new(kind, lexeme, self.line, start_column)
    }

    fn number_literal(&mut self) -> Token {
        let start = self.position;
        let start_column = self.column;

        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            self.advance();
        }

        // Check for float
        if !self.is_at_end() && self.current_char() == '.' && self.peek().is_ascii_digit() {
            self.advance(); // consume '.'
            while !self.is_at_end() && self.current_char().is_ascii_digit() {
                self.advance();
            }

            let lexeme: String = self.input[start..self.position].iter().collect();
            let value = lexeme.parse::<f64>().unwrap();
            Token::new(TokenKind::Float(value), lexeme, self.line, start_column)
        } else {
            let lexeme: String = self.input[start..self.position].iter().collect();
            let value = lexeme.parse::<i64>().unwrap();
            Token::new(TokenKind::Integer(value), lexeme, self.line, start_column)
        }
    }

    fn string_literal(&mut self) -> Token {
        let start_column = self.column;
        self.advance(); // consume opening "

        let start = self.position;
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Token::new(
                TokenKind::Error("Unterminated string".to_string()),
                "".to_string(),
                self.line,
                start_column,
            );
        }

        let value: String = self.input[start..self.position].iter().collect();
        self.advance(); // consume closing "

        Token::new(
            TokenKind::String(value.clone()),
            format!("\"{}\"", value),
            self.line,
            start_column,
        )
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\r' | '\t' => self.advance(),
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance(); // consume /
        self.advance(); // consume *

        while !self.is_at_end() {
            if self.current_char() == '*' && self.peek() == '/' {
                self.advance();
                self.advance();
                break;
            }
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            self.advance();
        }
    }

    fn is_identifier_char(&self, ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    fn current_char(&self) -> char {
        self.input[self.position]
    }

    fn peek(&self) -> char {
        if self.position + 1 < self.input.len() {
            self.input[self.position + 1]
        } else {
            '\0'
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.column += 1;
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        Token::new(kind, lexeme.to_string(), self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("fn let mut if else while return");
        assert!(matches!(lexer.next_token().kind, TokenKind::Fn));
        assert!(matches!(lexer.next_token().kind, TokenKind::Let));
        assert!(matches!(lexer.next_token().kind, TokenKind::Mut));
        assert!(matches!(lexer.next_token().kind, TokenKind::If));
        assert!(matches!(lexer.next_token().kind, TokenKind::Else));
        assert!(matches!(lexer.next_token().kind, TokenKind::While));
        assert!(matches!(lexer.next_token().kind, TokenKind::Return));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 0");
        assert!(matches!(lexer.next_token().kind, TokenKind::Integer(42)));
        assert!(matches!(lexer.next_token().kind, TokenKind::Float(f) if (f - 3.14).abs() < 1e-6));
        assert!(matches!(lexer.next_token().kind, TokenKind::Integer(0)));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello world" "test""#);
        if let TokenKind::String(s) = lexer.next_token().kind {
            assert_eq!(s, "hello world");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("-> => == != <= >= :: + - * /");
        assert!(matches!(lexer.next_token().kind, TokenKind::Arrow));
        assert!(matches!(lexer.next_token().kind, TokenKind::FatArrow));
        assert!(matches!(lexer.next_token().kind, TokenKind::EqEq));
        assert!(matches!(lexer.next_token().kind, TokenKind::Ne));
        assert!(matches!(lexer.next_token().kind, TokenKind::Le));
        assert!(matches!(lexer.next_token().kind, TokenKind::Ge));
        assert!(matches!(lexer.next_token().kind, TokenKind::DoubleColon));
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("// comment\nfn /* block */ let");
        assert!(matches!(lexer.next_token().kind, TokenKind::Fn));
        assert!(matches!(lexer.next_token().kind, TokenKind::Let));
    }
}
