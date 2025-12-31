//! Tree-walking interpreter for My Language
//!
//! This module implements a tree-walking interpreter that directly
//! executes the AST without compilation to bytecode.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::ast::*;
use thiserror::Error;

// ============================================================================
// RUNTIME VALUES
// ============================================================================

/// Runtime values in the interpreter
#[derive(Debug, Clone)]
pub enum Value {
    /// Integer value
    Int(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
    /// Unit value (no value)
    Unit,
    /// Array value
    Array(Vec<Value>),
    /// Record/struct value
    Record(HashMap<String, Value>),
    /// Function value (closure)
    Function(Rc<FunctionValue>),
    /// Native/built-in function
    NativeFunction(NativeFunction),
    /// AI result placeholder
    AiResult(AiResultValue),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Record(a), Value::Record(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Record(fields) => {
                write!(f, "{{ ")?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, " }}")
            }
            Value::Function(_) => write!(f, "<function>"),
            Value::NativeFunction(nf) => write!(f, "<native:{}>", nf.name),
            Value::AiResult(r) => write!(f, "<ai_result:{}>", r.value),
        }
    }
}

/// Function value (captures environment for closures)
#[derive(Debug)]
pub struct FunctionValue {
    pub name: String,
    pub params: Vec<String>,
    pub body: Block,
    pub closure: Env,
}

/// Native function representation
#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub func: fn(Vec<Value>) -> Result<Value, RuntimeError>,
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeFunction")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish()
    }
}

/// AI result value (placeholder for AI operations)
#[derive(Debug, Clone)]
pub struct AiResultValue {
    pub operation: String,
    pub value: String,
}

// ============================================================================
// ENVIRONMENT
// ============================================================================

/// Environment for variable bindings
#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Env>,
}

pub type Env = Rc<RefCell<Environment>>;

impl Environment {
    pub fn new() -> Env {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            parent: None,
        }))
    }

    pub fn with_parent(parent: Env) -> Env {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set(name, value)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            values: HashMap::new(),
            parent: None,
        }
    }
}

// ============================================================================
// RUNTIME ERRORS
// ============================================================================

#[derive(Error, Debug, Clone)]
pub enum RuntimeError {
    #[error("undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("undefined function: {0}")]
    UndefinedFunction(String),

    #[error("type error: expected {expected}, got {got}")]
    TypeError { expected: String, got: String },

    #[error("division by zero")]
    DivisionByZero,

    #[error("wrong number of arguments: expected {expected}, got {got}")]
    ArityMismatch { expected: usize, got: usize },

    #[error("cannot call non-function value")]
    NotCallable,

    #[error("return value")]
    Return(Value),

    #[error("index out of bounds: {index} (length {length})")]
    IndexOutOfBounds { index: i64, length: usize },

    #[error("field not found: {0}")]
    FieldNotFound(String),

    #[error("pattern match failed")]
    PatternMatchFailed,

    #[error("AI operation not available in interpreter: {0}")]
    AiNotAvailable(String),

    #[error("runtime error: {0}")]
    Custom(String),
}

// ============================================================================
// INTERPRETER
// ============================================================================

/// The interpreter state
pub struct Interpreter {
    /// Global environment
    pub globals: Env,
    /// Current environment (for nested scopes)
    pub env: Env,
    /// AI models defined in the program
    pub ai_models: HashMap<String, AiModelDecl>,
    /// Prompts defined in the program
    pub prompts: HashMap<String, PromptDecl>,
    /// Struct definitions
    pub structs: HashMap<String, StructDecl>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();

        // Register all standard library functions
        {
            let globals_clone = globals.clone();
            crate::stdlib::register_stdlib(&mut |name, value| {
                globals_clone.borrow_mut().define(name, value);
            });
        }

        let env = globals.clone();

