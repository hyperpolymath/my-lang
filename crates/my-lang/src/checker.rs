// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//! Type checker and semantic analyzer for My Language
//!
//! Performs name resolution, type checking, and validation of AI constructs.

use crate::ast::*;
use crate::scope::*;
use crate::token::Span;
use crate::types::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CheckError {
    #[error("undefined variable '{name}' at line {line}, column {column}")]
    UndefinedVariable {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("undefined type '{name}' at line {line}, column {column}")]
    UndefinedType {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("undefined function '{name}' at line {line}, column {column}")]
    UndefinedFunction {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("undefined AI model '{name}' at line {line}, column {column}")]
    UndefinedAiModel {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("undefined prompt '{name}' at line {line}, column {column}")]
    UndefinedPrompt {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("type mismatch: expected {expected}, found {found} at line {line}, column {column}")]
    TypeMismatch {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },

    #[error("duplicate definition of '{name}' at line {line}, column {column}")]
    DuplicateDefinition {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("cannot assign to immutable variable '{name}' at line {line}, column {column}")]
    ImmutableAssignment {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("wrong number of arguments: expected {expected}, found {found} at line {line}, column {column}")]
    WrongArgCount {
        expected: usize,
        found: usize,
        line: usize,
        column: usize,
    },

    #[error("invalid binary operation: {left} {op} {right} at line {line}, column {column}")]
    InvalidBinaryOp {
        left: String,
        op: String,
        right: String,
        line: usize,
        column: usize,
    },

    #[error("condition must be Bool, found {found} at line {line}, column {column}")]
    NonBoolCondition {
        found: String,
        line: usize,
        column: usize,
    },

    #[error("{message} at line {line}, column {column}")]
    Other {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("expression nesting depth exceeds limit ({limit}) at line {line}, column {column}; refactor deeply nested calls (e.g. chained str_concat/format) into intermediate `let` bindings")]
    ExpressionTooDeep {
        limit: usize,
        line: usize,
        column: usize,
    },
}

/// Maximum AST nesting depth the type checker will recurse through before
/// emitting [`CheckError::ExpressionTooDeep`] and bailing out of the
/// subexpression.
///
/// This bounds **stack recursion**, not heap growth. The #14 investigation
/// (see `docs/wiki/internals/checker-allocation-investigation.md`) measured
/// `check_expr` / `is_assignable_from` to be *linear* in AST size on Linux
/// and on a Windows CI leg — there is no super-linear allocation for the
/// guard to defend against. What deep nesting *does* threaten is the
/// recursive descent through `check_expr`. (The recursive `Drop` of a deep
/// AST is a *separate* overflow; it is now handled structurally and
/// shape-independently by [`crate::ast::drop_program_iteratively`], so it no
/// longer constrains this value — hyperpolymath/my-lang#37.)
///
/// # Re-derived from a measured stack budget (my-lang#37)
///
/// The previous value (256) was an inherited guess, not a budget. Probing one
/// recursive `check_expr` walk on a fixed-size thread stack
/// (`examples/measure_depth.rs`) measures **≈4.4 KiB of stack per nesting
/// level**. The binding constraint is the smallest stack the checker runs on:
/// the OS main thread (the checker is *not* dispatched onto a large-stack
/// thread anywhere), i.e. **1 MiB on Windows**. At 256 levels that walk needs
/// ≈1.14 MiB — it can overflow *before* the guard fires on Windows, so 256 was
/// not merely unjustified but unsafe there.
///
/// `128 × 4.4 KiB ≈ 569 KiB ≈ 54%` of a 1 MiB Windows stack, leaving ~46%
/// headroom for the rest of the call graph above `check_expr`; vastly safe on
/// Linux (8 MiB) and Rust's 2 MiB default threads. It is still 2× the parser's
/// independently-derived [`crate::parser::MAX_PARSE_EXPR_DEPTH`] (64) ceiling,
/// so — because no *parseable* program nests deeper than 64 — lowering it
/// rejects zero real programs; it only ever fires on programmatically-built
/// ASTs, which is its sole remaining purpose.
///
/// This budget is reconfirmed automatically rather than by a one-off manual
/// run: `examples/measure_depth.rs` is self-driving (it re-execs itself as
/// worker subprocesses to find each overflow cliff and asserts the budget), and
/// the `Stack Depth (#37)` CI workflow runs it — alongside the
/// `tests/stack_depth_37.rs` regression — on **both ubuntu-latest and
/// windows-latest**. So the binding 1 MiB msvc datapoint is produced on every
/// change and a future bump that breaks the budget fails CI on the affected OS.
pub const MAX_EXPR_DEPTH: usize = 128;

pub type CheckResult<T> = Result<T, CheckError>;

