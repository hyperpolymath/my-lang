// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
// Solo Language Parser
// Recursive descent parser with Pratt precedence for expressions

#![forbid(unsafe_code)]
mod error;
mod precedence;

pub use error::{ParseError, ParseResult};
pub use precedence::Precedence;

use my_lang_lexer::{Lexer, Token};
use my_lang_ast::*;
use precedence::{token_to_binary_op, is_right_associative};

/// The parser structure
pub struct Parser {
    tokens: Vec<(Token, String)>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from source code
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_all();
        Self { tokens, pos: 0 }
    }

    // ========== Token Navigation ==========

    /// Get the current token without consuming it
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).map(|(t, _)| t).unwrap_or(&Token::Eof)
    }

    /// Get the current token's text
    fn peek_text(&self) -> &str {
        self.tokens.get(self.pos).map(|(_, s)| s.as_str()).unwrap_or("")
    }

    /// Look ahead n tokens
    fn peek_ahead(&self, n: usize) -> &Token {
        self.tokens.get(self.pos + n).map(|(t, _)| t).unwrap_or(&Token::Eof)
    }

    /// Consume the current token and advance
    fn advance(&mut self) -> (Token, String) {
        let result = self.tokens.get(self.pos).cloned()
            .unwrap_or((Token::Eof, String::new()));
        self.pos += 1;
        result
    }

    /// Check if we're at end of input
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    /// Consume a specific token or return error
    fn expect(&mut self, expected: Token) -> ParseResult<String> {
        if self.peek() == &expected {
            Ok(self.advance().1)
        } else {
            Err(ParseError::unexpected(format!("{:?}", expected), self.peek()))
        }
    }

    /// Try to consume a specific token, return true if successful
    fn match_token(&mut self, expected: &Token) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume an identifier or return error
    fn expect_identifier(&mut self) -> ParseResult<String> {
        if matches!(self.peek(), Token::Identifier) {
            Ok(self.advance().1)
        } else {
            Err(ParseError::MissingIdentifier)
        }
    }

    // ========== Top-Level Parsing ==========

    /// Parse a complete program
    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut items = Vec::new();
        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }
        Ok(Program { items })
    }

    /// Parse a single top-level item
    fn parse_item(&mut self) -> ParseResult<Item> {
        // Check for visibility modifier
        let _is_pub = self.match_token(&Token::Pub);

        // Check for attributes
        let attributes = self.parse_attributes()?;

        // Check for async/comptime modifiers
        let is_async = self.match_token(&Token::Async);
        let is_comptime = self.match_token(&Token::Comptime);

        match self.peek() {
            Token::Fn => self.parse_function(attributes, is_async, is_comptime).map(Item::Function),
            Token::Struct => self.parse_struct().map(Item::Struct),
            Token::Enum => self.parse_enum().map(Item::Enum),
            Token::Trait => self.parse_trait().map(Item::Trait),
            Token::Impl => self.parse_impl().map(Item::Impl),
            Token::Mod => self.parse_module().map(Item::Module),
            Token::Import | Token::Use => self.parse_import().map(Item::Import),
            Token::Const => self.parse_const().map(Item::Const),
            Token::Type => self.parse_type_alias().map(Item::Type),
            Token::Agent => self.parse_agent().map(Item::Agent),
            Token::Workflow => self.parse_workflow().map(Item::Workflow),
            _ => Err(ParseError::InvalidItem),
        }
    }

    /// Parse attributes (#[...])
    fn parse_attributes(&mut self) -> ParseResult<Vec<Attribute>> {
        let mut attrs = Vec::new();
        while self.match_token(&Token::Hash) {
            self.expect(Token::LeftBracket)?;
            let name = self.expect_identifier()?;
            let mut args = Vec::new();

            if self.match_token(&Token::LeftParen) {
                while !self.match_token(&Token::RightParen) {
                    let arg_name = self.expect_identifier()?;
                    self.expect(Token::Equal)?;
                    let arg_value = self.parse_expression()?;
                    args.push((arg_name, arg_value));
                    if !self.match_token(&Token::Comma) {
                        self.expect(Token::RightParen)?;
                        break;
                    }
                }
            }

            self.expect(Token::RightBracket)?;
            attrs.push(Attribute { name, args });
        }
        Ok(attrs)
    }

    // ========== Function Parsing ==========

    fn parse_function(&mut self, attributes: Vec<Attribute>, is_async: bool, is_comptime: bool) -> ParseResult<Function> {
        self.expect(Token::Fn)?;
        let name = self.expect_identifier()?;

        // Parse generics
        let generics = self.parse_generics()?;

        // Parse parameters
        self.expect(Token::LeftParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RightParen)?;

        // Parse return type
        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse where clause
        let where_clause = self.parse_where_clause()?;

        // Parse contract (pre/post conditions)
        let contract = self.parse_contract()?;

        // Parse body
        let body = self.parse_block()?;

        Ok(Function {
            name,
            generics,
            params,
            return_type,
            where_clause,
            contract,
            body,
            is_async,
            is_comptime,
            attributes,
        })
    }

    fn parse_generics(&mut self) -> ParseResult<Vec<Generic>> {
        if !self.match_token(&Token::Less) {
            return Ok(Vec::new());
        }

        let mut generics = Vec::new();
        loop {
            if self.match_token(&Token::Greater) {
                break;
            }

            let name = self.expect_identifier()?;
            let mut bounds = Vec::new();

            if self.match_token(&Token::Colon) {
                // Parse trait bounds
                loop {
                    bounds.push(self.expect_identifier()?);
                    if !self.match_token(&Token::Plus) {
                        break;
                    }
                }
            }

            generics.push(Generic { name, bounds });

            if !self.match_token(&Token::Comma) {
                self.expect(Token::Greater)?;
                break;
            }
        }
        Ok(generics)
    }

    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        while !matches!(self.peek(), Token::RightParen) {
            let is_mut = self.match_token(&Token::Mut);
            let name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;

            params.push(Param { name, ty, is_mut });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_where_clause(&mut self) -> ParseResult<Option<WhereClause>> {
        if !self.match_token(&Token::Where) {
            return Ok(None);
        }

        let mut predicates = Vec::new();
        loop {
            let ty = self.parse_type()?;
            self.expect(Token::Colon)?;

            let mut bounds = Vec::new();
            loop {
                bounds.push(self.expect_identifier()?);
                if !self.match_token(&Token::Plus) {
                    break;
                }
            }

            predicates.push(WherePredicate { ty, bounds });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(Some(WhereClause { predicates }))
    }

    fn parse_contract(&mut self) -> ParseResult<Option<Contract>> {
        let mut preconditions = Vec::new();
        let mut postconditions = Vec::new();
        let mut invariants = Vec::new();

        loop {
            match self.peek() {
                Token::Pre | Token::Requires => {
                    self.advance();
                    preconditions.push(self.parse_expression()?);
                }
                Token::Post | Token::Ensures => {
                    self.advance();
                    postconditions.push(self.parse_expression()?);
                }
                Token::Invariant => {
                    self.advance();
                    invariants.push(self.parse_expression()?);
                }
                _ => break,
            }
        }

        if preconditions.is_empty() && postconditions.is_empty() && invariants.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Contract { preconditions, postconditions, invariants }))
        }
    }

    // ========== Struct Parsing ==========

    fn parse_struct(&mut self) -> ParseResult<Struct> {
        self.expect(Token::Struct)?;
        let name = self.expect_identifier()?;
        let generics = self.parse_generics()?;
        let where_clause = self.parse_where_clause()?;

        self.expect(Token::LeftBrace)?;
        let mut fields = Vec::new();

        while !self.match_token(&Token::RightBrace) {
            let visibility = if self.match_token(&Token::Pub) {
                Visibility::Public
            } else {
                Visibility::Private
            };

            let field_name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;

            fields.push(Field { name: field_name, ty, visibility });

            if !self.match_token(&Token::Comma) {
                self.expect(Token::RightBrace)?;
                break;
            }
        }

        Ok(Struct { name, generics, fields, where_clause })
    }

    // ========== Enum Parsing ==========

    fn parse_enum(&mut self) -> ParseResult<Enum> {
        self.expect(Token::Enum)?;
        let name = self.expect_identifier()?;
        let generics = self.parse_generics()?;

        self.expect(Token::LeftBrace)?;
        let mut variants = Vec::new();

        while !self.match_token(&Token::RightBrace) {
            let variant_name = self.expect_identifier()?;

            let data = if self.match_token(&Token::LeftParen) {
                // Tuple variant
                let mut types = Vec::new();
                while !self.match_token(&Token::RightParen) {
                    types.push(self.parse_type()?);
                    if !self.match_token(&Token::Comma) {
                        self.expect(Token::RightParen)?;
                        break;
                    }
                }
                VariantData::Tuple(types)
            } else if self.match_token(&Token::LeftBrace) {
                // Struct variant
                let mut fields = Vec::new();
                while !self.match_token(&Token::RightBrace) {
                    let field_name = self.expect_identifier()?;
                    self.expect(Token::Colon)?;
                    let ty = self.parse_type()?;
                    fields.push(Field { name: field_name, ty, visibility: Visibility::Public });
                    if !self.match_token(&Token::Comma) {
                        self.expect(Token::RightBrace)?;
                        break;
                    }
                }
                VariantData::Struct(fields)
            } else {
                VariantData::Unit
            };

            variants.push(Variant { name: variant_name, data });

            if !self.match_token(&Token::Comma) {
                self.expect(Token::RightBrace)?;
                break;
            }
        }

        Ok(Enum { name, generics, variants })
    }

    // ========== Trait Parsing ==========

    fn parse_trait(&mut self) -> ParseResult<Trait> {
        self.expect(Token::Trait)?;
        let name = self.expect_identifier()?;
        let generics = self.parse_generics()?;

        self.expect(Token::LeftBrace)?;
        let mut items = Vec::new();

        while !self.match_token(&Token::RightBrace) {
            match self.peek() {
                Token::Fn => {
                    items.push(TraitItem::Function(
                        self.parse_function(Vec::new(), false, false)?
                    ));
                }
                Token::Type => {
                    self.advance();
                    let type_name = self.expect_identifier()?;
                    self.expect(Token::Semicolon)?;
                    items.push(TraitItem::Type(type_name));
                }
                Token::Const => {
                    items.push(TraitItem::Const(self.parse_const()?));
                }
                _ => return Err(ParseError::InvalidItem),
            }
        }

        Ok(Trait { name, generics, items })
    }

    // ========== Impl Parsing ==========

    fn parse_impl(&mut self) -> ParseResult<Impl> {
        self.expect(Token::Impl)?;
        let generics = self.parse_generics()?;

        // Parse either "Type" or "Trait for Type"
        let first_type = self.parse_type()?;

        let (trait_name, self_ty) = if self.match_token(&Token::For) {
            let ty_name = match &first_type {
                Type::Named(n) => Some(n.clone()),
                _ => None,
            };
            (ty_name, self.parse_type()?)
        } else {
            (None, first_type)
        };

        self.expect(Token::LeftBrace)?;
        let mut items = Vec::new();

        while !self.match_token(&Token::RightBrace) {
            let is_pub = self.match_token(&Token::Pub);
            match self.peek() {
                Token::Fn | Token::Async => {
                    let is_async = self.match_token(&Token::Async);
                    items.push(ImplItem::Function(
                        self.parse_function(Vec::new(), is_async, false)?
                    ));
                }
                Token::Type => {
                    items.push(ImplItem::Type(self.parse_type_alias()?));
                }
                Token::Const => {
                    items.push(ImplItem::Const(self.parse_const()?));
                }
                _ => return Err(ParseError::InvalidItem),
            }
        }

        Ok(Impl { generics, trait_name, self_ty, items })
    }

    // ========== Module/Import Parsing ==========

    fn parse_module(&mut self) -> ParseResult<Module> {
        self.expect(Token::Mod)?;
        let name = self.expect_identifier()?;

        self.expect(Token::LeftBrace)?;
        let mut items = Vec::new();

        while !self.match_token(&Token::RightBrace) {
            items.push(self.parse_item()?);
        }

        Ok(Module { name, items })
    }

    fn parse_import(&mut self) -> ParseResult<Import> {
        self.advance(); // consume 'import' or 'use'

        let mut path = Vec::new();
        path.push(self.expect_identifier()?);

        while self.match_token(&Token::ColonColon) {
            path.push(self.expect_identifier()?);
        }

        let alias = if self.match_token(&Token::As) {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect(Token::Semicolon)?;
        Ok(Import { path, alias })
    }

    // ========== Const/Type Alias Parsing ==========

    fn parse_const(&mut self) -> ParseResult<Const> {
        self.expect(Token::Const)?;
        let name = self.expect_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Equal)?;
        let value = self.parse_expression()?;
        self.expect(Token::Semicolon)?;

        Ok(Const { name, ty, value })
    }

    fn parse_type_alias(&mut self) -> ParseResult<TypeAlias> {
        self.expect(Token::Type)?;
        let name = self.expect_identifier()?;
        let generics = self.parse_generics()?;
        self.expect(Token::Equal)?;
        let ty = self.parse_type()?;
        self.expect(Token::Semicolon)?;

        Ok(TypeAlias { name, generics, ty })
    }

    // ========== Agent/Workflow Parsing (Ensemble) ==========

    fn parse_agent(&mut self) -> ParseResult<Agent> {
        self.expect(Token::Agent)?;
        let name = self.expect_identifier()?;
        self.expect(Token::LeftBrace)?;

        let mut state = Vec::new();
        let mut capabilities = Vec::new();
        let mut goals = Vec::new();
        let mut communication = Vec::new();

        while !self.match_token(&Token::RightBrace) {
            match self.peek() {
                Token::State => {
                    self.advance();
                    self.expect(Token::LeftBrace)?;
                    while !self.match_token(&Token::RightBrace) {
                        let field_name = self.expect_identifier()?;
                        self.expect(Token::Colon)?;
                        let ty = self.parse_type()?;
                        state.push(StateField {
                            name: field_name,
                            ty,
                            persistence: Persistence::Persistent,
                        });
                        self.match_token(&Token::Comma);
                    }
                }
                Token::Capabilities => {
                    self.advance();
                    self.expect(Token::LeftBrace)?;
                    while !self.match_token(&Token::RightBrace) {
                        let cap_name = self.expect_identifier()?;
                        self.expect(Token::LeftParen)?;
                        let params = self.parse_params()?;
                        self.expect(Token::RightParen)?;
                        self.expect(Token::Arrow)?;
                        let return_type = self.parse_type()?;
                        capabilities.push(Capability {
                            name: cap_name,
                            params,
                            return_type,
                            kind: CapabilityKind::Action,
                        });
                        self.match_token(&Token::Comma);
                    }
                }
                Token::Goals => {
                    self.advance();
                    self.expect(Token::LeftBrace)?;
                    while !self.match_token(&Token::RightBrace) {
                        let goal_name = self.expect_identifier()?;
                        self.expect(Token::Colon)?;
                        let expr = self.parse_expression()?;
                        goals.push(Goal {
                            name: goal_name,
                            expr,
                            priority: Priority::Medium,
                        });
                        self.match_token(&Token::Comma);
                    }
                }
                Token::Communication => {
                    self.advance();
                    self.expect(Token::LeftBrace)?;
                    while !self.match_token(&Token::RightBrace) {
                        let handler = self.parse_block()?;
                        communication.push(CommunicationHandler {
                            pattern: MessagePattern::Any,
                            handler,
                        });
                    }
                }
                _ => return Err(ParseError::InvalidItem),
            }
        }

        Ok(Agent { name, state, capabilities, goals, communication })
    }

    fn parse_workflow(&mut self) -> ParseResult<Workflow> {
        self.expect(Token::Workflow)?;
        let name = self.expect_identifier()?;
        self.expect(Token::LeftBrace)?;

        let mut stages = Vec::new();
        let coordination = Vec::new();

        while !self.match_token(&Token::RightBrace) {
            // Simple: just parse agent names for now
            let agent_name = self.expect_identifier()?;
            stages.push(Stage::Agent(agent_name));
            self.match_token(&Token::Arrow);
        }

        Ok(Workflow { name, stages, coordination })
    }

    // ========== Block & Statement Parsing ==========

    fn parse_block(&mut self) -> ParseResult<Block> {
        self.expect(Token::LeftBrace)?;

        let mut stmts = Vec::new();
        let mut expr = None;

        while !matches!(self.peek(), Token::RightBrace) {
            // Check if this might be a trailing expression
            let stmt_or_expr = self.parse_statement()?;

            if matches!(self.peek(), Token::RightBrace) {
                // Last item - could be trailing expression
                if let Statement::Expression(e) = stmt_or_expr {
                    expr = Some(Box::new(e));
                } else {
                    stmts.push(stmt_or_expr);
                }
            } else {
                stmts.push(stmt_or_expr);
            }
        }

        self.expect(Token::RightBrace)?;
        Ok(Block { stmts, expr })
    }

    fn parse_statement(&mut self) -> ParseResult<Statement> {
        match self.peek() {
            Token::Let => self.parse_let_statement(),
            Token::Fn | Token::Struct | Token::Enum | Token::Trait | Token::Impl
            | Token::Mod | Token::Const | Token::Type => {
                Ok(Statement::Item(Box::new(self.parse_item()?)))
            }
            _ => {
                let expr = self.parse_expression()?;
                // Expression statements end with semicolon, trailing expressions don't
                if self.match_token(&Token::Semicolon) {
                    // It's a statement
                }
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_let_statement(&mut self) -> ParseResult<Statement> {
        self.expect(Token::Let)?;
        let is_mut = self.match_token(&Token::Mut);
        let pattern = self.parse_pattern()?;

        let ty = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let init = if self.match_token(&Token::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect(Token::Semicolon)?;

        Ok(Statement::Let { pattern, ty, init, is_mut })
    }

    // ========== Expression Parsing (Pratt Parser) ==========

    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_expression_with_precedence(Precedence::None)
    }

    fn parse_expression_with_precedence(&mut self, min_prec: Precedence) -> ParseResult<Expression> {
        let mut left = self.parse_prefix()?;

        while !self.is_at_end() {
            let prec = Precedence::of_binary(self.peek());
            if prec <= min_prec {
                break;
            }

            left = self.parse_infix(left, prec)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> ParseResult<Expression> {
        match self.peek().clone() {
            // Literals
            Token::IntLiteral => {
                let text = self.advance().1;
                let value: i64 = text.parse().map_err(|_| ParseError::InvalidLiteral(text))?;
                Ok(Expression::Literal(Literal::Int(value)))
            }
            Token::FloatLiteral => {
                let text = self.advance().1;
                let value: f64 = text.parse().map_err(|_| ParseError::InvalidLiteral(text))?;
                Ok(Expression::Literal(Literal::Float(value)))
            }
            Token::StringLiteral => {
                let text = self.advance().1;
                // Remove quotes
                let content = text[1..text.len()-1].to_string();
                Ok(Expression::Literal(Literal::String(content)))
            }
            Token::CharLiteral => {
                let text = self.advance().1;
                let ch = text.chars().nth(1).unwrap_or('\0');
                Ok(Expression::Literal(Literal::Char(ch)))
            }
            Token::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(false)))
            }

            // Identifiers
            Token::Identifier => {
                let name = self.advance().1;

                // Check for struct literal
                if self.match_token(&Token::LeftBrace) {
                    let fields = self.parse_struct_fields()?;
                    Ok(Expression::Struct { name, fields })
                } else {
                    Ok(Expression::Identifier(name))
                }
            }

            // Unary operators
            Token::Minus => {
                self.advance();
                let expr = self.parse_expression_with_precedence(Precedence::Unary)?;
                Ok(Expression::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_expression_with_precedence(Precedence::Unary)?;
                Ok(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            Token::Ampersand => {
                self.advance();
                let is_mut = self.match_token(&Token::Mut);
                let expr = self.parse_expression_with_precedence(Precedence::Unary)?;
                Ok(Expression::Unary {
                    op: if is_mut { UnaryOp::RefMut } else { UnaryOp::Ref },
                    expr: Box::new(expr),
                })
            }
            Token::Star => {
                self.advance();
                let expr = self.parse_expression_with_precedence(Precedence::Unary)?;
                Ok(Expression::Unary {
                    op: UnaryOp::Deref,
                    expr: Box::new(expr),
                })
            }

            // Grouping
            Token::LeftParen => {
                self.advance();
                if self.match_token(&Token::RightParen) {
                    return Ok(Expression::Literal(Literal::Unit));
                }

                let expr = self.parse_expression()?;

                // Check for tuple
                if self.match_token(&Token::Comma) {
                    let mut elements = vec![expr];
                    while !self.match_token(&Token::RightParen) {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            self.expect(Token::RightParen)?;
                            break;
                        }
                    }
                    Ok(Expression::Tuple(elements))
                } else {
                    self.expect(Token::RightParen)?;
                    Ok(expr)
                }
            }

            // Arrays
            Token::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !self.match_token(&Token::RightBracket) {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&Token::Comma) {
                        self.expect(Token::RightBracket)?;
                        break;
                    }
                }
                Ok(Expression::Array(elements))
            }

            // Block expression
            Token::LeftBrace => {
                Ok(Expression::Block(self.parse_block()?))
            }

            // Control flow
            Token::If => self.parse_if_expression(),
            Token::Match => self.parse_match_expression(),
            Token::Loop => self.parse_loop_expression(),
            Token::While => self.parse_while_expression(),
            Token::For => self.parse_for_expression(),
            Token::Return => self.parse_return_expression(),
            Token::Break => self.parse_break_expression(),
            Token::Continue => {
                self.advance();
                Ok(Expression::Continue)
            }

            // Ensemble-specific
            Token::Spawn => self.parse_spawn_expression(),
            Token::Send => self.parse_send_expression(),
            Token::Receive => self.parse_receive_expression(),

            _ => Err(ParseError::InvalidExpression),
        }
    }

    fn parse_infix(&mut self, left: Expression, prec: Precedence) -> ParseResult<Expression> {
        match self.peek().clone() {
            // Binary operators
            Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent
            | Token::EqualEqual | Token::NotEqual | Token::Less | Token::Greater
            | Token::LessEqual | Token::GreaterEqual | Token::And | Token::Or
            | Token::Ampersand | Token::Pipe | Token::Caret | Token::LeftShift | Token::RightShift
            | Token::Equal => {
                let (op_token, _) = self.advance();
                let op = token_to_binary_op(&op_token)
                    .ok_or_else(|| ParseError::InvalidOperator(format!("{:?}", op_token)))?;

                let next_prec = if is_right_associative(&op_token) {
                    prec
                } else {
                    prec.next()
                };

                let right = self.parse_expression_with_precedence(next_prec)?;

                Ok(Expression::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                })
            }

            // Function call
            Token::LeftParen => {
                self.advance();
                let args = self.parse_call_args()?;
                self.expect(Token::RightParen)?;
                Ok(Expression::Call {
                    func: Box::new(left),
                    args,
                })
            }

            // Index
            Token::LeftBracket => {
                self.advance();
                let index = self.parse_expression()?;
                self.expect(Token::RightBracket)?;
                Ok(Expression::Index {
                    expr: Box::new(left),
                    index: Box::new(index),
                })
            }

            // Field access / method call
            Token::Dot => {
                self.advance();
                let field = self.expect_identifier()?;

                if self.match_token(&Token::LeftParen) {
                    // Method call
                    let args = self.parse_call_args()?;
                    self.expect(Token::RightParen)?;
                    Ok(Expression::MethodCall {
                        receiver: Box::new(left),
                        method: field,
                        args,
                    })
                } else {
                    // Field access
                    Ok(Expression::Field {
                        expr: Box::new(left),
                        field,
                    })
                }
            }

            _ => Err(ParseError::InvalidExpression),
        }
    }

    fn parse_call_args(&mut self) -> ParseResult<Vec<Expression>> {
        let mut args = Vec::new();
        while !matches!(self.peek(), Token::RightParen) {
            args.push(self.parse_expression()?);
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        Ok(args)
    }

    fn parse_struct_fields(&mut self) -> ParseResult<Vec<(String, Expression)>> {
        let mut fields = Vec::new();
        while !self.match_token(&Token::RightBrace) {
            let name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let value = self.parse_expression()?;
            fields.push((name, value));
            if !self.match_token(&Token::Comma) {
                self.expect(Token::RightBrace)?;
                break;
            }
        }
        Ok(fields)
    }

    // ========== Control Flow Expressions ==========

    fn parse_if_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::If)?;
        let cond = self.parse_expression()?;
        let then_block = self.parse_block()?;

        let else_block = if self.match_token(&Token::Else) {
            if matches!(self.peek(), Token::If) {
                // else if
                let else_if = self.parse_if_expression()?;
                Some(Block {
                    stmts: vec![],
                    expr: Some(Box::new(else_if)),
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Expression::If {
            cond: Box::new(cond),
            then_block,
            else_block,
        })
    }

    fn parse_match_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::Match)?;
        let expr = self.parse_expression()?;
        self.expect(Token::LeftBrace)?;

        let mut arms = Vec::new();
        while !self.match_token(&Token::RightBrace) {
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(&Token::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.expect(Token::FatArrow)?;
            let body = self.parse_expression()?;

            arms.push(MatchArm { pattern, guard, body });

            if !self.match_token(&Token::Comma) {
                self.expect(Token::RightBrace)?;
                break;
            }
        }

        Ok(Expression::Match {
            expr: Box::new(expr),
            arms,
        })
    }

    fn parse_loop_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::Loop)?;
        let body = self.parse_block()?;
        Ok(Expression::Loop(body))
    }

    fn parse_while_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::While)?;
        let cond = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Expression::While {
            cond: Box::new(cond),
            body,
        })
    }

    fn parse_for_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::For)?;
        let pattern = self.parse_pattern()?;
        self.expect(Token::In)?;
        let iter = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Expression::For {
            pattern,
            iter: Box::new(iter),
            body,
        })
    }

    fn parse_return_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::Return)?;
        let value = if matches!(self.peek(), Token::Semicolon | Token::RightBrace) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        Ok(Expression::Return(value))
    }

    fn parse_break_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::Break)?;
        let value = if matches!(self.peek(), Token::Semicolon | Token::RightBrace) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        Ok(Expression::Break(value))
    }

    // ========== Ensemble Expressions ==========

    fn parse_spawn_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::Spawn)?;
        let agent = self.expect_identifier()?;
        let config = if self.match_token(&Token::LeftBrace) {
            self.parse_struct_fields()?
        } else {
            Vec::new()
        };
        Ok(Expression::Spawn { agent, config })
    }

    fn parse_send_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::Send)?;
        let message = self.parse_expression()?;
        // Expect 'to' keyword (using identifier check)
        if self.peek_text() == "to" {
            self.advance();
        }
        let recipient = self.parse_expression()?;
        Ok(Expression::Send {
            message: Box::new(message),
            recipient: Box::new(recipient),
        })
    }

    fn parse_receive_expression(&mut self) -> ParseResult<Expression> {
        self.expect(Token::Receive)?;
        let filter = None; // Simplified
        let timeout = None;
        Ok(Expression::Receive { filter, timeout })
    }

    // ========== Type Parsing ==========

    fn parse_type(&mut self) -> ParseResult<Type> {
        // Check for reference types
        if self.match_token(&Token::Ampersand) {
            let is_mut = self.match_token(&Token::Mut);
            let lifetime = if self.match_token(&Token::Lifetime) {
                Some(self.expect_identifier()?)
            } else {
                None
            };
            let ty = self.parse_type()?;
            return Ok(Type::Reference {
                ty: Box::new(ty),
                is_mut,
                lifetime,
            });
        }

        // Check for affine type
        if self.match_token(&Token::Affine) {
            let ty = self.parse_type()?;
            return Ok(Type::Affine(Box::new(ty)));
        }

        // Check for tuple type
        if self.match_token(&Token::LeftParen) {
            if self.match_token(&Token::RightParen) {
                return Ok(Type::Primitive(PrimitiveType::Unit));
            }

            let first = self.parse_type()?;
            if self.match_token(&Token::Comma) {
                let mut types = vec![first];
                while !self.match_token(&Token::RightParen) {
                    types.push(self.parse_type()?);
                    if !self.match_token(&Token::Comma) {
                        self.expect(Token::RightParen)?;
                        break;
                    }
                }
                return Ok(Type::Tuple(types));
            }
            self.expect(Token::RightParen)?;
            return Ok(first);
        }

        // Check for array type
        if self.match_token(&Token::LeftBracket) {
            let elem = self.parse_type()?;
            let size = if self.match_token(&Token::Semicolon) {
                if let Token::IntLiteral = self.peek() {
                    let text = self.advance().1;
                    Some(text.parse().unwrap_or(0))
                } else {
                    None
                }
            } else {
                None
            };
            self.expect(Token::RightBracket)?;
            return Ok(Type::Array {
                elem: Box::new(elem),
                size,
            });
        }

        // Primitive types
        let ty = match self.peek() {
            Token::I8 => { self.advance(); Type::Primitive(PrimitiveType::I8) }
            Token::I16 => { self.advance(); Type::Primitive(PrimitiveType::I16) }
            Token::I32 => { self.advance(); Type::Primitive(PrimitiveType::I32) }
            Token::I64 => { self.advance(); Type::Primitive(PrimitiveType::I64) }
            Token::I128 => { self.advance(); Type::Primitive(PrimitiveType::I128) }
            Token::Isize => { self.advance(); Type::Primitive(PrimitiveType::Isize) }
            Token::U8 => { self.advance(); Type::Primitive(PrimitiveType::U8) }
            Token::U16 => { self.advance(); Type::Primitive(PrimitiveType::U16) }
            Token::U32 => { self.advance(); Type::Primitive(PrimitiveType::U32) }
            Token::U64 => { self.advance(); Type::Primitive(PrimitiveType::U64) }
            Token::U128 => { self.advance(); Type::Primitive(PrimitiveType::U128) }
            Token::Usize => { self.advance(); Type::Primitive(PrimitiveType::Usize) }
            Token::F32 => { self.advance(); Type::Primitive(PrimitiveType::F32) }
            Token::F64 => { self.advance(); Type::Primitive(PrimitiveType::F64) }
            Token::Bool => { self.advance(); Type::Primitive(PrimitiveType::Bool) }
            Token::Char => { self.advance(); Type::Primitive(PrimitiveType::Char) }
            Token::Str => { self.advance(); Type::Primitive(PrimitiveType::Str) }
            Token::Identifier => {
                let name = self.advance().1;

                // Check for generic args
                if self.match_token(&Token::Less) {
                    let mut args = Vec::new();
                    while !self.match_token(&Token::Greater) {
                        args.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            self.expect(Token::Greater)?;
                            break;
                        }
                    }
                    Type::Generic { name, args }
                } else {
                    Type::Named(name)
                }
            }
            _ => return Err(ParseError::InvalidType(format!("{:?}", self.peek()))),
        };

        // Check for function type
        if self.match_token(&Token::Arrow) {
            let ret = self.parse_type()?;
            if let Type::Tuple(params) = ty {
                return Ok(Type::Function {
                    params,
                    ret: Box::new(ret),
                });
            }
        }

        Ok(ty)
    }

    // ========== Pattern Parsing ==========

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match self.peek().clone() {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::IntLiteral => {
                let text = self.advance().1;
                let value: i64 = text.parse().map_err(|_| ParseError::InvalidLiteral(text))?;
                Ok(Pattern::Literal(Literal::Int(value)))
            }
            Token::StringLiteral => {
                let text = self.advance().1;
                let content = text[1..text.len()-1].to_string();
                Ok(Pattern::Literal(Literal::String(content)))
            }
            Token::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            Token::Identifier => {
                let name = self.advance().1;

                // Check for struct pattern
                if self.match_token(&Token::LeftBrace) {
                    let mut fields = Vec::new();
                    while !self.match_token(&Token::RightBrace) {
                        let field_name = self.expect_identifier()?;
                        let pattern = if self.match_token(&Token::Colon) {
                            self.parse_pattern()?
                        } else {
                            Pattern::Identifier(field_name.clone())
                        };
                        fields.push((field_name, pattern));
                        if !self.match_token(&Token::Comma) {
                            self.expect(Token::RightBrace)?;
                            break;
                        }
                    }
                    Ok(Pattern::Struct { name, fields })
                } else {
                    Ok(Pattern::Identifier(name))
                }
            }
            Token::LeftParen => {
                self.advance();
                let mut patterns = Vec::new();
                while !self.match_token(&Token::RightParen) {
                    patterns.push(self.parse_pattern()?);
                    if !self.match_token(&Token::Comma) {
                        self.expect(Token::RightParen)?;
                        break;
                    }
                }
                Ok(Pattern::Tuple(patterns))
            }
            _ => Err(ParseError::InvalidPattern(format!("{:?}", self.peek()))),
        }
    }
}

/// Convenience function to parse source code
pub fn parse(source: &str) -> ParseResult<Program> {
    let mut parser = Parser::new(source);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_function() {
        let source = "fn main() {}";
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_function_with_params() {
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_let_statement() {
        let source = "fn main() { let x: i32 = 42; }";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_expression() {
        let source = "fn main() { if x > 0 { 1 } else { 0 } }";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_struct() {
        let source = "struct Point { x: i32, y: i32 }";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_enum() {
        let source = "enum Option<T> { Some(T), None }";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_binary_expression() {
        let source = "fn main() { 1 + 2 * 3 }";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function_call() {
        let source = "fn main() { foo(1, 2, 3) }";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_method_call() {
        let source = "fn main() { x.foo().bar(1) }";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_contract() {
        let source = "fn div(a: i32, b: i32) -> i32 pre b != 0 { a / b }";
        let result = parse(source);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        // Test with postcondition too
        let source2 = "fn f(x: i32) -> i32 post x > 0 { x }";
        let result2 = parse(source2);
        assert!(result2.is_ok(), "Parse with post failed: {:?}", result2);
    }
}
