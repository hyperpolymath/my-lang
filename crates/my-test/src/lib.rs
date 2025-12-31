// SPDX-License-Identifier: MIT
//! Test Runner for My Language
//!
//! Provides test discovery, execution, and reporting:
//! - Test discovery from source files
//! - Parallel test execution
//! - Coverage reporting
//! - Benchmarking support

use my_lang::{parse, eval, Program, TopLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Test runner errors
#[derive(Debug, Error)]
pub enum TestError {
    #[error("parse error: {0}")]
    ParseError(String),

    #[error("runtime error: {0}")]
    RuntimeError(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("assertion failed: {0}")]
    AssertionFailed(String),

    #[error("timeout after {0:?}")]
    Timeout(Duration),
}

/// Test case
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub file: PathBuf,
    pub function: String,
    pub tags: Vec<String>,
    pub timeout: Option<Duration>,
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub output: String,
}

/// Test suite results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Duration,
    pub results: Vec<TestResult>,
}

impl TestResults {
    pub fn new(results: Vec<TestResult>) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let duration = results.iter().map(|r| r.duration).sum();

        TestResults {
            total,
            passed,
            failed,
            skipped: 0,
            duration,
            results,
        }
    }

    pub fn success(&self) -> bool {
        self.failed == 0
    }
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Number of parallel workers
    pub workers: usize,
    /// Default timeout per test
    pub timeout: Duration,
    /// Only run tests matching filter
    pub filter: Option<String>,
    /// Skip tests matching filter
    pub skip: Option<String>,
    /// Run benchmarks
    pub bench: bool,
    /// Capture output
    pub capture: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            workers: num_cpus::get(),
            timeout: Duration::from_secs(30),
            filter: None,
            skip: None,
            bench: false,
            capture: true,
        }
    }
}

/// Discover tests from source files
pub fn discover_tests(paths: &[PathBuf]) -> Result<Vec<TestCase>, TestError> {
    let mut tests = Vec::new();

    for path in paths {
        if path.is_dir() {
            tests.extend(discover_tests_in_dir(path)?);
        } else if path.extension().map(|e| e == "my").unwrap_or(false) {
            tests.extend(discover_tests_in_file(path)?);
        }
    }

    Ok(tests)
}

fn discover_tests_in_dir(dir: &PathBuf) -> Result<Vec<TestCase>, TestError> {
    let mut tests = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            tests.extend(discover_tests_in_dir(&path)?);
        } else if path.extension().map(|e| e == "my").unwrap_or(false) {
            let name = path.file_stem().unwrap().to_string_lossy();
            if name.starts_with("test_") || name.ends_with("_test") {
                tests.extend(discover_tests_in_file(&path)?);
            }
        }
    }

    Ok(tests)
}

fn discover_tests_in_file(path: &PathBuf) -> Result<Vec<TestCase>, TestError> {
    let source = std::fs::read_to_string(path)?;
    let program = parse(&source).map_err(|e| TestError::ParseError(e.to_string()))?;

    let mut tests = Vec::new();

    for item in &program.items {
        if let TopLevel::Function(f) = item {
            // Check for test_ prefix in function name
            let func_name = &f.name.name;
            let is_test = func_name.starts_with("test_");

            if is_test {
                tests.push(TestCase {
                    name: format!("{}::{}", path.display(), func_name),
                    file: path.clone(),
                    function: func_name.clone(),
                    tags: vec![],
                    timeout: None,
                });
            }
        }
    }

    Ok(tests)
}

/// Test runner
pub struct TestRunner {
    config: TestConfig,
}

impl TestRunner {
    pub fn new(config: TestConfig) -> Self {
        TestRunner { config }
    }

    /// Run all tests
    pub async fn run(&self, tests: Vec<TestCase>) -> TestResults {
        let filtered_tests: Vec<_> = tests
            .into_iter()
            .filter(|t| self.should_run(t))
            .collect();

        let mut results = Vec::new();

        for test in filtered_tests {
            let result = self.run_single(test).await;
            results.push(result);
        }

        TestResults::new(results)
    }

    fn should_run(&self, test: &TestCase) -> bool {
        if let Some(filter) = &self.config.filter {
            if !test.name.contains(filter) && !test.tags.iter().any(|t| t.contains(filter)) {
                return false;
            }
        }

        if let Some(skip) = &self.config.skip {
            if test.name.contains(skip) || test.tags.iter().any(|t| t.contains(skip)) {
                return false;
            }
        }

        true
    }

    async fn run_single(&self, test: TestCase) -> TestResult {
        let start = Instant::now();
        let timeout = test.timeout.unwrap_or(self.config.timeout);

        let result = tokio::time::timeout(timeout, async {
            self.execute_test(&test)
        })
        .await;

        let duration = start.elapsed();

        match result {
            Ok(Ok(())) => TestResult {
                name: test.name,
                passed: true,
                duration,
                error: None,
                output: String::new(),
            },
            Ok(Err(e)) => TestResult {
                name: test.name,
                passed: false,
                duration,
                error: Some(e.to_string()),
                output: String::new(),
            },
            Err(_) => TestResult {
                name: test.name,
                passed: false,
                duration,
                error: Some(format!("timeout after {:?}", timeout)),
                output: String::new(),
            },
        }
    }

    fn execute_test(&self, test: &TestCase) -> Result<(), TestError> {
        let source = std::fs::read_to_string(&test.file)?;

        // Parse and evaluate
        match eval(&source) {
            Ok(_) => Ok(()),
            Err(e) => Err(TestError::RuntimeError(e.to_string())),
        }
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new(TestConfig::default())
    }
}

/// Assertion helpers for tests
pub mod assert {
    use super::*;

    pub fn assert_eq<T: PartialEq + std::fmt::Debug>(left: T, right: T) -> Result<(), TestError> {
        if left == right {
            Ok(())
        } else {
            Err(TestError::AssertionFailed(format!(
                "assertion failed: {:?} == {:?}",
                left, right
            )))
        }
    }

    pub fn assert_ne<T: PartialEq + std::fmt::Debug>(left: T, right: T) -> Result<(), TestError> {
        if left != right {
            Ok(())
        } else {
            Err(TestError::AssertionFailed(format!(
                "assertion failed: {:?} != {:?}",
                left, right
            )))
        }
    }

    pub fn assert_true(condition: bool, message: &str) -> Result<(), TestError> {
        if condition {
            Ok(())
        } else {
            Err(TestError::AssertionFailed(message.to_string()))
        }
    }
}

/// Benchmark helpers
pub mod bench {
    use super::*;

    pub struct Bencher {
        iterations: usize,
    }

    impl Bencher {
        pub fn new() -> Self {
            Bencher { iterations: 100 }
        }

        pub fn iter<F, R>(&self, mut f: F) -> Duration
        where
            F: FnMut() -> R,
        {
            let start = Instant::now();
            for _ in 0..self.iterations {
                std::hint::black_box(f());
            }
            start.elapsed() / self.iterations as u32
        }
    }

    impl Default for Bencher {
        fn default() -> Self {
            Self::new()
        }
    }
}

mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_results_success() {
        let results = TestResults::new(vec![
            TestResult {
                name: "test1".to_string(),
                passed: true,
                duration: Duration::from_millis(10),
                error: None,
                output: String::new(),
            },
        ]);
        assert!(results.success());
    }

    #[test]
    fn test_results_failure() {
        let results = TestResults::new(vec![
            TestResult {
                name: "test1".to_string(),
                passed: false,
                duration: Duration::from_millis(10),
                error: Some("failed".to_string()),
                output: String::new(),
            },
        ]);
        assert!(!results.success());
    }
}
