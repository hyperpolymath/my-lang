//! My Language: A programming language with first-class AI integration
//!
//! This crate provides a lexer and parser for a language that treats AI operations
//! as first-class citizens, with syntax-level support for:
//!
//! - AI model declarations and configuration
//! - Prompt templates
//! - AI expressions (query, verify, generate, embed, classify, optimize, test)
//! - AI type constraints (ai_check, ai_valid, ai_format, ai_infer)
//! - AI effect types (AI<T>)
//! - AI-based contracts (pre/post conditions with AI verification)

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

pub use ast::*;
pub use lexer::Lexer;
pub use parser::{ParseError, ParseResult, Parser};
pub use token::{Span, Token, TokenKind};

/// Parse source code into an AST
pub fn parse(source: &str) -> ParseResult<Program> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_example() {
        let source = r#"
            // AI Model Configuration
            ai_model claude {
                provider: "anthropic"
                model: "claude-3-opus"
                temperature: 0.7
                cache: true
            }

            // Prompt template
            prompt summarize { "Summarize the following text: {text}" }

            // Struct with AI validation
            #[derive(Debug, Clone)]
            struct Email {
                #[ai_validate("valid email format")]
                address: String,
                #[ai_embed]
                content: String,
            }

            // Effect declaration
            effect Logger {
                op log: String
            }

            // Function with AI features
            #[ai_optimize]
            fn process_email(email: Email) -> AI<String>
                where pre: email.address != "", ai_ensure: "email is processed safely"
            {
                let result = ai query {
                    model: claude
                    content: email.content
                };

                if result {
                    return result;
                } else {
                    return ai! { "Could not process email" };
                }
            }

            // Main function
            fn main() {
                let emails: [Email] = [];

                go {
                    let r = process_email(emails);
                }

                match result {
                    Ok(s) => s,
                    Err(e) => "error",
                };
            }
        "#;

        let program = parse(source).unwrap();
        assert!(program.items.len() >= 5);
    }

    #[test]
    fn test_ai_expressions() {
        let source = r#"
            fn test_ai() {
                // Quick AI query
                let x = ai! { "What is 2+2?" };

                // AI query with body
                let y = ai query {
                    prompt: "Analyze this"
                    context: data
                };

                // AI function call style
                let z = ai generate(template, params);

                // AI classify
                let category = ai classify(text);

                // AI embed
                let embedding = ai embed(document);

                // Prompt invocation
                let answer = my_prompt!(arg1, arg2);
            }
        "#;

        let program = parse(source).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            assert!(f.body.stmts.len() >= 6);
        }
    }

    #[test]
    fn test_type_system() {
        let source = r#"
            fn types_demo(
                a: Int,
                b: String,
                c: Float,
                d: [Bool],
                e: AI<String>
            ) -> AI<String> {
                let x: Int = 42;
            }
        "#;

        let program = parse(source).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            assert_eq!(f.params.len(), 5);
        }
    }
}
