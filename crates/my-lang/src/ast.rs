// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Abstract Syntax Tree definitions for My Language with AI integration.

use crate::token::Span;

/// A complete program consisting of top-level declarations
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Program {
    pub items: Vec<TopLevel>,
}

/// Top-level declarations
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TopLevel {
    Function(FnDecl),
    Struct(StructDecl),
    Effect(EffectDecl),
    Contract(ContractDecl),
    Import(ImportDecl),
    Comptime(ComptimeDecl),
    Arena(ArenaDecl),
    AiModel(AiModelDecl),
    Prompt(PromptDecl),
    Agent(AgentDecl),
    Ensemble(EnsembleDecl),
    Orchestrate(OrchestrationDecl),
}

// ============================================
// AI-First Extensions
// ============================================

/// AI Model Declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AiModelDecl {
    pub name: Ident,
    pub attributes: Vec<AiModelAttr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AiModelAttr {
    Provider(String),
    Model(String),
    Temperature(f64),
    Cache(bool),
}

/// Prompt Declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PromptDecl {
    pub name: Ident,
    pub template: String,
    pub span: Span,
}

// ============================================
// Ensemble Extensions
// ============================================

/// Agent Declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AgentDecl {
    pub name: Ident,
    pub capabilities: Vec<Ident>,
    pub constraints: Vec<Ident>,
    pub properties: Vec<AgentProperty>,
    pub functions: Vec<FnDecl>,
    pub span: Span,
}

/// Agent Properties (stateless, stateful, singleton, model)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AgentProperty {
    Stateless(bool),
    Stateful(bool),
    Singleton(bool),
    Model(String),
}

/// Ensemble Declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnsembleDecl {
    pub name: Ident,
    pub agents: Vec<Ident>,
    pub workflows: Vec<WorkflowDecl>,
    pub span: Span,
}

/// Workflow Declaration (inside ensemble)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkflowDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

/// Orchestration Declaration: `comptime orchestrate Name { ... }`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrchestrationDecl {
    pub name: Ident,
    pub agents: Vec<AgentSpec>,
    pub fusion: FusionRule,
    pub consensus: ConsensusRule,
    pub topology: TopologyType,
    pub span: Span,
}

/// Agent specification in orchestration: `Agent(config)`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AgentSpec {
    pub name: Ident,
    pub config_args: Vec<Expr>,
    pub span: Span,
}

/// Fusion rule for belief combination
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FusionRule {
    Dempster,
    Yager,
    DuboisPrade,
    Custom(String),
}

/// Consensus rule for decision-making
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConsensusRule {
    Threshold(f64),
    Unanimous,
    Quorum(i64, i64), // (required, total)
}

/// Topology type for agent network
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TopologyType {
    Ring,
    Star,
    FullyConnected,
    Custom(String),
}

/// AI Keywords for statements and expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AiKeyword {
    Query,
    Verify,
    Generate,
    Embed,
    Classify,
    Optimize,
    Test,
    Infer,
    Constrain,
    Validate,
}

