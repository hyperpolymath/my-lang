//! Common String Operations
//!
//! Generic string manipulation functions.

/// Get string length
pub fn len(s: &str) -> usize {
    s.len()
}

/// Get character count (Unicode-aware)
pub fn char_count(s: &str) -> usize {
    s.chars().count()
}

/// Concatenate two strings
pub fn concat(a: &str, b: &str) -> String {
    format!("{}{}", a, b)
}

/// Split string by delimiter
pub fn split(s: &str, delim: &str) -> Vec<String> {
    s.split(delim).map(|p| p.to_string()).collect()
}

/// Split string by whitespace
pub fn split_whitespace(s: &str) -> Vec<String> {
    s.split_whitespace().map(|p| p.to_string()).collect()
}

/// Join strings with delimiter
pub fn join(parts: &[String], delim: &str) -> String {
    parts.join(delim)
}

/// Trim whitespace from both ends
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

/// Trim whitespace from start
pub fn trim_start(s: &str) -> String {
    s.trim_start().to_string()
}

/// Trim whitespace from end
pub fn trim_end(s: &str) -> String {
    s.trim_end().to_string()
}

/// Convert to uppercase
pub fn to_upper(s: &str) -> String {
    s.to_uppercase()
}

/// Convert to lowercase
pub fn to_lower(s: &str) -> String {
    s.to_lowercase()
}

/// Capitalize first character
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

/// Check if contains substring
pub fn contains(s: &str, sub: &str) -> bool {
    s.contains(sub)
}

/// Check if starts with prefix
pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Check if ends with suffix
pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// Replace all occurrences
pub fn replace(s: &str, from: &str, to: &str) -> String {
    s.replace(from, to)
}

/// Replace first occurrence
pub fn replace_first(s: &str, from: &str, to: &str) -> String {
    s.replacen(from, to, 1)
}

/// Get substring by byte indices
pub fn substring(s: &str, start: usize, end: usize) -> Option<String> {
    if start <= end && end <= s.len() {
        Some(s[start..end].to_string())
    } else {
        None
    }
}

/// Get character at index
pub fn char_at(s: &str, idx: usize) -> Option<char> {
    s.chars().nth(idx)
}

/// Get byte at index
pub fn byte_at(s: &str, idx: usize) -> Option<u8> {
    s.as_bytes().get(idx).copied()
}

/// Find first occurrence of substring
pub fn find(s: &str, sub: &str) -> Option<usize> {
    s.find(sub)
}

/// Find last occurrence of substring
pub fn rfind(s: &str, sub: &str) -> Option<usize> {
    s.rfind(sub)
}

/// Reverse string
pub fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Repeat string n times
pub fn repeat(s: &str, n: usize) -> String {
    s.repeat(n)
}

/// Pad start to length
pub fn pad_start(s: &str, len: usize, pad: char) -> String {
    if s.len() >= len {
        s.to_string()
    } else {
        let padding: String = std::iter::repeat(pad).take(len - s.len()).collect();
        format!("{}{}", padding, s)
    }
}

/// Pad end to length
pub fn pad_end(s: &str, len: usize, pad: char) -> String {
    if s.len() >= len {
        s.to_string()
    } else {
        let padding: String = std::iter::repeat(pad).take(len - s.len()).collect();
        format!("{}{}", s, padding)
    }
}

/// Check if string is empty
pub fn is_empty(s: &str) -> bool {
    s.is_empty()
}

/// Check if string is blank (empty or only whitespace)
pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// Check if string is numeric
pub fn is_numeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_numeric())
}

/// Check if string is alphanumeric
pub fn is_alphanumeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric())
}

/// Check if string is alphabetic
pub fn is_alphabetic(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphabetic())
}

/// Convert to character array
pub fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

/// Convert to byte array
pub fn bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// Parse integer from string
pub fn parse_int(s: &str) -> Option<i64> {
    s.trim().parse().ok()
}

/// Parse float from string
pub fn parse_float(s: &str) -> Option<f64> {
    s.trim().parse().ok()
}

/// Format integer to string with radix
pub fn int_to_string_radix(n: i64, radix: u32) -> Option<String> {
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
        let c = char::from_digit(digit, radix).unwrap();
        result.insert(0, c);
        num /= radix as u64;
    }

    if n < 0 {
        result.insert(0, '-');
    }

    Some(result)
}

/// Count occurrences of substring
pub fn count(s: &str, sub: &str) -> usize {
    if sub.is_empty() {
        return 0;
    }
    s.matches(sub).count()
}

/// Get lines from string
pub fn lines(s: &str) -> Vec<String> {
    s.lines().map(|l| l.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(len("hello"), 5);
        assert_eq!(char_count("hello"), 5);
    }

    #[test]
    fn test_concat() {
        assert_eq!(concat("hello", " world"), "hello world");
    }

    #[test]
    fn test_split_join() {
        let parts = split("a,b,c", ",");
        assert_eq!(parts, vec!["a", "b", "c"]);
        assert_eq!(join(&parts, "-"), "a-b-c");
    }

    #[test]
    fn test_case() {
        assert_eq!(to_upper("hello"), "HELLO");
        assert_eq!(to_lower("HELLO"), "hello");
        assert_eq!(capitalize("hello"), "Hello");
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim("  hello  "), "hello");
        assert_eq!(trim_start("  hello"), "hello");
        assert_eq!(trim_end("hello  "), "hello");
    }

    #[test]
    fn test_contains_and_find() {
        assert!(contains("hello world", "world"));
        assert!(!contains("hello world", "foo"));
        assert_eq!(find("hello", "ll"), Some(2));
    }

    #[test]
    fn test_replace() {
        assert_eq!(replace("hello world", "world", "rust"), "hello rust");
    }

    #[test]
    fn test_reverse() {
        assert_eq!(reverse("hello"), "olleh");
    }

    #[test]
    fn test_pad() {
        assert_eq!(pad_start("42", 5, '0'), "00042");
        assert_eq!(pad_end("hi", 5, '!'), "hi!!!");
    }

    #[test]
    fn test_parse() {
        assert_eq!(parse_int("42"), Some(42));
        assert_eq!(parse_float("3.14"), Some(3.14));
    }

    #[test]
    fn test_radix() {
        assert_eq!(int_to_string_radix(255, 16), Some("ff".to_string()));
        assert_eq!(int_to_string_radix(10, 2), Some("1010".to_string()));
    }
}
