// SPDX-License-Identifier: MIT
//! My Language Formatter CLI

use clap::Parser;
use my_fmt::{format_file, check_formatted, FormatConfig, FormatError};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "my-fmt")]
#[command(about = "Format My Language source files")]
struct Args {
    /// Files to format
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Check if files are formatted (don't modify)
    #[arg(short, long)]
    check: bool,

    /// Maximum line width
    #[arg(long, default_value = "100")]
    max_width: usize,

    /// Use tabs instead of spaces
    #[arg(long)]
    tabs: bool,

    /// Spaces per indent level
    #[arg(long, default_value = "4")]
    indent_size: usize,
}

fn main() -> Result<(), FormatError> {
    let args = Args::parse();

    let config = FormatConfig {
        max_width: args.max_width,
        indent: if args.tabs {
            "\t".to_string()
        } else {
            " ".repeat(args.indent_size)
        },
        ..Default::default()
    };

    let mut has_errors = false;

    for file in &args.files {
        if args.check {
            match check_formatted(file, &config) {
                Ok(true) => {
                    println!("{}: OK", file.display());
                }
                Ok(false) => {
                    println!("{}: needs formatting", file.display());
                    has_errors = true;
                }
                Err(e) => {
                    eprintln!("{}: error: {}", file.display(), e);
                    has_errors = true;
                }
            }
        } else {
            match format_file(file, &config) {
                Ok(()) => {
                    println!("Formatted {}", file.display());
                }
                Err(e) => {
                    eprintln!("{}: error: {}", file.display(), e);
                    has_errors = true;
                }
            }
        }
    }

    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}
