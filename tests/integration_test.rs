// SPDX-License-Identifier: MIT
//! Integration tests for the My Language compilation pipeline

use my_lang::{parse, eval};

#[test]
fn test_parse_simple_function() {
    let source = r#"
        fn main() {
            let x = 42;
            x;
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_parse_function_with_params() {
    let source = r#"
        fn add(a: Int, b: Int) -> Int {
            a + b;
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_struct() {
    let source = r#"
        struct Point {
            x: Int,
            y: Int,
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_effect() {
    let source = r#"
        effect IO {
            op print: String -> Unit
            op read: Unit -> String
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_ai_model() {
    let source = r#"
        ai_model Assistant {
            provider: "anthropic"
            model: "claude-3-opus"
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_if_expression() {
    let source = r#"
        fn max(a: Int, b: Int) -> Int {
            if a > b {
                a;
            } else {
                b;
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_match_expression() {
    let source = r#"
        fn describe(n: Int) -> String {
            match n {
                0 => "zero",
                1 => "one",
                _ => "many"
            };
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_lambda() {
    let source = r#"
        fn apply(f: Int -> Int, x: Int) -> Int {
            f(x);
        }

        fn main() {
            let double = |x: Int| => x * 2;
            apply(double, 21);
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_hir_lowering() {
    let source = r#"
        fn add(a: Int, b: Int) -> Int {
            a + b;
        }
    "#;

    let program = parse(source).expect("parse failed");
    let hir = my_hir::lower(&program).expect("HIR lowering failed");

    assert_eq!(hir.items.len(), 1);
    if let my_hir::HirItem::Function(f) = &hir.items[0] {
        assert_eq!(f.name, "add");
        assert_eq!(f.params.len(), 2);
    } else {
        panic!("Expected function");
    }
}

#[test]
fn test_mir_lowering() {
    let source = r#"
        fn main() {
            let x = 1;
            let y = 2;
            x + y;
        }
    "#;

    let program = parse(source).expect("parse failed");
    let hir = my_hir::lower(&program).expect("HIR lowering failed");
    let mir = my_mir::lower(&hir).expect("MIR lowering failed");

    assert!(mir.functions.contains_key("main"));
    assert_eq!(mir.entry, Some("main".to_string()));
}

#[test]
fn test_full_pipeline() {
    let source = r#"
        fn factorial(n: Int) -> Int {
            if n <= 1 {
                1;
            } else {
                n * factorial(n - 1);
            }
        }

        fn main() {
            factorial(5);
        }
    "#;

    // Parse
    let program = parse(source).expect("parse failed");
    assert!(!program.items.is_empty());

    // Lower to HIR
    let hir = my_hir::lower(&program).expect("HIR lowering failed");
    assert!(!hir.items.is_empty());

    // Lower to MIR
    let mir = my_mir::lower(&hir).expect("MIR lowering failed");
    assert!(mir.entry.is_some());
}

#[test]
fn test_eval_simple() {
    let source = r#"
        fn main() {
            42
        }
    "#;

    // Just test that eval doesn't panic
    let _ = eval(source);
}

// AI Runtime tests (require API keys, so just test initialization)
#[test]
fn test_ai_runtime_creation() {
    let runtime = my_ai::AIRuntime::new();
    assert_eq!(runtime.default_model, "claude-3-opus");
}

#[test]
fn test_ai_runtime_from_env() {
    // This tests the env var detection (won't have actual keys in test)
    let runtime = my_ai::runtime_from_env();
    // Should not panic, even without API keys
    let _ = runtime;
}

// Package manager tests
#[test]
fn test_parse_manifest() {
    let toml = r#"
[package]
name = "test-app"
version = "0.1.0"

[dependencies]
std = "0.1"

[ai]
default-model = "claude-3-opus"
cache = true
"#;

    let manifest: my_pkg::Manifest = toml::from_str(toml).expect("parse failed");
    assert_eq!(manifest.package.name, "test-app");
    assert_eq!(manifest.package.version, "0.1.0");
    assert!(manifest.dependencies.contains_key("std"));
    assert_eq!(manifest.ai.default_model, Some("claude-3-opus".to_string()));
}
