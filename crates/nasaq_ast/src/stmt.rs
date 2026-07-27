use nasaq_syntax::{Span, Spanned};

use crate::{Expr, Pattern, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        mutable: bool,
        name: Spanned<String>,
        ty: Option<Spanned<Type>>,
        init: Spanned<Expr>,
    },
    Expr(Spanned<Expr>),
    Return {
        value: Option<Spanned<Expr>>,
        span: Span,
    },
    If {
        cond: Spanned<Expr>,
        then_block: Block,
        else_block: Option<Block>,
        span: Span,
    },
    While {
        cond: Spanned<Expr>,
        body: Block,
        span: Span,
    },
    Break { span: Span },
    Continue { span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Spanned<Stmt>>,
    pub span: Span,
}
