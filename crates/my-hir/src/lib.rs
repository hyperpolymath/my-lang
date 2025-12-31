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

use my_lang::{
    Program, TopLevel, FnDecl, StructDecl, EffectDecl, AiModelDecl,
    Type, PrimitiveType, AiModelAttr, Block, Stmt, Expr, Literal,
    BinaryOp, UnaryOp, Pattern, MatchArm, LambdaBody, AiExpr, AiKeyword,
};
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
        body: lower_block(&f.body)?,
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

fn lower_block(block: &Block) -> Result<HirBlock, HirError> {
    let mut stmts = Vec::new();
    let mut final_expr = None;

    for (i, stmt) in block.stmts.iter().enumerate() {
        let is_last = i == block.stmts.len() - 1;

        match stmt {
            // If last statement is expression without semicolon, it's the block's value
            Stmt::Expr(expr) if is_last => {
                final_expr = Some(Box::new(lower_expr(expr)?));
            }
            _ => {
                stmts.push(lower_stmt(stmt)?);
            }
        }
    }

    Ok(HirBlock {
        stmts,
        expr: final_expr,
    })
}

fn lower_stmt(stmt: &Stmt) -> Result<HirStmt, HirError> {
    match stmt {
        Stmt::Let { name, ty, value, .. } => Ok(HirStmt::Let {
            name: name.name.clone(),
            ty: ty.as_ref().map(lower_type),
            value: lower_expr(value)?,
        }),
        Stmt::Expr(expr) => Ok(HirStmt::Expr(lower_expr(expr)?)),
        Stmt::Return { value, .. } => Ok(HirStmt::Return(
            value.as_ref().map(lower_expr).transpose()?,
        )),
        Stmt::If { condition, then_block, else_block, .. } => {
            // Desugar if statement to expression statement
            let hir_if = HirExpr::If(
                Box::new(lower_expr(condition)?),
                lower_block(then_block)?,
                else_block.as_ref().map(lower_block).transpose()?,
            );
            Ok(HirStmt::Expr(hir_if))
        }
        Stmt::Go { block, .. } => {
            // Lower go blocks as async expressions (placeholder)
            Ok(HirStmt::Expr(HirExpr::Block(lower_block(block)?)))
        }
        Stmt::Await { value, .. } => {
            // Await is lowered as a regular expression for now
            Ok(HirStmt::Expr(lower_expr(value)?))
        }
        Stmt::Try { value, .. } => {
            Ok(HirStmt::Expr(lower_expr(value)?))
        }
        Stmt::Comptime { block, .. } => {
            // Comptime blocks are evaluated at compile time (placeholder)
            Ok(HirStmt::Expr(HirExpr::Block(lower_block(block)?)))
        }
        Stmt::Ai(ai_stmt) => {
            // Lower AI statement to AI expression
            let hir_ai = lower_ai_keyword_expr(ai_stmt.keyword, &ai_stmt.body)?;
            Ok(HirStmt::Expr(hir_ai))
        }
    }
}

