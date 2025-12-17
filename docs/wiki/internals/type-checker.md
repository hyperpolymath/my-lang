# Type Checker Internals

The type checker performs semantic analysis on the AST.

## Overview

Location: `src/checker.rs`, `src/types.rs`, `src/scope.rs`

The type checker is responsible for:
1. Name resolution (variables, types, functions)
2. Type checking and inference
3. Effect checking
4. AI construct validation
5. Contract validation

## Architecture

### Core Components

```rust
// src/checker.rs
pub struct Checker {
    symbols: SymbolTable,    // Variable/function bindings
    types: TypeEnv,          // Type definitions
    errors: Vec<CheckError>, // Accumulated errors
    current_function: Option<FunctionContext>,
}

// src/scope.rs
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

pub struct TypeEnv {
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, EnumDef>,
    traits: HashMap<String, TraitDef>,
    ai_models: HashMap<String, AiModelDef>,
    prompts: HashMap<String, PromptDef>,
}

// src/types.rs
pub enum Ty {
    Int, Float, String, Bool, Unit,
    Named(String),
    Function { params: Vec<Ty>, result: Box<Ty> },
    Array(Box<Ty>),
    Tuple(Vec<Ty>),
    Ref { mutable: bool, inner: Box<Ty> },
    AI(Box<Ty>),
    Effect(Box<Ty>),
    Var(usize),  // Type variable for inference
    Error,
    Unknown,
}
```

## Two-Pass Analysis

### Pass 1: Collection

Gather all top-level definitions:

```rust
impl Checker {
    fn collect_definitions(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Struct(s) => self.collect_struct(s),
                TopLevel::Enum(e) => self.collect_enum(e),
                TopLevel::Function(f) => self.collect_function(f),
                TopLevel::Trait(t) => self.collect_trait(t),
                TopLevel::AiModel(m) => self.collect_ai_model(m),
                TopLevel::Prompt(p) => self.collect_prompt(p),
                _ => {}
            }
        }
    }

    fn collect_struct(&mut self, s: &StructDecl) {
        let fields: Vec<(String, Ty)> = s.fields
            .iter()
            .map(|f| (f.name.name.clone(), self.resolve_type(&f.ty)))
            .collect();

        self.types.define_struct(s.name.name.clone(), StructDef {
            name: s.name.name.clone(),
            fields,
            generics: s.generics.clone(),
        });
    }

    fn collect_function(&mut self, f: &FnDecl) {
        let param_types: Vec<Ty> = f.params
            .iter()
            .map(|p| self.resolve_type(&p.ty))
            .collect();

        let return_type = f.return_ty
            .as_ref()
            .map(|t| self.resolve_type(t))
            .unwrap_or(Ty::Unit);

        let fn_type = Ty::Function {
            params: param_types,
            result: Box::new(return_type),
        };

        self.symbols.define(f.name.name.clone(), Symbol {
            name: f.name.name.clone(),
            ty: fn_type,
            kind: SymbolKind::Function,
            mutable: false,
        });
    }
}
```

### Pass 2: Type Checking

Check all expressions and statements:

```rust
impl Checker {
    fn check_program(&mut self, program: &Program) {
        for item in &program.items {
            self.check_item(item);
        }
    }

    fn check_function(&mut self, f: &FnDecl) {
        self.symbols.enter_scope();

        // Add parameters to scope
        for param in &f.params {
            let ty = self.resolve_type(&param.ty);
            self.symbols.define(param.name.name.clone(), Symbol {
                name: param.name.name.clone(),
                ty,
                kind: SymbolKind::Variable,
                mutable: param.mutable,
            });
        }

        // Set current function context
        let return_ty = f.return_ty
            .as_ref()
            .map(|t| self.resolve_type(t))
            .unwrap_or(Ty::Unit);

        self.current_function = Some(FunctionContext {
            return_type: return_ty.clone(),
            effects: f.effects.clone(),
        });

        // Check function body
        let body_ty = self.check_block(&f.body);

        // Verify return type matches
        if !self.types_compatible(&body_ty, &return_ty) {
            self.errors.push(CheckError::TypeMismatch {
                expected: return_ty,
                found: body_ty,
                line: f.span.line,
                column: f.span.column,
            });
        }

        self.current_function = None;
        self.symbols.exit_scope();
    }
}
```

## Expression Type Checking

