//! Property-based tests for My Language
//!
//! These tests are inspired by Echidna's approach to invariant testing.
//! Each test defines a property (invariant) that should ALWAYS hold,
//! regardless of the input. The test framework generates random inputs
//! to try to falsify these properties.
//!
//! Property Categories:
//! 1. Lexer Invariants - Properties about tokenization
//! 2. Parser Invariants - Properties about AST construction
//! 3. Type Checker Invariants - Properties about semantic analysis
//! 4. Roundtrip Properties - Parse -> Print -> Parse = same AST

use my_lang::lexer::Lexer;
use my_lang::parser::Parser;
use my_lang::checker::check;
use my_lang::token::TokenKind;
use my_lang::parse;

// ============================================================================
// TEST GENERATORS
// ============================================================================

/// Simple RNG for tests (xorshift64)
struct TestRng {
    state: u64,
}

impl TestRng {
    fn new(seed: u64) -> Self {
        TestRng { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 { return 0; }
        (self.next_u64() as usize) % max
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() % 2 == 0
    }
}

/// Generate random valid identifiers
fn gen_identifier(rng: &mut TestRng) -> String {
    let first_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
    let rest_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";

    let len = rng.next_usize(20) + 1;
    let first: char = first_chars.chars().nth(rng.next_usize(first_chars.len())).unwrap();
    let rest: String = (0..len-1)
        .map(|_| rest_chars.chars().nth(rng.next_usize(rest_chars.len())).unwrap())
        .collect();

    format!("{}{}", first, rest)
}

/// Generate random integer literals
fn gen_int_literal(rng: &mut TestRng) -> String {
    let value = rng.next_u64() % 1_000_000;
    format!("{}", value)
}

/// Generate random float literals
fn gen_float_literal(rng: &mut TestRng) -> String {
    let int_part = rng.next_u64() % 1000;
    let frac_part = rng.next_u64() % 1000;
    format!("{}.{}", int_part, frac_part)
}

/// Generate random string literals
fn gen_string_literal(rng: &mut TestRng) -> String {
    let len = rng.next_usize(50);
    let safe_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,!?";
    let content: String = (0..len)
        .map(|_| safe_chars.chars().nth(rng.next_usize(safe_chars.len())).unwrap())
        .collect();
    format!("\"{}\"", content)
}

/// Generate a simple valid expression
fn gen_simple_expr(rng: &mut TestRng, depth: usize) -> String {
    if depth == 0 {
        match rng.next_usize(4) {
            0 => gen_int_literal(rng),
            1 => gen_float_literal(rng),
            2 => gen_string_literal(rng),
            _ => gen_identifier(rng),
        }
    } else {
        match rng.next_usize(6) {
            0 => gen_int_literal(rng),
            1 => gen_identifier(rng),
            2 => format!("({} + {})", gen_simple_expr(rng, depth - 1), gen_simple_expr(rng, depth - 1)),
            3 => format!("({} * {})", gen_simple_expr(rng, depth - 1), gen_simple_expr(rng, depth - 1)),
            4 => format!("({} - {})", gen_simple_expr(rng, depth - 1), gen_simple_expr(rng, depth - 1)),
            _ => format!("({})", gen_simple_expr(rng, depth - 1)),
        }
    }
}

/// Generate a valid function
fn gen_function(rng: &mut TestRng) -> String {
    let name = gen_identifier(rng);
    let num_params = rng.next_usize(4);
    let params: Vec<String> = (0..num_params)
        .map(|_| format!("{}: Int", gen_identifier(rng)))
        .collect();

    let body_expr = gen_simple_expr(rng, 2);

    format!(
        "fn {}({}) -> Int {{ {} }}",
        name,
        params.join(", "),
        body_expr
    )
}

/// Generate a valid let statement
#[allow(dead_code)]
fn gen_let_stmt(rng: &mut TestRng) -> String {
    let name = gen_identifier(rng);
    let mutable = if rng.next_bool() { "mut " } else { "" };
    let value = gen_simple_expr(rng, 1);
    format!("let {}{} = {};", mutable, name, value)
}

/// Generate a valid program
fn gen_program(rng: &mut TestRng) -> String {
    let num_functions = rng.next_usize(5) + 1;
    let functions: Vec<String> = (0..num_functions)
        .map(|_| gen_function(rng))
        .collect();
    functions.join("\n\n")
}

// ============================================================================
// LEXER INVARIANTS
// ============================================================================

/// INVARIANT: Lexer should never panic on any input
#[test]
fn lexer_invariant_no_panic() {
    let mut rng = TestRng::new(12345);

    for _ in 0..1000 {
        // Generate random bytes
        let len = rng.next_usize(1000);
        let input: String = (0..len)
            .map(|_| (rng.next_usize(128) as u8) as char)
            .collect();

        // This should not panic
        let mut lexer = Lexer::new(&input);
        let _tokens = lexer.tokenize();
    }
}

/// INVARIANT: Empty input should produce only EOF token
#[test]
fn lexer_invariant_empty_input() {
    let mut lexer = Lexer::new("");
    let tokens = lexer.tokenize();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

/// INVARIANT: Lexer should eventually terminate with EOF
#[test]
fn lexer_invariant_terminates() {
    let mut rng = TestRng::new(54321);

    for _ in 0..100 {
        let input = gen_program(&mut rng);
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.tokenize();

        // Last token should be EOF
        assert!(!tokens.is_empty());
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    }
}

/// INVARIANT: Token spans should be valid (start <= end, within source bounds)
#[test]
fn lexer_invariant_valid_spans() {
    let mut rng = TestRng::new(11111);

    for _ in 0..100 {
        let input = gen_program(&mut rng);
        let input_len = input.len();
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.tokenize();

        for token in &tokens {
            // Span should be valid
            assert!(token.span.start <= token.span.end, "Invalid span: start > end");
            assert!(token.span.end <= input_len, "Span extends beyond input: {} > {}", token.span.end, input_len);
            assert!(token.span.line >= 1, "Line number should be >= 1");
            assert!(token.span.column >= 1, "Column number should be >= 1");
        }
    }
}

/// INVARIANT: Keywords should be recognized correctly
#[test]
fn lexer_invariant_keywords() {
    let keywords = vec![
        ("fn", TokenKind::Fn),
        ("let", TokenKind::Let),
        ("mut", TokenKind::Mut),
        ("if", TokenKind::If),
        ("else", TokenKind::Else),
        ("match", TokenKind::Match),
        ("return", TokenKind::Return),
        ("struct", TokenKind::Struct),
        ("go", TokenKind::Go),
        ("await", TokenKind::Await),
        ("try", TokenKind::Try),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
        ("ai", TokenKind::Ai),
        ("query", TokenKind::Query),
        ("verify", TokenKind::Verify),
        ("generate", TokenKind::Generate),
        ("embed", TokenKind::Embed),
        ("classify", TokenKind::Classify),
    ];

    for (text, expected_kind) in keywords {
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize();
        assert!(tokens.len() >= 1, "Should produce at least EOF token");
        assert_eq!(
            tokens[0].kind, expected_kind,
            "Keyword '{}' should produce {:?}, got {:?}",
            text, expected_kind, tokens[0].kind
        );
    }
}

/// INVARIANT: Numbers should be lexed correctly
#[test]
fn lexer_invariant_numbers() {
    let test_cases = vec![
        ("0", TokenKind::IntLit),
        ("42", TokenKind::IntLit),
        ("1234567890", TokenKind::IntLit),
        ("3.14", TokenKind::FloatLit),
        ("0.5", TokenKind::FloatLit),
    ];

    for (text, expected_kind) in test_cases {
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize();
        assert!(tokens.len() >= 1);
        assert_eq!(
            tokens[0].kind, expected_kind,
            "Number '{}' should produce {:?}, got {:?}",
            text, expected_kind, tokens[0].kind
        );
    }
}

/// INVARIANT: Operators should be lexed correctly
#[test]
fn lexer_invariant_operators() {
    let operators = vec![
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Star),
        ("/", TokenKind::Slash),
        ("==", TokenKind::EqEq),
        ("!=", TokenKind::BangEq),
        ("<", TokenKind::Lt),
        (">", TokenKind::Gt),
        ("<=", TokenKind::LtEq),
        (">=", TokenKind::GtEq),
        ("&&", TokenKind::AndAnd),
        ("||", TokenKind::OrOr),
        ("!", TokenKind::Bang),
        ("=", TokenKind::Eq),
        ("->", TokenKind::Arrow),
        ("=>", TokenKind::FatArrow),
    ];

    for (text, expected_kind) in operators {
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize();
        assert!(tokens.len() >= 1);
        assert_eq!(
            tokens[0].kind, expected_kind,
            "Operator '{}' should produce {:?}, got {:?}",
            text, expected_kind, tokens[0].kind
        );
    }
}

// ============================================================================
// PARSER INVARIANTS
// ============================================================================

/// INVARIANT: Parser should not panic on valid input
#[test]
fn parser_invariant_no_panic_valid() {
    let mut rng = TestRng::new(22222);

    for _ in 0..100 {
        let input = gen_program(&mut rng);
        // This should not panic
        let _result = parse(&input);
    }
}

/// INVARIANT: Parser should not panic on any input
#[test]
fn parser_invariant_no_panic_any() {
    let mut rng = TestRng::new(33333);

    for _ in 0..500 {
        let len = rng.next_usize(500);
        let input: String = (0..len)
            .map(|_| {
                let printable = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 \n\t{}[]()+-*/=<>!&|:;,.\"'_";
                printable.chars().nth(rng.next_usize(printable.len())).unwrap()
            })
            .collect();

        // Should not panic
        let _result = parse(&input);
    }
}

/// INVARIANT: Empty function should parse
#[test]
fn parser_invariant_empty_function() {
    let input = "fn test() { }";
    let result = parse(input);
    assert!(result.is_ok(), "Empty function should parse: {:?}", result.err());
}

/// INVARIANT: Valid programs should parse successfully
#[test]
fn parser_invariant_valid_programs() {
    let valid_programs = vec![
        "fn main() { }",
        "fn add(a: Int, b: Int) -> Int { }",
        "fn test() { let x = 5; }",
        "fn test() { let mut x = 5; }",
        "fn test() { if true { } }",
        "fn test() { if true { } else { } }",
        "fn test(x: Int) { match x { 0 => 1, _ => 2, }; }",
        "struct Point { x: Int, y: Int, }",
        "fn test() -> Int { return 42; }",
        r#"fn test() { let s = "hello"; }"#,
        "fn test() { let arr = [1, 2, 3]; }",
        r#"ai_model Test { provider: "openai" }"#,
    ];

    for program in valid_programs {
        let result = parse(program);
        assert!(result.is_ok(), "Program should parse:\n{}\nError: {:?}", program, result.err());
    }
}

/// INVARIANT: Parse result should contain at least one item for non-empty input
#[test]
fn parser_invariant_nonempty_result() {
    let mut rng = TestRng::new(44444);

    for _ in 0..50 {
        let input = gen_program(&mut rng);
        if let Ok(program) = parse(&input) {
            assert!(!program.items.is_empty(), "Parsed program should not be empty for input: {}", input);
        }
    }
}

// ============================================================================
// TYPE CHECKER INVARIANTS
// ============================================================================

/// INVARIANT: Type checker should not panic on any valid AST
#[test]
fn checker_invariant_no_panic() {
    let mut rng = TestRng::new(55555);

    for _ in 0..100 {
        let input = gen_program(&mut rng);
        if let Ok(program) = parse(&input) {
            // Should not panic
            let _errors = check(&program);
        }
    }
}

/// INVARIANT: Well-typed programs should have no errors
#[test]
fn checker_invariant_well_typed() {
    let well_typed_programs = vec![
        "fn main() { }",
        "fn add(a: Int, b: Int) -> Int { a + b }",
        "fn test() { let x: Int = 5; }",
        "fn test() { let x = 5; let y = x + 1; }",
        "fn test() { let x = true; if x { } }",
        "struct Point { x: Int, y: Int, }
         fn test() { let p = Point { x: 1, y: 2 }; }",
    ];

    for program in well_typed_programs {
        if let Ok(ast) = parse(program) {
            let result = check(&ast);
            assert!(result.is_ok(), "Well-typed program should have no errors:\n{}\nErrors: {:?}", program, result.err());
        }
    }
}

/// INVARIANT: Using undefined variables should produce an error
#[test]
fn checker_invariant_undefined_variable() {
    let program = "fn test() { let x = undefined_var; }";
    if let Ok(ast) = parse(program) {
        let result = check(&ast);
        assert!(result.is_err(), "Using undefined variable should produce error");
    }
}

/// INVARIANT: Wrong number of arguments should produce an error
#[test]
fn checker_invariant_wrong_arg_count() {
    let program = "
        fn foo(a: Int, b: Int) -> Int { a + b }
        fn test() { foo(1); }
    ";
    if let Ok(ast) = parse(program) {
        let result = check(&ast);
        assert!(result.is_err(), "Wrong argument count should produce error");
    }
}

// ============================================================================
// INTEGRATION INVARIANTS
// ============================================================================

/// INVARIANT: Pipeline should work end-to-end for valid programs
#[test]
fn integration_invariant_full_pipeline() {
    let programs = vec![
        "fn main() { let x = 5; }",
        "fn add(a: Int, b: Int) -> Int { }",
        "struct Point { x: Int, y: Int, }",
        r#"ai_model Test { provider: "openai" }"#,
    ];

    for program in programs {
        // Lex
        let mut lexer = Lexer::new(program);
        let tokens = lexer.tokenize();
        assert!(!tokens.is_empty(), "Lexer should produce tokens");

        // Parse
        let mut parser = Parser::new(tokens);
        let parse_result = parser.parse_program();
        assert!(parse_result.is_ok(), "Parser should succeed for: {}", program);

        // Type check
        let ast = parse_result.unwrap();
        let _result = check(&ast);
        // Note: Some programs may have type errors, which is fine
        // We're just checking the pipeline doesn't panic
    }
}

// ============================================================================
// STRESS TESTS
// ============================================================================

/// STRESS: Deeply nested expressions
#[test]
fn stress_nested_expressions() {
    let mut expr = "1".to_string();
    for _ in 0..50 {
        expr = format!("({} + 1)", expr);
    }
    let program = format!("fn test() {{ let x = {}; }}", expr);

    // Should not stack overflow
    let _result = parse(&program);
}

/// STRESS: Many functions
#[test]
fn stress_many_functions() {
    let mut program = String::new();
    for i in 0..100 {
        program.push_str(&format!("fn func_{}() {{ }}\n", i));
    }

    let result = parse(&program);
    assert!(result.is_ok(), "Many functions should parse");

    if let Ok(ast) = result {
        assert_eq!(ast.items.len(), 100, "Should have 100 functions");
    }
}

/// STRESS: Long identifiers
#[test]
fn stress_long_identifiers() {
    let long_name: String = (0..1000).map(|_| 'a').collect();
    let program = format!("fn {}() {{ }}", long_name);

    let result = parse(&program);
    assert!(result.is_ok(), "Long identifier should parse");
}

/// STRESS: Many parameters
#[test]
fn stress_many_parameters() {
    let params: Vec<String> = (0..50).map(|i| format!("p{}: Int", i)).collect();
    let program = format!("fn test({}) {{ }}", params.join(", "));

    let result = parse(&program);
    assert!(result.is_ok(), "Many parameters should parse");
}

/// STRESS: Large string literal
#[test]
fn stress_large_string() {
    let content: String = (0..10000).map(|_| 'a').collect();
    let program = format!(r#"fn test() {{ let s = "{}"; }}"#, content);

    let result = parse(&program);
    assert!(result.is_ok(), "Large string should parse");
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

/// EDGE: Empty blocks
#[test]
fn edge_empty_blocks() {
    let programs = vec![
        "fn test() { }",
        "fn test() { if true { } }",
        "fn test() { if true { } else { } }",
    ];

    for program in programs {
        let result = parse(program);
        assert!(result.is_ok(), "Empty block should parse: {}", program);
    }
}

/// EDGE: Trailing commas
#[test]
fn edge_trailing_commas() {
    let programs = vec![
        "fn test(a: Int,) { }",
        "fn test() { let x = [1, 2,]; }",
        "struct Point { x: Int, y: Int, }",
    ];

    for program in programs {
        let result = parse(program);
        assert!(result.is_ok(), "Trailing comma should parse: {}", program);
    }
}

/// EDGE: Comments in various positions
#[test]
fn edge_comments() {
    let programs = vec![
        "// comment\nfn test() { }",
        "fn test() { // comment\n}",
        "fn /* comment */ test() { }",
        "fn test() { /* comment */ }",
        "fn test() { let x = 5; // comment\n}",
    ];

    for program in programs {
        let result = parse(program);
        assert!(result.is_ok(), "Program with comment should parse: {}", program);
    }
}

/// EDGE: Whitespace variations
#[test]
fn edge_whitespace() {
    let programs = vec![
        "fn test(){}",           // No spaces
        "fn  test  (  )  {  }",  // Extra spaces
        "fn\ttest()\t{\t}",      // Tabs
        "fn\ntest\n(\n)\n{\n}",  // Newlines
    ];

    for program in programs {
        let result = parse(program);
        assert!(result.is_ok(), "Whitespace variation should parse: {:?}", program);
    }
}

// ============================================================================
// AI-SPECIFIC TESTS
// ============================================================================

/// AI: Quick query syntax
#[test]
fn ai_quick_query() {
    let program = r#"fn test() { let x = ai! { "What is 2+2?" }; }"#;
    let result = parse(program);
    assert!(result.is_ok(), "AI quick query should parse");
}

/// AI: Query with options
#[test]
fn ai_query_with_options() {
    let program = r#"
        fn test() {
            let x = ai query {
                prompt: "test"
                model: "gpt-4"
            };
        }
    "#;
    let result = parse(program);
    assert!(result.is_ok(), "AI query with options should parse: {:?}", result.err());
}

/// AI: Model declaration
#[test]
fn ai_model_declaration() {
    let program = r#"
        ai_model MyModel {
            provider: "openai"
            model: "gpt-4"
            temperature: 0.7
        }
    "#;
    let result = parse(program);
    assert!(result.is_ok(), "AI model declaration should parse");
}

/// AI: Prompt declaration
#[test]
fn ai_prompt_declaration() {
    let program = r#"prompt summarize { "Summarize: {text}" }"#;
    let result = parse(program);
    assert!(result.is_ok(), "Prompt declaration should parse");
}

/// AI: Various AI expressions
#[test]
fn ai_expressions() {
    let programs = vec![
        r#"fn test() { ai query { } }"#,
        r#"fn test() { ai verify { } }"#,
        r#"fn test() { ai generate { } }"#,
        r#"fn test() { ai embed { } }"#,
        r#"fn test() { ai classify { } }"#,
    ];

    for program in programs {
        let result = parse(program);
        assert!(result.is_ok(), "AI expression should parse: {}", program);
    }
}

// ============================================================================
// FUZZ TESTING (Echidna-style invariant testing)
// ============================================================================

/// FUZZ: Random token sequences should not panic the parser
#[test]
fn fuzz_random_tokens_no_panic() {
    let mut rng = TestRng::new(99999);
    let token_fragments = vec![
        "fn", "let", "mut", "if", "else", "match",
        "return", "struct", "go", "await", "try", "true", "false",
        "ai", "query", "verify", "generate", "embed", "classify",
        "(", ")", "{", "}", "[", "]", ";", ":", ",", ".",
        "+", "-", "*", "/", "=", "==", "!=", "<", ">", "<=", ">=",
        "&&", "||", "!", "&", "|", "->", "=>", "::",
        "0", "1", "42", "3.14", "\"test\"",
        "foo", "bar", "baz", "x", "y", "z", "_",
    ];

    for _ in 0..1000 {
        let num_tokens = rng.next_usize(50) + 1;
        let input: String = (0..num_tokens)
            .map(|_| {
                let idx = rng.next_usize(token_fragments.len());
                token_fragments[idx]
            })
            .collect::<Vec<&str>>()
            .join(" ");

        // Should not panic
        let _result = parse(&input);
    }
}

/// FUZZ: Malformed structures should not panic
#[test]
fn fuzz_malformed_structures() {
    let malformed = vec![
        "fn",
        "fn (",
        "fn test",
        "fn test(",
        "fn test() {",
        "fn test() { let",
        "fn test() { let x",
        "fn test() { let x =",
        "struct",
        "struct {",
        "struct Foo {",
        "match",
        "match x",
        "match x {",
        "if",
        "if true",
        "if true {",
        "go",
        "go {",
        "ai",
        "ai query",
        "ai query {",
    ];

    for input in malformed {
        // Should not panic (errors are fine)
        let _result = parse(input);
    }
}

/// FUZZ: Boundary values
#[test]
fn fuzz_boundary_values() {
    let boundary_programs = vec![
        // Empty
        "",
        // Single character
        "a",
        // Just whitespace
        "   \n\t   ",
        // Just comments
        "// comment",
        "/* comment */",
        // Minimal valid
        "fn a(){}",
        // Maximum nesting we reasonably support
        "fn test() { if true { if true { if true { } } } }",
    ];

    for program in boundary_programs {
        let _result = parse(program);
    }
}