fn lower_expr(expr: &Expr) -> Result<HirExpr, HirError> {
    match expr {
        Expr::Literal(lit) => Ok(HirExpr::Literal(lower_literal(lit))),
        Expr::Ident(ident) => Ok(HirExpr::Var(ident.name.clone())),
        Expr::Call { callee, args, .. } => Ok(HirExpr::Call(
            Box::new(lower_expr(callee)?),
            args.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?,
        )),
        Expr::Field { object, field, .. } => Ok(HirExpr::Field(
            Box::new(lower_expr(object)?),
            field.name.clone(),
        )),
        Expr::Binary { left, op, right, .. } => Ok(HirExpr::BinOp(
            Box::new(lower_expr(left)?),
            lower_binop(*op),
            Box::new(lower_expr(right)?),
        )),
        Expr::Unary { op, operand, .. } => Ok(HirExpr::UnOp(
            lower_unop(*op),
            Box::new(lower_expr(operand)?),
        )),
        Expr::Try { operand, .. } => {
            // Try expressions are lowered as the operand (error handling in MIR)
            lower_expr(operand)
        }
        Expr::Block(block) => Ok(HirExpr::Block(lower_block(block)?)),
        Expr::Restrict { operand, .. } => {
            // Restrict expressions pass through (checked separately)
            lower_expr(operand)
        }
        Expr::Ai(ai_expr) => lower_ai_expr(ai_expr),
        Expr::Lambda { params, body, .. } => {
            let hir_params: Vec<HirParam> = params
                .iter()
                .map(|p| HirParam {
                    name: p.name.name.clone(),
                    ty: lower_type(&p.ty),
                })
                .collect();

            let hir_body = match body {
                LambdaBody::Expr(e) => lower_expr(e)?,
                LambdaBody::Block(b) => HirExpr::Block(lower_block(b)?),
            };

            Ok(HirExpr::Lambda(hir_params, Box::new(hir_body)))
        }
        Expr::Match { scrutinee, arms, .. } => Ok(HirExpr::Match(
            Box::new(lower_expr(scrutinee)?),
            arms.iter().map(lower_match_arm).collect::<Result<Vec<_>, _>>()?,
        )),
        Expr::Array { elements, .. } => Ok(HirExpr::Array(
            elements.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?,
        )),
        Expr::Record { fields, .. } => Ok(HirExpr::Record(
            fields
                .iter()
                .map(|f| Ok((f.name.name.clone(), lower_expr(&f.value)?)))
                .collect::<Result<Vec<_>, HirError>>()?,
        )),
    }
}

fn lower_literal(lit: &Literal) -> HirLiteral {
    match lit {
        Literal::Int(v, _) => HirLiteral::Int(*v),
        Literal::Float(v, _) => HirLiteral::Float(*v),
        Literal::String(v, _) => HirLiteral::String(v.clone()),
        Literal::Bool(v, _) => HirLiteral::Bool(*v),
    }
}

fn lower_binop(op: BinaryOp) -> HirBinOp {
    match op {
        BinaryOp::Add => HirBinOp::Add,
        BinaryOp::Sub => HirBinOp::Sub,
        BinaryOp::Mul => HirBinOp::Mul,
        BinaryOp::Div => HirBinOp::Div,
        BinaryOp::Eq => HirBinOp::Eq,
        BinaryOp::Ne => HirBinOp::Ne,
        BinaryOp::Lt => HirBinOp::Lt,
        BinaryOp::Gt => HirBinOp::Gt,
        BinaryOp::Le => HirBinOp::Le,
        BinaryOp::Ge => HirBinOp::Ge,
        BinaryOp::And => HirBinOp::And,
        BinaryOp::Or => HirBinOp::Or,
        BinaryOp::Assign => HirBinOp::Add, // Assignments handled separately
    }
}

fn lower_unop(op: UnaryOp) -> HirUnOp {
    match op {
        UnaryOp::Neg => HirUnOp::Neg,
        UnaryOp::Not => HirUnOp::Not,
        UnaryOp::Ref => HirUnOp::Ref,
        UnaryOp::RefMut => HirUnOp::RefMut,
    }
}

fn lower_match_arm(arm: &MatchArm) -> Result<HirArm, HirError> {
    Ok(HirArm {
        pattern: lower_pattern(&arm.pattern)?,
        guard: None, // TODO: Add guard support if needed
        body: lower_expr(&arm.body)?,
    })
}

fn lower_pattern(pattern: &Pattern) -> Result<HirPattern, HirError> {
    match pattern {
        Pattern::Literal(lit) => Ok(HirPattern::Literal(lower_literal(lit))),
        Pattern::Ident(ident) => Ok(HirPattern::Var(ident.name.clone())),
        Pattern::Wildcard(_) => Ok(HirPattern::Wildcard),
        Pattern::Constructor { name, args, .. } => Ok(HirPattern::Constructor(
            name.name.clone(),
            args.iter().map(lower_pattern).collect::<Result<Vec<_>, _>>()?,
        )),
    }
}

