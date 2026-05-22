// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! AST Visitor framework for My Language.

use crate::ast::*;

/// Trait for visiting My Language AST nodes.
pub trait Visitor: Sized {
    fn visit_program(&mut self, program: &Program) {
        for item in &program.items { self.visit_top_level(item); }
    }
    fn visit_top_level(&mut self, item: &TopLevel) { walk_top_level(self, item); }
    fn visit_fn_decl(&mut self, func: &FnDecl) { walk_fn_decl(self, func); }
    fn visit_stmt(&mut self, stmt: &Stmt) { walk_stmt(self, stmt); }
    fn visit_expr(&mut self, expr: &Expr) { walk_expr(self, expr); }
    fn visit_pattern(&mut self, _pattern: &Pattern) {}
    fn visit_block(&mut self, block: &Block) {
        for stmt in &block.stmts { self.visit_stmt(stmt); }
    }
    fn visit_param(&mut self, _param: &Param) {}
    fn visit_type(&mut self, _ty: &Type) {}
}

pub fn walk_top_level<V: Visitor>(v: &mut V, item: &TopLevel) {
    match item {
        TopLevel::Function(f) => v.visit_fn_decl(f),
        TopLevel::Struct(s) => {
            for field in &s.fields { v.visit_type(&field.ty); }
        }
        TopLevel::Effect(e) => {
            for op in &e.ops { v.visit_type(&op.ty); }
        }
        TopLevel::Contract(c) => {
            for clause in &c.contract.clauses {
                match clause {
                    ContractClause::Pre(e) | ContractClause::Post(e)
                    | ContractClause::Invariant(e) => v.visit_expr(e),
                    _ => {}
                }
            }
        }
        TopLevel::Import(_) | TopLevel::Arena(_) | TopLevel::AiModel(_)
        | TopLevel::Prompt(_) => {}
        TopLevel::Comptime(c) => v.visit_block(&c.block),
        TopLevel::Agent(a) => {
            for f in &a.functions { v.visit_fn_decl(f); }
        }
        TopLevel::Ensemble(e) => {
            for w in &e.workflows { v.visit_block(&w.body); }
        }
        TopLevel::Orchestrate(_) => {}
    }
}

pub fn walk_fn_decl<V: Visitor>(v: &mut V, func: &FnDecl) {
    for param in &func.params { v.visit_param(param); }
    if let Some(ret) = &func.return_type { v.visit_type(ret); }
    v.visit_block(&func.body);
}

pub fn walk_stmt<V: Visitor>(v: &mut V, stmt: &Stmt) {
    match stmt {
        Stmt::Expr(e) => v.visit_expr(e),
        Stmt::Let { value, ty, .. } => {
            if let Some(t) = ty { v.visit_type(t); }
            v.visit_expr(value);
        }
        Stmt::If { condition, then_block, else_block, .. } => {
            v.visit_expr(condition);
            v.visit_block(then_block);
            if let Some(eb) = else_block { v.visit_block(eb); }
        }
        Stmt::Go { block, .. } => v.visit_block(block),
        Stmt::Return { value, .. } => {
            if let Some(val) = value { v.visit_expr(val); }
        }
        Stmt::Await { value, .. } => v.visit_expr(value),
        Stmt::Try { value, .. } => v.visit_expr(value),
        Stmt::Comptime { block, .. } => v.visit_block(block),
        Stmt::Ai(ai) => {
            match &ai.body {
                AiStmtBody::Block(b) => v.visit_block(b),
                AiStmtBody::Expr(e) => v.visit_expr(e),
            }
        }
        Stmt::Belief { ty, .. } => v.visit_type(ty),
    }
}

pub fn walk_expr<V: Visitor>(v: &mut V, expr: &Expr) {
    match expr {
        Expr::Literal(_) | Expr::Ident(_) => {}
        Expr::Call { callee, args, .. } => {
            v.visit_expr(callee);
            for arg in args { v.visit_expr(arg); }
        }
        Expr::Field { object, .. } => v.visit_expr(object),
        Expr::Binary { left, right, .. } => {
            v.visit_expr(left);
            v.visit_expr(right);
        }
        Expr::Unary { operand, .. } => v.visit_expr(operand),
        Expr::Try { operand, .. } => v.visit_expr(operand),
        Expr::Block(block) => v.visit_block(block),
        Expr::Restrict { operand, .. } => v.visit_expr(operand),
        Expr::Ai(ai_expr) => {
            match ai_expr {
                AiExpr::Block { body, .. } => {
                    for item in body {
                        if let AiBodyItem::Field { value, .. } = item {
                            v.visit_expr(value);
                        }
                    }
                }
                AiExpr::Call { args, .. } => {
                    for arg in args { v.visit_expr(arg); }
                }
                AiExpr::Quick { .. } => {}
                AiExpr::PromptInvocation { args, .. } => {
                    for arg in args { v.visit_expr(arg); }
                }
            }
        }
        Expr::Lambda { params, body, .. } => {
            for p in params { v.visit_param(p); }
            match body {
                LambdaBody::Expr(e) => v.visit_expr(e),
                LambdaBody::Block(b) => v.visit_block(b),
            }
        }
        Expr::Match { scrutinee, arms, .. } => {
            v.visit_expr(scrutinee);
            for arm in arms {
                v.visit_pattern(&arm.pattern);
                v.visit_expr(&arm.body);
            }
        }
        Expr::Array { elements, .. } => {
            for e in elements { v.visit_expr(e); }
        }
        Expr::Record { fields, .. } => {
            for f in fields { v.visit_expr(&f.value); }
        }
    }
}
