// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//! My Language Linter
//!
//! Static analysis and code quality checks for My Language.

#![forbid(unsafe_code)]
use my_lang::{Program, TopLevel, FnDecl, Stmt, Expr, Pattern};
use std::collections::HashSet;
use thiserror::Error;

/// Lint severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Lint diagnostic
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
}

/// Linter configuration
#[derive(Debug, Clone)]
pub struct LintConfig {
    pub check_unused_variables: bool,
    pub check_unused_functions: bool,
    pub check_shadowing: bool,
    pub check_naming_conventions: bool,
    pub check_ai_safety: bool,
    pub max_function_lines: usize,
    pub max_complexity: usize,
}

impl Default for LintConfig {
    fn default() -> Self {
        LintConfig {
            check_unused_variables: true,
            check_unused_functions: true,
            check_shadowing: true,
            check_naming_conventions: true,
            check_ai_safety: true,
            max_function_lines: 100,
            max_complexity: 10,
        }
    }
}

/// Linter for My Language
pub struct Linter {
    config: LintConfig,
    diagnostics: Vec<Diagnostic>,
    defined_functions: HashSet<String>,
    called_functions: HashSet<String>,
}

impl Linter {
    /// Create a new linter with default configuration
    pub fn new() -> Self {
        Self::with_config(LintConfig::default())
    }

    /// Create a new linter with custom configuration
    pub fn with_config(config: LintConfig) -> Self {
        Linter {
            config,
            diagnostics: Vec::new(),
            defined_functions: HashSet::new(),
            called_functions: HashSet::new(),
        }
    }

