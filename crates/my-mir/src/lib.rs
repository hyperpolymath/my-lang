// SPDX-License-Identifier: MIT
//! Mid-level Intermediate Representation for My Language
//!
//! MIR is the SSA-form IR used for optimization and codegen:
//! - Static Single Assignment (SSA) form
//! - Basic blocks with explicit control flow
//! - Explicit drop insertion for ownership
//! - Monomorphized generics
//! - Inlined small functions
//!
//! # Architecture
//!
//! ```text
//! HIR → MIR → LLVM IR
//!       ↑
//!   Optimizations
//! ```

use my_hir::{HirFunction, HirProgram, HirType};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use thiserror::Error;

/// MIR lowering errors
#[derive(Debug, Error)]
pub enum MirError {
    #[error("undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("unreachable code")]
    UnreachableCode,
}

/// MIR Program - collection of functions
#[derive(Debug)]
pub struct MirProgram {
    pub functions: HashMap<String, MirFunction>,
    pub entry: Option<String>,
}

/// MIR Function - CFG of basic blocks
#[derive(Debug, Clone)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<MirLocal>,
    pub return_type: MirType,
    pub locals: Vec<MirLocal>,
    pub blocks: DiGraph<BasicBlock, BranchKind>,
    pub entry_block: NodeIndex,
}

/// MIR local variable (SSA)
#[derive(Debug, Clone)]
pub struct MirLocal {
    pub id: LocalId,
    pub name: Option<String>,
    pub ty: MirType,
}

/// Local variable identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub usize);

/// MIR type (monomorphized)
#[derive(Debug, Clone, PartialEq)]
pub enum MirType {
    I32,
    I64,
    F32,
    F64,
    Bool,
    Ptr(Box<MirType>),
    Array(Box<MirType>, usize),
    Struct(String, Vec<MirType>),
    Function(Vec<MirType>, Box<MirType>),
    Unit,
    Never,
}

/// Basic block - sequence of instructions ending in terminator
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

/// Block identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

/// Branch kind for CFG edges
#[derive(Debug, Clone, Copy)]
pub enum BranchKind {
    Unconditional,
    True,
    False,
    SwitchCase(i64),
    SwitchDefault,
}

/// MIR instruction (SSA form)
#[derive(Debug, Clone)]
pub struct Instruction {
    pub dest: LocalId,
    pub kind: InstructionKind,
}

/// Instruction kinds
#[derive(Debug, Clone)]
pub enum InstructionKind {
    /// Load constant
    Const(MirConstant),

    /// Binary operation
    BinOp(BinOp, LocalId, LocalId),

    /// Unary operation
    UnOp(UnOp, LocalId),

    /// Function call
    Call(String, Vec<LocalId>),

    /// Indirect call through function pointer
    CallIndirect(LocalId, Vec<LocalId>),

    /// Load from memory
    Load(LocalId),

    /// Store to memory
    Store(LocalId, LocalId),

    /// Get element pointer
    GetElementPtr(LocalId, Vec<LocalId>),

    /// Allocate on stack
    Alloca(MirType),

    /// Cast between types
    Cast(LocalId, MirType),

    /// Phi node (SSA)
    Phi(Vec<(BlockId, LocalId)>),

    /// AI operation stub (to be replaced at runtime)
    AIStub(AIOperation, Vec<LocalId>),

    /// Drop value (ownership)
    Drop(LocalId),

    /// Copy value
    Copy(LocalId),

    /// Move value
    Move(LocalId),
}

/// MIR constant values
#[derive(Debug, Clone)]
pub enum MirConstant {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    Unit,
}

/// Binary operations
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Xor,
    Shl,
    Shr,
}

/// Unary operations
#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
    Deref,
    AddrOf,
    AddrOfMut,
}

/// AI operations (runtime dispatch)
#[derive(Debug, Clone)]
pub enum AIOperation {
    Query { model: Option<String> },
    Verify,
    Embed,
    Generate,
    Classify,
}

/// Block terminator
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Return from function
    Return(Option<LocalId>),

    /// Unconditional branch
    Goto(BlockId),

    /// Conditional branch
    If(LocalId, BlockId, BlockId),

    /// Switch on integer value
    Switch(LocalId, Vec<(i64, BlockId)>, BlockId),

    /// Unreachable (for exhaustive matches)
    Unreachable,

    /// Call that may unwind
    Invoke {
        func: String,
        args: Vec<LocalId>,
        dest: LocalId,
        normal: BlockId,
        unwind: BlockId,
    },
}

