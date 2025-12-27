//! Type checker and semantic analyzer for My Language
//!
//! Performs name resolution, type checking, and validation of AI constructs.

use crate::ast::*;
use crate::scope::*;
use crate::token::Span;
use crate::types::*;
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
}

pub type CheckResult<T> = Result<T, CheckError>;

/// The type checker and semantic analyzer
pub struct Checker {
    symbols: SymbolTable,
    types: TypeEnv,
    errors: Vec<CheckError>,
    /// Current function's return type (for checking return statements)
    current_return_type: Option<Ty>,
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

                if let Err(_) = self.types.define_effect(def) {
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

                if let Err(_) = self.types.define_ai_model(def) {
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

                if let Err(_) = self.types.define_prompt(def) {
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

                if let Err(_) = self.symbols.define(Symbol {
                    name: f.name.name.clone(),
                    kind: SymbolKind::Function,
                    ty: fn_type,
                    span: f.span,
                    mutable: false,
                }) {
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
            if let Err(_) = self.symbols.define(Symbol {
                name: param.name.name.clone(),
                kind: SymbolKind::Parameter,
                ty,
                span: param.span,
                mutable: false,
            }) {
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

                if let Err(_) = self.symbols.define(Symbol {
                    name: name.name.clone(),
                    kind: SymbolKind::Variable,
                    ty: final_ty,
                    span: *span,
                    mutable: *mutable,
                }) {
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
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Ty {
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
                            for (_i, (param, arg)) in params.iter().zip(arg_types.iter()).enumerate() {
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
