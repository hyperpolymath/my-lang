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
#[derive(Debug)]
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

fn lower_function(f: &HirFunction) -> Result<MirFunction, MirError> {
    let mut blocks = DiGraph::new();
    let mut locals = Vec::new();
    let mut local_counter = 0;

    // Create entry block
    let entry = BasicBlock {
        id: BlockId(0),
        instructions: vec![],
        terminator: Terminator::Return(None),
    };
    let entry_block = blocks.add_node(entry);

    // Create locals for parameters
    let params: Vec<MirLocal> = f
        .params
        .iter()
        .map(|p| {
            let id = LocalId(local_counter);
            local_counter += 1;
            MirLocal {
                id,
                name: Some(p.name.clone()),
                ty: lower_type(&p.ty),
            }
        })
        .collect();

    Ok(MirFunction {
        name: f.name.clone(),
        params,
        return_type: lower_type(&f.return_type),
        locals,
        blocks,
        entry_block,
    })
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
    pub fn dce(program: &mut MirProgram) {
        // TODO: Implement DCE
    }

    /// Constant folding
    pub fn const_fold(program: &mut MirProgram) {
        // TODO: Implement constant folding
    }

    /// Inline small functions
    pub fn inline(program: &mut MirProgram, threshold: usize) {
        // TODO: Implement inlining
    }

    /// Remove redundant phi nodes
    pub fn simplify_phi(program: &mut MirProgram) {
        // TODO: Implement phi simplification
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
