// SPDX-License-Identifier: MIT
//! Linter for My Language
//!
//! Provides static analysis and code quality checks:
//! - Unused variables and imports
//! - Missing effect annotations
//! - Deprecated AI model usage
//! - Contract violations
//! - Style recommendations

use my_lang::{parse, Program, TopLevel, FnDecl, Stmt, Expr, AiModelDecl, AiModelAttr};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Lint errors
#[derive(Debug, Error)]
pub enum LintError {
    #[error("parse error: {0}")]
    ParseError(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Lint diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub rule: String,
    pub message: String,
    pub severity: Severity,
    pub line: usize,
    pub column: usize,
    pub suggestion: Option<String>,
}

/// Lint rule trait
pub trait LintRule: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn severity(&self) -> Severity;
    fn check(&self, program: &Program) -> Vec<Diagnostic>;
}

/// Unused variable rule
pub struct UnusedVariable;

impl LintRule for UnusedVariable {
    fn name(&self) -> &str {
        "unused-variable"
    }

    fn description(&self) -> &str {
        "Detects variables that are declared but never used"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for item in &program.items {
            if let TopLevel::Function(f) = item {
                let mut declared: HashSet<String> = HashSet::new();
                let mut used: HashSet<String> = HashSet::new();

                // Collect declarations from parameters
                for param in &f.params {
                    declared.insert(param.name.name.clone());
                }

                // TODO: Analyze body for declarations and usages

                // Report unused (simplified - doesn't analyze usage yet)
                for name in declared.difference(&used) {
                    if !name.starts_with('_') {
                        diagnostics.push(Diagnostic {
                            rule: self.name().to_string(),
                            message: format!("variable '{}' may be unused", name),
                            severity: Severity::Hint,
                            line: 0,
                            column: 0,
                            suggestion: Some(format!("prefix with underscore: _{}", name)),
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

/// Missing effect annotation rule
pub struct MissingEffectAnnotation;

impl LintRule for MissingEffectAnnotation {
    fn name(&self) -> &str {
        "missing-effect-annotation"
    }

    fn description(&self) -> &str {
        "Detects functions that perform effects without declaring them"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, _program: &Program) -> Vec<Diagnostic> {
        // TODO: Implement effect analysis
        Vec::new()
    }
}

/// Deprecated AI model rule
pub struct DeprecatedAIModel;

impl LintRule for DeprecatedAIModel {
    fn name(&self) -> &str {
        "deprecated-ai-model"
    }

    fn description(&self) -> &str {
        "Detects usage of deprecated AI models"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, program: &Program) -> Vec<Diagnostic> {
        let deprecated_models = vec![
            "gpt-3.5-turbo",
            "claude-2",
            "claude-instant",
        ];

        let mut diagnostics = Vec::new();

        for item in &program.items {
            if let TopLevel::AiModel(m) = item {
                for attr in &m.attributes {
                    if let AiModelAttr::Model(model) = attr {
                        if deprecated_models.contains(&model.as_str()) {
                            diagnostics.push(Diagnostic {
                                rule: self.name().to_string(),
                                message: format!("AI model '{}' is deprecated", model),
                                severity: self.severity(),
                                line: 0,
                                column: 0,
                                suggestion: Some(
                                    "Consider using a newer model version".to_string(),
                                ),
                            });
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// Contract violation rule
pub struct ContractViolation;

impl LintRule for ContractViolation {
    fn name(&self) -> &str {
        "contract-violation"
    }

    fn description(&self) -> &str {
        "Detects potential contract violations"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, _program: &Program) -> Vec<Diagnostic> {
        // TODO: Implement contract analysis
        Vec::new()
    }
}

/// Linter configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LintConfig {
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    #[serde(default)]
    pub error_on_warnings: bool,
}

/// Linter
pub struct Linter {
    rules: Vec<Box<dyn LintRule>>,
    config: LintConfig,
}

impl Linter {
    pub fn new(config: LintConfig) -> Self {
        let mut linter = Linter {
            rules: Vec::new(),
            config,
        };
        linter.register_default_rules();
        linter
    }

    fn register_default_rules(&mut self) {
        self.rules.push(Box::new(UnusedVariable));
        self.rules.push(Box::new(MissingEffectAnnotation));
        self.rules.push(Box::new(DeprecatedAIModel));
        self.rules.push(Box::new(ContractViolation));
    }

    /// Lint source code
    pub fn lint(&self, source: &str) -> Result<Vec<Diagnostic>, LintError> {
        let program = parse(source).map_err(|e| LintError::ParseError(e.to_string()))?;

        let mut diagnostics = Vec::new();

        for rule in &self.rules {
            if !self.config.disabled_rules.contains(&rule.name().to_string()) {
                diagnostics.extend(rule.check(&program));
            }
        }

        Ok(diagnostics)
    }

    /// Lint a file
    pub fn lint_file(&self, path: &std::path::Path) -> Result<Vec<Diagnostic>, LintError> {
        let source = std::fs::read_to_string(path)?;
        self.lint(&source)
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new(LintConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linter_creation() {
        let linter = Linter::default();
        assert!(!linter.rules.is_empty());
    }
}
