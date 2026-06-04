// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//! My Language Formatter
//!
//! This tool formats My Language code according to style guidelines.

use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file to format
    #[arg(short, long)]
    input: PathBuf,

    /// Output file (default: overwrite input)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    // Read the input file
    let input_path = args.input;
    let content = match fs::read_to_string(&input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    // Format the content
    let formatted = format_my(&content);

    // Write the output file
    let output_path = args.output.unwrap_or(input_path);
    match fs::write(output_path, formatted) {
        Ok(_) => println!("Formatted {}", output_path.display()),
        Err(e) => eprintln!("Error writing file: {}", e),
    }
}

fn format_my(content: &str) -> String {
    // Basic formatting: indent lines that start with keywords
    let mut formatted = String::new();
    let keywords = ["fn", "let", "if", "else", "for", "while", "match"];

    for line in content.lines() {
        let trimmed = line.trim();
        if keywords.iter().any(|&kw| trimmed.starts_with(kw)) {
            formatted.push_str("  ");
        }
        formatted.push_str(trimmed);
        formatted.push('\n');
    }

    formatted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_my() {
        let input = "fn main() {\nlet x = 1;\n}";
        let expected = "  fn main() {\n  let x = 1;\n}\n";
        assert_eq!(format_my(input), expected);
    }
}
