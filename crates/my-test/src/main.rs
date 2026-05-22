// SPDX-License-Identifier: MPL-2.0
//! My Language Test Runner CLI

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "my-test")]
#[command(about = "Run My Language tests", long_about = None)]
#[command(version)]
struct Cli {
    /// Test files or directories
    paths: Vec<PathBuf>,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Output format
    #[arg(short = 'f', long, default_value = "human")]
    format: OutputFormat,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.paths.is_empty() {
        eprintln!("Error: No test files specified");
        std::process::exit(1);
    }

    let mut runner = my_test::TestRunner::new();

    // Discover and run tests
    for path in &cli.paths {
        if path.is_file() {
            run_test_file(&mut runner, path)?;
        } else if path.is_dir() {
            discover_and_run_tests(&mut runner, path)?;
        }
    }

    let results = runner.finish();

    // Output results
    match cli.format {
        OutputFormat::Human => print_human_results(&results, cli.verbose),
        OutputFormat::Json => print_json_results(&results)?,
    }

    if results.failed > 0 || results.errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn run_test_file(runner: &mut my_test::TestRunner, path: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    runner.run_test_file(&path.display().to_string(), &source)?;
    Ok(())
}

fn discover_and_run_tests(runner: &mut my_test::TestRunner, dir: &PathBuf) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "my" && path.file_name().unwrap().to_str().unwrap().contains("test") {
                    run_test_file(runner, &path)?;
                }
            }
        } else if path.is_dir() {
            discover_and_run_tests(runner, &path)?;
        }
    }
    Ok(())
}

fn print_human_results(results: &my_test::TestSuite, verbose: bool) {
    println!("\n{}", "=".repeat(60));
    println!("Test Results");
    println!("{}", "=".repeat(60));

    for test in &results.tests {
        let status = match &test.result {
            my_test::TestResult::Passed => "✓ PASS",
            my_test::TestResult::Failed { .. } => "✗ FAIL",
            my_test::TestResult::Error { .. } => "✗ ERROR",
        };

        println!(
            "{} {} ({:.2}ms)",
            status,
            test.name,
            test.duration.as_millis()
        );

        if verbose || !matches!(test.result, my_test::TestResult::Passed) {
            match &test.result {
                my_test::TestResult::Failed { message } => {
                    println!("  Failure: {}", message);
                }
                my_test::TestResult::Error { message } => {
                    println!("  Error: {}", message);
                }
                _ => {}
            }
        }
    }

    println!("{}", "=".repeat(60));
    println!(
        "Total: {} | Passed: {} | Failed: {} | Errors: {}",
        results.total(),
        results.passed,
        results.failed,
        results.errors
    );
    println!("Success rate: {:.1}%", results.success_rate());
    println!("{}", "=".repeat(60));
}

fn print_json_results(results: &my_test::TestSuite) -> Result<()> {
    let json_results = serde_json::json!({
        "total": results.total(),
        "passed": results.passed,
        "failed": results.failed,
        "errors": results.errors,
        "success_rate": results.success_rate(),
        "tests": results.tests.iter().map(|test| {
            serde_json::json!({
                "name": test.name,
                "file": test.file,
                "result": match &test.result {
                    my_test::TestResult::Passed => "passed",
                    my_test::TestResult::Failed { .. } => "failed",
                    my_test::TestResult::Error { .. } => "error",
                },
                "message": match &test.result {
                    my_test::TestResult::Passed => None,
                    my_test::TestResult::Failed { message } => Some(message),
                    my_test::TestResult::Error { message } => Some(message),
                },
                "duration_ms": test.duration.as_millis(),
            })
        }).collect::<Vec<_>>(),
    });

    println!("{}", serde_json::to_string_pretty(&json_results)?);
    Ok(())
}
