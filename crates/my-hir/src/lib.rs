// SPDX-License-Identifier: MIT
//! High-level Intermediate Representation for My Language
//!
//! HIR is the first IR after parsing, providing:
//! - Desugared match expressions
//! - Expanded macros and prompts
//! - Resolved imports
//! - Normalized control flow
//!
//! # Architecture
//!
//! ```text
//! AST → HIR → MIR → LLVM IR
//! ```

use my_lang::{Program, TopLevel, FnDecl, StructDecl, EffectDecl, AiModelDecl, Type, PrimitiveType, AiModelAttr};
use thiserror::Error;

/// HIR lowering errors
#[derive(Debug, Error)]
pub enum HirError {
    #[error("unresolved import: {0}")]
    UnresolvedImport(String),

    #[error("undefined macro: {0}")]
    UndefinedMacro(String),

    #[error("invalid pattern: {0}")]
    InvalidPattern(String),
}

/// HIR Program representation
#[derive(Debug, Clone)]
pub struct HirProgram {
    pub items: Vec<HirItem>,
}

/// HIR top-level items
#[derive(Debug, Clone)]
pub enum HirItem {
    Function(HirFunction),
    Struct(HirStruct),
    Effect(HirEffect),
    AIModel(HirAIModel),
}

/// HIR function representation
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: HirType,
    pub body: HirBlock,
    pub effects: Vec<String>,
}

/// HIR parameter
#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: String,
    pub ty: HirType,
}

/// HIR type representation
#[derive(Debug, Clone)]
pub enum HirType {
    Primitive(HirPrimitive),
    Function(Box<HirType>, Box<HirType>),
    Array(Box<HirType>),
    AI(Box<HirType>),
    Effect(Box<HirType>, Vec<String>),
    Named(String),
    Unit,
}

/// HIR primitive types
#[derive(Debug, Clone, Copy)]
pub enum HirPrimitive {
    Int,
    Float,
    String,
    Bool,
}

/// HIR struct definition
#[derive(Debug, Clone)]
pub struct HirStruct {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<HirField>,
}

/// HIR struct field
#[derive(Debug, Clone)]
pub struct HirField {
    pub name: String,
    pub ty: HirType,
}

/// HIR effect declaration
#[derive(Debug, Clone)]
pub struct HirEffect {
    pub name: String,
    pub operations: Vec<HirOperation>,
}

/// HIR effect operation
#[derive(Debug, Clone)]
pub struct HirOperation {
    pub name: String,
    pub params: Vec<HirType>,
    pub return_type: HirType,
}

/// HIR AI model declaration
#[derive(Debug, Clone)]
pub struct HirAIModel {
    pub name: String,
    pub provider: String,
    pub model: String,
    pub config: Vec<(String, HirExpr)>,
}

/// HIR block
#[derive(Debug, Clone)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
    pub expr: Option<Box<HirExpr>>,
}

/// HIR statement
#[derive(Debug, Clone)]
pub enum HirStmt {
    Let {
        name: String,
        ty: Option<HirType>,
        value: HirExpr,
    },
    Expr(HirExpr),
    Return(Option<HirExpr>),
}

/// HIR expression
#[derive(Debug, Clone)]
pub enum HirExpr {
    Literal(HirLiteral),
    Var(String),
    Call(Box<HirExpr>, Vec<HirExpr>),
    Lambda(Vec<HirParam>, Box<HirExpr>),
    If(Box<HirExpr>, HirBlock, Option<HirBlock>),
    Match(Box<HirExpr>, Vec<HirArm>),
    Block(HirBlock),
    Field(Box<HirExpr>, String),
    Array(Vec<HirExpr>),
    Record(Vec<(String, HirExpr)>),
    BinOp(Box<HirExpr>, HirBinOp, Box<HirExpr>),
    UnOp(HirUnOp, Box<HirExpr>),
    AI(HirAIExpr),
}

/// HIR literal
#[derive(Debug, Clone)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

/// HIR binary operator
#[derive(Debug, Clone, Copy)]
pub enum HirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

/// HIR unary operator
#[derive(Debug, Clone, Copy)]
pub enum HirUnOp {
    Neg,
    Not,
    Ref,
    RefMut,
}

/// HIR match arm
#[derive(Debug, Clone)]
pub struct HirArm {
    pub pattern: HirPattern,
    pub guard: Option<HirExpr>,
    pub body: HirExpr,
}