/// The type checker and semantic analyzer
pub struct Checker {
    symbols: SymbolTable,
    types: TypeEnv,
    errors: Vec<CheckError>,
    /// Current function's return type (for checking return statements)
    current_return_type: Option<Ty>,
    /// Current expression-recursion depth (bounds stack recursion on
    /// pathological nesting, see [`MAX_EXPR_DEPTH`]).
    expr_depth: usize,
    /// Set once an [`CheckError::ExpressionTooDeep`] has been reported, to
    /// avoid spamming a duplicate error for every parent expression on the
    /// way out of the recursion.
    too_deep_reported: bool,
    /// Content-addressed result cache for *environment-independent*
    /// subexpressions (hyperpolymath/my-lang#16).
    ///
    /// Keyed by a span-stripped structural hash. Only populated for the
    /// provably-safe subset built by [`Checker::memo_key`] (string/number/
    /// bool literals and calls that resolve to the genuine stdlib binding,
    /// recursively). For that subset the resulting [`Ty`] is a pure function
    /// of the structural form — it does not depend on the symbol environment,
    /// introduces no scope bindings, and (because we only insert when the
    /// check produced no new diagnostics) re-encountering the expression is
    /// observably identical to re-checking it. This turns the repeated
    /// `str_concat`/`format` templating sites from #1 from
    /// O(occurrences × subtree) into O(distinct × subtree).
    expr_cache: HashMap<u64, Ty>,
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker {
    pub fn new() -> Self {
        let mut checker = Self {
            symbols: SymbolTable::new(),
            types: TypeEnv::new(),
            errors: Vec::new(),
            current_return_type: None,
            expr_depth: 0,
            too_deep_reported: false,
            expr_cache: HashMap::new(),
        };
        checker.register_stdlib();
        checker
    }

    /// Register standard library functions in the symbol table
    fn register_stdlib(&mut self) {
        use crate::stdlib::stdlib_functions;

        // Define types for each stdlib function
        for name in stdlib_functions() {
            let ty = Self::stdlib_function_type(name);
            let _ = self.symbols.define(Symbol {
                name: name.to_string(),
                kind: SymbolKind::Function,
                ty,
                span: Span::default(),
                mutable: false,
            });
        }
    }

    /// Get the type signature for a stdlib function
    fn stdlib_function_type(name: &str) -> Ty {
        match name {
            // I/O functions
            "print" | "println" | "debug" => Ty::Function {
                params: vec![Ty::Unknown], // Accepts any type
                result: Box::new(Ty::Unit),
            },
            "input" => Ty::Function {
                params: vec![],
                result: Box::new(Ty::String),
            },
            "input_prompt" => Ty::Function {
                params: vec![Ty::String],
                result: Box::new(Ty::String),
            },

            // String functions
            "len" => Ty::Function {
                params: vec![Ty::Unknown], // String or Array
                result: Box::new(Ty::Int),
            },
            "str_concat" => Ty::Function {
                params: vec![Ty::Unknown, Ty::Unknown],
                result: Box::new(Ty::String),
            },
            "str_split" => Ty::Function {
                params: vec![Ty::String, Ty::String],
                result: Box::new(Ty::Array(Box::new(Ty::String))),
            },
            "str_join" => Ty::Function {
                params: vec![Ty::Array(Box::new(Ty::String)), Ty::String],
                result: Box::new(Ty::String),
            },
            "str_trim" | "str_upper" | "str_lower" => Ty::Function {
                params: vec![Ty::String],
                result: Box::new(Ty::String),
            },
            "str_contains" | "str_starts_with" | "str_ends_with" => Ty::Function {
                params: vec![Ty::String, Ty::String],
                result: Box::new(Ty::Bool),
            },
            "str_replace" | "str_substring" => Ty::Function {
                params: vec![Ty::String, Ty::Unknown, Ty::Unknown],
                result: Box::new(Ty::String),
            },
            "char_at" => Ty::Function {
                params: vec![Ty::String, Ty::Int],
                result: Box::new(Ty::String),
            },

            // Math functions
            "abs" | "floor" | "ceil" | "round" => Ty::Function {
                params: vec![Ty::Unknown], // Numeric
                result: Box::new(Ty::Unknown),
            },
            "min" | "max" | "pow" | "mod" => Ty::Function {
                params: vec![Ty::Unknown, Ty::Unknown],
                result: Box::new(Ty::Unknown),
            },
            "sqrt" | "sin" | "cos" | "tan" | "log" | "log10" | "exp" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Float),
            },
            "PI" | "E" | "TAU" => Ty::Float,

            // Array functions
            "push" => Ty::Function {
                params: vec![Ty::Array(Box::new(Ty::Unknown)), Ty::Unknown],
                result: Box::new(Ty::Array(Box::new(Ty::Unknown))),
            },
            "pop" | "reverse" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Unknown),
            },
            "first" | "last" => Ty::Function {
                params: vec![Ty::Array(Box::new(Ty::Unknown))],
                result: Box::new(Ty::Unknown),
            },
            "get" => Ty::Function {
                params: vec![Ty::Array(Box::new(Ty::Unknown)), Ty::Int],
                result: Box::new(Ty::Unknown),
            },
            "set" => Ty::Function {
                params: vec![Ty::Array(Box::new(Ty::Unknown)), Ty::Int, Ty::Unknown],
                result: Box::new(Ty::Array(Box::new(Ty::Unknown))),
            },
            "concat" => Ty::Function {
                params: vec![Ty::Array(Box::new(Ty::Unknown)), Ty::Array(Box::new(Ty::Unknown))],
                result: Box::new(Ty::Array(Box::new(Ty::Unknown))),
            },
            "slice" => Ty::Function {
                params: vec![Ty::Array(Box::new(Ty::Unknown)), Ty::Int, Ty::Int],
                result: Box::new(Ty::Array(Box::new(Ty::Unknown))),
            },
            "contains" => Ty::Function {
                params: vec![Ty::Array(Box::new(Ty::Unknown)), Ty::Unknown],
                result: Box::new(Ty::Bool),
            },
            "range" => Ty::Function {
                params: vec![Ty::Int, Ty::Int],
                result: Box::new(Ty::Array(Box::new(Ty::Int))),
            },
            "is_empty" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Bool),
            },

            // Type functions
            "type_of" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::String),
            },
            "to_string" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::String),
            },
            "to_int" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Int),
            },
            "to_float" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Float),
            },
            "to_bool" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Bool),
            },
            "is_int" | "is_float" | "is_string" | "is_bool" | "is_array" | "is_function" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Bool),
            },

            // Utility functions
            "assert" => Ty::Function {
                params: vec![Ty::Bool],
                result: Box::new(Ty::Unit),
            },
            "assert_eq" => Ty::Function {
                params: vec![Ty::Unknown, Ty::Unknown],
                result: Box::new(Ty::Unit),
            },
            "panic" => Ty::Function {
                params: vec![Ty::String],
                result: Box::new(Ty::Unit),
            },
            "identity" | "clone" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Unknown),
            },
            "default" => Ty::Function {
                params: vec![Ty::String],
                result: Box::new(Ty::Unknown),
            },
            "hash" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Int),
            },
            "time" | "random" => Ty::Function {
                params: vec![],
                result: Box::new(Ty::Float),
            },
            "sleep" => Ty::Function {
                params: vec![Ty::Unknown],
                result: Box::new(Ty::Unit),
            },
            "random_int" => Ty::Function {
                params: vec![Ty::Int, Ty::Int],
                result: Box::new(Ty::Int),
            },
            "env" => Ty::Function {
                params: vec![Ty::String],
                result: Box::new(Ty::String),
            },

            _ => Ty::Unknown,
        }
    }

    /// Check a complete program
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<CheckError>> {
        // First pass: collect all type definitions
        for item in &program.items {
            self.collect_definitions(item);
        }

        // Second pass: type check all items
        for item in &program.items {
            self.check_top_level(item);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// First pass: collect type and function definitions
    fn collect_definitions(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Struct(s) => {
                let fields: Vec<(String, Ty)> = s.fields
                    .iter()
                    .map(|f| (f.name.name.clone(), ast_type_to_ty(&f.ty)))
                    .collect();

                let def = StructDef {
                    name: s.name.name.clone(),
                    fields,
                    type_params: s.type_params.iter().map(|p| p.name.clone()).collect(),
                    span: s.span,
                };

                if let Err(_msg) = self.types.define_struct(def) {
                    self.errors.push(CheckError::DuplicateDefinition {
                        name: s.name.name.clone(),
                        line: s.span.line,
                        column: s.span.column,
                    });
                }

                // Also add as a type symbol
                let _ = self.symbols.define(Symbol {
                    name: s.name.name.clone(),
                    kind: SymbolKind::Struct,
                    ty: Ty::Named(s.name.name.clone()),
                    span: s.span,
                    mutable: false,
                });
            }

            TopLevel::Effect(e) => {
                let operations: Vec<(String, Ty)> = e.ops
                    .iter()
                    .map(|op| (op.name.name.clone(), ast_type_to_ty(&op.ty)))
                    .collect();

                let def = EffectDef {
                    name: e.name.name.clone(),
                    operations,
                    span: e.span,
                };

                if self.types.define_effect(def).is_err() {
                    self.errors.push(CheckError::DuplicateDefinition {
                        name: e.name.name.clone(),
                        line: e.span.line,
                        column: e.span.column,
                    });
                }

                let _ = self.symbols.define(Symbol {
                    name: e.name.name.clone(),
                    kind: SymbolKind::Effect,
                    ty: Ty::Named(e.name.name.clone()),
                    span: e.span,
                    mutable: false,
                });
            }

            TopLevel::AiModel(m) => {
                let def = AiModelDef {
                    name: m.name.name.clone(),
                    provider: m.attributes.iter().find_map(|a| {
                        if let AiModelAttr::Provider(p) = a { Some(p.clone()) } else { None }
                    }),
                    model: m.attributes.iter().find_map(|a| {
                        if let AiModelAttr::Model(m) = a { Some(m.clone()) } else { None }
                    }),
                    span: m.span,
                };

                if self.types.define_ai_model(def).is_err() {
                    self.errors.push(CheckError::DuplicateDefinition {
                        name: m.name.name.clone(),
                        line: m.span.line,
                        column: m.span.column,
                    });
                }

                let _ = self.symbols.define(Symbol {
                    name: m.name.name.clone(),
                    kind: SymbolKind::AiModel,
                    ty: Ty::Named(format!("ai_model:{}", m.name.name)),
                    span: m.span,
                    mutable: false,
                });
            }

            TopLevel::Prompt(p) => {
                let def = PromptDef {
                    name: p.name.name.clone(),
                    template: p.template.clone(),
                    span: p.span,
                };

                if self.types.define_prompt(def).is_err() {
                    self.errors.push(CheckError::DuplicateDefinition {
                        name: p.name.name.clone(),
                        line: p.span.line,
                        column: p.span.column,
                    });
                }

                let _ = self.symbols.define(Symbol {
                    name: p.name.name.clone(),
                    kind: SymbolKind::Prompt,
                    ty: Ty::Function {
                        params: vec![],  // Prompts can take any args
                        result: Box::new(Ty::AI(Box::new(Ty::String))),
                    },
                    span: p.span,
                    mutable: false,
                });
            }

            TopLevel::Function(f) => {
                // Collect function signature
                let param_types: Vec<Ty> = f.params
                    .iter()
                    .map(|p| ast_type_to_ty(&p.ty))
                    .collect();

                let return_type = f.return_type
                    .as_ref()
                    .map(ast_type_to_ty)
                    .unwrap_or(Ty::Unit);

                let fn_type = Ty::Function {
                    params: param_types,
                    result: Box::new(return_type),
                };

                if self.symbols.define(Symbol {
                    name: f.name.name.clone(),
                    kind: SymbolKind::Function,
                    ty: fn_type,
                    span: f.span,
                    mutable: false,
                }).is_err() {
                    self.errors.push(CheckError::DuplicateDefinition {
                        name: f.name.name.clone(),
                        line: f.span.line,
                        column: f.span.column,
                    });
                }
            }

            _ => {}
        }
    }

    /// Second pass: type check top-level items
    fn check_top_level(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function(f) => self.check_function(f),
            TopLevel::Struct(s) => self.check_struct(s),
            TopLevel::Comptime(c) => self.check_comptime(&c.block),
            _ => {} // Already handled in first pass
        }
    }

    fn check_function(&mut self, f: &FnDecl) {
        self.symbols.enter_scope();

        // Add parameters to scope
        for param in &f.params {
            let ty = ast_type_to_ty(&param.ty);
            if self.symbols.define(Symbol {
                name: param.name.name.clone(),
                kind: SymbolKind::Parameter,
                ty,
                span: param.span,
                mutable: false,
            }).is_err() {
                self.errors.push(CheckError::DuplicateDefinition {
                    name: param.name.name.clone(),
                    line: param.span.line,
                    column: param.span.column,
                });
            }
        }

        // Set return type context
        self.current_return_type = f.return_type.as_ref().map(ast_type_to_ty);

        // Check function body
        self.check_block(&f.body);

        self.current_return_type = None;
        self.symbols.exit_scope();
    }

    fn check_struct(&mut self, s: &StructDecl) {
        // Check that field types are valid
        for field in &s.fields {
            self.check_type_exists(&field.ty);
        }
    }

    fn check_comptime(&mut self, block: &Block) {
        self.symbols.enter_scope();
        self.check_block(block);
        self.symbols.exit_scope();
    }

    fn check_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }

            Stmt::Let { mutable, name, ty, value, span } => {
                let value_ty = self.check_expr(value);

                let declared_ty = ty.as_ref().map(ast_type_to_ty);

                let final_ty = if let Some(decl) = &declared_ty {
                    if !decl.is_assignable_from(&value_ty) && !value_ty.is_error_or_unknown() {
                        self.errors.push(CheckError::TypeMismatch {
                            expected: decl.to_string(),
                            found: value_ty.to_string(),
                            line: span.line,
                            column: span.column,
                        });
                    }
                    decl.clone()
                } else {
                    value_ty
                };

                if self.symbols.define(Symbol {
                    name: name.name.clone(),
                    kind: SymbolKind::Variable,
                    ty: final_ty,
                    span: *span,
                    mutable: *mutable,
                }).is_err() {
                    self.errors.push(CheckError::DuplicateDefinition {
                        name: name.name.clone(),
                        line: span.line,
                        column: span.column,
                    });
                }
            }

            Stmt::If { condition, then_block, else_block, span } => {
                let cond_ty = self.check_expr(condition);
                if cond_ty != Ty::Bool && !cond_ty.is_error_or_unknown() {
                    self.errors.push(CheckError::NonBoolCondition {
                        found: cond_ty.to_string(),
                        line: span.line,
                        column: span.column,
                    });
                }

                self.symbols.enter_scope();
                self.check_block(then_block);
                self.symbols.exit_scope();

                if let Some(else_blk) = else_block {
                    self.symbols.enter_scope();
                    self.check_block(else_blk);
                    self.symbols.exit_scope();
                }
            }

            Stmt::Go { block, .. } => {
                self.symbols.enter_scope();
                self.check_block(block);
                self.symbols.exit_scope();
            }

            Stmt::Return { value, span } => {
                let return_ty = value
                    .as_ref()
                    .map(|v| self.check_expr(v))
                    .unwrap_or(Ty::Unit);

                if let Some(expected) = &self.current_return_type {
                    if !expected.is_assignable_from(&return_ty) && !return_ty.is_error_or_unknown() {
                        self.errors.push(CheckError::TypeMismatch {
                            expected: expected.to_string(),
                            found: return_ty.to_string(),
                            line: span.line,
                            column: span.column,
                        });
                    }
                }
            }

            Stmt::Await { value, .. } => {
                self.check_expr(value);
            }

            Stmt::Try { value, .. } => {
                self.check_expr(value);
            }

            Stmt::Comptime { block, .. } => {
                self.symbols.enter_scope();
                self.check_block(block);
                self.symbols.exit_scope();
            }

            Stmt::Ai(ai_stmt) => {
                self.check_ai_stmt(ai_stmt);
            }

            Stmt::Belief { name, ty, confidence, span } => {
                // Type check belief statement: belief name: Type where confidence(0.85);
                let belief_ty = ast_type_to_ty(ty);

                // Check confidence is between 0 and 1
                if *confidence < 0.0 || *confidence > 1.0 {
                    self.errors.push(CheckError::TypeMismatch {
                        expected: "confidence between 0.0 and 1.0".to_string(),
                        found: format!("{}", confidence),
                        line: span.line,
                        column: span.column,
                    });
                }

                // Define the belief variable with its type
                let _ = self.symbols.define(Symbol {
                    name: name.name.clone(),
                    kind: SymbolKind::Variable,
                    ty: belief_ty,
                    span: *span,
                    mutable: false,
                });
            }
        }
    }

    /// Type-check an expression, consulting the content-addressed result
    /// cache for environment-independent subexpressions first
    /// (hyperpolymath/my-lang#16).
    ///
    /// A cache hit returns immediately *without recursing*, which is the
    /// whole point: structurally-identical `str_concat`/`format` templating
    /// trees that recur across functions are checked once. A cache hit also
    /// cannot trip the depth guard or allocate, so a repeated deep-but-legal
    /// pure chain no longer pays its full recursive cost on every occurrence.
    /// We only *insert* a result when the underlying check produced no new
    /// diagnostics and a concrete type, so memoisation never suppresses or
    /// reorders an error relative to the un-cached behaviour.
    fn check_expr(&mut self, expr: &Expr) -> Ty {
        let Some(key) = self.memo_key(expr) else {
            return self.check_expr_guarded(expr);
        };
        if let Some(ty) = self.expr_cache.get(&key) {
            return ty.clone();
        }
        let errors_before = self.errors.len();
        let ty = self.check_expr_guarded(expr);
        if self.errors.len() == errors_before && !ty.is_error_or_unknown() {
            self.expr_cache.insert(key, ty.clone());
        }
        ty
    }

    /// Span-stripped structural hash for the subset of expressions whose type
    /// is a pure function of their form, independent of the symbol
    /// environment and free of scope side effects. Returns `None` for
    /// anything outside that subset (in particular: every expression that
    /// resolves an identifier against local scope, and every scope-
    /// introducing form — lambdas, blocks, matches, records), so those are
    /// always re-checked normally and the cache stays sound.
    ///
    /// The subset:
    /// - literals (type fixed by the literal kind), and
    /// - calls whose callee resolves to the *genuine* stdlib binding (so the
    ///   result type is the fixed stdlib signature regardless of scope, and a
    ///   local/user shadow is correctly excluded) and whose arguments are
    ///   themselves in the subset.
    ///
    /// Spans are never fed to the hasher, so structurally-equal expressions
    /// at different source locations share a key. The key is a 64-bit hash;
    /// a collision (≈2⁻⁶⁴) could return a wrong cached type, which is the
    /// standard, accepted trade-off for content-addressed memoisation and is
    /// acceptable here because this is an optional optimisation layer over an
    /// already-linear checker (see #14).
    fn memo_key(&self, expr: &Expr) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash_pure(expr, &mut hasher)?;
        Some(hasher.finish())
    }

    fn hash_pure(&self, expr: &Expr, hasher: &mut DefaultHasher) -> Option<()> {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::Int(v, _) => {
                        0u8.hash(hasher);
                        v.hash(hasher);
                    }
                    Literal::Float(v, _) => {
                        1u8.hash(hasher);
                        v.to_bits().hash(hasher);
                    }
                    Literal::String(s, _) => {
                        2u8.hash(hasher);
                        s.hash(hasher);
                    }
                    Literal::Bool(b, _) => {
                        3u8.hash(hasher);
                        b.hash(hasher);
                    }
                }
                Some(())
            }
            Expr::Call { callee, args, .. } => {
                let Expr::Ident(id) = callee.as_ref() else {
                    return None;
                };
                // Only cacheable if the callee resolves to the real stdlib
                // function symbol. Comparing against the canonical stdlib
                // signature rejects a local `let str_concat = ...` or a
                // user-defined function that shadows the name, so the name
                // alone is a sound proxy for the resolved (scope-invariant)
                // result type.
                let sig = Self::stdlib_function_type(&id.name);
                if !matches!(sig, Ty::Function { .. }) {
                    return None;
                }
                match self.symbols.lookup(&id.name) {
                    Some(sym) if sym.ty == sig => {}
                    _ => return None,
                }
                4u8.hash(hasher);
                id.name.hash(hasher);
                (args.len() as u64).hash(hasher);
                for arg in args {
                    self.hash_pure(arg, hasher)?;
                }
                Some(())
            }
            _ => None,
        }
    }

    fn check_expr_guarded(&mut self, expr: &Expr) -> Ty {
        // Depth guard: stops pathological inputs (e.g. deeply chained
        // str_concat/format) from driving the checker into a stack overflow
        // via unbounded recursion (allocation is linear — see #14).
        // Reports a single ExpressionTooDeep
        // error for the whole offending subtree and returns Ty::Error so the
        // surrounding code keeps type-checking in error-recovery mode.
        if self.expr_depth >= MAX_EXPR_DEPTH {
            if !self.too_deep_reported {
                let span = expr_span(expr);
                self.errors.push(CheckError::ExpressionTooDeep {
                    limit: MAX_EXPR_DEPTH,
                    line: span.line,
                    column: span.column,
                });
                self.too_deep_reported = true;
            }
            return Ty::Error;
        }

        self.expr_depth += 1;
        let ty = self.check_expr_inner(expr);
        self.expr_depth -= 1;
        ty
    }

    fn check_expr_inner(&mut self, expr: &Expr) -> Ty {
        match expr {
            Expr::Literal(lit) => self.check_literal(lit),

            Expr::Ident(ident) => {
                if let Some(symbol) = self.symbols.lookup(&ident.name) {
                    symbol.ty.clone()
                } else {
                    self.errors.push(CheckError::UndefinedVariable {
                        name: ident.name.clone(),
                        line: ident.span.line,
                        column: ident.span.column,
                    });
                    Ty::Error
                }
            }

            Expr::Call { callee, args, span } => {
                let callee_ty = self.check_expr(callee);
                let arg_types: Vec<Ty> = args.iter().map(|a| self.check_expr(a)).collect();

                match callee_ty {
                    Ty::Function { params, result } => {
                        if params.len() != arg_types.len() {
                            self.errors.push(CheckError::WrongArgCount {
                                expected: params.len(),
                                found: arg_types.len(),
                                line: span.line,
                                column: span.column,
                            });
                        } else {
                            for (param, arg) in params.iter().zip(arg_types.iter()) {
                                if !param.is_assignable_from(arg) && !arg.is_error_or_unknown() {
                                    self.errors.push(CheckError::TypeMismatch {
                                        expected: param.to_string(),
                                        found: arg.to_string(),
                                        line: span.line,
                                        column: span.column,
                                    });
                                }
                            }
                        }
                        *result
                    }
                    Ty::Error | Ty::Unknown => Ty::Error,
                    _ => {
                        self.errors.push(CheckError::Other {
                            message: format!("Cannot call non-function type '{}'", callee_ty),
                            line: span.line,
                            column: span.column,
                        });
                        Ty::Error
                    }
                }
            }

            Expr::Field { object, field, span } => {
                let obj_ty = self.check_expr(object);

                match &obj_ty {
                    Ty::Named(name) => {
                        if let Some(struct_def) = self.types.get_struct(name) {
                            if let Some((_, field_ty)) = struct_def.fields.iter()
                                .find(|(n, _)| n == &field.name)
                            {
                                field_ty.clone()
                            } else {
                                self.errors.push(CheckError::Other {
                                    message: format!("No field '{}' on type '{}'", field.name, name),
                                    line: span.line,
                                    column: span.column,
                                });
                                Ty::Error
                            }
                        } else {
                            Ty::Unknown
                        }
                    }
                    Ty::Record(fields) => {
                        if let Some((_, field_ty)) = fields.iter()
                            .find(|(n, _)| n == &field.name)
                        {
                            field_ty.clone()
                        } else {
                            self.errors.push(CheckError::Other {
                                message: format!("No field '{}' in record", field.name),
                                line: span.line,
                                column: span.column,
                            });
                            Ty::Error
                        }
                    }
                    Ty::Error | Ty::Unknown => Ty::Error,
                    _ => {
                        self.errors.push(CheckError::Other {
                            message: format!("Cannot access field on type '{}'", obj_ty),
                            line: span.line,
                            column: span.column,
                        });
                        Ty::Error
                    }
                }
            }

            Expr::Binary { left, op, right, span } => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);

                // Enforce immutability of assignment targets: assigning to a
                // binding declared without `mut` is a static error. `Symbol.mutable`
                // is recorded at `let` time but was never consulted here, so
                // `CheckError::ImmutableAssignment` had no construction site.
                // NOTE: currently latent — the parser does not yet emit
                // `BinaryOp::Assign` (KNOWN_PARSE_GAP, conformance/valid/v04_let.my);
                // this gate activates once assignment parsing lands, and is
                // reachable today only via a programmatically-built AST.
                if matches!(op, BinaryOp::Assign) {
                    if let Expr::Ident(ident) = left.as_ref() {
                        // Copy out of the immutable lookup borrow before touching
                        // self.errors. Unknown names are already reported by
                        // check_expr(left), so default to "not immutable".
                        let is_immutable = self
                            .symbols
                            .lookup(&ident.name)
                            .map(|s| !s.mutable)
                            .unwrap_or(false);
                        if is_immutable {
                            self.errors.push(CheckError::ImmutableAssignment {
                                name: ident.name.clone(),
                                line: ident.span.line,
                                column: ident.span.column,
                            });
                        }
                    }
                }

                self.check_binary_op(*op, &left_ty, &right_ty, *span)
            }

            Expr::Unary { op, operand, span } => {
                let operand_ty = self.check_expr(operand);
                self.check_unary_op(*op, &operand_ty, *span)
            }

            Expr::Try { operand, .. } => {
                self.check_expr(operand)
            }

            Expr::Block(block) => {
                self.symbols.enter_scope();
                self.check_block(block);
                self.symbols.exit_scope();
                Ty::Unit
            }

            Expr::Restrict { operand, .. } => {
                self.check_expr(operand)
            }

            Expr::Ai(ai_expr) => {
                self.check_ai_expr(ai_expr)
            }

            Expr::Lambda { params, body, span: _ } => {
                self.symbols.enter_scope();

                let param_types: Vec<Ty> = params.iter().map(|p| {
                    let ty = ast_type_to_ty(&p.ty);
                    let _ = self.symbols.define(Symbol {
                        name: p.name.name.clone(),
                        kind: SymbolKind::Parameter,
                        ty: ty.clone(),
                        span: p.span,
                        mutable: false,
                    });
                    ty
                }).collect();

                let result_ty = match body {
                    LambdaBody::Expr(e) => self.check_expr(e),
                    LambdaBody::Block(b) => {
                        self.check_block(b);
                        Ty::Unit
                    }
                };

                self.symbols.exit_scope();

                Ty::Function {
                    params: param_types,
                    result: Box::new(result_ty),
                }
            }

            Expr::Match { scrutinee, arms, span: _ } => {
                let scrutinee_ty = self.check_expr(scrutinee);

                let mut result_ty: Option<Ty> = None;

                for arm in arms {
                    self.symbols.enter_scope();

                    // Check pattern and introduce bindings
                    self.check_pattern(&arm.pattern, &scrutinee_ty);

                    let arm_ty = self.check_expr(&arm.body);

                    if let Some(ref expected) = result_ty {
                        if !expected.is_assignable_from(&arm_ty) && !arm_ty.is_error_or_unknown() {
                            self.errors.push(CheckError::TypeMismatch {
                                expected: expected.to_string(),
                                found: arm_ty.to_string(),
                                line: arm.span.line,
                                column: arm.span.column,
                            });
                        }
                    } else {
                        result_ty = Some(arm_ty);
                    }

                    self.symbols.exit_scope();
                }

                result_ty.unwrap_or(Ty::Unit)
            }

            Expr::Array { elements, span } => {
                if elements.is_empty() {
                    Ty::Array(Box::new(Ty::Unknown))
                } else {
                    let first_ty = self.check_expr(&elements[0]);
                    for elem in elements.iter().skip(1) {
                        let elem_ty = self.check_expr(elem);
                        if !first_ty.is_assignable_from(&elem_ty) && !elem_ty.is_error_or_unknown() {
                            self.errors.push(CheckError::TypeMismatch {
                                expected: first_ty.to_string(),
                                found: elem_ty.to_string(),
                                line: span.line,
                                column: span.column,
                            });
                        }
                    }
                    Ty::Array(Box::new(first_ty))
                }
            }

            Expr::Record { fields, span: _ } => {
                let field_types: Vec<(String, Ty)> = fields
                    .iter()
                    .map(|f| (f.name.name.clone(), self.check_expr(&f.value)))
                    .collect();
                Ty::Record(field_types)
            }
        }
    }

    fn check_literal(&self, lit: &Literal) -> Ty {
        match lit {
            Literal::Int(_, _) => Ty::Int,
            Literal::Float(_, _) => Ty::Float,
            Literal::String(_, _) => Ty::String,
            Literal::Bool(_, _) => Ty::Bool,
        }
    }

    fn check_binary_op(&mut self, op: BinaryOp, left: &Ty, right: &Ty, span: Span) -> Ty {
        use BinaryOp::*;

        // Error recovery
        if left.is_error_or_unknown() || right.is_error_or_unknown() {
            return Ty::Error;
        }

        match op {
            Add | Sub | Mul | Div => {
                if left.is_numeric() && right.is_numeric() {
                    if left == right {
                        left.clone()
                    } else {
                        Ty::Float // Numeric promotion
                    }
                } else if matches!(op, Add) && left == &Ty::String && right == &Ty::String {
                    Ty::String // String concatenation
                } else {
                    self.errors.push(CheckError::InvalidBinaryOp {
                        left: left.to_string(),
                        op: format!("{:?}", op),
                        right: right.to_string(),
                        line: span.line,
                        column: span.column,
                    });
                    Ty::Error
                }
            }

            Eq | Ne => {
                if left == right || left.is_assignable_from(right) {
                    Ty::Bool
                } else {
                    self.errors.push(CheckError::InvalidBinaryOp {
                        left: left.to_string(),
                        op: format!("{:?}", op),
                        right: right.to_string(),
                        line: span.line,
                        column: span.column,
                    });
                    Ty::Error
                }
            }

            Lt | Gt | Le | Ge => {
                if left.is_numeric() && right.is_numeric() {
                    Ty::Bool
                } else {
                    self.errors.push(CheckError::InvalidBinaryOp {
                        left: left.to_string(),
                        op: format!("{:?}", op),
                        right: right.to_string(),
                        line: span.line,
                        column: span.column,
                    });
                    Ty::Error
                }
            }

            And | Or => {
                if left == &Ty::Bool && right == &Ty::Bool {
                    Ty::Bool
                } else {
                    self.errors.push(CheckError::InvalidBinaryOp {
                        left: left.to_string(),
                        op: format!("{:?}", op),
                        right: right.to_string(),
                        line: span.line,
                        column: span.column,
                    });
                    Ty::Error
                }
            }

            Assign => {
                // Assignment returns the assigned value
                if left.is_assignable_from(right) {
                    left.clone()
                } else {
                    self.errors.push(CheckError::TypeMismatch {
                        expected: left.to_string(),
                        found: right.to_string(),
                        line: span.line,
                        column: span.column,
                    });
                    Ty::Error
                }
            }
        }
    }

    fn check_unary_op(&mut self, op: UnaryOp, operand: &Ty, span: Span) -> Ty {
        use UnaryOp::*;

        if operand.is_error_or_unknown() {
            return Ty::Error;
        }

        match op {
            Neg => {
                if operand.is_numeric() {
                    operand.clone()
                } else {
                    self.errors.push(CheckError::Other {
                        message: format!("Cannot negate type '{}'", operand),
                        line: span.line,
                        column: span.column,
                    });
                    Ty::Error
                }
            }

            Not => {
                if operand == &Ty::Bool {
                    Ty::Bool
                } else {
                    self.errors.push(CheckError::Other {
                        message: format!("Cannot apply '!' to type '{}'", operand),
                        line: span.line,
                        column: span.column,
                    });
                    Ty::Error
                }
            }

            Ref => Ty::Ref {
                mutable: false,
                inner: Box::new(operand.clone()),
            },

            RefMut => Ty::Ref {
                mutable: true,
                inner: Box::new(operand.clone()),
            },
        }
    }

    fn check_pattern(&mut self, pattern: &Pattern, expected: &Ty) {
        match pattern {
            Pattern::Literal(_) => {
                // Literal patterns: check type compatibility
            }
            Pattern::Ident(ident) => {
                // Bind the identifier to the expected type
                let _ = self.symbols.define(Symbol {
                    name: ident.name.clone(),
                    kind: SymbolKind::Variable,
                    ty: expected.clone(),
                    span: ident.span,
                    mutable: false,
                });
            }
            Pattern::Wildcard(_) => {
                // Wildcard matches anything
            }
            Pattern::Constructor { name, args, span: _ } => {
                // Check constructor pattern
                // Clone the field types to avoid borrow issues
                let field_types: Vec<Ty> = self.types
                    .get_struct(&name.name)
                    .map(|s| s.fields.iter().map(|(_, ty)| ty.clone()).collect())
                    .unwrap_or_default();

                for (i, arg) in args.iter().enumerate() {
                    if let Some(field_ty) = field_types.get(i) {
                        self.check_pattern(arg, field_ty);
                    }
                }
            }
        }
    }

    fn check_ai_stmt(&mut self, stmt: &AiStmt) {
        match &stmt.body {
            AiStmtBody::Block(block) => {
                self.symbols.enter_scope();
                self.check_block(block);
                self.symbols.exit_scope();
            }
            AiStmtBody::Expr(expr) => {
                self.check_expr(expr);
            }
        }
    }

    fn check_ai_expr(&mut self, expr: &AiExpr) -> Ty {
        match expr {
            AiExpr::Block { keyword, body, span: _ } => {
                // Check AI body items
                for item in body {
                    match item {
                        AiBodyItem::Field { name, value } => {
                            // Check for model reference
                            if name.name == "model" {
                                if let Expr::Ident(ident) = value {
                                    if self.types.get_ai_model(&ident.name).is_none() {
                                        self.errors.push(CheckError::UndefinedAiModel {
                                            name: ident.name.clone(),
                                            line: ident.span.line,
                                            column: ident.span.column,
                                        });
                                    }
                                }
                            } else {
                                self.check_expr(value);
                            }
                        }
                        AiBodyItem::Literal(_) => {}
                    }
                }

                // AI expressions return AI<T> where T depends on the keyword
                match keyword {
                    AiKeyword::Query | AiKeyword::Generate => Ty::AI(Box::new(Ty::String)),
                    AiKeyword::Verify | AiKeyword::Validate => Ty::AI(Box::new(Ty::Bool)),
                    AiKeyword::Embed => Ty::AI(Box::new(Ty::Array(Box::new(Ty::Float)))),
                    AiKeyword::Classify => Ty::AI(Box::new(Ty::String)),
                    _ => Ty::AI(Box::new(Ty::Unknown)),
                }
            }

            AiExpr::Call { keyword, args, span: _ } => {
                for arg in args {
                    self.check_expr(arg);
                }

                match keyword {
                    AiKeyword::Query | AiKeyword::Generate => Ty::AI(Box::new(Ty::String)),
                    AiKeyword::Verify | AiKeyword::Validate => Ty::AI(Box::new(Ty::Bool)),
                    AiKeyword::Embed => Ty::AI(Box::new(Ty::Array(Box::new(Ty::Float)))),
                    AiKeyword::Classify => Ty::AI(Box::new(Ty::String)),
                    _ => Ty::AI(Box::new(Ty::Unknown)),
                }
            }

            AiExpr::Quick { query: _, span: _ } => {
                Ty::AI(Box::new(Ty::String))
            }

            AiExpr::PromptInvocation { name, args, span: _ } => {
                // Check that the prompt exists
                if self.types.get_prompt(&name.name).is_none() {
                    self.errors.push(CheckError::UndefinedPrompt {
                        name: name.name.clone(),
                        line: name.span.line,
                        column: name.span.column,
                    });
                }

                for arg in args {
                    self.check_expr(arg);
                }

                Ty::AI(Box::new(Ty::String))
            }
        }
    }

    fn check_type_exists(&mut self, ty: &Type) {
        match ty {
            Type::Named(ident) => {
                if !self.symbols.is_defined(&ident.name)
                    && self.types.get_struct(&ident.name).is_none()
                    && self.types.get_effect(&ident.name).is_none()
                {
                    self.errors.push(CheckError::UndefinedType {
                        name: ident.name.clone(),
                        line: ident.span.line,
                        column: ident.span.column,
                    });
                }
            }
            Type::Array { element, .. } => self.check_type_exists(element),
            Type::Reference { inner, .. } => self.check_type_exists(inner),
            Type::Ai { inner, .. } => self.check_type_exists(inner),
            Type::Effect { inner, .. } => self.check_type_exists(inner),
            Type::Function { param, result, .. } => {
                self.check_type_exists(param);
                self.check_type_exists(result);
            }
            Type::Tuple { elements, .. } => {
                for elem in elements {
                    self.check_type_exists(elem);
                }
            }
            Type::Record { fields, .. } => {
                for field in fields {
                    self.check_type_exists(&field.ty);
                }
            }
            Type::Constrained { base, .. } => {
                self.check_type_exists(base);
            }
            Type::Primitive(_) => {}
        }
    }
}