```rust
impl Checker {
    fn check_expr(&mut self, expr: &Expr) -> Ty {
        match expr {
            Expr::Literal { value, .. } => self.check_literal(value),

            Expr::Ident { name } => self.check_ident(name),

            Expr::Binary { lhs, op, rhs, span } => {
                let lhs_ty = self.check_expr(lhs);
                let rhs_ty = self.check_expr(rhs);
                self.check_binary_op(op, lhs_ty, rhs_ty, span)
            }

            Expr::Unary { op, expr, span } => {
                let expr_ty = self.check_expr(expr);
                self.check_unary_op(op, expr_ty, span)
            }

            Expr::Call { callee, args, span } => {
                let callee_ty = self.check_expr(callee);
                self.check_call(callee_ty, args, span)
            }

            Expr::If { cond, then_branch, else_branch, span } => {
                let cond_ty = self.check_expr(cond);
                if !matches!(cond_ty, Ty::Bool) {
                    self.errors.push(CheckError::NonBoolCondition {
                        found: cond_ty,
                        line: span.line,
                        column: span.column,
                    });
                }

                let then_ty = self.check_block(then_branch);

                if let Some(else_branch) = else_branch {
                    let else_ty = self.check_block(else_branch);
                    if !self.types_compatible(&then_ty, &else_ty) {
                        self.errors.push(CheckError::BranchTypeMismatch {
                            then_type: then_ty.clone(),
                            else_type: else_ty,
                            line: span.line,
                            column: span.column,
                        });
                    }
                    then_ty
                } else {
                    Ty::Unit
                }
            }

            Expr::Match { expr, arms, span: _ } => {
                let scrutinee_ty = self.check_expr(expr);
                self.check_match(scrutinee_ty, arms)
            }

            Expr::AI { expr, span: _ } => {
                self.check_ai_expr(expr)
            }

            Expr::Lambda { params, body, span: _ } => {
                self.check_lambda(params, body)
            }

            // ... more expressions
            _ => Ty::Unknown,
        }
    }
}
```

## AI Expression Checking

```rust
impl Checker {
    fn check_ai_expr(&mut self, expr: &AiExpr) -> Ty {
        match expr {
            AiExpr::Quick { query, span: _ } => {
                self.check_expr(query);
                Ty::AI(Box::new(Ty::String))
            }

            AiExpr::Block { keyword, fields, span: _ } => {
                for (name, value) in fields {
                    self.check_expr(value);
                }

                match keyword {
                    AiKeyword::Query | AiKeyword::Generate => {
                        Ty::AI(Box::new(Ty::String))
                    }
                    AiKeyword::Verify | AiKeyword::Validate => {
                        Ty::AI(Box::new(Ty::Bool))
                    }
                    AiKeyword::Embed => {
                        Ty::AI(Box::new(Ty::Array(Box::new(Ty::Float))))
                    }
                    AiKeyword::Classify => {
                        Ty::AI(Box::new(Ty::String))
                    }
                    _ => Ty::AI(Box::new(Ty::Unknown)),
                }
            }

            AiExpr::Call { keyword, args, span: _ } => {
                for arg in args {
                    self.check_expr(arg);
                }

                match keyword {
                    AiKeyword::Query | AiKeyword::Generate => {
                        Ty::AI(Box::new(Ty::String))
                    }
                    AiKeyword::Verify | AiKeyword::Validate => {
                        Ty::AI(Box::new(Ty::Bool))
                    }
                    AiKeyword::Embed => {
                        Ty::AI(Box::new(Ty::Array(Box::new(Ty::Float))))
                    }
                    AiKeyword::Classify => {
                        Ty::AI(Box::new(Ty::String))
                    }
                    _ => Ty::AI(Box::new(Ty::Unknown)),
                }
            }

            AiExpr::PromptInvocation { name, args, span } => {
                // Check prompt exists
                if self.types.get_prompt(&name.name).is_none() {
                    self.errors.push(CheckError::UndefinedPrompt {
                        name: name.name.clone(),
                        line: span.line,
                        column: span.column,
                    });
                }

                for arg in args {
                    self.check_expr(arg);
                }

                Ty::AI(Box::new(Ty::String))
            }
        }
    }
}
```

## Symbol Table

```rust
// src/scope.rs
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

pub struct Scope {
    symbols: HashMap<String, Symbol>,
}

pub struct Symbol {
    pub name: String,
    pub ty: Ty,
    pub kind: SymbolKind,
    pub mutable: bool,
}

pub enum SymbolKind {
    Variable,
    Function,
    Type,
    Constant,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![Scope::new()], // Global scope
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, symbol: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.symbols.insert(name, symbol);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        // Only search current scope
        self.scopes.last()?.symbols.get(name)
    }
}
```

## Type Environment

```rust
pub struct TypeEnv {
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, EnumDef>,
    traits: HashMap<String, TraitDef>,
    ai_models: HashMap<String, AiModelDef>,
    prompts: HashMap<String, PromptDef>,
}

pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Ty)>,
    pub generics: Option<Generics>,
}

pub struct AiModelDef {
    pub name: String,
    pub provider: String,
    pub model: String,
    pub config: HashMap<String, Expr>,
}

pub struct PromptDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: Option<Type>,
}

impl TypeEnv {
    pub fn get_struct(&self, name: &str) -> Option<&StructDef> {
        self.structs.get(name)
    }

    pub fn get_ai_model(&self, name: &str) -> Option<&AiModelDef> {
        self.ai_models.get(name)
    }

    pub fn get_prompt(&self, name: &str) -> Option<&PromptDef> {
        self.prompts.get(name)
    }
}
```

## Pattern Checking

