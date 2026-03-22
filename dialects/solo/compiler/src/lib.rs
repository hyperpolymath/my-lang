/// Solo Dialect Compiler
///
/// Systems programming language with affine types and arena allocation.
///
/// # Features
///
/// - Affine type system (linear ownership)
/// - Arena-based memory management (no GC)
/// - Epistemic types (belief states)
/// - Compile-time memory safety
///
/// # Architecture
///
/// ```text
/// Source → Lexer → Parser → Type Checker → Code Generator → Binary
/// ```

#![forbid(unsafe_code)]
pub mod token;
pub mod lexer;

pub use token::{Token, TokenKind};
pub use lexer::Lexer;

/// Compile Solo source code to executable
pub fn compile(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);

    // Tokenize
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if matches!(token.kind, TokenKind::Eof) {
            break;
        }
        if let TokenKind::Error(ref msg) = token.kind {
            return Err(format!("Lexer error at {}:{}: {}", token.line, token.column, msg));
        }
        tokens.push(token);
    }

    // TODO(#parser): Implement recursive descent parser
    // Tracking: See ROADMAP.adoc "Phase 1: Solo Compiler"
    // Status: In progress - AST types needed first

    // TODO(#typeck): Implement type checker with affine types
    // Tracking: See ROADMAP.adoc "Phase 1: Solo Compiler"
    // Depends on: Parser completion

    // TODO(#codegen): Implement QBE IR code generation
    // Tracking: See ROADMAP.adoc "Phase 1: Solo Compiler"
    // Depends on: Type checker completion

    Ok(())
}