/// Best-effort span lookup for any [`Expr`] variant. Used by error reporting
/// when we don't have the surrounding context's span available (e.g. from the
/// expression-depth guard).
fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Literal(lit) => lit.span(),
        Expr::Ident(i) => i.span,
        Expr::Call { span, .. }
        | Expr::Field { span, .. }
        | Expr::Binary { span, .. }
        | Expr::Unary { span, .. }
        | Expr::Try { span, .. }
        | Expr::Restrict { span, .. }
        | Expr::Lambda { span, .. }
        | Expr::Match { span, .. }
        | Expr::Array { span, .. }
        | Expr::Record { span, .. } => *span,
        Expr::Block(b) => b.span,
        Expr::Ai(ai) => match ai {
            AiExpr::Block { span, .. }
            | AiExpr::Call { span, .. }
            | AiExpr::Quick { span, .. }
            | AiExpr::PromptInvocation { span, .. } => *span,
        },
    }
}

/// Public function to check a program
pub fn check(program: &Program) -> Result<(), Vec<CheckError>> {
    let mut checker = Checker::new();
    checker.check_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn check_source(source: &str) -> Result<(), Vec<CheckError>> {
        let program = parse(source).expect("Parse failed");
        check(&program)
    }

    // The former shape-specific `drop_program_iteratively` test helper (it
    // only flattened 2-arg `Call` chains) is replaced by the general,
    // shape-independent `crate::ast::drop_program_iteratively`, already in
    // scope here via `super::*` (hyperpolymath/my-lang#37).

    #[test]
    fn test_basic_function() {
        let result = check_source("fn main() { let x: Int = 42; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_variable() {
        let result = check_source("fn main() { let x: Int = y; }");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, CheckError::UndefinedVariable { .. })));
    }

    #[test]
    fn test_type_mismatch() {
        let result = check_source(r#"fn main() { let x: Int = "hello"; }"#);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn test_ai_model_defined() {
        let result = check_source(r#"
            ai_model gpt4 {
                provider: "openai"
                model: "gpt-4"
            }
            fn main() {
                let x = ai query {
                    model: gpt4
                    prompt: "test"
                };
            }
        "#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_ai_model() {
        let result = check_source(r#"
            fn main() {
                let x = ai query {
                    model: undefined_model
                    prompt: "test"
                };
            }
        "#);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, CheckError::UndefinedAiModel { .. })));
    }

    #[test]
    fn test_prompt_defined() {
        let result = check_source(r#"
            prompt greeting { "Hello {name}" }
            fn main() {
                let x = greeting!("World");
            }
        "#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_call_type_check() {
        let result = check_source(r#"
            fn add(a: Int, b: Int) -> Int {
                return a + b;
            }
            fn main() {
                let x: Int = add(1, 2);
            }
        "#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wrong_arg_count() {
        let result = check_source(r#"
            fn add(a: Int, b: Int) -> Int {
                return a + b;
            }
            fn main() {
                let x = add(1);
            }
        "#);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, CheckError::WrongArgCount { .. })));
    }

    #[test]
    fn test_deeply_nested_str_concat_reports_too_deep_instead_of_oom() {
        // Regression for hyperpolymath/my-lang#1: deeply nested str_concat /
        // format chains used to drive the type checker into runaway memory
        // allocation. Now we should get a clean ExpressionTooDeep error.
        //
        // We build the AST programmatically rather than from source: the
        // recursive-descent parser would itself overflow the stack on inputs
        // this deep before the checker ever ran.
        use crate::token::Span;

        let span = Span::default();
        let leaf = Expr::Literal(Literal::String("end".to_string(), span));
        let mut expr = leaf;
        for _ in 0..(MAX_EXPR_DEPTH + 16) {
            expr = Expr::Call {
                callee: Box::new(Expr::Ident(Ident::new("str_concat", span))),
                args: vec![Expr::Literal(Literal::String("a".to_string(), span)), expr],
                span,
            };
        }

        let program = Program {
            items: vec![TopLevel::Function(FnDecl {
                modifiers: vec![],
                name: Ident::new("main", span),
                params: vec![],
                return_type: None,
                contract: None,
                body: Block {
                    stmts: vec![Stmt::Let {
                        mutable: false,
                        name: Ident::new("s", span),
                        ty: None,
                        value: expr,
                        span,
                    }],
                    span,
                },
                span,
            })],
        };

        let errors = check(&program).expect_err("expected ExpressionTooDeep error");
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, CheckError::ExpressionTooDeep { .. })),
            "expected ExpressionTooDeep, got: {:?}",
            errors
        );
        // And only one — we don't spam the user with N copies of the same
        // diagnostic on the way back out of the recursion.
        let count = errors
            .iter()
            .filter(|e| matches!(e, CheckError::ExpressionTooDeep { .. }))
            .count();
        assert_eq!(count, 1, "expected exactly one ExpressionTooDeep error");

        // Drop the deep AST iteratively: the auto-generated recursive Drop on
        // the chain of Box<Expr> would itself overflow the test thread's
        // stack. (That's a separate, drop-side instance of the same nesting
        // problem; not what this test is about.)
        drop_program_iteratively(program);
    }

    #[test]
    fn test_deep_non_call_ast_teardown_does_not_overflow() {
        // Regression for hyperpolymath/my-lang#37, subtlety 1: the recursive
        // `Drop` overflow is *shape-independent*. The former test helper only
        // flattened 2-arg `Call` chains; a differently-shaped deep AST (here a
        // `Unary::Not` chain, with no `Call` anywhere) would still overflow.
        // The general `ast::drop_program_iteratively` must tear it down with
        // O(1) stack regardless of shape.
        //
        // Depth is far beyond the measured recursive-Drop cliff (≈4–6k at a
        // 512 KiB stack) so a recursion-based teardown would abort the test
        // process; survival proves the teardown is non-recursive.
        use crate::token::Span;

        let span = Span::default();
        let mut expr = Expr::Literal(Literal::Bool(true, span));
        for _ in 0..50_000 {
            expr = Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(expr),
                span,
            };
        }

        let program = Program {
            items: vec![TopLevel::Function(FnDecl {
                modifiers: vec![],
                name: Ident::new("main", span),
                params: vec![],
                return_type: None,
                contract: None,
                body: Block {
                    stmts: vec![Stmt::Let {
                        mutable: false,
                        name: Ident::new("s", span),
                        ty: None,
                        value: expr,
                        span,
                    }],
                    span,
                },
                span,
            })],
        };

        // The sole assertion is that this returns at all: a recursive teardown
        // would `STATUS_STACK_OVERFLOW` / SIGABRT the test runner first.
        drop_program_iteratively(program);
    }

    #[test]
    fn test_moderately_nested_str_concat_still_checks() {
        // Sanity: well below the limit, deeply chained str_concat must still
        // type-check successfully via the normal source path.
        let depth = 32;
        let mut src = String::from("fn main() { let s = ");
        for _ in 0..depth {
            src.push_str("str_concat(\"a\", ");
        }
        src.push_str("\"end\"");
        for _ in 0..depth {
            src.push(')');
        }
        src.push_str("; }");

        assert!(check_source(&src).is_ok());
    }

    #[test]
    fn test_memoised_pure_chain_is_cached() {
        // #16: a structurally-identical pure str_concat chain repeated across
        // many functions must be checked once and reused. We can observe the
        // cache directly via the Checker API.
        let src = r#"
            fn a() { let x = str_concat("p", str_concat("q", "r")); }
            fn b() { let y = str_concat("p", str_concat("q", "r")); }
            fn c() { let z = str_concat("p", str_concat("q", "r")); }
        "#;
        let program = parse(src).expect("parse");
        let mut checker = Checker::new();
        checker.check_program(&program).expect("should type-check");
        // The repeated `str_concat(...)` tree (and its inner subtree) collapse
        // to a small number of distinct keys, not one per occurrence.
        assert!(
            !checker.expr_cache.is_empty(),
            "expected pure str_concat chain to be memoised"
        );
        // Three identical chains => the distinct-key count stays at the small
        // fixed set of subexpressions ("p", "q", "r", inner + outer call),
        // not 3x that. The exact figure is an implementation detail; the
        // invariant is that it does not grow with the number of occurrences.
        assert!(
            checker.expr_cache.len() <= 6,
            "identical chains should share keys regardless of occurrence count, got {} entries",
            checker.expr_cache.len()
        );
    }

    #[test]
    fn test_memoisation_does_not_change_results() {
        // Correctness: every existing positive/negative case must behave
        // identically with the cache in place (the cache is consulted on
        // every expression via check_expr).
        assert!(check_source(r#"fn main() { let s = str_concat("a", "b"); }"#).is_ok());
        assert!(check_source(
            r#"fn main() { let s = format(str_concat("<", str_concat("x", ">"))); }"#
        )
        .is_ok());
        // A pure stdlib call with a genuine type error must still report it
        // (never cached, since the result is an error type).
        let err = check_source(r#"fn main() { let s = str_concat("a"); }"#).unwrap_err();
        assert!(err.iter().any(|e| matches!(e, CheckError::WrongArgCount { .. })));
    }

    #[test]
    fn test_memoisation_is_scope_sound() {
        // The soundness hazard #16 calls out: two structurally-identical
        // expressions can have different types because identifiers resolve
        // against scope. `add(p, 1)` is `Int` in one function and `Float` in
        // the other. Because identifier arguments are outside the cacheable
        // subset, each call site is checked against its own scope and the
        // return-type checks must both pass.
        let src = r#"
            fn add(a: Int, b: Int) -> Int { return a + b; }
            fn addf(a: Float, b: Float) -> Float { return a + b; }
            fn use_int(p: Int) -> Int { return add(p, p); }
            fn use_float(p: Float) -> Float { return addf(p, p); }
        "#;
        assert!(
            check_source(src).is_ok(),
            "scope-sensitive look-alikes must each be checked in their own scope"
        );

        // And the negative: a genuine mismatch is still caught, not masked by
        // a cache entry from the structurally-similar good call.
        let bad = r#"
            fn add(a: Int, b: Int) -> Int { return a + b; }
            fn ok() -> Int { return add(1, 2); }
            fn bad() -> Int { return add(1, "two"); }
        "#;
        let err = check_source(bad).unwrap_err();
        assert!(err.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn test_non_bool_condition() {
        let result = check_source(r#"
            fn main() {
                if 42 {
                    let x = 1;
                }
            }
        "#);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, CheckError::NonBoolCondition { .. })));
    }
}
