// SPDX-License-Identifier: MIT
//! My Language Linter CLI

use clap::Parser;
use my_lint::{Linter, LintConfig, LintError, Severity};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "my-lint")]
#[command(about = "Lint My Language source files")]
struct Args {
    /// Files to lint
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Treat warnings as errors
    #[arg(short = 'W', long)]
    warnings_as_errors: bool,

    /// Disable specific rules (comma-separated)
    #[arg(long)]
    disable: Option<String>,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    format: String,
}

fn main() -> Result<(), LintError> {
    let args = Args::parse();

    let disabled_rules: Vec<String> = args
        .disable
        .map(|s| s.split(',').map(|r| r.trim().to_string()).collect())
        .unwrap_or_default();

    let config = LintConfig {
        disabled_rules,
        error_on_warnings: args.warnings_as_errors,
    };

    let linter = Linter::new(config);
    let mut has_errors = false;
    let mut all_diagnostics = Vec::new();

    for file in &args.files {
        match linter.lint_file(file) {
            Ok(diagnostics) => {
                for diag in &diagnostics {
                    if diag.severity == Severity::Error
                        || (args.warnings_as_errors && diag.severity == Severity::Warning)
                    {
                        has_errors = true;
                    }
                }

                if args.format == "json" {
                    all_diagnostics.extend(diagnostics);
                } else {
                    for diag in diagnostics {
                        let severity = match diag.severity {
                            Severity::Error => "error",
                            Severity::Warning => "warning",
                            Severity::Info => "info",
                            Severity::Hint => "hint",
                        };

                        println!(
                            "{}:{}:{}: {}: {} [{}]",
                            file.display(),
                            diag.line,
                            diag.column,
                            severity,
                            diag.message,
                            diag.rule
                        );

                        if let Some(suggestion) = &diag.suggestion {
                            println!("  suggestion: {}", suggestion);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}: error: {}", file.display(), e);
                has_errors = true;
            }
        }
    }

    if args.format == "json" {
        println!("{}", serde_json::to_string_pretty(&all_diagnostics).unwrap());
    }

    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}
