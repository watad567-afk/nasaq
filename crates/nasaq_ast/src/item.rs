use nasaq_syntax::{Span, Spanned};

use crate::{Block, Param, Type};
use crate::component::ComponentDecl;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(FnDecl),
    Extern(ExternFn),
    Struct(StructDecl),
    Component(ComponentDecl),
    Import(ImportDecl),
    Export(Spanned<Box<Item>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    pub name: Spanned<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: Spanned<String>,
    pub params: Vec<Spanned<Param>>,
    pub return_type: Option<Spanned<Type>>,
    pub body: Block,
    pub exported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternFn {
    pub name: Spanned<String>,
    pub params: Vec<Spanned<Param>>,
    pub return_type: Option<Spanned<Type>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub path: Spanned<String>,
    pub alias: Option<Spanned<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: Spanned<String>,
    pub fields: Vec<Spanned<StructField>>,
    pub exported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
}
