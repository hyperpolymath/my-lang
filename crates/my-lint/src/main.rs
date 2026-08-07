// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! My Language Linter
//!
//! This tool lints My Language code for syntax and style issues.

use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file to lint
    #[arg(short, long)]
    input: PathBuf,
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

    // Lint the content
    let issues = lint_my(&content);

    // Print the issues
    for issue in &issues {
        println!("{}:{}: {}", input_path.display(), issue.line, issue.message);
    }

    if issues.is_empty() {
        println!("No issues found");
    }
}

#[derive(Debug)]
struct LintIssue {
    line: usize,
    message: String,
}

fn lint_my(content: &str) -> Vec<LintIssue> {
    let mut issues = Vec::new();

    // Check for missing semicolons
    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.ends_with(';') && !trimmed.starts_with("fn") && !trimmed.starts_with("if") && !trimmed.starts_with("for") && !trimmed.starts_with("while") && !trimmed.starts_with("match") {
            issues.push(LintIssue {
                line: line_num,
                message: "Missing semicolon".to_string(),
            });
        }
    }

    // Check for uppercase keywords
    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        let keywords = ["fn", "let", "if", "else", "for", "while", "match"];
        for keyword in keywords {
            if line.to_lowercase().contains(&format!(" {} ", keyword)) {
                issues.push(LintIssue {
                    line: line_num,
                    message: format!("Keyword '{}' should be lowercase", keyword),
                });
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_my() {
        // "let x = 1" has no trailing ';' and does not start with an excluded
        // keyword, so the missing-semicolon lint fires. The keyword-casing lint
        // requires the keyword to be surrounded by spaces (" let "), which this
        // line is not, so it does not fire here.
        let input = "let x = 1";
        let issues = lint_my(input);
        assert_eq!(issues.len(), 1, "expected only the missing-semicolon lint");
        assert_eq!(issues[0].message, "Missing semicolon");
    }
}
