// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Standard Library for My Language
//!
//! This module provides built-in functions and types that are automatically
//! available in every program.

use crate::interpreter::{NativeFunction, RuntimeError, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Program argv forwarded by the host (e.g. my-cli's `Run` command).
/// When set, `env_args()` returns these instead of `std::env::args().skip(1)`.
/// This lets `my run script.my -- --flag value` propagate `--flag value` to
/// the program without my-cli's clap consuming it.
static PROGRAM_ARGS: OnceLock<Vec<String>> = OnceLock::new();

/// Set the program argv visible to `env_args()`. Idempotent (first call wins).
/// Hosts (my-cli, embedders) call this before running a program.
pub fn set_program_args(args: Vec<String>) {
    let _ = PROGRAM_ARGS.set(args);
}

/// Register all standard library functions into an environment
pub fn register_stdlib(define: &mut impl FnMut(String, Value)) {
    // I/O Functions
    register_io_functions(define);

    // String Functions
    register_string_functions(define);

    // Math Functions
    register_math_functions(define);

    // Array Functions
    register_array_functions(define);

    // Type Functions
    register_type_functions(define);

    // Utility Functions
    register_utility_functions(define);

    // File System Functions (added 2026-04-30 per idaptik-port stdlib PR)
    register_fs_functions(define);

    // Environment Extras (env_args; complements existing env(name))
    register_env_extras(define);

    // Map / Dict Functions (string-keyed maps over Value::Record; my-lang#46)
    register_map_functions(define);

    // JSON Functions (json_parse / json_stringify; my-lang#47)
    register_json_functions(define);

    // Date Functions (date_today; my-lang#48)
    register_date_functions(define);
}

// ============================================================================
// I/O FUNCTIONS
// ============================================================================

fn register_io_functions(define: &mut impl FnMut(String, Value)) {
    // print(value) - Print without newline
    define(
        "print".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "print".to_string(),
            arity: 1,
            func: |args| {
                print!("{}", args[0]);
                Ok(Value::Unit)
            },
        }),
    );

    // println(value) - Print with newline
    define(
        "println".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "println".to_string(),
            arity: 1,
            func: |args| {
                println!("{}", args[0]);
                Ok(Value::Unit)
            },
        }),
    );

    // debug(value) - Print debug representation
    define(
        "debug".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "debug".to_string(),
            arity: 1,
            func: |args| {
                println!("{:?}", args[0]);
                Ok(Value::Unit)
            },
        }),
    );

    // input() - Read line from stdin (returns empty string on error)
    define(
        "input".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "input".to_string(),
            arity: 0,
            func: |_| {
                let mut line = String::new();
                match std::io::stdin().read_line(&mut line) {
                    Ok(_) => Ok(Value::String(line.trim_end().to_string())),
                    Err(_) => Ok(Value::String(String::new())),
                }
            },
        }),
    );

    // input_prompt(prompt) - Print prompt and read line
    define(
        "input_prompt".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "input_prompt".to_string(),
            arity: 1,
            func: |args| {
                print!("{}", args[0]);
                use std::io::Write;
                let _ = std::io::stdout().flush();
                let mut line = String::new();
                match std::io::stdin().read_line(&mut line) {
                    Ok(_) => Ok(Value::String(line.trim_end().to_string())),
                    Err(_) => Ok(Value::String(String::new())),
                }
            },
        }),
    );
}

// ============================================================================
// STRING FUNCTIONS
// ============================================================================

