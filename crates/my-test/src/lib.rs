// SPDX-License-Identifier: PMPL-1.0-or-later
//! My Language Test Runner
//!
//! Discovers and executes tests written in My Language.

#![forbid(unsafe_code)]
use my_lang::Program;
use std::time::{Duration, Instant};

/// Test result
#[derive(Debug, Clone)]
pub enum TestResult {
    Passed,
    Failed { message: String },
    Error { message: String },
}

/// Test case
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub file: String,
    pub result: TestResult,
    pub duration: Duration,
}

/// Test suite results
#[derive(Debug)]
pub struct TestSuite {
    pub tests: Vec<TestCase>,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
}

impl TestSuite {
    pub fn new() -> Self {
        TestSuite {
            tests: Vec::new(),
            passed: 0,
            failed: 0,
            errors: 0,
        }
    }

    pub fn add_result(&mut self, test: TestCase) {
        match &test.result {
            TestResult::Passed => self.passed += 1,
            TestResult::Failed { .. } => self.failed += 1,
            TestResult::Error { .. } => self.errors += 1,
        }
        self.tests.push(test);
    }

    pub fn total(&self) -> usize {
        self.tests.len()
    }

    pub fn success_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            (self.passed as f64) / (self.total() as f64) * 100.0
        }
    }
}

impl Default for TestSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Test runner
pub struct TestRunner {
    suite: TestSuite,
}

impl TestRunner {
    pub fn new() -> Self {
        TestRunner {
            suite: TestSuite::new(),
        }
    }

    /// Run a single test file
    pub fn run_test_file(&mut self, path: &str, source: &str) -> Result<(), anyhow::Error> {
        let start = Instant::now();

        // Parse the test file
        let program = match my_lang::parse(source) {
            Ok(p) => p,
            Err(e) => {
                self.suite.add_result(TestCase {
                    name: path.to_string(),
                    file: path.to_string(),
                    result: TestResult::Error {
                        message: format!("Parse error: {:?}", e),
                    },
                    duration: start.elapsed(),
                });
                return Ok(());
            }
        };

        // Type check
        if let Err(e) = my_lang::check(&program) {
            self.suite.add_result(TestCase {
                name: path.to_string(),
                file: path.to_string(),
                result: TestResult::Error {
                    message: format!("Type check error: {:?}", e),
                },
                duration: start.elapsed(),
            });
            return Ok(());
        }

        // Run test through MIR interpreter
        match self.run_test_mir(&program) {
            Ok(()) => {
                self.suite.add_result(TestCase {
                    name: path.to_string(),
                    file: path.to_string(),
                    result: TestResult::Passed,
                    duration: start.elapsed(),
                });
            }
            Err(msg) => {
                self.suite.add_result(TestCase {
                    name: path.to_string(),
                    file: path.to_string(),
                    result: TestResult::Failed { message: msg },
                    duration: start.elapsed(),
                });
            }
        }

        Ok(())
    }

    fn run_test_mir(&self, program: &Program) -> Result<(), String> {
        // Lower to HIR
        let hir = my_hir::lower(program)
            .map_err(|e| format!("HIR lowering failed: {:?}", e))?;

        // Lower to MIR
        let mir = my_mir::lower(&hir)
            .map_err(|e| format!("MIR lowering failed: {:?}", e))?;

        // Run with MIR interpreter
        let mut interpreter = my_mir::interpreter::Interpreter::new(mir);
        let result = interpreter.run()
            .map_err(|e| format!("Runtime error: {:?}", e))?;

        // For now, any successful execution is a pass
        // In the future, we can check specific return values for assertions
        match result {
            my_mir::interpreter::Value::I64(0) => Ok(()),
            my_mir::interpreter::Value::Bool(true) => Ok(()),
            _ => Err(format!("Test returned non-success value: {:?}", result)),
        }
    }

    /// Get the test suite results
    pub fn results(&self) -> &TestSuite {
        &self.suite
    }

    /// Consume the runner and return the results
    pub fn finish(self) -> TestSuite {
        self.suite
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}
