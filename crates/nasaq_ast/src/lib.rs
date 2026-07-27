//! Abstract syntax tree nodes for Nasaq programs.

mod component;
mod expr;
mod item;
mod stmt;
mod ty;

pub use component::{
    AttrValue, ComponentDecl, HtmlAttr, HtmlElement, StateDecl, StyleBlock, ViewBlock, ViewNode,
};
pub use expr::{AssignOp, BinOp, Expr, FieldInit, Literal, MatchArm, Param, Pattern, UnaryOp};
pub use item::{ExternFn, FnDecl, ImportDecl, Item, ModuleDecl, StructDecl, StructField};
pub use stmt::{Block, Stmt};
pub use ty::Type;

use nasaq_syntax::{Span, Spanned};

/// A parsed Nasaq source file.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub module: Option<Spanned<ModuleDecl>>,
    pub items: Vec<Spanned<Item>>,
    pub span: Span,
}