fn register_string_functions(define: &mut impl FnMut(String, Value)) {
    // len(string|array) - Get length
    define(
        "len".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "len".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
                _ => Err(RuntimeError::TypeError {
                    expected: "string or array".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // str_concat(a, b) - Concatenate strings
    define(
        "str_concat".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_concat".to_string(),
            arity: 2,
            func: |args| {
                let a = match &args[0] {
                    Value::String(s) => s.clone(),
                    v => format!("{}", v),
                };
                let b = match &args[1] {
                    Value::String(s) => s.clone(),
                    v => format!("{}", v),
                };
                Ok(Value::String(format!("{}{}", a, b)))
            },
        }),
    );

    // str_split(string, delimiter) - Split string into array
    define(
        "str_split".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_split".to_string(),
            arity: 2,
            func: |args| {
                let (s, delim) = match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(d)) => (s, d),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "string, string".to_string(),
                            got: format!("{:?}, {:?}", args[0], args[1]),
                        })
                    }
                };
                let parts: Vec<Value> = s
                    .split(delim.as_str())
                    .map(|p| Value::String(p.to_string()))
                    .collect();
                Ok(Value::Array(parts))
            },
        }),
    );

    // str_join(array, delimiter) - Join array into string
    define(
        "str_join".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_join".to_string(),
            arity: 2,
            func: |args| {
                let (arr, delim) = match (&args[0], &args[1]) {
                    (Value::Array(a), Value::String(d)) => (a, d),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "array, string".to_string(),
                            got: format!("{:?}, {:?}", args[0], args[1]),
                        })
                    }
                };
                let parts: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
                Ok(Value::String(parts.join(delim)))
            },
        }),
    );

    // str_trim(string) - Remove whitespace from both ends
    define(
        "str_trim".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_trim".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(s) => Ok(Value::String(s.trim().to_string())),
                _ => Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // str_upper(string) - Convert to uppercase
    define(
        "str_upper".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_upper".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_uppercase())),
                _ => Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // str_lower(string) - Convert to lowercase
    define(
        "str_lower".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_lower".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_lowercase())),
                _ => Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // str_contains(string, substring) - Check if contains substring
    define(
        "str_contains".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_contains".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::String(s), Value::String(sub)) => Ok(Value::Bool(s.contains(sub.as_str()))),
                _ => Err(RuntimeError::TypeError {
                    expected: "string, string".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // str_replace(string, from, to) - Replace all occurrences
    define(
        "str_replace".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_replace".to_string(),
            arity: 3,
            func: |args| match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::String(from), Value::String(to)) => {
                    Ok(Value::String(s.replace(from.as_str(), to.as_str())))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "string, string, string".to_string(),
                    got: format!("{:?}, {:?}, {:?}", args[0], args[1], args[2]),
                }),
            },
        }),
    );

    // str_starts_with(string, prefix) - Check if starts with prefix
    define(
        "str_starts_with".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_starts_with".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::String(s), Value::String(prefix)) => {
                    Ok(Value::Bool(s.starts_with(prefix.as_str())))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "string, string".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // str_ends_with(string, suffix) - Check if ends with suffix
    define(
        "str_ends_with".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_ends_with".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::String(s), Value::String(suffix)) => {
                    Ok(Value::Bool(s.ends_with(suffix.as_str())))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "string, string".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // str_substring(string, start, end) - Get substring
    define(
        "str_substring".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "str_substring".to_string(),
            arity: 3,
            func: |args| match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::Int(start), Value::Int(end)) => {
                    let start = *start as usize;
                    let end = (*end as usize).min(s.len());
                    if start <= end && start <= s.len() {
                        Ok(Value::String(s[start..end].to_string()))
                    } else {
                        Ok(Value::String(String::new()))
                    }
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "string, int, int".to_string(),
                    got: format!("{:?}, {:?}, {:?}", args[0], args[1], args[2]),
                }),
            },
        }),
    );

    // char_at(string, index) - Get character at index
    define(
        "char_at".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "char_at".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::String(s), Value::Int(idx)) => {
                    // Index by *character*, not byte: `s.len()` is the byte
                    // length, so the old `idx < s.len()` guard let a multi-byte
                    // UTF-8 string reach `chars().nth(idx) == None` and panic.
                    // `try_from` also rejects negative indices. Length is
                    // reported as the char count to match char-indexing.
                    match usize::try_from(*idx).ok().and_then(|i| s.chars().nth(i)) {
                        Some(c) => Ok(Value::String(c.to_string())),
                        None => Err(RuntimeError::IndexOutOfBounds {
                            index: *idx,
                            length: s.chars().count(),
                        }),
                    }
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "string, int".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // format(template, args) - positional {} substitution; {{ and }} escape
    define(
        "format".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "format".to_string(),
            arity: 2,
            func: |args| {
                let template = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "string".to_string(),
                            got: format!("{:?}", args[0]),
                        })
                    }
                };
                let values = match &args[1] {
                    Value::Array(a) => a.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "array".to_string(),
                            got: format!("{:?}", args[1]),
                        })
                    }
                };
                let mut result = String::new();
                let mut chars = template.chars().peekable();
                let mut idx = 0usize;
                while let Some(c) = chars.next() {
                    if c == '{' && chars.peek() == Some(&'{') {
                        chars.next();
                        result.push('{');
                    } else if c == '{' && chars.peek() == Some(&'}') {
                        chars.next();
                        if idx >= values.len() {
                            return Err(RuntimeError::Custom(format!(
                                "format: not enough args for placeholder #{}",
                                idx
                            )));
                        }
                        match &values[idx] {
                            Value::String(s) => result.push_str(s),
                            v => result.push_str(&format!("{}", v)),
                        }
                        idx += 1;
                    } else if c == '}' && chars.peek() == Some(&'}') {
                        chars.next();
                        result.push('}');
                    } else {
                        result.push(c);
                    }
                }
                Ok(Value::String(result))
            },
        }),
    );
}

