// SPDX-License-Identifier: MIT
//! My Language Test Runner CLI

use clap::Parser;
use my_test::{discover_tests, TestRunner, TestConfig, TestError};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "my-test")]
#[command(about = "Run My Language tests")]
struct Args {
    /// Files or directories to test
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,

    /// Only run tests matching filter
    #[arg(short, long)]
    filter: Option<String>,

    /// Skip tests matching filter
    #[arg(long)]
    skip: Option<String>,

    /// Number of parallel workers
    #[arg(short = 'j', long)]
    jobs: Option<usize>,

    /// Timeout per test in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Run benchmarks
    #[arg(long)]
    bench: bool,

    /// Don't capture output
    #[arg(long)]
    nocapture: bool,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    format: String,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), TestError> {
    let args = Args::parse();

    let config = TestConfig {
        workers: args.jobs.unwrap_or_else(|| std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1)),
        timeout: Duration::from_secs(args.timeout),
        filter: args.filter,
        skip: args.skip,
        bench: args.bench,
        capture: !args.nocapture,
    };

    // Discover tests
    let tests = discover_tests(&args.paths)?;

    if tests.is_empty() {
        println!("No tests found");
        return Ok(());
    }

    if args.verbose {
        println!("Found {} tests:", tests.len());
        for test in &tests {
            println!("  {}", test.name);
        }
        println!();
    }

    // Run tests
    let runner = TestRunner::new(config);
    let results = runner.run(tests).await;

    // Output results
    if args.format == "json" {
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    } else {
        println!("\nTest Results:");
        println!("{}", "─".repeat(50));

        for result in &results.results {
            let status = if result.passed {
                "\x1b[32m✓\x1b[0m"
            } else {
                "\x1b[31m✗\x1b[0m"
            };

            println!("{} {} ({:?})", status, result.name, result.duration);

            if let Some(error) = &result.error {
                println!("  \x1b[31m{}\x1b[0m", error);
            }
        }

        println!("{}", "─".repeat(50));
        println!(
            "\x1b[{}m{} passed\x1b[0m, \x1b[{}m{} failed\x1b[0m, {} total in {:?}",
            if results.passed > 0 { "32" } else { "0" },
            results.passed,
            if results.failed > 0 { "31" } else { "0" },
            results.failed,
            results.total,
            results.duration
        );
    }

    if !results.success() {
        std::process::exit(1);
    }

    Ok(())
}