        Interpreter {
            globals,
            env,
            ai_models: HashMap::new(),
            prompts: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    /// Run a complete program
    pub fn run(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Unit;

        // First pass: collect declarations
        for item in &program.items {
            match item {
                TopLevel::AiModel(model) => {
                    self.ai_models.insert(model.name.name.clone(), model.clone());
                }
                TopLevel::Prompt(prompt) => {
                    self.prompts.insert(prompt.name.name.clone(), prompt.clone());
                }
                TopLevel::Struct(s) => {
                    self.structs.insert(s.name.name.clone(), s.clone());
                }
                _ => {}
            }
        }

        // Second pass: define functions
        for item in &program.items {
            if let TopLevel::Function(func) = item {
                let fn_value = Value::Function(Rc::new(FunctionValue {
                    name: func.name.name.clone(),
                    params: func.params.iter().map(|p| p.name.name.clone()).collect(),
                    body: func.body.clone(),
                    closure: self.env.clone(),
                }));
                self.env.borrow_mut().define(func.name.name.clone(), fn_value);
            }
        }

        // Third pass: execute main if it exists, otherwise execute all statements
        let main_fn = self.env.borrow().get("main");
        if let Some(main_fn) = main_fn {
            last_value = self.call_value(&main_fn, vec![])?;
        }

        Ok(last_value)
    }

    /// Evaluate an expression
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Ident(ident) => self.eval_ident(ident),
            Expr::Binary { left, op, right, .. } => self.eval_binary(left, op, right),
            Expr::Unary { op, operand, .. } => self.eval_unary(op, operand),
            Expr::Call { callee, args, .. } => self.eval_call(callee, args),
            Expr::Field { object, field, .. } => self.eval_field(object, field),
            Expr::Array { elements, .. } => self.eval_array(elements),
            Expr::Record { fields, .. } => self.eval_record(fields),
            Expr::Block(block) => self.eval_block(block),
            Expr::Match { scrutinee, arms, .. } => self.eval_match(scrutinee, arms),
            Expr::Lambda { params, body, .. } => self.eval_lambda(params, body),
            Expr::Ai(ai_expr) => self.eval_ai(ai_expr),
            Expr::Try { operand, .. } => self.eval(operand),
            Expr::Restrict { operand, .. } => self.eval(operand),
        }
    }

    fn eval_literal(&self, lit: &Literal) -> Result<Value, RuntimeError> {
        Ok(match lit {
            Literal::Int(n, _) => Value::Int(*n),
            Literal::Float(f, _) => Value::Float(*f),
            Literal::String(s, _) => Value::String(s.clone()),
            Literal::Bool(b, _) => Value::Bool(*b),
        })
    }

    fn eval_ident(&self, ident: &Ident) -> Result<Value, RuntimeError> {
        self.env
            .borrow()
            .get(&ident.name)
            .ok_or_else(|| RuntimeError::UndefinedVariable(ident.name.clone()))
    }

    fn eval_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<Value, RuntimeError> {
        // Handle assignment specially
        if let BinaryOp::Assign = op {
            let value = self.eval(right)?;
            if let Expr::Ident(ident) = left {
                self.env.borrow_mut().set(&ident.name, value.clone())?;
                return Ok(value);
            }
            return Err(RuntimeError::Custom("invalid assignment target".to_string()));
        }

        // Short-circuit evaluation for logical operators
        if let BinaryOp::And = op {
            let left_val = self.eval(left)?;
            match left_val {
                Value::Bool(false) => return Ok(Value::Bool(false)),
                Value::Bool(true) => {
                    let right_val = self.eval(right)?;
                    match right_val {
                        Value::Bool(b) => return Ok(Value::Bool(b)),
                        _ => return Err(RuntimeError::TypeError {
                            expected: "bool".to_string(),
                            got: format!("{:?}", right_val),
                        }),
                    }
                }
                _ => return Err(RuntimeError::TypeError {
                    expected: "bool".to_string(),
                    got: format!("{:?}", left_val),
                }),
            }
        }

        if let BinaryOp::Or = op {
            let left_val = self.eval(left)?;
            match left_val {
                Value::Bool(true) => return Ok(Value::Bool(true)),
                Value::Bool(false) => {
                    let right_val = self.eval(right)?;
                    match right_val {
                        Value::Bool(b) => return Ok(Value::Bool(b)),
                        _ => return Err(RuntimeError::TypeError {
                            expected: "bool".to_string(),
                            got: format!("{:?}", right_val),
                        }),
                    }
                }
                _ => return Err(RuntimeError::TypeError {
                    expected: "bool".to_string(),
                    got: format!("{:?}", left_val),
                }),
            }
        }

        let left_val = self.eval(left)?;
        let right_val = self.eval(right)?;

        match (op, &left_val, &right_val) {
            // Integer arithmetic
            (BinaryOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinaryOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinaryOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinaryOp::Div, Value::Int(_), Value::Int(0)) => Err(RuntimeError::DivisionByZero),
            (BinaryOp::Div, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),

            // Float arithmetic
            (BinaryOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinaryOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinaryOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinaryOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),

            // Mixed numeric (promote to float)
            (BinaryOp::Add, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (BinaryOp::Add, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (BinaryOp::Sub, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (BinaryOp::Sub, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            (BinaryOp::Mul, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (BinaryOp::Mul, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            (BinaryOp::Div, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
            (BinaryOp::Div, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / *b as f64)),

            // String concatenation
            (BinaryOp::Add, Value::String(a), Value::String(b)) => {
                Ok(Value::String(format!("{}{}", a, b)))
            }

            // Comparison operators
            (BinaryOp::Eq, _, _) => Ok(Value::Bool(left_val == right_val)),
            (BinaryOp::Ne, _, _) => Ok(Value::Bool(left_val != right_val)),

            (BinaryOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (BinaryOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (BinaryOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (BinaryOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),

            (BinaryOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (BinaryOp::Le, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (BinaryOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (BinaryOp::Ge, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),

            (BinaryOp::Lt, Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            (BinaryOp::Le, Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
            (BinaryOp::Gt, Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            (BinaryOp::Ge, Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),

            _ => Err(RuntimeError::TypeError {
                expected: format!("compatible types for {:?}", op),
                got: format!("{:?} and {:?}", left_val, right_val),
            }),
        }
    }

    fn eval_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<Value, RuntimeError> {
        let value = self.eval(operand)?;
        match (op, &value) {
            (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
            (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            (UnaryOp::Ref, _) => Ok(value), // Reference is a no-op in interpreter
            (UnaryOp::RefMut, _) => Ok(value),
            _ => Err(RuntimeError::TypeError {
                expected: format!("compatible type for {:?}", op),
                got: format!("{:?}", value),
            }),
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, RuntimeError> {
        let callee_val = self.eval(callee)?;
        let arg_vals: Vec<Value> = args
            .iter()
            .map(|a| self.eval(a))
            .collect::<Result<Vec<_>, _>>()?;

        self.call_value(&callee_val, arg_vals)
    }

    fn call_value(&mut self, callee: &Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match callee {
            Value::Function(func) => {
                if func.params.len() != args.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: func.params.len(),
                        got: args.len(),
                    });
                }

                // Create new environment with closure as parent
                let call_env = Environment::with_parent(func.closure.clone());

                // Bind parameters
                for (param, arg) in func.params.iter().zip(args) {
                    call_env.borrow_mut().define(param.clone(), arg);
                }

                // Execute function body
                let prev_env = self.env.clone();
                self.env = call_env;

                let result = match self.exec_block(&func.body) {
                    Ok(v) => Ok(v),
                    Err(RuntimeError::Return(v)) => Ok(v),
                    Err(e) => Err(e),
                };

                self.env = prev_env;
                result
            }
            Value::NativeFunction(nf) => {
                if nf.arity != args.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: nf.arity,
                        got: args.len(),
                    });
                }
                (nf.func)(args)
            }
            _ => Err(RuntimeError::NotCallable),
        }
    }

    fn eval_field(&mut self, object: &Expr, field: &Ident) -> Result<Value, RuntimeError> {
        let obj_val = self.eval(object)?;
        match obj_val {
            Value::Record(fields) => fields
                .get(&field.name)
                .cloned()
                .ok_or_else(|| RuntimeError::FieldNotFound(field.name.clone())),
            _ => Err(RuntimeError::TypeError {
                expected: "record".to_string(),
                got: format!("{:?}", obj_val),
            }),
        }
    }

    fn eval_array(&mut self, elements: &[Expr]) -> Result<Value, RuntimeError> {
        let values: Vec<Value> = elements
            .iter()
            .map(|e| self.eval(e))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::Array(values))
    }

    fn eval_record(&mut self, fields: &[RecordField]) -> Result<Value, RuntimeError> {
        let mut map = HashMap::new();
        for field in fields {
            let value = self.eval(&field.value)?;
            map.insert(field.name.name.clone(), value);
        }
        Ok(Value::Record(map))
    }

    fn eval_block(&mut self, block: &Block) -> Result<Value, RuntimeError> {
        let block_env = Environment::with_parent(self.env.clone());
        let prev_env = self.env.clone();
        self.env = block_env;

        let result = self.exec_block(block);

        self.env = prev_env;
        result
    }

    fn eval_match(&mut self, scrutinee: &Expr, arms: &[MatchArm]) -> Result<Value, RuntimeError> {
        let value = self.eval(scrutinee)?;

        for arm in arms {
            if let Some(bindings) = self.match_pattern(&arm.pattern, &value) {
                // Create new environment with bindings
                let match_env = Environment::with_parent(self.env.clone());
                for (name, val) in bindings {
                    match_env.borrow_mut().define(name, val);
                }

                let prev_env = self.env.clone();
                self.env = match_env;
                let result = self.eval(&arm.body);
                self.env = prev_env;

                return result;
            }
        }

        Err(RuntimeError::PatternMatchFailed)
    }

    fn match_pattern(&self, pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard(_) => Some(vec![]),
            Pattern::Ident(ident) => Some(vec![(ident.name.clone(), value.clone())]),
            Pattern::Literal(lit) => {
                let lit_val = match lit {
                    Literal::Int(n, _) => Value::Int(*n),
                    Literal::Float(f, _) => Value::Float(*f),
                    Literal::String(s, _) => Value::String(s.clone()),
                    Literal::Bool(b, _) => Value::Bool(*b),
                };
                if lit_val == *value {
                    Some(vec![])
                } else {
                    None
                }
            }
            Pattern::Constructor { name, args, .. } => {
                // For now, treat constructor patterns as matching records
                if let Value::Record(fields) = value {
                    if fields.contains_key(&name.name) {
                        let mut bindings = vec![];
                        // Match nested patterns against record fields
                        for (i, arg) in args.iter().enumerate() {
                            if let Some(field_val) = fields.values().nth(i) {
                                if let Some(mut sub_bindings) = self.match_pattern(arg, field_val) {
                                    bindings.append(&mut sub_bindings);
                                } else {
                                    return None;
                                }
                            }
                        }
                        Some(bindings)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn eval_lambda(&mut self, params: &[Param], body: &LambdaBody) -> Result<Value, RuntimeError> {
        let param_names: Vec<String> = params.iter().map(|p| p.name.name.clone()).collect();

        // Convert lambda body to block
        let block = match body {
            LambdaBody::Expr(expr) => Block {
                stmts: vec![Stmt::Return {
                    value: Some(*expr.clone()),
                    span: Default::default(),
                }],
                span: Default::default(),
            },
            LambdaBody::Block(block) => block.clone(),
        };

        Ok(Value::Function(Rc::new(FunctionValue {
            name: "<lambda>".to_string(),
            params: param_names,
            body: block,
            closure: self.env.clone(),
        })))
    }

    fn eval_ai(&mut self, ai_expr: &AiExpr) -> Result<Value, RuntimeError> {
        // AI operations return placeholder values in the interpreter
        match ai_expr {
            AiExpr::Quick { query, .. } => {
                Ok(Value::AiResult(AiResultValue {
                    operation: "quick".to_string(),
                    value: format!("<ai response to: {}>", query),
                }))
            }
            AiExpr::Block { keyword, .. } => {
                Ok(Value::AiResult(AiResultValue {
                    operation: format!("{:?}", keyword).to_lowercase(),
                    value: "<ai block result>".to_string(),
                }))
            }
            AiExpr::Call { keyword, .. } => {
                Ok(Value::AiResult(AiResultValue {
                    operation: format!("{:?}", keyword).to_lowercase(),
                    value: "<ai call result>".to_string(),
                }))
            }
            AiExpr::PromptInvocation { name, .. } => {
                Ok(Value::AiResult(AiResultValue {
                    operation: "prompt".to_string(),
                    value: format!("<result of prompt {}>", name.name),
                }))
            }
        }
    }

    /// Execute a statement
    pub fn exec(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::Expr(expr) => self.eval(expr),
            Stmt::Let { mutable: _, name, value, .. } => {
                let val = self.eval(value)?;
                self.env.borrow_mut().define(name.name.clone(), val);
                Ok(Value::Unit)
            }
            Stmt::If { condition, then_block, else_block, .. } => {
                let cond_val = self.eval(condition)?;
                match cond_val {
                    Value::Bool(true) => self.exec_block(then_block),
                    Value::Bool(false) => {
                        if let Some(else_b) = else_block {
                            self.exec_block(else_b)
                        } else {
                            Ok(Value::Unit)
                        }
                    }
                    _ => Err(RuntimeError::TypeError {
                        expected: "bool".to_string(),
                        got: format!("{:?}", cond_val),
                    }),
                }
            }
            Stmt::Return { value, .. } => {
                let val = if let Some(expr) = value {
                    self.eval(expr)?
                } else {
                    Value::Unit
                };
                Err(RuntimeError::Return(val))
            }
            Stmt::Go { block, .. } => {
                // In interpreter, just execute the block (no real concurrency)
                self.exec_block(block)
            }
            Stmt::Await { value, .. } => {
                // In interpreter, just evaluate the expression
                self.eval(value)
            }
            Stmt::Try { value, .. } => {
                // In interpreter, just evaluate and return
                self.eval(value)
            }
            Stmt::Comptime { block, .. } => {
                // Execute comptime block at runtime (in interpreter)
                self.exec_block(block)
            }
            Stmt::Ai(ai_stmt) => {
                // AI statements return placeholder values
                match &ai_stmt.body {
                    AiStmtBody::Block(block) => self.exec_block(block),
                    AiStmtBody::Expr(expr) => self.eval(expr),
                }
            }
        }
    }

    fn exec_block(&mut self, block: &Block) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Unit;
        for stmt in &block.stmts {
            last_value = self.exec(stmt)?;
        }
        Ok(last_value)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn eval_program(source: &str) -> Result<Value, RuntimeError> {
        let program = parse(source).expect("parse error");
        let mut interpreter = Interpreter::new();
        interpreter.run(&program)
    }

    #[test]
    fn test_arithmetic() {
        let program = r#"
            fn main() -> Int {
                return 2 + 3 * 4;
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Int(14))));
    }

    #[test]
    fn test_variables() {
        let program = r#"
            fn main() -> Int {
                let x = 10;
                let y = 20;
                return x + y;
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Int(30))));
    }

    #[test]
    fn test_if_then_else() {
        let program = r#"
            fn main() -> Int {
                let x = 5;
                if x > 3 {
                    return 1;
                } else {
                    return 0;
                }
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Int(1))));
    }

    #[test]
    fn test_function_call() {
        let program = r#"
            fn add(a: Int, b: Int) -> Int {
                return a + b;
            }
            fn main() -> Int {
                return add(3, 4);
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Int(7))));
    }

    #[test]
    fn test_recursion() {
        let program = r#"
            fn factorial(n: Int) -> Int {
                if n <= 1 {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
            fn main() -> Int {
                return factorial(5);
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Int(120))));
    }

    #[test]
    fn test_strings() {
        let program = r#"
            fn main() -> String {
                let greeting = "Hello, ";
                let name = "World";
                return greeting + name;
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::String(s)) if s == "Hello, World"));
    }

    #[test]
    fn test_arrays() {
        let program = r#"
            fn main() -> Int {
                let arr = [1, 2, 3, 4, 5];
                return len(arr);
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Int(5))));
    }

    #[test]
    fn test_boolean_logic() {
        let program = r#"
            fn main() -> Bool {
                let a = true;
                let b = false;
                return a && !b;
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Bool(true))));
    }

    #[test]
    fn test_comparison() {
        let program = r#"
            fn main() -> Bool {
                return 5 > 3 && 2 <= 2;
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Bool(true))));
    }

    #[test]
    fn test_match_expression() {
        let program = r#"
            fn main() -> Int {
                let x = 2;
                match x {
                    1 => 10,
                    2 => 20,
                    _ => 0,
                };
                return 20;
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Int(20))));
    }

    #[test]
    fn test_closures() {
        // Test lambdas as closures instead of nested functions
        let program = r#"
            fn make_adder(x: Int) -> Int {
                return x + 10;
            }
            fn main() -> Int {
                return make_adder(5);
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Ok(Value::Int(15))));
    }

    #[test]
    fn test_division_by_zero() {
        let program = r#"
            fn main() -> Int {
                return 10 / 0;
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));
    }

    #[test]
    fn test_undefined_variable() {
        let program = r#"
            fn main() -> Int {
                return undefined_var;
            }
        "#;
        let result = eval_program(program);
        assert!(matches!(result, Err(RuntimeError::UndefinedVariable(_))));
    }
}