// ============================================================================
// MATH FUNCTIONS
// ============================================================================

fn register_math_functions(define: &mut impl FnMut(String, Value)) {
    // abs(number) - Absolute value
    define(
        "abs".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "abs".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Int(n) => Ok(Value::Int(n.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // min(a, b) - Minimum of two values
    define(
        "min".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "min".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.min(*b as f64))),
                _ => Err(RuntimeError::TypeError {
                    expected: "number, number".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // max(a, b) - Maximum of two values
    define(
        "max".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "max".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.max(*b as f64))),
                _ => Err(RuntimeError::TypeError {
                    expected: "number, number".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // floor(float) - Round down
    define(
        "floor".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "floor".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Int(f.floor() as i64)),
                Value::Int(n) => Ok(Value::Int(*n)),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // ceil(float) - Round up
    define(
        "ceil".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "ceil".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Int(f.ceil() as i64)),
                Value::Int(n) => Ok(Value::Int(*n)),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // round(float) - Round to nearest
    define(
        "round".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "round".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Int(f.round() as i64)),
                Value::Int(n) => Ok(Value::Int(*n)),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // sqrt(number) - Square root
    define(
        "sqrt".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "sqrt".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.sqrt())),
                Value::Int(n) => Ok(Value::Float((*n as f64).sqrt())),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // pow(base, exp) - Power
    define(
        "pow".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "pow".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Int(base), Value::Int(exp)) => {
                    if *exp >= 0 {
                        Ok(Value::Int(base.pow(*exp as u32)))
                    } else {
                        Ok(Value::Float((*base as f64).powi(*exp as i32)))
                    }
                }
                (Value::Float(base), Value::Int(exp)) => Ok(Value::Float(base.powi(*exp as i32))),
                (Value::Float(base), Value::Float(exp)) => Ok(Value::Float(base.powf(*exp))),
                (Value::Int(base), Value::Float(exp)) => {
                    Ok(Value::Float((*base as f64).powf(*exp)))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "number, number".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // mod(a, b) - Modulo
    define(
        "mod".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "mod".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Int(a % b))
                    }
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
                _ => Err(RuntimeError::TypeError {
                    expected: "number, number".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // sin(float) - Sine
    define(
        "sin".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "sin".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.sin())),
                Value::Int(n) => Ok(Value::Float((*n as f64).sin())),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // cos(float) - Cosine
    define(
        "cos".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "cos".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.cos())),
                Value::Int(n) => Ok(Value::Float((*n as f64).cos())),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // tan(float) - Tangent
    define(
        "tan".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "tan".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.tan())),
                Value::Int(n) => Ok(Value::Float((*n as f64).tan())),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // log(float) - Natural logarithm
    define(
        "log".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "log".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.ln())),
                Value::Int(n) => Ok(Value::Float((*n as f64).ln())),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // log10(float) - Base 10 logarithm
    define(
        "log10".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "log10".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.log10())),
                Value::Int(n) => Ok(Value::Float((*n as f64).log10())),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // exp(float) - e^x
    define(
        "exp".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "exp".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.exp())),
                Value::Int(n) => Ok(Value::Float((*n as f64).exp())),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // Constants
    define("PI".to_string(), Value::Float(std::f64::consts::PI));
    define("E".to_string(), Value::Float(std::f64::consts::E));
    define("TAU".to_string(), Value::Float(std::f64::consts::TAU));
}

// ============================================================================
// ARRAY FUNCTIONS
// ============================================================================

