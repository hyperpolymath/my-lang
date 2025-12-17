//! Standard Library for My Language
//!
//! This module provides built-in functions and types that are automatically
//! available in every program.

use crate::interpreter::{NativeFunction, RuntimeError, Value};
use std::collections::HashMap;

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
                    let idx = *idx as usize;
                    if idx < s.len() {
                        Ok(Value::String(s.chars().nth(idx).unwrap().to_string()))
                    } else {
                        Err(RuntimeError::IndexOutOfBounds {
                            index: idx as i64,
                            length: s.len(),
                        })
                    }
                }
                _ => Err(RuntimeError::TypeError {
                    expected: "string, int".to_string(),
                    got: format!("{:?}, {:?}", args[0], args[1]),
                }),
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
    ]
}