fn lower_ai_expr(ai_expr: &AiExpr) -> Result<HirExpr, HirError> {
    match ai_expr {
        AiExpr::Block { keyword, body, .. } => {
            // Extract prompt from body items
            let prompt = body
                .iter()
                .filter_map(|item| match item {
                    my_lang::AiBodyItem::Literal(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ");

            Ok(HirExpr::AI(match keyword {
                AiKeyword::Query => HirAIExpr::Query {
                    model: None,
                    prompt: Box::new(HirExpr::Literal(HirLiteral::String(prompt))),
                },
                AiKeyword::Verify => HirAIExpr::Verify {
                    condition: Box::new(HirExpr::Literal(HirLiteral::String(prompt))),
                },
                AiKeyword::Embed => HirAIExpr::Embed {
                    input: Box::new(HirExpr::Literal(HirLiteral::String(prompt))),
                },
                AiKeyword::Generate => HirAIExpr::Generate {
                    template: prompt,
                    params: vec![],
                },
                _ => HirAIExpr::Query {
                    model: None,
                    prompt: Box::new(HirExpr::Literal(HirLiteral::String(prompt))),
                },
            }))
        }
        AiExpr::Call { keyword, args, .. } => {
            let hir_args: Vec<HirExpr> = args
                .iter()
                .map(lower_expr)
                .collect::<Result<Vec<_>, _>>()?;

            let prompt = if hir_args.is_empty() {
                Box::new(HirExpr::Literal(HirLiteral::String(String::new())))
            } else {
                Box::new(hir_args.into_iter().next().unwrap())
            };

            Ok(HirExpr::AI(match keyword {
                AiKeyword::Query => HirAIExpr::Query { model: None, prompt },
                AiKeyword::Verify => HirAIExpr::Verify { condition: prompt },
                AiKeyword::Embed => HirAIExpr::Embed { input: prompt },
                _ => HirAIExpr::Query { model: None, prompt },
            }))
        }
        AiExpr::Quick { query, .. } => Ok(HirExpr::AI(HirAIExpr::Query {
            model: None,
            prompt: Box::new(HirExpr::Literal(HirLiteral::String(query.clone()))),
        })),
        AiExpr::PromptInvocation { name, args, .. } => {
            let hir_args: Vec<HirExpr> = args
                .iter()
                .map(lower_expr)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(HirExpr::AI(HirAIExpr::Generate {
                template: name.name.clone(),
                params: hir_args,
            }))
        }
    }
}

fn lower_ai_keyword_expr(keyword: AiKeyword, body: &my_lang::AiStmtBody) -> Result<HirExpr, HirError> {
    match body {
        my_lang::AiStmtBody::Block(block) => {
            let hir_block = lower_block(block)?;
            Ok(HirExpr::AI(match keyword {
                AiKeyword::Query => HirAIExpr::Query {
                    model: None,
                    prompt: Box::new(HirExpr::Block(hir_block)),
                },
                AiKeyword::Verify => HirAIExpr::Verify {
                    condition: Box::new(HirExpr::Block(hir_block)),
                },
                _ => HirAIExpr::Query {
                    model: None,
                    prompt: Box::new(HirExpr::Block(hir_block)),
                },
            }))
        }
        my_lang::AiStmtBody::Expr(expr) => {
            let hir_expr = lower_expr(expr)?;
            Ok(HirExpr::AI(match keyword {
                AiKeyword::Query => HirAIExpr::Query {
                    model: None,
                    prompt: Box::new(hir_expr),
                },
                AiKeyword::Verify => HirAIExpr::Verify {
                    condition: Box::new(hir_expr),
                },
                _ => HirAIExpr::Query {
                    model: None,
                    prompt: Box::new(hir_expr),
                },
            }))
        }
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
