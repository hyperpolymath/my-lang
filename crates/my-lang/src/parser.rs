//! Parser for My Language with AI integration
//!
//! Implements a recursive descent parser for the complete grammar.

use crate::ast::*;
use crate::token::{Span, Token, TokenKind};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("unexpected token: expected {expected}, found {found} at line {line}, column {column}")]
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid literal: {0}")]
    InvalidLiteral(String),
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut items = Vec::new();
        while !self.is_at_end() {
            items.push(self.parse_top_level()?);
        }
        Ok(Program { items })
    }

    // ============================================
    // Top-Level Declarations
    // ============================================

    fn parse_top_level(&mut self) -> ParseResult<TopLevel> {
        // Check for modifiers/attributes first
        if self.check(TokenKind::HashBracket) {
            let modifiers = self.parse_attributes()?;
            return self.parse_top_level_with_modifiers(modifiers);
        }

        match self.peek_kind() {
            Some(TokenKind::Fn) => {
                Ok(TopLevel::Function(self.parse_fn_decl(vec![])?))
            }
            Some(TokenKind::Ident) if self.peek_literal() == Some("async") => {
                Ok(TopLevel::Function(self.parse_fn_decl(vec![])?))
            }
            Some(TokenKind::Struct) => Ok(TopLevel::Struct(self.parse_struct_decl(vec![])?)),
            Some(TokenKind::Effect) => Ok(TopLevel::Effect(self.parse_effect_decl()?)),
            Some(TokenKind::Use) => Ok(TopLevel::Import(self.parse_import_decl()?)),
            Some(TokenKind::Comptime) => Ok(TopLevel::Comptime(self.parse_comptime_decl()?)),
            Some(TokenKind::Let) => {
                // Could be arena declaration
                let start = self.current_span();
                self.advance();
                let name = self.parse_ident()?;
                self.expect(TokenKind::Eq)?;
                // Check for Arena::new()
                if self.check(TokenKind::Ident) && self.peek_literal() == Some("Arena") {
                    self.advance();
                    self.expect(TokenKind::ColonColon)?;
                    self.expect_ident("new")?;
                    self.expect(TokenKind::LParen)?;
                    self.expect(TokenKind::RParen)?;
                    self.expect(TokenKind::Semicolon)?;
                    let span = self.span_from(start);
                    Ok(TopLevel::Arena(ArenaDecl { name, span }))
                } else {
                    Err(self.error("Arena::new()"))
                }
            }
            Some(TokenKind::AiModel) => Ok(TopLevel::AiModel(self.parse_ai_model_decl()?)),
            Some(TokenKind::Prompt) => Ok(TopLevel::Prompt(self.parse_prompt_decl()?)),
            _ => Err(self.error("top-level declaration")),
        }
    }

    fn parse_top_level_with_modifiers(&mut self, attrs: Vec<Attribute>) -> ParseResult<TopLevel> {
        match self.peek_kind() {
            Some(TokenKind::Fn) => {
                let modifiers = self.attrs_to_fn_modifiers(attrs);
                Ok(TopLevel::Function(self.parse_fn_decl(modifiers)?))
            }
            Some(TokenKind::Ident) if self.peek_literal() == Some("async") => {
                let modifiers = self.attrs_to_fn_modifiers(attrs);
                Ok(TopLevel::Function(self.parse_fn_decl(modifiers)?))
            }
            Some(TokenKind::Struct) => {
                let modifiers = self.attrs_to_struct_modifiers(attrs);
                Ok(TopLevel::Struct(self.parse_struct_decl(modifiers)?))
            }
            _ => Err(self.error("fn or struct after attributes")),
        }
    }

    fn attrs_to_fn_modifiers(&self, attrs: Vec<Attribute>) -> Vec<FnModifier> {
        attrs.into_iter().filter_map(|a| match a {
            Attribute::Safe => Some(FnModifier::Safe),
            Attribute::AiOptimize => Some(FnModifier::AiOptimize),
            Attribute::AiTest => Some(FnModifier::AiTest),
            Attribute::AiHint(s) => Some(FnModifier::AiHint(s)),
            Attribute::AiCache => Some(FnModifier::AiCache),
            Attribute::Comptime => Some(FnModifier::Comptime),
            _ => None,
        }).collect()
    }

    fn attrs_to_struct_modifiers(&self, attrs: Vec<Attribute>) -> Vec<StructModifier> {
        attrs.into_iter().filter_map(|a| match a {
            Attribute::AiGenerate => Some(StructModifier::AiGenerate),
            Attribute::Derive(items) => Some(StructModifier::Derive(items)),
            _ => None,
        }).collect()
    }

    // ============================================
    // AI Declarations
    // ============================================

    fn parse_ai_model_decl(&mut self) -> ParseResult<AiModelDecl> {
        let start = self.current_span();
        self.expect(TokenKind::AiModel)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut attributes = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            attributes.push(self.parse_ai_model_attr()?);
        }

        self.expect(TokenKind::RBrace)?;
        let span = self.span_from(start);

        Ok(AiModelDecl { name, attributes, span })
    }

    fn parse_ai_model_attr(&mut self) -> ParseResult<AiModelAttr> {
        let attr_name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;

        match attr_name.name.as_str() {
            "provider" => {
                let value = self.parse_string_lit()?;
                Ok(AiModelAttr::Provider(value))
            }
            "model" => {
                let value = self.parse_string_lit()?;
                Ok(AiModelAttr::Model(value))
            }
            "temperature" => {
                let value = self.parse_float_lit()?;
                Ok(AiModelAttr::Temperature(value))
            }
            "cache" => {
                let value = self.parse_bool_lit()?;
                Ok(AiModelAttr::Cache(value))
            }
            _ => Err(self.error("valid ai_model attribute")),
        }
    }

    fn parse_prompt_decl(&mut self) -> ParseResult<PromptDecl> {
        let start = self.current_span();
        self.expect(TokenKind::Prompt)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;
        let template = self.parse_string_lit()?;
        self.expect(TokenKind::RBrace)?;
        let span = self.span_from(start);

        Ok(PromptDecl { name, template, span })
    }

    // ============================================
    // Function Declaration
    // ============================================

    fn parse_fn_decl(&mut self, mut modifiers: Vec<FnModifier>) -> ParseResult<FnDecl> {
        let start = self.current_span();

        // Check for async modifier
        if self.check(TokenKind::Ident) && self.peek_literal() == Some("async") {
            self.advance();
            modifiers.push(FnModifier::Async);
        }

        self.expect(TokenKind::Fn)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RParen)?;

        let return_type = if self.check(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let contract = if self.check(TokenKind::Where) {
            Some(self.parse_contract()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let span = self.span_from(start);

        Ok(FnDecl {
            modifiers,
            name,
            params,
            return_type,
            contract,
            body,
            span,
        })
    }

    fn parse_param_list(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            params.push(self.parse_param()?);
            while self.check(TokenKind::Comma) {
                self.advance();
                if self.check(TokenKind::RParen) {
                    break;
                }
                params.push(self.parse_param()?);
            }
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> ParseResult<Param> {
        let start = self.current_span();
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        let span = self.span_from(start);
        Ok(Param { name, ty, span })
    }

    // ============================================
    // Struct Declaration
    // ============================================

    fn parse_struct_decl(&mut self, modifiers: Vec<StructModifier>) -> ParseResult<StructDecl> {
        let start = self.current_span();
        self.expect(TokenKind::Struct)?;
        let name = self.parse_ident()?;

        let type_params = if self.check(TokenKind::Lt) {
            self.advance();
            let params = self.parse_type_params()?;
            self.expect(TokenKind::Gt)?;
            params
        } else {
            vec![]
        };

        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            fields.push(self.parse_struct_field()?);
        }
        self.expect(TokenKind::RBrace)?;

        let span = self.span_from(start);

        Ok(StructDecl {
            modifiers,
            name,
            type_params,
            fields,
            span,
        })
    }

    fn parse_type_params(&mut self) -> ParseResult<Vec<Ident>> {
        let mut params = vec![self.parse_ident()?];
        while self.check(TokenKind::Comma) {
            self.advance();
            params.push(self.parse_ident()?);
        }
        Ok(params)
    }

    fn parse_struct_field(&mut self) -> ParseResult<StructField> {
        let start = self.current_span();

        let modifiers = if self.check(TokenKind::HashBracket) {
            self.parse_field_modifiers()?
        } else {
            vec![]
        };

        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;

        // Optional trailing comma
        if self.check(TokenKind::Comma) {
            self.advance();
        }

        let span = self.span_from(start);

        Ok(StructField {
            modifiers,
            name,
            ty,
            span,
        })
    }

    fn parse_field_modifiers(&mut self) -> ParseResult<Vec<FieldModifier>> {
        let mut modifiers = Vec::new();
        while self.check(TokenKind::HashBracket) {
            self.advance();
            let name = self.parse_ident()?;
            match name.name.as_str() {
                "ai_validate" => {
                    self.expect(TokenKind::LParen)?;
                    let constraint = self.parse_string_lit()?;
                    self.expect(TokenKind::RParen)?;
                    modifiers.push(FieldModifier::AiValidate(constraint));
                }
                "ai_embed" => {
                    modifiers.push(FieldModifier::AiEmbed);
                }
                _ => {}
            }
            self.expect(TokenKind::RBracket)?;
        }
        Ok(modifiers)
    }

    // ============================================
    // Effect Declaration
    // ============================================

    fn parse_effect_decl(&mut self) -> ParseResult<EffectDecl> {
        let start = self.current_span();
        self.expect(TokenKind::Effect)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut ops = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            ops.push(self.parse_effect_op()?);
        }

        self.expect(TokenKind::RBrace)?;
        let span = self.span_from(start);

        Ok(EffectDecl { name, ops, span })
    }

    fn parse_effect_op(&mut self) -> ParseResult<EffectOp> {
        let start = self.current_span();
        self.expect(TokenKind::Op)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        let span = self.span_from(start);
        Ok(EffectOp { name, ty, span })
    }

    // ============================================
    // Import Declaration
    // ============================================

    fn parse_import_decl(&mut self) -> ParseResult<ImportDecl> {
        let start = self.current_span();
        self.expect(TokenKind::Use)?;

        let mut path = vec![self.parse_ident()?];
        while self.check(TokenKind::ColonColon) {
            self.advance();
            if self.check(TokenKind::LBrace) {
                break;
            }
            path.push(self.parse_ident()?);
        }

        let items = if self.check(TokenKind::ColonColon) {
            self.advance();
            self.expect(TokenKind::LBrace)?;
            let items = self.parse_import_list()?;
            self.expect(TokenKind::RBrace)?;
            Some(items)
        } else if self.check(TokenKind::LBrace) {
            self.expect(TokenKind::LBrace)?;
            let items = self.parse_import_list()?;
            self.expect(TokenKind::RBrace)?;
            Some(items)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;
        let span = self.span_from(start);

        Ok(ImportDecl { path, items, span })
    }

    fn parse_import_list(&mut self) -> ParseResult<Vec<Ident>> {
        let mut items = vec![self.parse_ident()?];
        while self.check(TokenKind::Comma) {
            self.advance();
            if self.check(TokenKind::RBrace) {
                break;
            }
            items.push(self.parse_ident()?);
        }
        Ok(items)
    }

    // ============================================
    // Comptime Declaration
    // ============================================

    fn parse_comptime_decl(&mut self) -> ParseResult<ComptimeDecl> {
        let start = self.current_span();
        self.expect(TokenKind::Comptime)?;
        let block = self.parse_block()?;
        let span = self.span_from(start);
        Ok(ComptimeDecl { block, span })
    }

    // ============================================
    // Contract
    // ============================================

    fn parse_contract(&mut self) -> ParseResult<Contract> {
        let start = self.current_span();
        self.expect(TokenKind::Where)?;

        let mut clauses = vec![self.parse_contract_clause()?];
        while self.check(TokenKind::Comma) {
            self.advance();
            clauses.push(self.parse_contract_clause()?);
        }

        let span = self.span_from(start);
        Ok(Contract { clauses, span })
    }

    fn parse_contract_clause(&mut self) -> ParseResult<ContractClause> {
        match self.peek_kind() {
            Some(TokenKind::Pre) => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(ContractClause::Pre(self.parse_expr()?))
            }
            Some(TokenKind::Post) => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(ContractClause::Post(self.parse_expr()?))
            }
            Some(TokenKind::Invariant) => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(ContractClause::Invariant(self.parse_expr()?))
            }
            Some(TokenKind::AiCheck) => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(ContractClause::AiCheck(self.parse_string_lit()?))
            }
            Some(TokenKind::AiEnsure) => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(ContractClause::AiEnsure(self.parse_string_lit()?))
            }
            _ => Err(self.error("contract clause")),
        }
    }

    // ============================================
    // Blocks and Statements
    // ============================================

    fn parse_block(&mut self) -> ParseResult<Block> {
        let start = self.current_span();
        self.expect(TokenKind::LBrace)?;

        let mut stmts = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }

        self.expect(TokenKind::RBrace)?;
        let span = self.span_from(start);

        Ok(Block { stmts, span })
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek_kind() {
            Some(TokenKind::Let) => self.parse_let_stmt(),
            Some(TokenKind::If) => self.parse_if_stmt(),
            Some(TokenKind::Go) => self.parse_go_stmt(),
            Some(TokenKind::Return) => self.parse_return_stmt(),
            Some(TokenKind::Await) => self.parse_await_stmt(),
            Some(TokenKind::Try) => self.parse_try_stmt(),
            Some(TokenKind::Comptime) => self.parse_comptime_stmt(),
            Some(TokenKind::Ai) => self.parse_ai_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.current_span();
        self.expect(TokenKind::Let)?;

        let mutable = if self.check(TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let name = self.parse_ident()?;

        let ty = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Eq)?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;

        let span = self.span_from(start);
        Ok(Stmt::Let {
            mutable,
            name,
            ty,
            value,
            span,
        })
    }

    fn parse_if_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.current_span();
        self.expect(TokenKind::If)?;
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_block = if self.check(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = self.span_from(start);
        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
            span,
        })
    }

    fn parse_go_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.current_span();
        self.expect(TokenKind::Go)?;
        let block = self.parse_block()?;
        let span = self.span_from(start);
        Ok(Stmt::Go { block, span })
    }

    fn parse_return_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.current_span();
        self.expect(TokenKind::Return)?;

        let value = if !self.check(TokenKind::Semicolon) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;
        let span = self.span_from(start);
        Ok(Stmt::Return { value, span })
    }

    fn parse_await_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.current_span();
        self.expect(TokenKind::Await)?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        let span = self.span_from(start);
        Ok(Stmt::Await { value, span })
    }

    fn parse_try_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.current_span();
        self.expect(TokenKind::Try)?;
        let value = self.parse_expr()?;

        let propagate = if self.check(TokenKind::Question) {
            self.advance();
            true
        } else {
            false
        };

        // Semicolon is optional with ?
        if self.check(TokenKind::Semicolon) {
            self.advance();
        }

        let span = self.span_from(start);
        Ok(Stmt::Try {
            value,
            propagate,
            span,
        })
    }

    fn parse_comptime_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.current_span();
        self.expect(TokenKind::Comptime)?;
        let block = self.parse_block()?;
        let span = self.span_from(start);
        Ok(Stmt::Comptime { block, span })
    }

    fn parse_ai_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.current_span();
        self.expect(TokenKind::Ai)?;
        let keyword = self.parse_ai_keyword()?;

        let body = if self.check(TokenKind::LBrace) {
            AiStmtBody::Block(self.parse_block()?)
        } else {
            AiStmtBody::Expr(Box::new(self.parse_expr()?))
        };

        let span = self.span_from(start);
        Ok(Stmt::Ai(AiStmt { keyword, body, span }))
    }

    fn parse_ai_keyword(&mut self) -> ParseResult<AiKeyword> {
        match self.peek_kind() {
            Some(TokenKind::Query) => { self.advance(); Ok(AiKeyword::Query) }
            Some(TokenKind::Verify) => { self.advance(); Ok(AiKeyword::Verify) }
            Some(TokenKind::Generate) => { self.advance(); Ok(AiKeyword::Generate) }
            Some(TokenKind::Embed) => { self.advance(); Ok(AiKeyword::Embed) }
            Some(TokenKind::Classify) => { self.advance(); Ok(AiKeyword::Classify) }
            Some(TokenKind::Optimize) => { self.advance(); Ok(AiKeyword::Optimize) }
            Some(TokenKind::Test) => { self.advance(); Ok(AiKeyword::Test) }
            Some(TokenKind::Infer) => { self.advance(); Ok(AiKeyword::Infer) }
            Some(TokenKind::Constrain) => { self.advance(); Ok(AiKeyword::Constrain) }
            Some(TokenKind::Validate) => { self.advance(); Ok(AiKeyword::Validate) }
            _ => Err(self.error("AI keyword")),
        }
    }

    fn parse_expr_stmt(&mut self) -> ParseResult<Stmt> {
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Expr(expr))
    }

    // ============================================
    // Expressions
    // ============================================

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_and_expr()?;

        while self.check(TokenKind::OrOr) {
            let start = self.current_span();
            self.advance();
            let right = self.parse_and_expr()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_and_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_equality_expr()?;

        while self.check(TokenKind::AndAnd) {
            let start = self.current_span();
            self.advance();
            let right = self.parse_equality_expr()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_equality_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_comparison_expr()?;

        while let Some(op) = self.match_equality_op() {
            let start = self.current_span();
            self.advance();
            let right = self.parse_comparison_expr()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn match_equality_op(&self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::EqEq) => Some(BinaryOp::Eq),
            Some(TokenKind::BangEq) => Some(BinaryOp::Ne),
            _ => None,
        }
    }

    fn parse_comparison_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_additive_expr()?;

        while let Some(op) = self.match_comparison_op() {
            let start = self.current_span();
            self.advance();
            let right = self.parse_additive_expr()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn match_comparison_op(&self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Lt) => Some(BinaryOp::Lt),
            Some(TokenKind::Gt) => Some(BinaryOp::Gt),
            Some(TokenKind::LtEq) => Some(BinaryOp::Le),
            Some(TokenKind::GtEq) => Some(BinaryOp::Ge),
            _ => None,
        }
    }

    fn parse_additive_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_multiplicative_expr()?;

        while let Some(op) = self.match_additive_op() {
            let start = self.current_span();
            self.advance();
            let right = self.parse_multiplicative_expr()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn match_additive_op(&self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Plus) => Some(BinaryOp::Add),
            Some(TokenKind::Minus) => Some(BinaryOp::Sub),
            _ => None,
        }
    }

    fn parse_multiplicative_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_unary_expr()?;

        while let Some(op) = self.match_multiplicative_op() {
            let start = self.current_span();
            self.advance();
            let right = self.parse_unary_expr()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn match_multiplicative_op(&self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Star) => Some(BinaryOp::Mul),
            Some(TokenKind::Slash) => Some(BinaryOp::Div),
            _ => None,
        }
    }

    fn parse_unary_expr(&mut self) -> ParseResult<Expr> {
        match self.peek_kind() {
            Some(TokenKind::Minus) => {
                let start = self.current_span();
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = self.span_from(start);
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span,
                })
            }
            Some(TokenKind::Bang) => {
                let start = self.current_span();
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = self.span_from(start);
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span,
                })
            }
            Some(TokenKind::Ampersand) => {
                let start = self.current_span();
                self.advance();
                let mutable = if self.check(TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                let operand = self.parse_unary_expr()?;
                let span = self.span_from(start);
                Ok(Expr::Unary {
                    op: if mutable { UnaryOp::RefMut } else { UnaryOp::Ref },
                    operand: Box::new(operand),
                    span,
                })
            }
            Some(TokenKind::Try) => {
                let start = self.current_span();
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = self.span_from(start);
                Ok(Expr::Try {
                    operand: Box::new(operand),
                    span,
                })
            }
            Some(TokenKind::Restrict) => {
                let start = self.current_span();
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = self.span_from(start);
                Ok(Expr::Restrict {
                    operand: Box::new(operand),
                    span,
                })
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            match self.peek_kind() {
                Some(TokenKind::LParen) => {
                    let start = self.current_span();
                    self.advance();
                    let args = self.parse_expr_list()?;
                    self.expect(TokenKind::RParen)?;
                    let span = self.span_from(start);
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        span,
                    };
                }
                Some(TokenKind::Dot) => {
                    let start = self.current_span();
                    self.advance();
                    let field = self.parse_ident()?;
                    let span = self.span_from(start);
                    expr = Expr::Field {
                        object: Box::new(expr),
                        field,
                        span,
                    };
                }
                Some(TokenKind::Bang) => {
                    // Check if this is a prompt invocation (ident!)
                    if let Expr::Ident(ident) = &expr {
                        let start = ident.span;
                        self.advance();
                        let args = if self.check(TokenKind::LParen) {
                            self.advance();
                            let args = self.parse_expr_list()?;
                            self.expect(TokenKind::RParen)?;
                            args
                        } else {
                            vec![]
                        };
                        let span = self.span_from(start);
                        expr = Expr::Ai(AiExpr::PromptInvocation {
                            name: ident.clone(),
                            args,
                            span,
                        });
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary_expr(&mut self) -> ParseResult<Expr> {
        match self.peek_kind() {
            Some(TokenKind::IntLit) => self.parse_int_literal(),
            Some(TokenKind::FloatLit) => self.parse_float_literal(),
            Some(TokenKind::StringLit) => self.parse_string_literal(),
            Some(TokenKind::True) | Some(TokenKind::False) => self.parse_bool_literal(),
            Some(TokenKind::Ident) => self.parse_ident_expr(),
            Some(TokenKind::LParen) => self.parse_paren_expr(),
            Some(TokenKind::LBrace) => self.parse_block_or_record_expr(),
            Some(TokenKind::LBracket) => self.parse_array_expr(),
            Some(TokenKind::Pipe) => self.parse_lambda_expr(),
            Some(TokenKind::Match) => self.parse_match_expr(),
            Some(TokenKind::Ai) => self.parse_ai_expr(),
            Some(TokenKind::AiBang) => self.parse_ai_quick_expr(),
            _ => Err(self.error("expression")),
        }
    }

    fn parse_int_literal(&mut self) -> ParseResult<Expr> {
        let token = self.advance().ok_or(ParseError::UnexpectedEof)?;
        let value: i64 = token.literal.parse()
            .map_err(|_| ParseError::InvalidLiteral(token.literal.clone()))?;
        Ok(Expr::Literal(Literal::Int(value, token.span)))
    }

    fn parse_float_literal(&mut self) -> ParseResult<Expr> {
        let token = self.advance().ok_or(ParseError::UnexpectedEof)?;
        let value: f64 = token.literal.parse()
            .map_err(|_| ParseError::InvalidLiteral(token.literal.clone()))?;
        Ok(Expr::Literal(Literal::Float(value, token.span)))
    }

    fn parse_string_literal(&mut self) -> ParseResult<Expr> {
        let token = self.advance().ok_or(ParseError::UnexpectedEof)?;
        Ok(Expr::Literal(Literal::String(token.literal.clone(), token.span)))
    }

    fn parse_bool_literal(&mut self) -> ParseResult<Expr> {
        let token = self.advance().ok_or(ParseError::UnexpectedEof)?;
        let value = token.kind == TokenKind::True;
        Ok(Expr::Literal(Literal::Bool(value, token.span)))
    }

    fn parse_ident_expr(&mut self) -> ParseResult<Expr> {
        let ident = self.parse_ident()?;
        Ok(Expr::Ident(ident))
    }

    fn parse_paren_expr(&mut self) -> ParseResult<Expr> {
        self.expect(TokenKind::LParen)?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        Ok(expr)
    }

    fn parse_block_or_record_expr(&mut self) -> ParseResult<Expr> {
        let start = self.current_span();
        self.expect(TokenKind::LBrace)?;

        // Check if this is a record literal (starts with ident:)
        if self.check(TokenKind::Ident) {
            let saved_pos = self.pos;
            let ident = self.parse_ident()?;

            if self.check(TokenKind::Colon) {
                // This is a record literal
                self.advance();
                let value = self.parse_expr()?;
                let mut fields = vec![RecordField { name: ident, value }];

                while self.check(TokenKind::Comma) {
                    self.advance();
                    if self.check(TokenKind::RBrace) {
                        break;
                    }
                    let name = self.parse_ident()?;
                    self.expect(TokenKind::Colon)?;
                    let value = self.parse_expr()?;
                    fields.push(RecordField { name, value });
                }

                self.expect(TokenKind::RBrace)?;
                let span = self.span_from(start);
                return Ok(Expr::Record { fields, span });
            }

            // Not a record, restore position and parse as block
            self.pos = saved_pos;
        }

        // Parse as block
        let mut stmts = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RBrace)?;
        let span = self.span_from(start);

        Ok(Expr::Block(Block { stmts, span }))
    }

    fn parse_array_expr(&mut self) -> ParseResult<Expr> {
        let start = self.current_span();
        self.expect(TokenKind::LBracket)?;
        let elements = self.parse_expr_list()?;
        self.expect(TokenKind::RBracket)?;
        let span = self.span_from(start);
        Ok(Expr::Array { elements, span })
    }

    fn parse_lambda_expr(&mut self) -> ParseResult<Expr> {
        let start = self.current_span();
        self.expect(TokenKind::Pipe)?;

        let params = if !self.check(TokenKind::Pipe) {
            self.parse_param_list()?
        } else {
            vec![]
        };

        self.expect(TokenKind::Pipe)?;

        let body = if self.check(TokenKind::FatArrow) {
            self.advance();
            LambdaBody::Expr(Box::new(self.parse_expr()?))
        } else {
            LambdaBody::Block(self.parse_block()?)
        };

        let span = self.span_from(start);
        Ok(Expr::Lambda { params, body, span })
    }

    fn parse_match_expr(&mut self) -> ParseResult<Expr> {
        let start = self.current_span();
        self.expect(TokenKind::Match)?;
        let scrutinee = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            arms.push(self.parse_match_arm()?);
        }

        self.expect(TokenKind::RBrace)?;
        let span = self.span_from(start);

        Ok(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span,
        })
    }

    fn parse_match_arm(&mut self) -> ParseResult<MatchArm> {
        let start = self.current_span();
        let pattern = self.parse_pattern()?;
        self.expect(TokenKind::FatArrow)?;
        let body = self.parse_expr()?;

        // Optional trailing comma
        if self.check(TokenKind::Comma) {
            self.advance();
        }

        let span = self.span_from(start);
        Ok(MatchArm { pattern, body, span })
    }

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match self.peek_kind() {
            Some(TokenKind::IntLit) => {
                let expr = self.parse_int_literal()?;
                if let Expr::Literal(lit) = expr {
                    Ok(Pattern::Literal(lit))
                } else {
                    unreachable!()
                }
            }
            Some(TokenKind::StringLit) => {
                let expr = self.parse_string_literal()?;
                if let Expr::Literal(lit) = expr {
                    Ok(Pattern::Literal(lit))
                } else {
                    unreachable!()
                }
            }
            Some(TokenKind::True) | Some(TokenKind::False) => {
                let expr = self.parse_bool_literal()?;
                if let Expr::Literal(lit) = expr {
                    Ok(Pattern::Literal(lit))
                } else {
                    unreachable!()
                }
            }
            Some(TokenKind::Ident) => {
                let ident = self.parse_ident()?;
                if ident.name == "_" {
                    return Ok(Pattern::Wildcard(ident.span));
                }

                if self.check(TokenKind::LParen) {
                    let start = ident.span;
                    self.advance();
                    let args = self.parse_pattern_list()?;
                    self.expect(TokenKind::RParen)?;
                    let span = self.span_from(start);
                    Ok(Pattern::Constructor {
                        name: ident,
                        args,
                        span,
                    })
                } else {
                    Ok(Pattern::Ident(ident))
                }
            }
            _ => Err(self.error("pattern")),
        }
    }

    fn parse_pattern_list(&mut self) -> ParseResult<Vec<Pattern>> {
        let mut patterns = Vec::new();
        if !self.check(TokenKind::RParen) {
            patterns.push(self.parse_pattern()?);
            while self.check(TokenKind::Comma) {
                self.advance();
                if self.check(TokenKind::RParen) {
                    break;
                }
                patterns.push(self.parse_pattern()?);
            }
        }
        Ok(patterns)
    }

    fn parse_ai_expr(&mut self) -> ParseResult<Expr> {
        let start = self.current_span();
        self.expect(TokenKind::Ai)?;
        let keyword = self.parse_ai_keyword()?;

        if self.check(TokenKind::LBrace) {
            // ai keyword { body }
            self.advance();
            let body = self.parse_ai_body()?;
            self.expect(TokenKind::RBrace)?;
            let span = self.span_from(start);
            Ok(Expr::Ai(AiExpr::Block { keyword, body, span }))
        } else if self.check(TokenKind::LParen) {
            // ai keyword(args)
            self.advance();
            let args = self.parse_expr_list()?;
            self.expect(TokenKind::RParen)?;
            let span = self.span_from(start);
            Ok(Expr::Ai(AiExpr::Call { keyword, args, span }))
        } else {
            Err(self.error("{ or ( after AI keyword"))
        }
    }

    fn parse_ai_quick_expr(&mut self) -> ParseResult<Expr> {
        let start = self.current_span();
        self.expect(TokenKind::AiBang)?;
        self.expect(TokenKind::LBrace)?;
        let query = self.parse_string_lit()?;
        self.expect(TokenKind::RBrace)?;
        let span = self.span_from(start);
        Ok(Expr::Ai(AiExpr::Quick { query, span }))
    }

    fn parse_ai_body(&mut self) -> ParseResult<Vec<AiBodyItem>> {
        let mut items = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            items.push(self.parse_ai_body_item()?);
        }
        Ok(items)
    }

    fn parse_ai_body_item(&mut self) -> ParseResult<AiBodyItem> {
        if self.check(TokenKind::StringLit) {
            let lit = self.parse_string_lit()?;
            Ok(AiBodyItem::Literal(lit))
        } else {
            let name = self.parse_ident()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_expr()?;
            Ok(AiBodyItem::Field { name, value })
        }
    }

    fn parse_expr_list(&mut self) -> ParseResult<Vec<Expr>> {
        let mut exprs = Vec::new();
        if !self.check(TokenKind::RParen) && !self.check(TokenKind::RBracket) {
            exprs.push(self.parse_expr()?);
            while self.check(TokenKind::Comma) {
                self.advance();
                if self.check(TokenKind::RParen) || self.check(TokenKind::RBracket) {
                    break;
                }
                exprs.push(self.parse_expr()?);
            }
        }
        Ok(exprs)
    }

    // ============================================
    // Types
    // ============================================

    fn parse_type(&mut self) -> ParseResult<Type> {
        let base = self.parse_base_type()?;

        // Check for function type arrow
        if self.check(TokenKind::Arrow) {
            let start = self.current_span();
            self.advance();
            let result = self.parse_type()?;
            let span = self.span_from(start);
            return Ok(Type::Function {
                param: Box::new(base),
                result: Box::new(result),
                span,
            });
        }

        // Check for type constraints - but only if followed by AI constraint keywords
        // (not contract clause keywords like pre, post, invariant)
        if self.check(TokenKind::Where) && self.is_ai_constraint_following() {
            let start = self.current_span();
            self.advance();
            let constraints = self.parse_ai_constraints()?;
            let span = self.span_from(start);
            return Ok(Type::Constrained {
                base: Box::new(base),
                constraints,
                span,
            });
        }

        Ok(base)
    }

    fn is_ai_constraint_following(&self) -> bool {
        // Look ahead to see if after 'where' we have an AI constraint keyword
        if self.pos + 1 >= self.tokens.len() {
            return false;
        }
        matches!(
            self.tokens[self.pos + 1].kind,
            TokenKind::AiCheck
                | TokenKind::AiValid
                | TokenKind::AiFormat
                | TokenKind::AiInfer
                | TokenKind::Ident
        ) && !matches!(
            self.tokens[self.pos + 1].kind,
            TokenKind::Pre | TokenKind::Post | TokenKind::Invariant
        )
    }

    fn parse_base_type(&mut self) -> ParseResult<Type> {
        match self.peek_kind() {
            Some(TokenKind::Int) => {
                self.advance();
                Ok(Type::Primitive(PrimitiveType::Int))
            }
            Some(TokenKind::String) => {
                self.advance();
                Ok(Type::Primitive(PrimitiveType::String))
            }
            Some(TokenKind::Bool) => {
                self.advance();
                Ok(Type::Primitive(PrimitiveType::Bool))
            }
            Some(TokenKind::Float) => {
                self.advance();
                Ok(Type::Primitive(PrimitiveType::Float))
            }
            Some(TokenKind::AI) => {
                let start = self.current_span();
                self.advance();
                self.expect(TokenKind::Lt)?;
                let inner = self.parse_type()?;
                self.expect(TokenKind::Gt)?;
                let span = self.span_from(start);
                Ok(Type::Ai {
                    inner: Box::new(inner),
                    span,
                })
            }
            Some(TokenKind::Ident) => {
                let ident = self.parse_ident()?;
                if ident.name == "Effect" && self.check(TokenKind::Lt) {
                    let start = ident.span;
                    self.advance();
                    let inner = self.parse_type()?;
                    self.expect(TokenKind::Gt)?;
                    let span = self.span_from(start);
                    Ok(Type::Effect {
                        inner: Box::new(inner),
                        span,
                    })
                } else {
                    Ok(Type::Named(ident))
                }
            }
            Some(TokenKind::Ampersand) => {
                let start = self.current_span();
                self.advance();
                let mutable = if self.check(TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                let inner = self.parse_base_type()?;
                let span = self.span_from(start);
                Ok(Type::Reference {
                    mutable,
                    inner: Box::new(inner),
                    span,
                })
            }
            Some(TokenKind::LBracket) => {
                let start = self.current_span();
                self.advance();
                let element = self.parse_type()?;
                self.expect(TokenKind::RBracket)?;
                let span = self.span_from(start);
                Ok(Type::Array {
                    element: Box::new(element),
                    span,
                })
            }
            Some(TokenKind::LBrace) => {
                let start = self.current_span();
                self.advance();
                let fields = self.parse_type_fields()?;
                self.expect(TokenKind::RBrace)?;
                let span = self.span_from(start);
                Ok(Type::Record { fields, span })
            }
            Some(TokenKind::LParen) => {
                let start = self.current_span();
                self.advance();
                let mut elements = vec![self.parse_type()?];
                while self.check(TokenKind::Comma) {
                    self.advance();
                    elements.push(self.parse_type()?);
                }
                self.expect(TokenKind::RParen)?;
                let span = self.span_from(start);
                Ok(Type::Tuple { elements, span })
            }
            _ => Err(self.error("type")),
        }
    }

    fn parse_type_fields(&mut self) -> ParseResult<Vec<TypeField>> {
        let mut fields = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let name = self.parse_ident()?;
            self.expect(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            fields.push(TypeField { name, ty });

            if !self.check(TokenKind::RBrace) {
                // Allow optional comma
                if self.check(TokenKind::Comma) {
                    self.advance();
                }
            }
        }
        Ok(fields)
    }

    fn parse_ai_constraints(&mut self) -> ParseResult<Vec<AiConstraint>> {
        let mut constraints = vec![self.parse_ai_constraint()?];
        while self.check(TokenKind::Comma) {
            self.advance();
            constraints.push(self.parse_ai_constraint()?);
        }
        Ok(constraints)
    }

    fn parse_ai_constraint(&mut self) -> ParseResult<AiConstraint> {
        match self.peek_kind() {
            Some(TokenKind::AiCheck) => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(AiConstraint::Check(self.parse_string_lit()?))
            }
            Some(TokenKind::AiValid) => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(AiConstraint::Valid(self.parse_string_lit()?))
            }
            Some(TokenKind::AiFormat) => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(AiConstraint::Format(self.parse_string_lit()?))
            }
            Some(TokenKind::AiInfer) => {
                self.advance();
                Ok(AiConstraint::Infer)
            }
            Some(TokenKind::Ident) => {
                let name = self.parse_ident()?;
                self.expect(TokenKind::Colon)?;
                let value = self.parse_expr()?;
                Ok(AiConstraint::Custom { name, value })
            }
            _ => Err(self.error("AI constraint")),
        }
    }

    // ============================================
    // Attributes
    // ============================================

    fn parse_attributes(&mut self) -> ParseResult<Vec<Attribute>> {
        let mut attrs = Vec::new();
        while self.check(TokenKind::HashBracket) {
            attrs.push(self.parse_attribute()?);
        }
        Ok(attrs)
    }

    fn parse_attribute(&mut self) -> ParseResult<Attribute> {
        self.expect(TokenKind::HashBracket)?;
        let name = self.parse_ident()?;

        let attr = match name.name.as_str() {
            "safe" => Attribute::Safe,
            "ai_optimize" => Attribute::AiOptimize,
            "ai_test" => Attribute::AiTest,
            "ai_cache" => Attribute::AiCache,
            "comptime" => Attribute::Comptime,
            "ai_generate" => Attribute::AiGenerate,
            "ai_hint" => {
                self.expect(TokenKind::LParen)?;
                let hint = self.parse_string_lit()?;
                self.expect(TokenKind::RParen)?;
                Attribute::AiHint(hint)
            }
            "derive" => {
                self.expect(TokenKind::LParen)?;
                let items = self.parse_derive_list()?;
                self.expect(TokenKind::RParen)?;
                Attribute::Derive(items)
            }
            _ => Attribute::Custom(name.name),
        };

        self.expect(TokenKind::RBracket)?;
        Ok(attr)
    }

    fn parse_derive_list(&mut self) -> ParseResult<Vec<Ident>> {
        let mut items = vec![self.parse_ident()?];
        while self.check(TokenKind::Comma) {
            self.advance();
            items.push(self.parse_ident()?);
        }
        Ok(items)
    }

    // ============================================
    // Helper Methods
    // ============================================

    fn parse_ident(&mut self) -> ParseResult<Ident> {
        // Allow certain keywords to be used as identifiers
        if self.is_keyword_as_ident() || self.check(TokenKind::Ident) {
            let token = self.advance().ok_or(ParseError::UnexpectedEof)?;
            Ok(Ident::new(token.literal, token.span))
        } else {
            Err(self.error("identifier"))
        }
    }

    fn is_keyword_as_ident(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(TokenKind::Query)
                | Some(TokenKind::Verify)
                | Some(TokenKind::Generate)
                | Some(TokenKind::Embed)
                | Some(TokenKind::Classify)
                | Some(TokenKind::Optimize)
                | Some(TokenKind::Test)
                | Some(TokenKind::Infer)
                | Some(TokenKind::Constrain)
                | Some(TokenKind::Validate)
                | Some(TokenKind::Prompt)
                | Some(TokenKind::Pre)
                | Some(TokenKind::Post)
                | Some(TokenKind::Invariant)
                | Some(TokenKind::Comptime)
        )
    }

    fn expect_ident(&mut self, expected: &str) -> ParseResult<()> {
        if self.peek_literal() != Some(expected) {
            return Err(self.error(expected));
        }
        self.advance();
        Ok(())
    }

    fn parse_string_lit(&mut self) -> ParseResult<String> {
        if !self.check(TokenKind::StringLit) {
            return Err(self.error("string literal"));
        }
        let token = self.advance().ok_or(ParseError::UnexpectedEof)?;
        Ok(token.literal)
    }

    fn parse_float_lit(&mut self) -> ParseResult<f64> {
        if !self.check(TokenKind::FloatLit) && !self.check(TokenKind::IntLit) {
            return Err(self.error("number"));
        }
        let token = self.advance().ok_or(ParseError::UnexpectedEof)?;
        token.literal.parse()
            .map_err(|_| ParseError::InvalidLiteral(token.literal))
    }

    fn parse_bool_lit(&mut self) -> ParseResult<bool> {
        match self.peek_kind() {
            Some(TokenKind::True) => {
                self.advance();
                Ok(true)
            }
            Some(TokenKind::False) => {
                self.advance();
                Ok(false)
            }
            _ => Err(self.error("boolean")),
        }
    }

    fn expect(&mut self, kind: TokenKind) -> ParseResult<Token> {
        if !self.check(kind.clone()) {
            return Err(self.error(&kind.to_string()));
        }
        self.advance().ok_or(ParseError::UnexpectedEof)
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(kind)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind.clone())
    }

    fn peek_literal(&self) -> Option<&str> {
        self.peek().map(|t| t.literal.as_str())
    }

    fn advance(&mut self) -> Option<Token> {
        if self.is_at_end() {
            return None;
        }
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        Some(token)
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind() == Some(TokenKind::Eof) || self.pos >= self.tokens.len()
    }

    fn current_span(&self) -> Span {
        self.peek().map(|t| t.span).unwrap_or_default()
    }

    fn span_from(&self, start: Span) -> Span {
        let end = if self.pos > 0 {
            self.tokens[self.pos - 1].span.end
        } else {
            start.end
        };
        Span::new(start.start, end, start.line, start.column)
    }

    fn error(&self, expected: &str) -> ParseError {
        let (found, line, column) = if let Some(token) = self.peek() {
            (token.kind.to_string(), token.span.line, token.span.column)
        } else {
            ("end of input".to_string(), 0, 0)
        };

        ParseError::UnexpectedToken {
            expected: expected.to_string(),
            found,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Attribute {
    Safe,
    AiOptimize,
    AiTest,
    AiHint(String),
    AiCache,
    Comptime,
    AiGenerate,
    Derive(Vec<Ident>),
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(input: &str) -> ParseResult<Program> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    #[test]
    fn test_simple_function() {
        let program = parse("fn main() { }").unwrap();
        assert_eq!(program.items.len(), 1);
        if let TopLevel::Function(f) = &program.items[0] {
            assert_eq!(f.name.name, "main");
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_function_with_params() {
        let program = parse("fn add(x: Int, y: Int) -> Int { }").unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            assert_eq!(f.params.len(), 2);
            assert!(f.return_type.is_some());
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_ai_model_decl() {
        let input = r#"
            ai_model gpt4 {
                provider: "openai"
                model: "gpt-4"
                temperature: 0.7
                cache: true
            }
        "#;
        let program = parse(input).unwrap();
        if let TopLevel::AiModel(m) = &program.items[0] {
            assert_eq!(m.name.name, "gpt4");
            assert_eq!(m.attributes.len(), 4);
        } else {
            panic!("Expected ai_model");
        }
    }

    #[test]
    fn test_prompt_decl() {
        let input = r#"prompt greeting { "Hello, {name}!" }"#;
        let program = parse(input).unwrap();
        if let TopLevel::Prompt(p) = &program.items[0] {
            assert_eq!(p.name.name, "greeting");
            assert_eq!(p.template, "Hello, {name}!");
        } else {
            panic!("Expected prompt");
        }
    }

    #[test]
    fn test_struct_with_ai_attrs() {
        let input = r#"
            #[ai_generate]
            struct User {
                #[ai_validate("valid email")]
                email: String,
                #[ai_embed]
                bio: String,
            }
        "#;
        let program = parse(input).unwrap();
        if let TopLevel::Struct(s) = &program.items[0] {
            assert_eq!(s.name.name, "User");
            assert_eq!(s.modifiers.len(), 1);
            assert_eq!(s.fields.len(), 2);
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_ai_type() {
        let input = "fn query() -> AI<String> { }";
        let program = parse(input).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            if let Some(Type::Ai { inner, .. }) = &f.return_type {
                assert!(matches!(inner.as_ref(), Type::Primitive(PrimitiveType::String)));
            } else {
                panic!("Expected AI type");
            }
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_contract() {
        let input = r#"
            fn divide(a: Int, b: Int) -> Int
                where pre: b != 0, ai_check: "divisor is not zero"
            { }
        "#;
        let program = parse(input).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            let contract = f.contract.as_ref().unwrap();
            assert_eq!(contract.clauses.len(), 2);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_let_stmt() {
        let input = "fn main() { let x: Int = 42; }";
        let program = parse(input).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            assert_eq!(f.body.stmts.len(), 1);
            if let Stmt::Let { name, .. } = &f.body.stmts[0] {
                assert_eq!(name.name, "x");
            } else {
                panic!("Expected let stmt");
            }
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_ai_quick_expr() {
        let input = r#"fn main() { ai! { "What is 2+2?" }; }"#;
        let program = parse(input).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            if let Stmt::Expr(Expr::Ai(AiExpr::Quick { query, .. })) = &f.body.stmts[0] {
                assert_eq!(query, "What is 2+2?");
            } else {
                panic!("Expected ai! expression");
            }
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_ai_stmt() {
        let input = "fn main() { ai query { } }";
        let program = parse(input).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            if let Stmt::Ai(ai_stmt) = &f.body.stmts[0] {
                assert_eq!(ai_stmt.keyword, AiKeyword::Query);
            } else {
                panic!("Expected ai statement");
            }
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_lambda() {
        let input = "fn main() { |x: Int| => x + 1; }";
        let program = parse(input).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            if let Stmt::Expr(Expr::Lambda { params, .. }) = &f.body.stmts[0] {
                assert_eq!(params.len(), 1);
            } else {
                panic!("Expected lambda");
            }
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_match_expr() {
        let input = r#"
            fn main() {
                match x {
                    1 => "one",
                    2 => "two",
                    _ => "other",
                };
            }
        "#;
        let program = parse(input).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            if let Stmt::Expr(Expr::Match { arms, .. }) = &f.body.stmts[0] {
                assert_eq!(arms.len(), 3);
            } else {
                panic!("Expected match");
            }
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_type_constraint() {
        let input = r#"fn check(email: String where ai_valid: "email") { }"#;
        let program = parse(input).unwrap();
        if let TopLevel::Function(f) = &program.items[0] {
            if let Type::Constrained { constraints, .. } = &f.params[0].ty {
                assert_eq!(constraints.len(), 1);
            } else {
                panic!("Expected constrained type");
            }
        } else {
            panic!("Expected function");
        }
    }
}
