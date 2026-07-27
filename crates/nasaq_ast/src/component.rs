use nasaq_syntax::{Span, Spanned};

use crate::{Expr, Param, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentDecl {
    pub name: Spanned<String>,
    pub params: Vec<Spanned<Param>>,
    pub states: Vec<Spanned<StateDecl>>,
    pub view: Option<Spanned<ViewBlock>>,
    pub style: Option<Spanned<StyleBlock>>,
    pub exported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateDecl {
    pub mutable: bool,
    pub name: Spanned<String>,
    pub ty: Option<Spanned<Type>>,
    pub init: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewBlock {
    pub nodes: Vec<Spanned<ViewNode>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewNode {
    Element(HtmlElement),
    Text(Spanned<String>),
    Interpolation(Spanned<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HtmlElement {
    pub tag: Spanned<String>,
    pub attrs: Vec<Spanned<HtmlAttr>>,
    pub children: Vec<Spanned<ViewNode>>,
    pub self_closing: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HtmlAttr {
    Attribute {
        name: Spanned<String>,
        value: Spanned<AttrValue>,
    },
    Event {
        event: Spanned<String>,
        handler: Spanned<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    String(String),
    Expr(Spanned<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleBlock {
    pub scoped: bool,
    pub css: Spanned<String>,
    pub span: Span,
}