```rust
impl Checker {
    fn check_pattern(&mut self, pattern: &Pattern, expected_ty: &Ty) {
        match pattern {
            Pattern::Wildcard { .. } => {
                // Matches anything
            }

            Pattern::Ident { name, .. } => {
                // Bind variable with expected type
                self.symbols.define(name.name.clone(), Symbol {
                    name: name.name.clone(),
                    ty: expected_ty.clone(),
                    kind: SymbolKind::Variable,
                    mutable: false,
                });
            }

            Pattern::Literal { value, span } => {
                let lit_ty = self.check_literal(value);
                if !self.types_compatible(&lit_ty, expected_ty) {
                    self.errors.push(CheckError::PatternTypeMismatch {
                        expected: expected_ty.clone(),
                        found: lit_ty,
                        line: span.line,
                        column: span.column,
                    });
                }
            }

            Pattern::Tuple { elements, span: _ } => {
                if let Ty::Tuple(element_types) = expected_ty {
                    for (elem, ty) in elements.iter().zip(element_types) {
                        self.check_pattern(elem, ty);
                    }
                }
            }

            Pattern::Struct { name, fields, span } => {
                if let Some(struct_def) = self.types.get_struct(&name.name()) {
                    let field_types: Vec<Ty> = struct_def.fields
                        .iter()
                        .map(|(_, ty)| ty.clone())
                        .collect();

                    for (i, (_, pattern)) in fields.iter().enumerate() {
                        if let Some(ty) = field_types.get(i) {
                            self.check_pattern(pattern, ty);
                        }
                    }
                } else {
                    self.errors.push(CheckError::UndefinedType {
                        name: name.name(),
                        line: span.line,
                        column: span.column,
                    });
                }
            }

            // ... more patterns
            _ => {}
        }
    }
}
```

## Error Types

```rust
pub enum CheckError {
    UndefinedVariable {
        name: String,
        line: usize,
        column: usize,
    },
    TypeMismatch {
        expected: Ty,
        found: Ty,
        line: usize,
        column: usize,
    },
    UndefinedAiModel {
        name: String,
        line: usize,
        column: usize,
    },
    UndefinedPrompt {
        name: String,
        line: usize,
        column: usize,
    },
    WrongArgumentCount {
        expected: usize,
        found: usize,
        line: usize,
        column: usize,
    },
    NonBoolCondition {
        found: Ty,
        line: usize,
        column: usize,
    },
    MutabilityError {
        name: String,
        line: usize,
        column: usize,
    },
    // ... more error types
}

impl Display for CheckError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CheckError::UndefinedVariable { name, line, column } => {
                write!(f, "undefined variable '{}' at line {}, column {}",
                       name, line, column)
            }
            CheckError::TypeMismatch { expected, found, line, column } => {
                write!(f, "type mismatch: expected {:?}, found {:?} at line {}, column {}",
                       expected, found, line, column)
            }
            // ... more error formatting
            _ => write!(f, "type error"),
        }
    }
}
```

## Type Compatibility

```rust
impl Checker {
    fn types_compatible(&self, t1: &Ty, t2: &Ty) -> bool {
        match (t1, t2) {
            // Same types
            (Ty::Int, Ty::Int) => true,
            (Ty::Float, Ty::Float) => true,
            (Ty::String, Ty::String) => true,
            (Ty::Bool, Ty::Bool) => true,
            (Ty::Unit, Ty::Unit) => true,

            // Named types
            (Ty::Named(n1), Ty::Named(n2)) => n1 == n2,

            // Arrays
            (Ty::Array(e1), Ty::Array(e2)) => self.types_compatible(e1, e2),

            // Tuples
            (Ty::Tuple(es1), Ty::Tuple(es2)) => {
                es1.len() == es2.len() &&
                es1.iter().zip(es2).all(|(t1, t2)| self.types_compatible(t1, t2))
            }

            // Functions
            (Ty::Function { params: p1, result: r1 },
             Ty::Function { params: p2, result: r2 }) => {
                p1.len() == p2.len() &&
                p1.iter().zip(p2).all(|(t1, t2)| self.types_compatible(t1, t2)) &&
                self.types_compatible(r1, r2)
            }

            // AI types
            (Ty::AI(t1), Ty::AI(t2)) => self.types_compatible(t1, t2),

            // Unknown/Error are compatible with everything (for error recovery)
            (Ty::Unknown, _) | (_, Ty::Unknown) => true,
            (Ty::Error, _) | (_, Ty::Error) => true,

            _ => false,
        }
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn check(source: &str) -> Vec<CheckError> {
        let program = Parser::new(source).parse().unwrap();
        let mut checker = Checker::new();
        checker.check(&program)
    }

    #[test]
    fn test_undefined_variable() {
        let errors = check("fn main() { let x = y; }");
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CheckError::UndefinedVariable { .. }));
    }

    #[test]
    fn test_type_mismatch() {
        let errors = check("fn add(a: Int, b: Int) -> String { a + b }");
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CheckError::TypeMismatch { .. }));
    }

    #[test]
    fn test_ai_model_defined() {
        let errors = check(r#"
            ai_model MyModel { provider: "openai" }
            fn test() { ai query { model: MyModel } }
        "#);
        assert!(errors.is_empty());
    }
}
```
