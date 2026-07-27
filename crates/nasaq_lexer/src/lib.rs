//! Lexical analysis for `.nasaq` source files.

mod lexer;
mod token;

pub use lexer::{LexError, Lexer, lex};
pub use token::{Token, TokenKind};
