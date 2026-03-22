//! Common Type Conversion and Checking Operations
//!
//! Generic type conversion and type checking functions.

/// Type tag enum for runtime type information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeTag {
    Int,
    Float,
    String,
    Bool,
    Array,
    Record,
    Function,
    Unit,
    Unknown,
}

impl std::fmt::Display for TypeTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeTag::Int => write!(f, "Int"),
            TypeTag::Float => write!(f, "Float"),
            TypeTag::String => write!(f, "String"),
            TypeTag::Bool => write!(f, "Bool"),
            TypeTag::Array => write!(f, "Array"),
            TypeTag::Record => write!(f, "Record"),
            TypeTag::Function => write!(f, "Function"),
            TypeTag::Unit => write!(f, "Unit"),
            TypeTag::Unknown => write!(f, "Unknown"),
        }
    }
}

// ============================================================================
// Integer Conversions
// ============================================================================

/// Convert string to integer
pub fn str_to_int(s: &str) -> Option<i64> {
    s.trim().parse().ok()
}

/// Convert string to integer with radix
pub fn str_to_int_radix(s: &str, radix: u32) -> Option<i64> {
    i64::from_str_radix(s.trim(), radix).ok()
}

/// Convert float to integer (truncate)
pub fn float_to_int(f: f64) -> i64 {
    f as i64
}

/// Convert bool to integer (0 or 1)
pub fn bool_to_int(b: bool) -> i64 {
    if b { 1 } else { 0 }
}

/// Convert char to integer (Unicode code point)
pub fn char_to_int(c: char) -> i64 {
    c as i64
}

// ============================================================================
// Float Conversions
// ============================================================================

/// Convert string to float
pub fn str_to_float(s: &str) -> Option<f64> {
    s.trim().parse().ok()
}

/// Convert integer to float
pub fn int_to_float(n: i64) -> f64 {
    n as f64
}

/// Convert bool to float (0.0 or 1.0)
pub fn bool_to_float(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ============================================================================
// String Conversions
// ============================================================================

/// Convert integer to string
pub fn int_to_str(n: i64) -> String {
    n.to_string()
}

/// Convert integer to string with radix
pub fn int_to_str_radix(n: i64, radix: u32) -> Option<String> {
    if radix < 2 || radix > 36 {
        return None;
    }

    if n == 0 {
        return Some("0".to_string());
    }

    let mut result = String::new();
    let mut num = n.abs() as u64;

    while num > 0 {
        let digit = (num % radix as u64) as u32;
        let c = char::from_digit(digit, radix)?;
        result.insert(0, c);
        num /= radix as u64;
    }

    if n < 0 {
        result.insert(0, '-');
    }

    Some(result)
}

/// Convert float to string
pub fn float_to_str(f: f64) -> String {
    f.to_string()
}

/// Convert float to string with precision
pub fn float_to_str_precision(f: f64, precision: usize) -> String {
    format!("{:.prec$}", f, prec = precision)
}

/// Convert bool to string
pub fn bool_to_str(b: bool) -> String {
    b.to_string()
}

/// Convert char to string
pub fn char_to_str(c: char) -> String {
    c.to_string()
}

// ============================================================================
// Boolean Conversions
// ============================================================================

/// Convert string to bool
pub fn str_to_bool(s: &str) -> Option<bool> {
    match s.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Convert integer to bool (non-zero is true)
pub fn int_to_bool(n: i64) -> bool {
    n != 0
}

/// Convert float to bool (non-zero is true)
pub fn float_to_bool(f: f64) -> bool {
    f != 0.0 && !f.is_nan()
}

// ============================================================================
// Character Conversions
// ============================================================================

/// Convert integer to char (Unicode code point)
pub fn int_to_char(n: i64) -> Option<char> {
    if n >= 0 && n <= char::MAX as i64 {
        char::from_u32(n as u32)
    } else {
        None
    }
}

/// Convert string to char (first character)
pub fn str_to_char(s: &str) -> Option<char> {
    s.chars().next()
}

// ============================================================================
// Type Checking
// ============================================================================

/// Check if string can be parsed as integer
pub fn is_int_str(s: &str) -> bool {
    s.trim().parse::<i64>().is_ok()
}

/// Check if string can be parsed as float
pub fn is_float_str(s: &str) -> bool {
    s.trim().parse::<f64>().is_ok()
}

/// Check if string can be parsed as bool
pub fn is_bool_str(s: &str) -> bool {
    str_to_bool(s).is_some()
}

/// Check if integer is even
pub fn is_even(n: i64) -> bool {
    n % 2 == 0
}

/// Check if integer is odd
pub fn is_odd(n: i64) -> bool {
    n % 2 != 0
}

/// Check if integer is positive
pub fn is_positive(n: i64) -> bool {
    n > 0
}

/// Check if integer is negative
pub fn is_negative(n: i64) -> bool {
    n < 0
}

/// Check if integer is zero
pub fn is_zero(n: i64) -> bool {
    n == 0
}

/// Check if float is integer (no fractional part)
pub fn is_integer(f: f64) -> bool {
    f.fract() == 0.0 && f.is_finite()
}

// ============================================================================
// Default Values
// ============================================================================

/// Get default integer value
pub fn default_int() -> i64 {
    0
}

/// Get default float value
pub fn default_float() -> f64 {
    0.0
}

/// Get default string value
pub fn default_string() -> String {
    String::new()
}

/// Get default bool value
pub fn default_bool() -> bool {
    false
}

// ============================================================================
// Hashing
// ============================================================================

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Hash a string
pub fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Hash an integer
pub fn hash_int(n: i64) -> u64 {
    let mut hasher = DefaultHasher::new();
    n.hash(&mut hasher);
    hasher.finish()
}

/// Hash bytes
pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_conversions() {
        assert_eq!(str_to_int("42"), Some(42));
        assert_eq!(str_to_int("-17"), Some(-17));
        assert_eq!(str_to_int("abc"), None);
        assert_eq!(float_to_int(3.7), 3);
        assert_eq!(bool_to_int(true), 1);
        assert_eq!(bool_to_int(false), 0);
    }

    #[test]
    fn test_float_conversions() {
        assert_eq!(str_to_float("3.14"), Some(3.14));
        assert_eq!(int_to_float(42), 42.0);
    }

    #[test]
    fn test_str_conversions() {
        assert_eq!(int_to_str(42), "42");
        assert_eq!(float_to_str(3.14), "3.14");
        assert_eq!(bool_to_str(true), "true");
    }

    #[test]
    fn test_radix() {
        assert_eq!(str_to_int_radix("ff", 16), Some(255));
        assert_eq!(str_to_int_radix("1010", 2), Some(10));
        assert_eq!(int_to_str_radix(255, 16), Some("ff".to_string()));
    }

    #[test]
    fn test_bool_conversions() {
        assert_eq!(str_to_bool("true"), Some(true));
        assert_eq!(str_to_bool("false"), Some(false));
        assert_eq!(str_to_bool("yes"), Some(true));
        assert_eq!(str_to_bool("no"), Some(false));
        assert_eq!(int_to_bool(0), false);
        assert_eq!(int_to_bool(1), true);
    }

    #[test]
    fn test_type_checks() {
        assert!(is_int_str("42"));
        assert!(!is_int_str("3.14"));
        assert!(is_float_str("3.14"));
        assert!(is_even(4));
        assert!(is_odd(5));
    }
}