fn register_array_functions(define: &mut impl FnMut(String, Value)) {
    // push(array, element) - Add element to end (returns new array)
    define(
        "push".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "push".to_string(),
            arity: 2,
            func: |args| match &args[0] {
                Value::Array(arr) => {
                    let mut new_arr = arr.clone();
                    new_arr.push(args[1].clone());
                    Ok(Value::Array(new_arr))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "array".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // pop(array) - Remove last element (returns new array)
    define(
        "pop".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "pop".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Array(arr) => {
                    let mut new_arr = arr.clone();
                    new_arr.pop();
                    Ok(Value::Array(new_arr))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "array".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // first(array) - Get first element
    define(
        "first".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "first".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Array(arr) => arr.first().cloned().ok_or(RuntimeError::IndexOutOfBounds {
                    index: 0,
                    length: 0,
                }),
                _ => Err(RuntimeError::TypeError {
                    expected: "array".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // last(array) - Get last element
    define(
        "last".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "last".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Array(arr) => arr.last().cloned().ok_or(RuntimeError::IndexOutOfBounds {
                    index: 0,
                    length: 0,
                }),
                _ => Err(RuntimeError::TypeError {
                    expected: "array".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // get(array, index) - Get element at index
    define(
        "get".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "get".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Array(arr), Value::Int(idx)) => {
                    let idx = *idx as usize;
                    arr.get(idx).cloned().ok_or(RuntimeError::IndexOutOfBounds {
                        index: idx as i64,
                        length: arr.len(),
                    })
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "array, int".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // set(array, index, value) - Set element at index (returns new array)
    define(
        "set".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "set".to_string(),
            arity: 3,
            func: |args| match (&args[0], &args[1]) {
                (Value::Array(arr), Value::Int(idx)) => {
                    let idx = *idx as usize;
                    if idx < arr.len() {
                        let mut new_arr = arr.clone();
                        new_arr[idx] = args[2].clone();
                        Ok(Value::Array(new_arr))
                    } else {
                        Err(RuntimeError::IndexOutOfBounds {
                            index: idx as i64,
                            length: arr.len(),
                        })
                    }
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "array, int, value".to_string(),
                    got: format!("{:?}, {:?}, {:?}", args[0], args[1], args[2]),
                }),
            },
        }),
    );

    // concat(array1, array2) - Concatenate arrays
    define(
        "concat".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "concat".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Array(a), Value::Array(b)) => {
                    let mut result = a.clone();
                    result.extend(b.iter().cloned());
                    Ok(Value::Array(result))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "array, array".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // slice(array, start, end) - Get slice of array
    define(
        "slice".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "slice".to_string(),
            arity: 3,
            func: |args| match (&args[0], &args[1], &args[2]) {
                (Value::Array(arr), Value::Int(start), Value::Int(end)) => {
                    let start = (*start as usize).min(arr.len());
                    let end = (*end as usize).min(arr.len());
                    if start <= end {
                        Ok(Value::Array(arr[start..end].to_vec()))
                    } else {
                        Ok(Value::Array(vec![]))
                    }
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "array, int, int".to_string(),
                    got: format!("{:?}, {:?}, {:?}", args[0], args[1], args[2]),
                }),
            },
        }),
    );

    // reverse(array) - Reverse array
    define(
        "reverse".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "reverse".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Array(arr) => {
                    let mut result = arr.clone();
                    result.reverse();
                    Ok(Value::Array(result))
                }
                Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
                _ => Err(RuntimeError::TypeError {
                    expected: "array or string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // contains(array, element) - Check if array contains element
    define(
        "contains".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "contains".to_string(),
            arity: 2,
            func: |args| match &args[0] {
                Value::Array(arr) => Ok(Value::Bool(arr.contains(&args[1]))),
                _ => Err(RuntimeError::TypeError {
                    expected: "array".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // range(start, end) - Create array [start, start+1, ..., end-1]
    define(
        "range".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "range".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Int(start), Value::Int(end)) => {
                    let arr: Vec<Value> = (*start..*end).map(Value::Int).collect();
                    Ok(Value::Array(arr))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "int, int".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // is_empty(array|string) - Check if empty
    define(
        "is_empty".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "is_empty".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Array(arr) => Ok(Value::Bool(arr.is_empty())),
                Value::String(s) => Ok(Value::Bool(s.is_empty())),
                _ => Err(RuntimeError::TypeError {
                    expected: "array or string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );
}

// ============================================================================
// TYPE FUNCTIONS
// ============================================================================

fn register_type_functions(define: &mut impl FnMut(String, Value)) {
    // type_of(value) - Get type name
    define(
        "type_of".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "type_of".to_string(),
            arity: 1,
            func: |args| {
                let type_name = match &args[0] {
                    Value::Int(_) => "Int",
                    Value::Float(_) => "Float",
                    Value::String(_) => "String",
                    Value::Bool(_) => "Bool",
                    Value::Unit => "Unit",
                    Value::Array(_) => "Array",
                    Value::Record(_) => "Record",
                    Value::Function(_) => "Function",
                    Value::NativeFunction(_) => "NativeFunction",
                    Value::AiResult(_) => "AiResult",
                };
                Ok(Value::String(type_name.to_string()))
            },
        }),
    );

    // to_string(value) - Convert to string
    define(
        "to_string".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "to_string".to_string(),
            arity: 1,
            func: |args| Ok(Value::String(format!("{}", args[0]))),
        }),
    );

    // to_int(value) - Convert to int
    define(
        "to_int".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "to_int".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Int(n) => Ok(Value::Int(*n)),
                Value::Float(f) => Ok(Value::Int(*f as i64)),
                Value::String(s) => s.parse::<i64>().map(Value::Int).map_err(|_| {
                    RuntimeError::TypeError {
                        expected: "integer string".to_string(),
                        got: s.clone(),
                    }
                }),
                Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
                _ => Err(RuntimeError::TypeError {
                    expected: "convertible to int".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // to_float(value) - Convert to float
    define(
        "to_float".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "to_float".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Int(n) => Ok(Value::Float(*n as f64)),
                Value::Float(f) => Ok(Value::Float(*f)),
                Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
                    RuntimeError::TypeError {
                        expected: "float string".to_string(),
                        got: s.clone(),
                    }
                }),
                _ => Err(RuntimeError::TypeError {
                    expected: "convertible to float".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // to_bool(value) - Convert to bool
    define(
        "to_bool".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "to_bool".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Bool(b) => Ok(Value::Bool(*b)),
                Value::Int(n) => Ok(Value::Bool(*n != 0)),
                Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
                Value::String(s) => Ok(Value::Bool(!s.is_empty())),
                Value::Array(arr) => Ok(Value::Bool(!arr.is_empty())),
                Value::Unit => Ok(Value::Bool(false)),
                _ => Ok(Value::Bool(true)),
            },
        }),
    );

    // is_int(value) - Check if int
    define(
        "is_int".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "is_int".to_string(),
            arity: 1,
            func: |args| Ok(Value::Bool(matches!(args[0], Value::Int(_)))),
        }),
    );

    // is_float(value) - Check if float
    define(
        "is_float".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "is_float".to_string(),
            arity: 1,
            func: |args| Ok(Value::Bool(matches!(args[0], Value::Float(_)))),
        }),
    );

    // is_string(value) - Check if string
    define(
        "is_string".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "is_string".to_string(),
            arity: 1,
            func: |args| Ok(Value::Bool(matches!(args[0], Value::String(_)))),
        }),
    );

    // is_bool(value) - Check if bool
    define(
        "is_bool".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "is_bool".to_string(),
            arity: 1,
            func: |args| Ok(Value::Bool(matches!(args[0], Value::Bool(_)))),
        }),
    );

    // is_array(value) - Check if array
    define(
        "is_array".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "is_array".to_string(),
            arity: 1,
            func: |args| Ok(Value::Bool(matches!(args[0], Value::Array(_)))),
        }),
    );

    // is_function(value) - Check if function
    define(
        "is_function".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "is_function".to_string(),
            arity: 1,
            func: |args| {
                Ok(Value::Bool(matches!(
                    args[0],
                    Value::Function(_) | Value::NativeFunction(_)
                )))
            },
        }),
    );
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn register_utility_functions(define: &mut impl FnMut(String, Value)) {
    // assert(condition) - Assert condition is true
    define(
        "assert".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "assert".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Bool(true) => Ok(Value::Unit),
                Value::Bool(false) => {
                    Err(RuntimeError::Custom("assertion failed".to_string()))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "bool".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // assert_eq(a, b) - Assert equality
    define(
        "assert_eq".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "assert_eq".to_string(),
            arity: 2,
            func: |args| {
                if args[0] == args[1] {
                    Ok(Value::Unit)
                } else {
                    Err(RuntimeError::Custom(format!(
                        "assertion failed: {} != {}",
                        args[0], args[1]
                    )))
                }
            },
        }),
    );

    // panic(message) - Panic with message
    define(
        "panic".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "panic".to_string(),
            arity: 1,
            func: |args| Err(RuntimeError::Custom(format!("panic: {}", args[0]))),
        }),
    );

    // identity(value) - Return value unchanged
    define(
        "identity".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "identity".to_string(),
            arity: 1,
            func: |args| Ok(args[0].clone()),
        }),
    );

    // clone(value) - Clone value (same as identity for now)
    define(
        "clone".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "clone".to_string(),
            arity: 1,
            func: |args| Ok(args[0].clone()),
        }),
    );

    // default(type_name) - Get default value for type
    define(
        "default".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "default".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(s) => match s.as_str() {
                    "Int" => Ok(Value::Int(0)),
                    "Float" => Ok(Value::Float(0.0)),
                    "String" => Ok(Value::String(String::new())),
                    "Bool" => Ok(Value::Bool(false)),
                    "Array" => Ok(Value::Array(vec![])),
                    "Record" => Ok(Value::Record(HashMap::new())),
                    _ => Err(RuntimeError::Custom(format!("unknown type: {}", s))),
                },
                _ => Err(RuntimeError::TypeError {
                    expected: "string (type name)".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // hash(value) - Simple hash function
    define(
        "hash".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "hash".to_string(),
            arity: 1,
            func: |args| {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                format!("{:?}", args[0]).hash(&mut hasher);
                Ok(Value::Int(hasher.finish() as i64))
            },
        }),
    );

    // time() - Current Unix timestamp in seconds (as float)
    define(
        "time".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "time".to_string(),
            arity: 0,
            func: |_| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();
                Ok(Value::Float(
                    duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1_000_000_000.0,
                ))
            },
        }),
    );

    // sleep(seconds) - Sleep for given seconds
    define(
        "sleep".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "sleep".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Int(n) => {
                    std::thread::sleep(std::time::Duration::from_secs(*n as u64));
                    Ok(Value::Unit)
                }
                Value::Float(f) => {
                    std::thread::sleep(std::time::Duration::from_secs_f64(*f));
                    Ok(Value::Unit)
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // random() - Random float between 0 and 1
    define(
        "random".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "random".to_string(),
            arity: 0,
            func: |_| {
                // Simple LCG random (not cryptographically secure)
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                let random = (seed.wrapping_mul(6364136223846793005).wrapping_add(1)) as f64
                    / u64::MAX as f64;
                Ok(Value::Float(random))
            },
        }),
    );

    // random_int(min, max) - Random int between min and max (inclusive)
    define(
        "random_int".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "random_int".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Int(min), Value::Int(max)) => {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;
                    let range = (max - min + 1) as u64;
                    let random =
                        min + (seed.wrapping_mul(6364136223846793005).wrapping_add(1) % range)
                            as i64;
                    Ok(Value::Int(random))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "int, int".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // env(name) - Get environment variable
    define(
        "env".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "env".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(name) => {
                    Ok(Value::String(std::env::var(name).unwrap_or_default()))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );
}

// ============================================================================
// FILE SYSTEM FUNCTIONS
// ============================================================================

fn register_fs_functions(define: &mut impl FnMut(String, Value)) {
    // fs_write_file(path, content) - write content to path; auto-create parent dirs
    define(
        "fs_write_file".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "fs_write_file".to_string(),
            arity: 2,
            func: |args| {
                let (path, content) = match (&args[0], &args[1]) {
                    (Value::String(p), Value::String(c)) => (p.clone(), c.clone()),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "string, string".to_string(),
                            got: format!("{:?}, {:?}", args[0], args[1]),
                        })
                    }
                };
                if let Some(parent) = std::path::Path::new(&path).parent() {
                    if !parent.as_os_str().is_empty() && !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            RuntimeError::Custom(format!(
                                "fs_write_file: create_dir_all({}) failed: {}",
                                parent.display(),
                                e
                            ))
                        })?;
                    }
                }
                std::fs::write(&path, content).map_err(|e| {
                    RuntimeError::Custom(format!("fs_write_file({}) failed: {}", path, e))
                })?;
                Ok(Value::Unit)
            },
        }),
    );

    // fs_read_file(path) -> String
    define(
        "fs_read_file".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "fs_read_file".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(path) => std::fs::read_to_string(path).map(Value::String).map_err(
                    |e| RuntimeError::Custom(format!("fs_read_file({}) failed: {}", path, e)),
                ),
                _ => Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // fs_create_dir_all(path) -> Unit
    define(
        "fs_create_dir_all".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "fs_create_dir_all".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(path) => std::fs::create_dir_all(path)
                    .map(|_| Value::Unit)
                    .map_err(|e| {
                        RuntimeError::Custom(format!("fs_create_dir_all({}) failed: {}", path, e))
                    }),
                _ => Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // fs_exists(path) -> Bool
    define(
        "fs_exists".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "fs_exists".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(path) => Ok(Value::Bool(std::path::Path::new(path).exists())),
                _ => Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // fs_list_dir(path) -> Array<String> - entry names (not full paths) in
    // `path`, sorted for deterministic tooling output. Errors if `path` is not
    // a readable directory. See hyperpolymath/my-lang#55.
    define(
        "fs_list_dir".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "fs_list_dir".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(path) => {
                    let rd = std::fs::read_dir(path).map_err(|e| {
                        RuntimeError::Custom(format!("fs_list_dir({}) failed: {}", path, e))
                    })?;
                    let mut names: Vec<String> = Vec::new();
                    for entry in rd {
                        let entry = entry.map_err(|e| {
                            RuntimeError::Custom(format!("fs_list_dir({}) failed: {}", path, e))
                        })?;
                        names.push(entry.file_name().to_string_lossy().into_owned());
                    }
                    names.sort();
                    Ok(Value::Array(names.into_iter().map(Value::String).collect()))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );
}

// ============================================================================
// ENV EXTRAS
// ============================================================================

fn register_env_extras(define: &mut impl FnMut(String, Value)) {
    // env_args() -> Array<String> - argv visible to the My program.
    // Prefers host-supplied args (my-cli forwards trailing args after `--`);
    // falls back to OS argv.skip(1) for backwards compat.
    define(
        "env_args".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "env_args".to_string(),
            arity: 0,
            func: |_| {
                let args: Vec<Value> = match PROGRAM_ARGS.get() {
                    Some(host_args) => host_args.iter().cloned().map(Value::String).collect(),
                    None => std::env::args().skip(1).map(Value::String).collect(),
                };
                Ok(Value::Array(args))
            },
        }),
    );
}

// ============================================================================
// MAP / DICT FUNCTIONS
// ============================================================================
//
// String-keyed maps backed by `Value::Record(HashMap<String, Value>)`. The
// interpreter's `eval_field` only resolves *static* identifier keys, so tooling
// (config tables, lookup tables, JSON-shaped data) has no way to use dynamic /
// computed keys without these builtins. See hyperpolymath/my-lang#46 / #45.
//
// Mutation builtins follow the same immutable convention as `push`: they clone
// the underlying map and return a new `Record` rather than mutating in place.

fn register_map_functions(define: &mut impl FnMut(String, Value)) {
    // map_new() -> Record - an empty string-keyed map.
    define(
        "map_new".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "map_new".to_string(),
            arity: 0,
            func: |_| Ok(Value::Record(HashMap::new())),
        }),
    );

    // map_set(map, key, value) -> Record - new map with `key` set to `value`.
    define(
        "map_set".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "map_set".to_string(),
            arity: 3,
            func: |args| match (&args[0], &args[1]) {
                (Value::Record(map), Value::String(key)) => {
                    let mut new_map = map.clone();
                    new_map.insert(key.clone(), args[2].clone());
                    Ok(Value::Record(new_map))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "map, string, value".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // map_get(map, key) -> value - errors if the key is absent (use map_has to test).
    define(
        "map_get".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "map_get".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Record(map), Value::String(key)) => map
                    .get(key)
                    .cloned()
                    .ok_or_else(|| RuntimeError::FieldNotFound(key.clone())),
                _ => Err(RuntimeError::TypeError {
                    expected: "map, string".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // map_has(map, key) -> Bool - true iff `key` is present.
    define(
        "map_has".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "map_has".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Record(map), Value::String(key)) => Ok(Value::Bool(map.contains_key(key))),
                _ => Err(RuntimeError::TypeError {
                    expected: "map, string".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // map_keys(map) -> Array<String> - keys in sorted order (deterministic for tooling).
    define(
        "map_keys".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "map_keys".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Record(map) => {
                    let mut keys: Vec<String> = map.keys().cloned().collect();
                    keys.sort();
                    Ok(Value::Array(keys.into_iter().map(Value::String).collect()))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "map".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // map_remove(map, key) -> Record - new map without `key` (no-op if absent).
    define(
        "map_remove".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "map_remove".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Record(map), Value::String(key)) => {
                    let mut new_map = map.clone();
                    new_map.remove(key);
                    Ok(Value::Record(new_map))
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "map, string".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
            },
        }),
    );

    // map_len(map) -> Int - number of entries.
    define(
        "map_len".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "map_len".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Record(map) => Ok(Value::Int(map.len() as i64)),
                _ => Err(RuntimeError::TypeError {
                    expected: "map".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );
}

// ============================================================================
// JSON FUNCTIONS
// ============================================================================
//
// json_parse(s) -> Value and json_stringify(v) -> String, backed by serde_json
// (a non-optional dependency of this crate). See hyperpolymath/my-lang#47 / #45.
//
// Representation mapping (paired with the Map/dict builtins, #46):
//   JSON object  <-> Value::Record   JSON array  <-> Value::Array
//   JSON string  <-> Value::String   JSON bool   <-> Value::Bool
//   JSON null    <-> Value::Unit
//   JSON number  ->  Value::Int when integral and i64-representable, else Float
//                <-  Int as integer, Float as fractional
// Object keys serialize in sorted order (serde_json's default Map is a
// BTreeMap), matching the deterministic ordering of `map_keys`.

/// serde_json::Value -> My Value (total; never fails).
fn json_to_value(j: serde_json::Value) -> Value {
    match j {
        serde_json::Value::Null => Value::Unit,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                // u64 too large for i64, or a real: fall back to float.
                Value::Float(n.as_f64().unwrap_or(f64::NAN))
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => Value::Array(arr.into_iter().map(json_to_value).collect()),
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_value(v));
            }
            Value::Record(map)
        }
    }
}

/// My Value -> serde_json::Value. Errors on values with no JSON analogue
/// (functions, native functions, AI results).
fn value_to_json(v: &Value) -> Result<serde_json::Value, RuntimeError> {
    match v {
        Value::Unit => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Int(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| {
                RuntimeError::Custom(format!(
                    "json_stringify: {} has no JSON representation (NaN/Infinity)",
                    f
                ))
            }),
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for item in arr {
                out.push(value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(out))
        }
        Value::Record(map) => {
            let mut obj = serde_json::Map::new();
            for (k, val) in map {
                obj.insert(k.clone(), value_to_json(val)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        other => Err(RuntimeError::TypeError {
            expected: "JSON-representable value".to_string(),
            got: format!("{:?}", other),
        }),
    }
}

fn register_json_functions(define: &mut impl FnMut(String, Value)) {
    // json_parse(s) -> Value - parse a JSON document into a My value.
    define(
        "json_parse".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "json_parse".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::String(s) => serde_json::from_str::<serde_json::Value>(s)
                    .map(json_to_value)
                    .map_err(|e| RuntimeError::Custom(format!("json_parse: invalid JSON: {}", e))),
                _ => Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                }),
            },
        }),
    );

    // json_stringify(v) -> String - serialize a My value to a JSON string
    // (compact, object keys sorted for deterministic output).
    define(
        "json_stringify".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "json_stringify".to_string(),
            arity: 1,
            func: |args| {
                let j = value_to_json(&args[0])?;
                serde_json::to_string(&j)
                    .map(Value::String)
                    .map_err(|e| RuntimeError::Custom(format!("json_stringify: {}", e)))
            },
        }),
    );
}

