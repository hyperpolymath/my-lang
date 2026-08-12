// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

// Abstract Syntax Tree for Solo/Duet/Ensemble

#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    Module(Module),
    Import(Import),
    Const(Const),
    Type(TypeAlias),
    // Duet-specific
    SynthFunction(SynthFunction),
    VerifyFunction(VerifyFunction),
    // Ensemble-specific
    Agent(Agent),
    Workflow(Workflow),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub generics: Vec<Generic>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub where_clause: Option<WhereClause>,
    pub contract: Option<Contract>,
    pub body: Block,
    pub is_async: bool,
    pub is_comptime: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub is_mut: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub stmts: Vec<Statement>,
    pub expr: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Let {
        pattern: Pattern,
        ty: Option<Type>,
        init: Option<Expression>,
        is_mut: bool,
    },
    Expression(Expression),
    Item(Box<Item>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Call {
        func: Box<Expression>,
        args: Vec<Expression>,
    },
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    If {
        cond: Box<Expression>,
        then_block: Block,
        else_block: Option<Block>,
    },
    Match {
        expr: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    Loop(Block),
    While {
        cond: Box<Expression>,
        body: Block,
    },
    For {
        pattern: Pattern,
        iter: Box<Expression>,
        body: Block,
    },
    Return(Option<Box<Expression>>),
    Break(Option<Box<Expression>>),
    Continue,
    Block(Block),
    Tuple(Vec<Expression>),
    Array(Vec<Expression>),
    Index {
        expr: Box<Expression>,
        index: Box<Expression>,
    },
    Field {
        expr: Box<Expression>,
        field: String,
    },
    Struct {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    // Duet-specific
    Intent {
        description: String,
        options: Vec<(String, Expression)>,
    },
    Synth {
        config: Vec<(String, Expression)>,
        expr: Box<Expression>,
    },
    Verify {
        config: Vec<(String, Expression)>,
        expr: Box<Expression>,
    },
    Hybrid {
        symbolic: Box<Expression>,
        neural: Box<Expression>,
        fusion: FusionStrategy,
    },
    // Ensemble-specific
    Spawn {
        agent: String,
        config: Vec<(String, Expression)>,
    },
    Send {
        message: Box<Expression>,
        recipient: Box<Expression>,
    },
    Receive {
        filter: Option<Box<Expression>>,
        timeout: Option<Box<Expression>>,
    },
    Broadcast {
        message: Box<Expression>,
        targets: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Primitive(PrimitiveType),
    Named(String),
    Generic {
        name: String,
        args: Vec<Type>,
    },
    Tuple(Vec<Type>),
    Array {
        elem: Box<Type>,
        size: Option<usize>,
    },
    Reference {
        ty: Box<Type>,
        is_mut: bool,
        lifetime: Option<String>,
    },
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    Affine(Box<Type>),
    // Duet-specific
    Inferred,
    Learned(Vec<Expression>),
    Fuzzy(Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveType {
    I8, I16, I32, I64, I128, Isize,
    U8, U16, U32, U64, U128, Usize,
    F32, F64,
    Bool,
    Char,
    Str,
    Unit,
    Never,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Assign,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg, Not, Deref, Ref, RefMut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    pub preconditions: Vec<Expression>,
    pub postconditions: Vec<Expression>,
    pub invariants: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Struct {
    pub name: String,
    pub generics: Vec<Generic>,
    pub fields: Vec<Field>,
    pub where_clause: Option<WhereClause>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub generics: Vec<Generic>,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub name: String,
    pub data: VariantData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariantData {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<Field>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trait {
    pub name: String,
    pub generics: Vec<Generic>,
    pub items: Vec<TraitItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraitItem {
    Function(Function),
    Type(String),
    Const(Const),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Impl {
    pub generics: Vec<Generic>,
    pub trait_name: Option<String>,
    pub self_ty: Type,
    pub items: Vec<ImplItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplItem {
    Function(Function),
    Type(TypeAlias),
    Const(Const),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Const {
    pub name: String,
    pub ty: Type,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeAlias {
    pub name: String,
    pub generics: Vec<Generic>,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Generic {
    pub name: String,
    pub bounds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhereClause {
    pub predicates: Vec<WherePredicate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WherePredicate {
    pub ty: Type,
    pub bounds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<(String, Expression)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Crate,
}

// Duet-specific AST nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SynthFunction {
    pub func: Function,
    pub spec: Option<Expression>,
    pub examples: Vec<(Expression, Expression)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyFunction {
    pub func: Function,
    pub property: Expression,
    pub method: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FusionStrategy {
    Weighted(f64, f64),
    Voting,
    Cascade,
    Parallel,
}

// Ensemble-specific AST nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub state: Vec<StateField>,
    pub capabilities: Vec<Capability>,
    pub goals: Vec<Goal>,
    pub communication: Vec<CommunicationHandler>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateField {
    pub name: String,
    pub ty: Type,
    pub persistence: Persistence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Persistence {
    Persistent,
    Ephemeral,
    Shared,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub kind: CapabilityKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CapabilityKind {
    Action,
    Perception,
    Communication,
    Reasoning,
    Learning,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goal {
    pub name: String,
    pub expr: Expression,
    pub priority: Priority,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommunicationHandler {
    pub pattern: MessagePattern,
    pub handler: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessagePattern {
    Type(String),
    From(String),
    Any,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub stages: Vec<Stage>,
    pub coordination: Vec<CoordinationRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stage {
    Agent(String),
    Parallel(Vec<Stage>),
    Conditional {
        cond: Expression,
        then_stage: Box<Stage>,
        else_stage: Option<Box<Stage>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoordinationRule {
    Consensus {
        threshold: f64,
        on: Expression,
    },
    Voting {
        method: VotingMethod,
        on: Expression,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VotingMethod {
    Majority,
    Unanimous,
    Weighted,
}
