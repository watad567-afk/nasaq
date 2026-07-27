use nasaq_syntax::{Span, Spanned};

use crate::{Block, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Spanned<Literal>),
    Ident(Spanned<String>),
    Binary {
        op: BinOp,
        left: Box<Spanned<Expr>>,
        right: Box<Spanned<Expr>>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Spanned<Expr>>,
        span: Span,
    },
    Call {
        callee: Box<Spanned<Expr>>,
        args: Vec<Spanned<Expr>>,
        span: Span,
    },
    Assign {
        target: Box<Spanned<Expr>>,
        op: AssignOp,
        value: Box<Spanned<Expr>>,
        span: Span,
    },
    Field {
        object: Box<Spanned<Expr>>,
        field: Spanned<String>,
        span: Span,
    },
    Index {
        object: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
        span: Span,
    },
    Block(Block),
    If {
        cond: Box<Spanned<Expr>>,
        then_block: Block,
        else_expr: Option<Box<Spanned<Expr>>>,
        span: Span,
    },
    Match {
        scrutinee: Box<Spanned<Expr>>,
        arms: Vec<Spanned<MatchArm>>,
        span: Span,
    },
    StructLit {
        name: Spanned<String>,
        fields: Vec<Spanned<FieldInit>>,
        span: Span,
    },
    Array(Vec<Spanned<Expr>>, Span),
    Tuple(Vec<Spanned<Expr>>, Span),
    Group(Box<Spanned<Expr>>, Span),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(String),
    String(String),
    Char(char),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
    pub default: Option<Spanned<super::Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInit {
    pub name: Spanned<String>,
    pub value: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Spanned<Pattern>,
    pub guard: Option<Spanned<Expr>>,
    pub body: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Ident {
        name: Spanned<String>,
        mutable: bool,
    },
    Literal(Spanned<Literal>),
    Struct {
        name: Spanned<String>,
        fields: Vec<(Spanned<String>, Spanned<Pattern>)>,
    },
}
