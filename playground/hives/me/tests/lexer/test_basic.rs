use my_lang_lexer::{Lexer, Token};

#[test]
fn test_hello_world() {
    let source = r#"
fn main() {
    println("Hello, World!");
}
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all();

    // Should have tokens: fn, main, (, ), {, println, (, "Hello, World!", ), ;, }
    assert!(tokens.len() > 0);
    assert_eq!(tokens[0].0, Token::Fn);
}

#[test]
fn test_variables() {
    let source = "let x: i32 = 42;";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all();

    assert_eq!(tokens[0].0, Token::Let);
    assert_eq!(tokens[1].0, Token::Identifier);
    assert_eq!(tokens[2].0, Token::Colon);
    assert_eq!(tokens[3].0, Token::I32);
    assert_eq!(tokens[4].0, Token::Equal);
    assert_eq!(tokens[5].0, Token::IntLiteral);
}

#[test]
fn test_affine_keyword() {
    let source = "affine File";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all();

    assert_eq!(tokens[0].0, Token::Affine);
    assert_eq!(tokens[1].0, Token::Identifier);
}

#[test]
fn test_duet_keywords() {
    let source = "intent synth verify";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all();

    assert_eq!(tokens[0].0, Token::Intent);
    assert_eq!(tokens[1].0, Token::Synth);
    assert_eq!(tokens[2].0, Token::Verify);
}

#[test]
fn test_ensemble_keywords() {
    let source = "agent workflow spawn";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all();

    assert_eq!(tokens[0].0, Token::Agent);
    assert_eq!(tokens[1].0, Token::Workflow);
    assert_eq!(tokens[2].0, Token::Spawn);
}
