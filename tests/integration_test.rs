// SPDX-License-Identifier: MIT
//! Integration tests for the My Language compilation pipeline

use my_lang::{eval, parse, Value};

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

// Solo Map/dict stdlib builtins (hyperpolymath/my-lang#46): string-keyed maps
// over Value::Record. Exercises new/set (incl. overwrite)/get/remove/len and the
// immutability convention (map_remove returns a fresh map, m3 is unchanged).
// map_has / map_keys return Bool / Array and are covered by examples/map.my.
#[test]
fn test_eval_map_builtins() {
    let source = r#"
        fn main() -> Int {
            let m0 = map_new();
            let m1 = map_set(m0, "a", 1);
            let m2 = map_set(m1, "b", 2);
            let m3 = map_set(m2, "a", 99);
            let m4 = map_remove(m3, "b");
            let a = map_get(m3, "a");
            let n0 = map_len(m0);
            let n3 = map_len(m3);
            let n4 = map_len(m4);
            return a + n0 + n3 + n4;
        }
    "#;

    // 99 (overwritten "a") + 0 (empty m0) + 2 (len m3) + 1 (len m4, "b" removed)
    match eval(source) {
        Ok(Value::Int(n)) => assert_eq!(n, 102),
        other => panic!("expected Int(102), got {:?}", other),
    }
}

// String-literal escape decoding (hyperpolymath/my-lang#47). The lexer stores
// the raw slice; parse_string_literal must decode \" \\ \n etc. Regression for
// the latent bug where `"a\"b"` yielded the literal 4 chars a \ " b.
#[test]
fn test_eval_string_escapes() {
    let source = r#"
        fn main() -> String {
            return "a\"b\\c\nd\tend";
        }
    "#;
    match eval(source) {
        Ok(Value::String(s)) => assert_eq!(s, "a\"b\\c\nd\tend"),
        other => panic!("expected decoded escapes, got {:?}", other),
    }
}

// JSON stdlib builtins (hyperpolymath/my-lang#47): json_parse / json_stringify.
// Round-trips an object literal (requires escape decoding) and asserts the
// deterministic sorted-key serialization.
#[test]
fn test_eval_json_roundtrip() {
    let source = r#"
        fn main() -> String {
            let v = json_parse("{\"b\": 2, \"a\": [1, 2, 3], \"n\": null, \"t\": true}");
            return json_stringify(v);
        }
    "#;
    match eval(source) {
        Ok(Value::String(s)) => {
            assert_eq!(s, r#"{"a":[1,2,3],"b":2,"n":null,"t":true}"#)
        }
        other => panic!("expected sorted-key JSON string, got {:?}", other),
    }
}

// Solo date_today() stdlib builtin (hyperpolymath/my-lang#48): ISO YYYY-MM-DD
// (UTC). Asserts the format/shape and a sane range rather than an exact value
// (the result depends on the wall clock).
#[test]
fn test_eval_date_today() {
    let source = r#"
        fn main() -> String {
            return date_today();
        }
    "#;
    match eval(source) {
        Ok(Value::String(s)) => {
            assert_eq!(s.len(), 10, "expected YYYY-MM-DD, got {:?}", s);
            let b = s.as_bytes();
            assert_eq!(b[4], b'-');
            assert_eq!(b[7], b'-');
            for (i, c) in s.char_indices() {
                if i != 4 && i != 7 {
                    assert!(c.is_ascii_digit(), "non-digit at {}: {:?}", i, s);
                }
            }
            let year: i32 = s[0..4].parse().unwrap();
            let month: u32 = s[5..7].parse().unwrap();
            let day: u32 = s[8..10].parse().unwrap();
            assert!((2000..=3000).contains(&year), "implausible year: {}", s);
            assert!((1..=12).contains(&month), "bad month: {}", s);
            assert!((1..=31).contains(&day), "bad day: {}", s);
        }
        other => panic!("expected ISO date string, got {:?}", other),
    }
}

// Solo fs_list_dir() stdlib builtin (hyperpolymath/my-lang#55): enumerate a
// directory, sorted entry names. Self-contained: builds a dir with fs_* builtins
// then lists it (exercises fs_create_dir_all/fs_write_file/fs_list_dir/str_join).
#[test]
fn test_eval_fs_list_dir() {
    let source = r#"
        fn main() -> String {
            let dir = "target/.it_fs_list_dir";
            fs_create_dir_all(dir);
            fs_write_file("target/.it_fs_list_dir/b.txt", "x");
            fs_write_file("target/.it_fs_list_dir/a.txt", "y");
            let names = fs_list_dir(dir);
            return str_join(names, ",");
        }
    "#;
    match eval(source) {
        Ok(Value::String(s)) => assert_eq!(s, "a.txt,b.txt"),
        other => panic!("expected sorted dir listing, got {:?}", other),
    }
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