// ============================================================================
// DATE FUNCTIONS
// ============================================================================
//
// date_today() -> String, ISO `YYYY-MM-DD` in UTC. `time()` only yields a Unix
// timestamp float; tooling that stamps generated artifacts needs a calendar
// date. See hyperpolymath/my-lang#48 / #45.
//
// The civil date is derived from epoch seconds with Howard Hinnant's
// `civil_from_days` algorithm (public domain) — no date crate dependency, and
// correct for all Gregorian dates.

/// (year, month [1..=12], day [1..=31]) for a count of days since 1970-01-01.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (y + if m <= 2 { 1 } else { 0 }, m, d)
}

fn register_date_functions(define: &mut impl FnMut(String, Value)) {
    // date_today() -> String - current UTC date as ISO `YYYY-MM-DD`.
    define(
        "date_today".to_string(),
        Value::NativeFunction(NativeFunction {
            name: "date_today".to_string(),
            arity: 0,
            func: |_| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let days = secs.div_euclid(86_400);
                let (y, m, d) = civil_from_days(days);
                Ok(Value::String(format!("{:04}-{:02}-{:02}", y, m, d)))
            },
        }),
    );
}

/// Get a list of all stdlib function names
pub fn stdlib_functions() -> Vec<&'static str> {
    vec![
        // I/O
        "print",
        "println",
        "debug",
        "input",
        "input_prompt",
        // String
        "len",
        "str_concat",
        "str_split",
        "str_join",
        "str_trim",
        "str_upper",
        "str_lower",
        "str_contains",
        "str_replace",
        "str_starts_with",
        "str_ends_with",
        "str_substring",
        "char_at",
        "format",
        // Math
        "abs",
        "min",
        "max",
        "floor",
        "ceil",
        "round",
        "sqrt",
        "pow",
        "mod",
        "sin",
        "cos",
        "tan",
        "log",
        "log10",
        "exp",
        // Math constants
        "PI",
        "E",
        "TAU",
        // Array
        "push",
        "pop",
        "first",
        "last",
        "get",
        "set",
        "concat",
        "slice",
        "reverse",
        "contains",
        "range",
        "is_empty",
        // Type
        "type_of",
        "to_string",
        "to_int",
        "to_float",
        "to_bool",
        "is_int",
        "is_float",
        "is_string",
        "is_bool",
        "is_array",
        "is_function",
        // Utility
        "assert",
        "assert_eq",
        "panic",
        "identity",
        "clone",
        "default",
        "hash",
        "time",
        "sleep",
        "random",
        "random_int",
        "env",
        // FS
        "fs_write_file",
        "fs_read_file",
        "fs_create_dir_all",
        "fs_exists",
        "fs_list_dir",
        // Env extras
        "env_args",
        // Map / dict
        "map_new",
        "map_set",
        "map_get",
        "map_has",
        "map_keys",
        "map_remove",
        "map_len",
        // JSON
        "json_parse",
        "json_stringify",
        // Date
        "date_today",
    ]
}
