//! Error recovery helpers for the parser.

use nasaq_diagnostics::Diagnostic;
use nasaq_lexer::TokenKind;
use nasaq_syntax::Span;

use crate::parser::Parser;

impl<'src> Parser<'src> {
    pub(super) fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                self.advance();
                return;
            }
            match self.current().kind {
                TokenKind::Fn
                | TokenKind::Let
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Return
                | TokenKind::Export
                | TokenKind::Extern
                | TokenKind::Struct
                | TokenKind::Import
                | TokenKind::Module
                | TokenKind::Component => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub(super) fn push_error(&mut self, message: impl Into<String>, span: Span) {
        self.diagnostics.push(nasaq_diagnostics::Diagnostic::error(
            message,
            span,
        ));
    }
}