/// AI Statement
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AiStmt {
    pub keyword: AiKeyword,
    pub body: AiStmtBody,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AiStmtBody {
    Block(Block),
    Expr(Box<Expr>),
}

/// AI Expression
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AiExpr {
    /// ai keyword { body }
    Block {
        keyword: AiKeyword,
        body: Vec<AiBodyItem>,
        span: Span,
    },
    /// ai keyword(args)
    Call {
        keyword: AiKeyword,
        args: Vec<Expr>,
        span: Span,
    },
    /// ai! { "quick query" }
    Quick {
        query: String,
        span: Span,
    },
    /// prompt_name!(args)
    PromptInvocation {
        name: Ident,
        args: Vec<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AiBodyItem {
    Field { name: Ident, value: Expr },
    Literal(String),
}

// ============================================
// Statements
// ============================================

/// A block of statements
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

/// Statement types
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Stmt {
    /// Expression statement: `expr;`
    Expr(Expr),
    /// Let binding: `let [mut] name [: type] = expr;`
    Let {
        mutable: bool,
        name: Ident,
        ty: Option<Type>,
        value: Expr,
        span: Span,
    },
    /// If statement: `if cond { } [else { }]`
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,
        span: Span,
    },
    /// Go statement: `go { }`
    Go {
        block: Block,
        span: Span,
    },
    /// Return statement: `return expr;`
    Return {
        value: Option<Expr>,
        span: Span,
    },
    /// Await statement: `await expr;`
    Await {
        value: Expr,
        span: Span,
    },
    /// Try statement: `try expr [?]`
    Try {
        value: Expr,
        propagate: bool,
        span: Span,
    },
    /// Comptime block: `comptime { }`
    Comptime {
        block: Block,
        span: Span,
    },
    /// AI statement
    Ai(AiStmt),
    /// Belief statement: `belief name: Type where confidence(0.85);`
    Belief {
        name: Ident,
        ty: Type,
        confidence: f64,
        span: Span,
    },
}

// ============================================
// Expressions
// ============================================

/// Expression types
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    /// Identifier
    Ident(Ident),
    /// Function/method call: `expr(args)`
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    /// Field access: `expr.field`
    Field {
        object: Box<Expr>,
        field: Ident,
        span: Span,
    },
    /// Binary operation: `expr op expr`
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    /// Unary operation: `op expr`
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    /// Try expression: `try expr`
    Try {
        operand: Box<Expr>,
        span: Span,
    },
    /// Block expression
    Block(Block),
    /// Restrict expression: `restrict expr`
    Restrict {
        operand: Box<Expr>,
        span: Span,
    },
    /// AI expression
    Ai(AiExpr),
    /// Lambda expression: `|params| => expr` or `|params| { block }`
    Lambda {
        params: Vec<Param>,
        body: LambdaBody,
        span: Span,
    },
    /// Match expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// Array literal
    Array {
        elements: Vec<Expr>,
        span: Span,
    },
    /// Record literal: `{ field: value, ... }`
    Record {
        fields: Vec<RecordField>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LambdaBody {
    Expr(Box<Expr>),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Pattern {
    Literal(Literal),
    Ident(Ident),
    Wildcard(Span),
    Constructor {
        name: Ident,
        args: Vec<Pattern>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecordField {
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BinaryOp {
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
    Assign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnaryOp {
    Neg,
    Not,
    Ref,
    RefMut,
}

// ============================================
// Types
// ============================================

/// Type expressions
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Type {
    /// Primitive types: Int, String, Bool, Float
    Primitive(PrimitiveType),
    /// Named type (identifier)
    Named(Ident),
    /// Function type: `T -> U`
    Function {
        param: Box<Type>,
        result: Box<Type>,
        span: Span,
    },
    /// Effect type: `Effect<T>`
    Effect {
        inner: Box<Type>,
        span: Span,
    },
    /// AI effect type: `AI<T>`
    Ai {
        inner: Box<Type>,
        span: Span,
    },
    /// Reference type: `&T` or `&mut T`
    Reference {
        mutable: bool,
        inner: Box<Type>,
        span: Span,
    },
    /// Array type: `[T]`
    Array {
        element: Box<Type>,
        span: Span,
    },
    /// Record type: `{ field: Type, ... }`
    Record {
        fields: Vec<TypeField>,
        span: Span,
    },
    /// Tuple type: `(T, U, ...)`
    Tuple {
        elements: Vec<Type>,
        span: Span,
    },
    /// Type with AI constraints: `T where ai_check: "..."`
    Constrained {
        base: Box<Type>,
        constraints: Vec<AiConstraint>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveType {
    Int,
    String,
    Bool,
    Float,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeField {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AiConstraint {
    Check(String),
    Valid(String),
    Format(String),
    Infer,
    Custom { name: Ident, value: Expr },
}

// ============================================
// Declarations
// ============================================

/// Function declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FnDecl {
    pub modifiers: Vec<FnModifier>,
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub contract: Option<Contract>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FnModifier {
    Async,
    Safe,
    AiOptimize,
    AiTest,
    AiHint(String),
    AiCache,
    Comptime,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

/// Struct declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructDecl {
    pub modifiers: Vec<StructModifier>,
    pub name: Ident,
    pub type_params: Vec<Ident>,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StructModifier {
    AiGenerate,
    Derive(Vec<Ident>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructField {
    pub modifiers: Vec<FieldModifier>,
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldModifier {
    AiValidate(String),
    AiEmbed,
}

/// Effect declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EffectDecl {
    pub name: Ident,
    pub ops: Vec<EffectOp>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EffectOp {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

/// Contract (pre/post conditions)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Contract {
    pub clauses: Vec<ContractClause>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContractClause {
    Pre(Expr),
    Post(Expr),
    Invariant(Expr),
    AiCheck(String),
    AiEnsure(String),
}

/// Contract declaration (standalone)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContractDecl {
    pub name: Ident,
    pub contract: Contract,
    pub span: Span,
}

/// Comptime declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComptimeDecl {
    pub block: Block,
    pub span: Span,
}

/// Arena declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArenaDecl {
    pub name: Ident,
    pub span: Span,
}

/// Import declaration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportDecl {
    pub path: Vec<Ident>,
    pub items: Option<Vec<Ident>>,
    pub span: Span,
}

// ============================================
// Common Types
// ============================================

/// Identifier
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Literal {
    Int(i64, Span),
    Float(f64, Span),
    String(String, Span),
    Bool(bool, Span),
}

impl Literal {
    pub fn span(&self) -> Span {
        match self {
            Literal::Int(_, s) | Literal::Float(_, s) | Literal::String(_, s) | Literal::Bool(_, s) => *s,
        }
    }
}

// ============================================
// Non-recursive AST teardown (hyperpolymath/my-lang#37)
// ============================================

/// One pending node in the iterative teardown worklist.
enum DropNode {
    Expr(Expr),
    Block(Block),
}

/// Drop an entire [`Program`] without recursing through the AST.
///
/// # Why this exists
///
/// `Expr` is a tree of `Box<Expr>` (plus `Block`/`Stmt`/`Match`/… edges).
/// The compiler-derived `Drop` recurses once per nesting level, so a
/// sufficiently deep AST overflows the thread stack *on drop* — independently
/// of the type checker, and for **any** shape, not just the `Call` chains the
/// former test-only `drop_program_iteratively` helper special-cased
/// (hyperpolymath/my-lang#37, subtlety 1).
///
/// Implementing `Drop for Expr` would be the other option, but it is rejected
/// deliberately: a `Drop` impl forbids by-value destructuring of `Expr`
/// (`cannot move out of a type which implements Drop`), which the parser /
/// HIR / MIR crates rely on pervasively. A standalone teardown keeps the AST a
/// plain movable tree while still giving any owner a non-overflowing drop.
///
/// # How it works
///
/// Owned nodes are pushed onto an explicit heap worklist. Each node is popped
/// **by value** and destructured (legal precisely because `Expr` has no `Drop`
/// impl); every recursive child is moved into the worklist *before* the
/// shallow remainder of the node goes out of scope. The derived drop therefore
/// only ever runs at depth 1. O(n) time, O(n) heap, **O(1) stack** — no
/// recursion and no assumptions about AST shape.
pub fn drop_program_iteratively(mut program: Program) {
    let mut work: Vec<DropNode> = Vec::new();

    // Seed from function bodies — the only place unbounded nesting can arise.
    // Every other `TopLevel` is a shallow declaration and drops at O(1) depth.
    for item in program.items.drain(..) {
        if let TopLevel::Function(f) = item {
            work.push(DropNode::Block(f.body));
        }
    }

    while let Some(node) = work.pop() {
        match node {
            DropNode::Expr(e) => detach_expr_children(e, &mut work),
            DropNode::Block(b) => detach_block_children(b, &mut work),
        }
        // `node` is fully consumed above; its non-recursive remainder (spans,
        // operators, identifiers, literals) drops here at depth 1.
    }
}

fn detach_expr_children(e: Expr, work: &mut Vec<DropNode>) {
    match e {
        Expr::Call { callee, args, .. } => {
            work.push(DropNode::Expr(*callee));
            work.extend(args.into_iter().map(DropNode::Expr));
        }
        Expr::Field { object, .. } => work.push(DropNode::Expr(*object)),
        Expr::Binary { left, right, .. } => {
            work.push(DropNode::Expr(*left));
            work.push(DropNode::Expr(*right));
        }
        Expr::Unary { operand, .. }
        | Expr::Try { operand, .. }
        | Expr::Restrict { operand, .. } => work.push(DropNode::Expr(*operand)),
        Expr::Block(b) => work.push(DropNode::Block(b)),
        Expr::Ai(ai) => detach_ai_expr(ai, work),
        Expr::Lambda { body, .. } => match body {
            LambdaBody::Expr(b) => work.push(DropNode::Expr(*b)),
            LambdaBody::Block(bl) => work.push(DropNode::Block(bl)),
        },
        Expr::Match { scrutinee, arms, .. } => {
            work.push(DropNode::Expr(*scrutinee));
            work.extend(arms.into_iter().map(|a| DropNode::Expr(a.body)));
        }
        Expr::Array { elements, .. } => {
            work.extend(elements.into_iter().map(DropNode::Expr));
        }
        Expr::Record { fields, .. } => {
            work.extend(fields.into_iter().map(|f| DropNode::Expr(f.value)));
        }
        Expr::Literal(_) | Expr::Ident(_) => {}
    }
}

fn detach_ai_expr(ai: AiExpr, work: &mut Vec<DropNode>) {
    match ai {
        AiExpr::Block { body, .. } => {
            for item in body {
                if let AiBodyItem::Field { value, .. } = item {
                    work.push(DropNode::Expr(value));
                }
            }
        }
        AiExpr::Call { args, .. } | AiExpr::PromptInvocation { args, .. } => {
            work.extend(args.into_iter().map(DropNode::Expr));
        }
        AiExpr::Quick { .. } => {}
    }
}

fn detach_block_children(b: Block, work: &mut Vec<DropNode>) {
    for stmt in b.stmts {
        match stmt {
            Stmt::Expr(e)
            | Stmt::Let { value: e, .. }
            | Stmt::Await { value: e, .. }
            | Stmt::Try { value: e, .. } => work.push(DropNode::Expr(e)),
            Stmt::If { condition, then_block, else_block, .. } => {
                work.push(DropNode::Expr(condition));
                work.push(DropNode::Block(then_block));
                if let Some(eb) = else_block {
                    work.push(DropNode::Block(eb));
                }
            }
            Stmt::Go { block, .. } | Stmt::Comptime { block, .. } => {
                work.push(DropNode::Block(block))
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    work.push(DropNode::Expr(v));
                }
            }
            Stmt::Ai(ai) => match ai.body {
                AiStmtBody::Block(bl) => work.push(DropNode::Block(bl)),
                AiStmtBody::Expr(e) => work.push(DropNode::Expr(*e)),
            },
            Stmt::Belief { .. } => {}
        }
    }
}
