use nasaq_syntax::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Module,
    Import,
    Export,
    Extern,
    Struct,
    Enum,
    Match,
    Async,
    Await,
    Pub,
    As,
    In,
    Break,
    Continue,
    Component,
    State,
    View,
    Style,
    Scoped,
    Type,
    Interface,
    Trait,
    Impl,
    For,

    // Type keywords
    Int,
    Float,
    Bool,
    String,
    Char,
    Void,

    // Literals
    IntLit(i64),
    FloatLit(String),
    StringLit(String),
    CharLit(char),

    Ident(String),
    /// Raw UTF-8 text in view blocks (Arabic, emoji, punctuation runs)
    ViewText(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    Bang,
    Dot,
    Comma,
    Colon,
    Semicolon,
    Arrow,
    FatArrow,
    PlusEq,
    MinusEq,
    Pipe,
    Amp,
    Question,
    At,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Eof,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Fn
                | TokenKind::Let
                | TokenKind::Mut
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::While
                | TokenKind::Return
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Module
                | TokenKind::Import
                | TokenKind::Export
                | TokenKind::Extern
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Match
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Pub
                | TokenKind::As
                | TokenKind::In
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Component
                | TokenKind::State
                | TokenKind::View
                | TokenKind::Style
                | TokenKind::Scoped
                | TokenKind::Type
                | TokenKind::Interface
                | TokenKind::Trait
                | TokenKind::Impl
                | TokenKind::For
                | TokenKind::Int
                | TokenKind::Float
                | TokenKind::Bool
                | TokenKind::String
                | TokenKind::Char
                | TokenKind::Void
        )
    }
}
