use nasaq_ast::{ComponentDecl, Item, StateDecl, Type};
use nasaq_lexer::TokenKind;
use nasaq_syntax::{Span, Spanned};

use crate::parser::Parser;

impl<'src> Parser<'src> {
    pub(super) fn parse_component_decl(&mut self, exported: bool) -> ComponentDecl {
        let name = self.parse_ident("expected component name");
        self.expect(TokenKind::LParen, "expected `(` after component name");
        let params = self.parse_params();
        self.expect(TokenKind::RParen, "expected `)` after component parameters");
        self.expect(TokenKind::LBrace, "expected `{` to start component body");

        let mut states = Vec::new();
        let mut view = None;
        let mut style = None;

        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            if self.check(TokenKind::State) {
                states.push(self.parse_state_decl());
            } else if self.check(TokenKind::View) {
                view = Some(self.parse_view_block());
            } else if self.check(TokenKind::Style) {
                style = Some(self.parse_style_block());
            } else {
                self.push_error(
                    format!(
                        "expected `state`, `view`, or `style` in component body, found `{}`",
                        self.token_text(self.current())
                    ),
                    self.current().span,
                );
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace, "expected `}` to close component body");
        ComponentDecl {
            name,
            params,
            states,
            view,
            style,
            exported,
        }
    }

    fn parse_state_decl(&mut self) -> Spanned<StateDecl> {
        let start = self.span_start();
        self.expect(TokenKind::State, "expected `state`");
        let mutable = self.match_kind(TokenKind::Mut);
        let name = self.parse_ident("expected state variable name");
        let ty = if self.match_kind(TokenKind::Colon) {
            Some(self.parse_type())
        } else {
            None
        };
        self.expect(TokenKind::Eq, "expected `=` in state declaration");
        let init = self.parse_expr();
        if !self.match_kind(TokenKind::Semicolon) && !self.check(TokenKind::RBrace) {
            if !(self.check(TokenKind::State)
                || self.check(TokenKind::View)
                || self.check(TokenKind::Style))
            {
                self.expect(TokenKind::Semicolon, "expected `;` after state declaration");
            }
        }
        Spanned::new(
            StateDecl {
                mutable,
                name,
                ty,
                init,
            },
            Span::new(start, self.previous().span.end),
        )
    }
}