/// Lower HIR to MIR
pub fn lower(hir: &HirProgram) -> Result<MirProgram, MirError> {
    let mut functions = HashMap::new();

    for item in &hir.items {
        if let my_hir::HirItem::Function(f) = item {
            let mir_func = lower_function(f)?;
            functions.insert(mir_func.name.clone(), mir_func);
        }
    }

    let entry = functions.get("main").map(|_| "main".to_string());

    Ok(MirProgram { functions, entry })
}

/// MIR builder for constructing CFGs
struct MirBuilder {
    blocks: DiGraph<BasicBlock, BranchKind>,
    locals: Vec<MirLocal>,
    local_counter: usize,
    block_counter: usize,
    current_block: Option<NodeIndex>,
    current_instructions: Vec<Instruction>,
    var_map: HashMap<String, LocalId>,
}

impl MirBuilder {
    fn new() -> Self {
        MirBuilder {
            blocks: DiGraph::new(),
            locals: Vec::new(),
            local_counter: 0,
            block_counter: 0,
            current_block: None,
            current_instructions: Vec::new(),
            var_map: HashMap::new(),
        }
    }

    fn new_local(&mut self, name: Option<String>, ty: MirType) -> LocalId {
        let id = LocalId(self.local_counter);
        self.local_counter += 1;
        self.locals.push(MirLocal { id, name: name.clone(), ty });
        if let Some(n) = name {
            self.var_map.insert(n, id);
        }
        id
    }

    fn new_temp(&mut self, ty: MirType) -> LocalId {
        self.new_local(None, ty)
    }

    fn new_block(&mut self) -> (BlockId, NodeIndex) {
        let id = BlockId(self.block_counter);
        self.block_counter += 1;
        let block = BasicBlock {
            id,
            instructions: vec![],
            terminator: Terminator::Unreachable,
        };
        let node = self.blocks.add_node(block);
        (id, node)
    }

    fn emit(&mut self, dest: LocalId, kind: InstructionKind) {
        self.current_instructions.push(Instruction { dest, kind });
    }

    fn finish_block(&mut self, terminator: Terminator) -> NodeIndex {
        if let Some(node) = self.current_block {
            let block = self.blocks.node_weight_mut(node).unwrap();
            block.instructions = std::mem::take(&mut self.current_instructions);
            block.terminator = terminator;
            node
        } else {
            let (_, node) = self.new_block();
            let block = self.blocks.node_weight_mut(node).unwrap();
            block.instructions = std::mem::take(&mut self.current_instructions);
            block.terminator = terminator;
            node
        }
    }

    fn set_current_block(&mut self, node: NodeIndex) {
        self.current_block = Some(node);
        self.current_instructions.clear();
    }

    fn lookup_var(&self, name: &str) -> Option<LocalId> {
        self.var_map.get(name).copied()
    }
}

fn lower_function(f: &HirFunction) -> Result<MirFunction, MirError> {
    let mut builder = MirBuilder::new();

    // Create locals for parameters
    let params: Vec<MirLocal> = f
        .params
        .iter()
        .map(|p| {
            let ty = lower_type(&p.ty);
            let id = builder.new_local(Some(p.name.clone()), ty.clone());
            MirLocal {
                id,
                name: Some(p.name.clone()),
                ty,
            }
        })
        .collect();

    // Create entry block
    let (_, entry_node) = builder.new_block();
    builder.set_current_block(entry_node);

    // Lower function body
    let result = lower_block(&mut builder, &f.body)?;

    // Finish with return
    let terminator = if let Some(result_id) = result {
        Terminator::Return(Some(result_id))
    } else {
        Terminator::Return(None)
    };
    builder.finish_block(terminator);

    Ok(MirFunction {
        name: f.name.clone(),
        params,
        return_type: lower_type(&f.return_type),
        locals: builder.locals,
        blocks: builder.blocks,
        entry_block: entry_node,
    })
}