/// HIR pattern
#[derive(Debug, Clone)]
pub enum HirPattern {
    Wildcard,
    Var(String),
    Literal(HirLiteral),
    Constructor(String, Vec<HirPattern>),
}

/// HIR AI expression
#[derive(Debug, Clone)]
pub enum HirAIExpr {
    Query {
        model: Option<String>,
        prompt: Box<HirExpr>,
    },
    Verify {
        condition: Box<HirExpr>,
    },
    Embed {
        input: Box<HirExpr>,
    },
    Generate {
        template: String,
        params: Vec<HirExpr>,
    },
}

/// Lower AST to HIR
pub fn lower(program: &Program) -> Result<HirProgram, HirError> {
    let mut items = Vec::new();

    for item in &program.items {
        match item {
            TopLevel::Function(f) => {
                items.push(HirItem::Function(lower_function(f)?));
            }
            TopLevel::Struct(s) => {
                items.push(HirItem::Struct(lower_struct(s)?));
            }
            TopLevel::Effect(e) => {
                items.push(HirItem::Effect(lower_effect(e)?));
            }
            TopLevel::AiModel(m) => {
                items.push(HirItem::AIModel(lower_ai_model(m)?));
            }
            _ => {
                // TODO: Handle other top-level items
            }
        }
    }

    Ok(HirProgram { items })
}

fn lower_function(f: &FnDecl) -> Result<HirFunction, HirError> {
    Ok(HirFunction {
        name: f.name.name.clone(),
        params: f
            .params
            .iter()
            .map(|p| HirParam {
                name: p.name.name.clone(),
                ty: lower_type(&p.ty),
            })
            .collect(),
        return_type: f
            .return_type
            .as_ref()
            .map(|t| lower_type(t))
            .unwrap_or(HirType::Unit),
        body: HirBlock {
            stmts: vec![],
            expr: None,
        },
        effects: vec![], // TODO: Extract from contract or modifiers
    })
}

fn lower_struct(s: &StructDecl) -> Result<HirStruct, HirError> {
    Ok(HirStruct {
        name: s.name.name.clone(),
        type_params: s.type_params.iter().map(|p| p.name.clone()).collect(),
        fields: s
            .fields
            .iter()
            .map(|f| HirField {
                name: f.name.name.clone(),
                ty: lower_type(&f.ty),
            })
            .collect(),
    })
}

fn lower_effect(e: &EffectDecl) -> Result<HirEffect, HirError> {
    Ok(HirEffect {
        name: e.name.name.clone(),
        operations: e
            .ops
            .iter()
            .map(|op| HirOperation {
                name: op.name.name.clone(),
                params: vec![],
                return_type: lower_type(&op.ty),
            })
            .collect(),
    })
}

fn lower_ai_model(m: &AiModelDecl) -> Result<HirAIModel, HirError> {
    let mut provider = String::new();
    let mut model = String::new();

    for attr in &m.attributes {
        match attr {
            AiModelAttr::Provider(p) => provider = p.clone(),
            AiModelAttr::Model(m) => model = m.clone(),
            _ => {}
        }
    }

    Ok(HirAIModel {
        name: m.name.name.clone(),
        provider,
        model,
        config: vec![],
    })
}

fn lower_type(ty: &Type) -> HirType {
    match ty {
        Type::Primitive(p) => match p {
            PrimitiveType::Int => HirType::Primitive(HirPrimitive::Int),
            PrimitiveType::Float => HirType::Primitive(HirPrimitive::Float),
            PrimitiveType::String => HirType::Primitive(HirPrimitive::String),
            PrimitiveType::Bool => HirType::Primitive(HirPrimitive::Bool),
        },
        Type::Array { element, .. } => HirType::Array(Box::new(lower_type(element))),
        Type::Ai { inner, .. } => HirType::AI(Box::new(lower_type(inner))),
        Type::Function { param, result, .. } => {
            HirType::Function(Box::new(lower_type(param)), Box::new(lower_type(result)))
        }
        Type::Named(name) => HirType::Named(name.name.clone()),
        Type::Effect { inner, .. } => HirType::Effect(Box::new(lower_type(inner)), vec![]),
        _ => HirType::Unit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_empty_program() {
        let program = Program { items: vec![] };
        let hir = lower(&program).unwrap();
        assert!(hir.items.is_empty());
    }
}