    /// Lint a program and return diagnostics
    pub fn lint(&mut self, program: &Program) -> Vec<Diagnostic> {
        self.diagnostics.clear();
        self.defined_functions.clear();
        self.called_functions.clear();

        // First pass: collect function definitions
        for item in &program.items {
            if let TopLevel::Function(f) = item {
                self.defined_functions.insert(f.name.name.clone());
            }
        }

        // Second pass: analyze each top-level item
        for item in &program.items {
            self.lint_top_level(item);
        }

        // Check for unused functions
        if self.config.check_unused_functions {
            for func_name in &self.defined_functions {
                if func_name != "main" && !self.called_functions.contains(func_name) {
                    self.diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "unused-function".to_string(),
                        message: format!("Function '{}' is never called", func_name),
                        line: 0,
                        column: 0,
                    });
                }
            }
        }

        self.diagnostics.clone()
    }

    fn lint_top_level(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function(f) => self.lint_function(f),
            TopLevel::Struct(s) => {
                if self.config.check_naming_conventions {
                    if s.name.name.chars().next().is_some_and(|c| !c.is_uppercase()) {
                        self.diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            code: "naming-convention".to_string(),
                            message: format!("Struct name '{}' should start with uppercase", s.name.name),
                            line: 0,
                            column: 0,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn lint_function(&mut self, func: &FnDecl) {
        // Check naming conventions
        if self.config.check_naming_conventions {
            if func.name.name.chars().next().is_some_and(|c| !c.is_lowercase()) {
                self.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: "naming-convention".to_string(),
                    message: format!("Function name '{}' should start with lowercase", func.name.name),
                    line: 0,
                    column: 0,
                });
            }
        }

        // Check function length
        let line_count = count_lines_in_block(&func.body);
        if line_count > self.config.max_function_lines {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                code: "function-too-long".to_string(),
                message: format!(
                    "Function '{}' has {} lines, exceeds maximum of {}",
                    func.name.name, line_count, self.config.max_function_lines
                ),
                line: 0,
                column: 0,
            });
        }

        // Analyze function body
        let mut scope = HashSet::new();
        for param in &func.params {
            scope.insert(param.name.name.clone());
        }
        self.lint_block(&func.body, &mut scope);
    }

    fn lint_block(&mut self, block: &my_lang::Block, scope: &mut HashSet<String>) {
        for stmt in &block.stmts {
            self.lint_stmt(stmt, scope);
        }
    }

    fn lint_stmt(&mut self, stmt: &Stmt, scope: &mut HashSet<String>) {
        match stmt {
            Stmt::Let { name, value, .. } => {
                // Check for shadowing
                if self.config.check_shadowing && scope.contains(&name.name) {
                    self.diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "shadowing".to_string(),
                        message: format!("Variable '{}' shadows previous binding", name.name),
                        line: 0,
                        column: 0,
                    });
                }

                self.lint_expr(value, scope);
                scope.insert(name.name.clone());
            }
            Stmt::Expr(expr) => self.lint_expr(expr, scope),
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.lint_expr(v, scope);
                }
            }
            Stmt::If { condition, then_block, else_block, .. } => {
                self.lint_expr(condition, scope);
                self.lint_block(then_block, &mut scope.clone());
                if let Some(else_b) = else_block {
                    self.lint_block(else_b, &mut scope.clone());
                }
            }
            Stmt::Go { block, .. } => self.lint_block(block, &mut scope.clone()),
            Stmt::Await { value, .. } => self.lint_expr(value, scope),
            Stmt::Try { value, .. } => self.lint_expr(value, scope),
            Stmt::Comptime { block, .. } => self.lint_block(block, scope),
            Stmt::Ai(ai_stmt) => {
                if self.config.check_ai_safety {
                    self.diagnostics.push(Diagnostic {
                        severity: Severity::Info,
                        code: "ai-usage".to_string(),
                        message: "AI operation detected - ensure proper error handling".to_string(),
                        line: 0,
                        column: 0,
                    });
                }
            }
            Stmt::Belief { name, confidence, .. } => {
                // Lint: warn if confidence is outside valid range [0.0, 1.0]
                if *confidence < 0.0 || *confidence > 1.0 {
                    self.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "invalid-confidence".to_string(),
                        message: format!(
                            "Belief '{}' has confidence {} outside valid range [0.0, 1.0]",
                            name.name, confidence
                        ),
                        line: 0,
                        column: 0,
                    });
                }
                scope.insert(name.name.clone());
            }
        }
    }

    fn lint_expr(&mut self, expr: &Expr, scope: &HashSet<String>) {
        match expr {
            Expr::Ident(ident) => {
                // Check if variable is defined
                if !scope.contains(&ident.name) && !self.defined_functions.contains(&ident.name) {
                    self.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "undefined-variable".to_string(),
                        message: format!("Variable '{}' is not defined", ident.name),
                        line: 0,
                        column: 0,
                    });
                }
            }
            Expr::Call { callee, args, .. } => {
                // Track function calls
                if let Expr::Ident(func_name) = callee.as_ref() {
                    self.called_functions.insert(func_name.name.clone());

                    // Check if function exists
                    if !self.defined_functions.contains(&func_name.name) {
                        self.diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            code: "undefined-function".to_string(),
                            message: format!("Function '{}' is not defined", func_name.name),
                            line: 0,
                            column: 0,
                        });
                    }
                }

                for arg in args {
                    self.lint_expr(arg, scope);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.lint_expr(left, scope);
                self.lint_expr(right, scope);
            }
            Expr::Unary { operand, .. } => {
                self.lint_expr(operand, scope);
            }
            Expr::Field { object, .. } => {
                self.lint_expr(object, scope);
            }
            Expr::Block(block) => {
                self.lint_block(block, &mut scope.clone());
            }
            Expr::Lambda { body, .. } => {
                match body {
                    my_lang::LambdaBody::Expr(e) => self.lint_expr(e, scope),
                    my_lang::LambdaBody::Block(b) => self.lint_block(b, &mut scope.clone()),
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.lint_expr(scrutinee, scope);
                for arm in arms {
                    self.lint_expr(&arm.body, scope);
                }
            }
            Expr::Array { elements, .. } => {
                for elem in elements {
                    self.lint_expr(elem, scope);
                }
            }
            Expr::Record { fields, .. } => {
                for field in fields {
                    self.lint_expr(&field.value, scope);
                }
            }
            Expr::Try { operand, .. } | Expr::Restrict { operand, .. } => {
                self.lint_expr(operand, scope);
            }
            Expr::Ai(_) => {
                if self.config.check_ai_safety {
                    self.diagnostics.push(Diagnostic {
                        severity: Severity::Info,
                        code: "ai-usage".to_string(),
                        message: "AI expression detected - ensure proper error handling".to_string(),
                        line: 0,
                        column: 0,
                    });
                }
            }
            _ => {}
        }
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

fn count_lines_in_block(block: &my_lang::Block) -> usize {
    block.stmts.len()
}

/// Lint a program with default configuration
pub fn lint(program: &Program) -> Vec<Diagnostic> {
    let mut linter = Linter::new();
    linter.lint(program)
}

/// Lint a program with custom configuration
pub fn lint_with_config(program: &Program, config: LintConfig) -> Vec<Diagnostic> {
    let mut linter = Linter::with_config(config);
    linter.lint(program)
}