fn lower_block(builder: &mut MirBuilder, block: &my_hir::HirBlock) -> Result<Option<LocalId>, MirError> {
    for stmt in &block.stmts {
        lower_stmt(builder, stmt)?;
    }

    if let Some(expr) = &block.expr {
        let result = lower_expr(builder, expr)?;
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

fn lower_stmt(builder: &mut MirBuilder, stmt: &my_hir::HirStmt) -> Result<(), MirError> {
    match stmt {
        my_hir::HirStmt::Let { name, ty, value } => {
            let val_id = lower_expr(builder, value)?;
            let mir_ty = ty.as_ref().map(lower_type).unwrap_or(MirType::Unit);
            let local_id = builder.new_local(Some(name.clone()), mir_ty);
            builder.emit(local_id, InstructionKind::Copy(val_id));
            Ok(())
        }
        my_hir::HirStmt::Expr(expr) => {
            lower_expr(builder, expr)?;
            Ok(())
        }
        my_hir::HirStmt::Return(value) => {
            let result = value.as_ref().map(|e| lower_expr(builder, e)).transpose()?;
            builder.finish_block(Terminator::Return(result));
            // Start a new unreachable block
            let (_, node) = builder.new_block();
            builder.set_current_block(node);
            Ok(())
        }
    }
}

fn lower_expr(builder: &mut MirBuilder, expr: &my_hir::HirExpr) -> Result<LocalId, MirError> {
    match expr {
        my_hir::HirExpr::Literal(lit) => {
            let (constant, ty) = lower_literal(lit);
            let dest = builder.new_temp(ty);
            builder.emit(dest, InstructionKind::Const(constant));
            Ok(dest)
        }
        my_hir::HirExpr::Var(name) => {
            builder.lookup_var(name).ok_or_else(|| MirError::UndefinedVariable(name.clone()))
        }
        my_hir::HirExpr::Call(callee, args) => {
            let arg_ids: Vec<LocalId> = args
                .iter()
                .map(|a| lower_expr(builder, a))
                .collect::<Result<_, _>>()?;

            // Check if callee is a direct function name
            if let my_hir::HirExpr::Var(func_name) = callee.as_ref() {
                let dest = builder.new_temp(MirType::Unit); // TODO: Infer return type
                builder.emit(dest, InstructionKind::Call(func_name.clone(), arg_ids));
                Ok(dest)
            } else {
                let callee_id = lower_expr(builder, callee)?;
                let dest = builder.new_temp(MirType::Unit);
                builder.emit(dest, InstructionKind::CallIndirect(callee_id, arg_ids));
                Ok(dest)
            }
        }
        my_hir::HirExpr::BinOp(left, op, right) => {
            let left_id = lower_expr(builder, left)?;
            let right_id = lower_expr(builder, right)?;
            let dest = builder.new_temp(MirType::I64); // TODO: Proper type
            builder.emit(dest, InstructionKind::BinOp(lower_binop(*op), left_id, right_id));
            Ok(dest)
        }
        my_hir::HirExpr::UnOp(op, operand) => {
            let operand_id = lower_expr(builder, operand)?;
            let dest = builder.new_temp(MirType::I64);
            builder.emit(dest, InstructionKind::UnOp(lower_unop(*op), operand_id));
            Ok(dest)
        }
        my_hir::HirExpr::If(cond, then_block, else_block) => {
            let cond_id = lower_expr(builder, cond)?;

            // Create blocks
            let (then_bid, then_node) = builder.new_block();
            let (else_bid, else_node) = builder.new_block();
            let (merge_bid, merge_node) = builder.new_block();

            // Finish current block with conditional branch
            builder.finish_block(Terminator::If(cond_id, then_bid, else_bid));

            // Lower then branch
            builder.set_current_block(then_node);
            let then_result = lower_block(builder, then_block)?;
            builder.finish_block(Terminator::Goto(merge_bid));
            builder.blocks.add_edge(then_node, merge_node, BranchKind::Unconditional);

            // Lower else branch
            builder.set_current_block(else_node);
            let else_result = if let Some(eb) = else_block {
                lower_block(builder, eb)?
            } else {
                None
            };
            builder.finish_block(Terminator::Goto(merge_bid));
            builder.blocks.add_edge(else_node, merge_node, BranchKind::Unconditional);

            // Set merge block as current
            builder.set_current_block(merge_node);

            // Create phi if both branches have values
            if let (Some(then_id), Some(else_id)) = (then_result, else_result) {
                let dest = builder.new_temp(MirType::Unit);
                builder.emit(dest, InstructionKind::Phi(vec![
                    (then_bid, then_id),
                    (else_bid, else_id),
                ]));
                Ok(dest)
            } else {
                let dest = builder.new_temp(MirType::Unit);
                builder.emit(dest, InstructionKind::Const(MirConstant::Unit));
                Ok(dest)
            }
        }
        my_hir::HirExpr::Block(block) => {
            let result = lower_block(builder, block)?;
            if let Some(id) = result {
                Ok(id)
            } else {
                let dest = builder.new_temp(MirType::Unit);
                builder.emit(dest, InstructionKind::Const(MirConstant::Unit));
                Ok(dest)
            }
        }
        my_hir::HirExpr::Field(object, field) => {
            let obj_id = lower_expr(builder, object)?;
            let dest = builder.new_temp(MirType::Unit);
            // Field access becomes a GEP in MIR
            let field_idx = builder.new_temp(MirType::I64);
            builder.emit(field_idx, InstructionKind::Const(MirConstant::I64(0))); // TODO: Field index
            builder.emit(dest, InstructionKind::GetElementPtr(obj_id, vec![field_idx]));
            Ok(dest)
        }
        my_hir::HirExpr::Array(elements) => {
            let elem_ids: Vec<LocalId> = elements
                .iter()
                .map(|e| lower_expr(builder, e))
                .collect::<Result<_, _>>()?;

            // Allocate array and store elements
            let arr_ty = MirType::Array(Box::new(MirType::I64), elem_ids.len());
            let arr = builder.new_temp(arr_ty);
            builder.emit(arr, InstructionKind::Alloca(MirType::Array(Box::new(MirType::I64), elem_ids.len())));

            for (i, elem_id) in elem_ids.iter().enumerate() {
                let idx = builder.new_temp(MirType::I64);
                builder.emit(idx, InstructionKind::Const(MirConstant::I64(i as i64)));
                let ptr = builder.new_temp(MirType::Ptr(Box::new(MirType::I64)));
                builder.emit(ptr, InstructionKind::GetElementPtr(arr, vec![idx]));
                let store_dest = builder.new_temp(MirType::Unit);
                builder.emit(store_dest, InstructionKind::Store(ptr, *elem_id));
            }

            Ok(arr)
        }
        my_hir::HirExpr::Record(fields) => {
            // Lower record as a struct allocation
            let dest = builder.new_temp(MirType::Unit);
            builder.emit(dest, InstructionKind::Alloca(MirType::Unit));

            for (_, value) in fields {
                lower_expr(builder, value)?;
            }

            Ok(dest)
        }
        my_hir::HirExpr::Lambda(params, body) => {
            // Lambdas are lowered to closures (function pointer + environment)
            let dest = builder.new_temp(MirType::Unit);
            builder.emit(dest, InstructionKind::Const(MirConstant::Unit));
            // TODO: Full lambda lowering with closure conversion
            Ok(dest)
        }
        my_hir::HirExpr::Match(scrutinee, arms) => {
            let scrut_id = lower_expr(builder, scrutinee)?;

            // Simple lowering: chain of if-else
            // TODO: Full match compilation with decision trees
            if arms.is_empty() {
                let dest = builder.new_temp(MirType::Unit);
                builder.emit(dest, InstructionKind::Const(MirConstant::Unit));
                return Ok(dest);
            }

            // For now, just lower the first arm's body
            let result = lower_expr(builder, &arms[0].body)?;
            Ok(result)
        }
        my_hir::HirExpr::AI(ai_expr) => {
            lower_ai_expr(builder, ai_expr)
        }
    }
}

fn lower_ai_expr(builder: &mut MirBuilder, ai_expr: &my_hir::HirAIExpr) -> Result<LocalId, MirError> {
    match ai_expr {
        my_hir::HirAIExpr::Query { model, prompt } => {
            let prompt_id = lower_expr(builder, prompt)?;
            let dest = builder.new_temp(MirType::Ptr(Box::new(MirType::I32))); // String result
            builder.emit(dest, InstructionKind::AIStub(
                AIOperation::Query { model: model.clone() },
                vec![prompt_id],
            ));
            Ok(dest)
        }
        my_hir::HirAIExpr::Verify { condition } => {
            let cond_id = lower_expr(builder, condition)?;
            let dest = builder.new_temp(MirType::Bool);
            builder.emit(dest, InstructionKind::AIStub(AIOperation::Verify, vec![cond_id]));
            Ok(dest)
        }
        my_hir::HirAIExpr::Embed { input } => {
            let input_id = lower_expr(builder, input)?;
            let dest = builder.new_temp(MirType::Array(Box::new(MirType::F32), 0));
            builder.emit(dest, InstructionKind::AIStub(AIOperation::Embed, vec![input_id]));
            Ok(dest)
        }
        my_hir::HirAIExpr::Generate { template, params } => {
            let param_ids: Vec<LocalId> = params
                .iter()
                .map(|p| lower_expr(builder, p))
                .collect::<Result<_, _>>()?;
            let dest = builder.new_temp(MirType::Ptr(Box::new(MirType::I32)));
            builder.emit(dest, InstructionKind::AIStub(AIOperation::Generate, param_ids));
            Ok(dest)
        }
    }
}

fn lower_literal(lit: &my_hir::HirLiteral) -> (MirConstant, MirType) {
    match lit {
        my_hir::HirLiteral::Int(v) => (MirConstant::I64(*v), MirType::I64),
        my_hir::HirLiteral::Float(v) => (MirConstant::F64(*v), MirType::F64),
        my_hir::HirLiteral::String(v) => (MirConstant::String(v.clone()), MirType::Ptr(Box::new(MirType::I32))),
        my_hir::HirLiteral::Bool(v) => (MirConstant::Bool(*v), MirType::Bool),
    }
}

fn lower_binop(op: my_hir::HirBinOp) -> BinOp {
    match op {
        my_hir::HirBinOp::Add => BinOp::Add,
        my_hir::HirBinOp::Sub => BinOp::Sub,
        my_hir::HirBinOp::Mul => BinOp::Mul,
        my_hir::HirBinOp::Div => BinOp::Div,
        my_hir::HirBinOp::Eq => BinOp::Eq,
        my_hir::HirBinOp::Ne => BinOp::Ne,
        my_hir::HirBinOp::Lt => BinOp::Lt,
        my_hir::HirBinOp::Gt => BinOp::Gt,
        my_hir::HirBinOp::Le => BinOp::Le,
        my_hir::HirBinOp::Ge => BinOp::Ge,
        my_hir::HirBinOp::And => BinOp::And,
        my_hir::HirBinOp::Or => BinOp::Or,
    }
}

fn lower_unop(op: my_hir::HirUnOp) -> UnOp {
    match op {
        my_hir::HirUnOp::Neg => UnOp::Neg,
        my_hir::HirUnOp::Not => UnOp::Not,
        my_hir::HirUnOp::Ref => UnOp::AddrOf,
        my_hir::HirUnOp::RefMut => UnOp::AddrOfMut,
    }
}

fn lower_type(ty: &HirType) -> MirType {
    match ty {
        HirType::Primitive(p) => match p {
            my_hir::HirPrimitive::Int => MirType::I64,
            my_hir::HirPrimitive::Float => MirType::F64,
            my_hir::HirPrimitive::String => MirType::Ptr(Box::new(MirType::I32)), // i8*
            my_hir::HirPrimitive::Bool => MirType::Bool,
        },
        HirType::Array(inner) => MirType::Array(Box::new(lower_type(inner)), 0),
        HirType::AI(inner) => lower_type(inner), // AI types are erased at runtime
        HirType::Function(param, ret) => {
            MirType::Function(vec![lower_type(param)], Box::new(lower_type(ret)))
        }
        HirType::Unit => MirType::Unit,
        _ => MirType::Unit, // TODO: Handle all types
    }
}

/// Optimization passes
pub mod passes {
    use super::*;

    /// Dead code elimination
    pub fn dce(_program: &mut MirProgram) {
        // TODO: Implement DCE
    }

    /// Constant folding
    pub fn const_fold(_program: &mut MirProgram) {
        // TODO: Implement constant folding
    }

    /// Inline small functions
    pub fn inline(_program: &mut MirProgram, _threshold: usize) {
        // TODO: Implement inlining
    }

    /// Remove redundant phi nodes
    pub fn simplify_phi(_program: &mut MirProgram) {
        // TODO: Implement phi simplification
    }
}

/// MIR Interpreter - executes MIR programs directly
pub mod interpreter {
    use super::*;
    use std::collections::HashMap;

    /// Runtime value
    #[derive(Debug, Clone)]
    pub enum Value {
        I32(i32),
        I64(i64),
        F32(f32),
        F64(f64),
        Bool(bool),
        String(String),
        Array(Vec<Value>),
        Struct(HashMap<String, Value>),
        Ptr(usize),
        Unit,
    }

    /// Interpreter error
    #[derive(Debug, thiserror::Error)]
    pub enum InterpreterError {
        #[error("undefined function: {0}")]
        UndefinedFunction(String),

        #[error("undefined local: {0}")]
        UndefinedLocal(usize),

        #[error("type error: {0}")]
        TypeError(String),

        #[error("stack overflow")]
        StackOverflow,

        #[error("division by zero")]
        DivisionByZero,

        #[error("AI operations require runtime")]
        AINotSupported,
    }

    /// Call frame
    struct Frame {
        function: String,
        locals: HashMap<usize, Value>,
        block_idx: NodeIndex,
        ip: usize, // instruction pointer within block
    }

    /// MIR interpreter
    pub struct Interpreter {
        program: MirProgram,
        stack: Vec<Frame>,
        heap: Vec<Value>,
        max_stack: usize,
    }

    impl Interpreter {
        pub fn new(program: MirProgram) -> Self {
            Interpreter {
                program,
                stack: Vec::new(),
                heap: Vec::new(),
                max_stack: 1000,
            }
        }

        /// Run the program starting from main
        pub fn run(&mut self) -> Result<Value, InterpreterError> {
            let entry = self.program.entry.clone()
                .ok_or_else(|| InterpreterError::UndefinedFunction("main".to_string()))?;

            self.call(&entry, vec![])
        }

        /// Call a function with arguments
        pub fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, InterpreterError> {
            if self.stack.len() >= self.max_stack {
                return Err(InterpreterError::StackOverflow);
            }

            let func = self.program.functions.get(name).cloned()
                .ok_or_else(|| InterpreterError::UndefinedFunction(name.to_string()))?;

            // Create new frame
            let mut locals = HashMap::new();

            // Bind parameters
            for (i, param) in func.params.iter().enumerate() {
                if let Some(arg) = args.get(i) {
                    locals.insert(param.id.0, arg.clone());
                }
            }

            let frame = Frame {
                function: name.to_string(),
                locals,
                block_idx: func.entry_block,
                ip: 0,
            };

            self.stack.push(frame);

            // Execute blocks
            let result = self.execute_function(&func)?;

            self.stack.pop();

            Ok(result)
        }

        fn execute_function(&mut self, func: &MirFunction) -> Result<Value, InterpreterError> {
            loop {
                // Get current block info without holding borrow
                let (block_idx, ip) = {
                    let frame = self.stack.last().unwrap();
                    (frame.block_idx, frame.ip)
                };

                let block = func.blocks.node_weight(block_idx)
                    .ok_or_else(|| InterpreterError::TypeError("invalid block".to_string()))?
                    .clone();

                // Execute instructions
                let mut current_ip = ip;
                while current_ip < block.instructions.len() {
                    let instr = block.instructions[current_ip].clone();
                    let value = self.execute_instruction(&instr)?;

                    let frame = self.stack.last_mut().unwrap();
                    frame.locals.insert(instr.dest.0, value);
                    frame.ip = current_ip + 1;
                    current_ip += 1;
                }

                // Execute terminator - extract values first to avoid borrow issues
                let terminator = block.terminator.clone();
                match terminator {
                    Terminator::Return(local) => {
                        let value = if let Some(id) = local {
                            self.get_local(id.0)?
                        } else {
                            Value::Unit
                        };
                        return Ok(value);
                    }
                    Terminator::Goto(target) => {
                        let next_block = Self::find_block_node_static(func, target)?;
                        let frame = self.stack.last_mut().unwrap();
                        frame.block_idx = next_block;
                        frame.ip = 0;
                    }
                    Terminator::If(cond, then_block, else_block) => {
                        let cond_val = self.get_local(cond.0)?;
                        let target = match cond_val {
                            Value::Bool(true) => then_block,
                            Value::Bool(false) => else_block,
                            _ => return Err(InterpreterError::TypeError("expected bool".to_string())),
                        };
                        let next_block = Self::find_block_node_static(func, target)?;
                        let frame = self.stack.last_mut().unwrap();
                        frame.block_idx = next_block;
                        frame.ip = 0;
                    }
                    Terminator::Switch(_, _, default) => {
                        let next_block = Self::find_block_node_static(func, default)?;
                        let frame = self.stack.last_mut().unwrap();
                        frame.block_idx = next_block;
                        frame.ip = 0;
                    }
                    Terminator::Unreachable => {
                        return Err(InterpreterError::TypeError("unreachable code".to_string()));
                    }
                    Terminator::Invoke { func: fn_name, args, dest, normal, .. } => {
                        let arg_values: Result<Vec<_>, _> = args.iter()
                            .map(|a| self.get_local(a.0))
                            .collect();
                        let result = self.call(&fn_name, arg_values?)?;
                        let next_block = Self::find_block_node_static(func, normal)?;
                        let frame = self.stack.last_mut().unwrap();
                        frame.locals.insert(dest.0, result);
                        frame.block_idx = next_block;
                        frame.ip = 0;
                    }
                }
            }
        }

        fn find_block_node_static(func: &MirFunction, id: BlockId) -> Result<NodeIndex, InterpreterError> {
            for node in func.blocks.node_indices() {
                if let Some(block) = func.blocks.node_weight(node) {
                    if block.id == id {
                        return Ok(node);
                    }
                }
            }
            Err(InterpreterError::TypeError(format!("block {:?} not found", id)))
        }

        fn execute_instruction(&mut self, instr: &Instruction) -> Result<Value, InterpreterError> {
            match &instr.kind {
                InstructionKind::Const(c) => Ok(self.const_to_value(c)),

                InstructionKind::BinOp(op, left, right) => {
                    let l = self.get_local(left.0)?;
                    let r = self.get_local(right.0)?;
                    self.eval_binop(*op, l, r)
                }

                InstructionKind::UnOp(op, operand) => {
                    let v = self.get_local(operand.0)?;
                    self.eval_unop(*op, v)
                }

                InstructionKind::Call(name, args) => {
                    let arg_values: Result<Vec<_>, _> = args.iter()
                        .map(|a| self.get_local(a.0))
                        .collect();

                    // Check for builtins
                    match name.as_str() {
                        "print" => {
                            for arg in arg_values? {
                                println!("{:?}", arg);
                            }
                            Ok(Value::Unit)
                        }
                        _ => self.call(name, arg_values?)
                    }
                }

                InstructionKind::CallIndirect(_, _) => {
                    Ok(Value::Unit) // TODO: Function pointers
                }

                InstructionKind::Load(ptr) => {
                    let addr = self.get_local(ptr.0)?;
                    if let Value::Ptr(idx) = addr {
                        self.heap.get(idx).cloned()
                            .ok_or_else(|| InterpreterError::TypeError("invalid pointer".to_string()))
                    } else {
                        Err(InterpreterError::TypeError("expected pointer".to_string()))
                    }
                }

                InstructionKind::Store(ptr, val) => {
                    let addr = self.get_local(ptr.0)?;
                    let value = self.get_local(val.0)?;
                    if let Value::Ptr(idx) = addr {
                        if idx < self.heap.len() {
                            self.heap[idx] = value;
                        }
                    }
                    Ok(Value::Unit)
                }

                InstructionKind::Alloca(_) => {
                    let idx = self.heap.len();
                    self.heap.push(Value::Unit);
                    Ok(Value::Ptr(idx))
                }

                InstructionKind::GetElementPtr(_, _) => {
                    Ok(Value::Ptr(0)) // TODO: Proper GEP
                }

                InstructionKind::Cast(val, _) => {
                    self.get_local(val.0)
                }

                InstructionKind::Phi(branches) => {
                    // In interpreter, just take first available value
                    for (_, local) in branches {
                        if let Ok(v) = self.get_local(local.0) {
                            return Ok(v);
                        }
                    }
                    Ok(Value::Unit)
                }

                InstructionKind::AIStub(_, _) => {
                    Err(InterpreterError::AINotSupported)
                }

                InstructionKind::Drop(_) => Ok(Value::Unit),
                InstructionKind::Copy(src) => self.get_local(src.0),
                InstructionKind::Move(src) => self.get_local(src.0),
            }
        }

        fn const_to_value(&self, c: &MirConstant) -> Value {
            match c {
                MirConstant::I32(v) => Value::I32(*v),
                MirConstant::I64(v) => Value::I64(*v),
                MirConstant::F32(v) => Value::F32(*v),
                MirConstant::F64(v) => Value::F64(*v),
                MirConstant::Bool(v) => Value::Bool(*v),
                MirConstant::String(v) => Value::String(v.clone()),
                MirConstant::Unit => Value::Unit,
            }
        }

        fn get_local(&self, id: usize) -> Result<Value, InterpreterError> {
            self.stack.last()
                .and_then(|f| f.locals.get(&id).cloned())
                .ok_or(InterpreterError::UndefinedLocal(id))
        }

        fn eval_binop(&self, op: BinOp, left: Value, right: Value) -> Result<Value, InterpreterError> {
            match (left, right) {
                (Value::I64(l), Value::I64(r)) => match op {
                    BinOp::Add => Ok(Value::I64(l + r)),
                    BinOp::Sub => Ok(Value::I64(l - r)),
                    BinOp::Mul => Ok(Value::I64(l * r)),
                    BinOp::Div => {
                        if r == 0 { return Err(InterpreterError::DivisionByZero); }
                        Ok(Value::I64(l / r))
                    }
                    BinOp::Rem => Ok(Value::I64(l % r)),
                    BinOp::Eq => Ok(Value::Bool(l == r)),
                    BinOp::Ne => Ok(Value::Bool(l != r)),
                    BinOp::Lt => Ok(Value::Bool(l < r)),
                    BinOp::Le => Ok(Value::Bool(l <= r)),
                    BinOp::Gt => Ok(Value::Bool(l > r)),
                    BinOp::Ge => Ok(Value::Bool(l >= r)),
                    _ => Err(InterpreterError::TypeError("invalid op for i64".to_string())),
                },
                (Value::F64(l), Value::F64(r)) => match op {
                    BinOp::Add => Ok(Value::F64(l + r)),
                    BinOp::Sub => Ok(Value::F64(l - r)),
                    BinOp::Mul => Ok(Value::F64(l * r)),
                    BinOp::Div => Ok(Value::F64(l / r)),
                    BinOp::Eq => Ok(Value::Bool(l == r)),
                    BinOp::Ne => Ok(Value::Bool(l != r)),
                    BinOp::Lt => Ok(Value::Bool(l < r)),
                    BinOp::Le => Ok(Value::Bool(l <= r)),
                    BinOp::Gt => Ok(Value::Bool(l > r)),
                    BinOp::Ge => Ok(Value::Bool(l >= r)),
                    _ => Err(InterpreterError::TypeError("invalid op for f64".to_string())),
                },
                (Value::Bool(l), Value::Bool(r)) => match op {
                    BinOp::And => Ok(Value::Bool(l && r)),
                    BinOp::Or => Ok(Value::Bool(l || r)),
                    BinOp::Eq => Ok(Value::Bool(l == r)),
                    BinOp::Ne => Ok(Value::Bool(l != r)),
                    _ => Err(InterpreterError::TypeError("invalid op for bool".to_string())),
                },
                _ => Err(InterpreterError::TypeError("type mismatch in binop".to_string())),
            }
        }

        fn eval_unop(&self, op: UnOp, val: Value) -> Result<Value, InterpreterError> {
            match (op, val) {
                (UnOp::Neg, Value::I64(v)) => Ok(Value::I64(-v)),
                (UnOp::Neg, Value::F64(v)) => Ok(Value::F64(-v)),
                (UnOp::Not, Value::Bool(v)) => Ok(Value::Bool(!v)),
                _ => Err(InterpreterError::TypeError("invalid unary op".to_string())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_empty_program() {
        let hir = my_hir::HirProgram { items: vec![] };
        let mir = lower(&hir).unwrap();
        assert!(mir.functions.is_empty());
    }
}
