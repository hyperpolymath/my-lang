//! Common I/O Operations
//!
//! Generic input/output functions for any language runtime.

use std::io::{self, BufRead, Write};

/// Print a string to stdout without newline
pub fn print_str(s: &str) {
    print!("{}", s);
    let _ = io::stdout().flush();
}

/// Print a string to stdout with newline
pub fn println_str(s: &str) {
    println!("{}", s);
}

/// Print debug representation
pub fn debug_print<T: std::fmt::Debug>(value: &T) {
    println!("{:?}", value);
}

/// Read a line from stdin
pub fn read_line() -> String {
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(_) => line.trim_end().to_string(),
        Err(_) => String::new(),
    }
}

/// Read a line with prompt
pub fn read_line_prompt(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    read_line()
}

/// Read all lines from stdin until EOF
pub fn read_all_lines() -> Vec<String> {
    io::stdin()
        .lock()
        .lines()
        .filter_map(|l| l.ok())
        .collect()
}

/// Write to stderr
pub fn eprint_str(s: &str) {
    eprint!("{}", s);
}

/// Write to stderr with newline
pub fn eprintln_str(s: &str) {
    eprintln!("{}", s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_functions_exist() {
        // These are side-effect functions, just verify they compile
        let _ = println_str;
        let _ = print_str;
        let _ = debug_print::<i32>;
    }
}
