use nasaq_ast::{AttrValue, Expr, HtmlAttr, HtmlElement, ViewBlock, ViewNode};

use nasaq_lexer::TokenKind;

use nasaq_syntax::{Span, Spanned};



use crate::parser::Parser;



impl<'src> Parser<'src> {

    pub(super) fn parse_view_block(&mut self) -> Spanned<ViewBlock> {

        let start = self.span_start();

        self.expect(TokenKind::View, "expected `view`");

        self.expect(TokenKind::LBrace, "expected `{` after `view`");

        let mut nodes = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.is_at_end() {

            if let Some(node) = self.parse_view_node() {

                nodes.push(node);

            } else if self.check(TokenKind::RBrace) {

                break;

            } else {

                self.push_error(

                    format!("expected view node, found `{}`", self.token_text(self.current())),

                    self.current().span,

                );

                self.advance();

            }

        }

        self.expect(TokenKind::RBrace, "expected `}` to close view block");

        let span = Span::new(start, self.previous().span.end);

        Spanned::new(ViewBlock { nodes, span }, span)

    }



    fn parse_view_node(&mut self) -> Option<Spanned<ViewNode>> {

        if self.match_kind(TokenKind::LBrace) {

            let expr = self.parse_expr();

            self.expect(TokenKind::RBrace, "expected `}` after interpolation");

            return Some(Spanned::new(

                ViewNode::Interpolation(expr.clone()),

                expr.span,

            ));

        }

        if self.check(TokenKind::Lt) {

            return Some(self.parse_element());

        }

        if let TokenKind::Ident(text) = self.current().kind.clone() {

            let span = self.current().span;

            self.advance();

            return Some(Spanned::new(

                ViewNode::Text(Spanned::new(text, span)),

                span,

            ));

        }

        if let TokenKind::StringLit(text) = self.current().kind.clone() {

            let span = self.current().span;

            self.advance();

            return Some(Spanned::new(

                ViewNode::Text(Spanned::new(text, span)),

                span,

            ));

        }

        if let TokenKind::ViewText(text) = self.current().kind.clone() {

            let span = self.current().span;

            self.advance();

            return Some(Spanned::new(

                ViewNode::Text(Spanned::new(text, span)),

                span,

            ));

        }

        None

    }



    fn parse_element(&mut self) -> Spanned<ViewNode> {

        let start = self.span_start();

        self.expect(TokenKind::Lt, "expected `<`");

        let tag = self.parse_ident("expected HTML tag name");

        let mut attrs = Vec::new();

        while !self.check(TokenKind::Gt) && !self.check(TokenKind::Slash) && !self.is_at_end() {

            let before = self.current;

            attrs.push(self.parse_html_attr());

            if self.current == before && !self.is_at_end() {

                self.advance();

            }

        }

        let self_closing = self.match_kind(TokenKind::Slash);

        self.expect(TokenKind::Gt, "expected `>` after opening tag");



        let mut children = Vec::new();

        if !self_closing {

            loop {

                if self.check(TokenKind::Lt)

                    && self.peek(1).is_some_and(|t| matches!(t.kind, TokenKind::Slash))

                {

                    self.advance();

                    self.advance();

                    let close = self.parse_ident("expected closing tag name");

                    if close.node != tag.node {

                        self.push_error(

                            format!(

                                "closing tag `{}` does not match `{}`",

                                close.node, tag.node

                            ),

                            close.span,

                        );

                    }

                    self.expect(TokenKind::Gt, "expected `>` after closing tag");

                    break;

                }

                if self.check(TokenKind::RBrace) {

                    break;

                }

                if let Some(child) = self.parse_view_node() {

                    children.push(child);

                } else if !self.is_at_end() {

                    self.advance();

                } else {

                    break;

                }

            }

        }



        let span = Span::new(start, self.previous().span.end);

        Spanned::new(

            ViewNode::Element(HtmlElement {

                tag,

                attrs,

                children,

                self_closing,

                span,

            }),

            span,

        )

    }



    fn parse_html_attr(&mut self) -> Spanned<HtmlAttr> {

        let start = self.span_start();

        if self.check_ident("on")

            && self

                .peek(1)

                .is_some_and(|t| matches!(t.kind, TokenKind::Colon))

        {

            self.advance();

            self.expect(TokenKind::Colon, "expected `:` after `on`");

            let event = self.parse_ident("expected event name");

            self.expect(TokenKind::Eq, "expected `=` after event");

            let handler = self.parse_expr_in_braces();

            return Spanned::new(

                HtmlAttr::Event { event, handler },

                Span::new(start, self.previous().span.end),

            );

        }

        let name = self.parse_ident("expected attribute name");

        let value = if self.match_kind(TokenKind::Eq) {

            self.parse_attr_value()

        } else {

            Spanned::new(AttrValue::String(String::new()), name.span)

        };

        Spanned::new(

            HtmlAttr::Attribute { name, value },

            Span::new(start, self.previous().span.end),

        )

    }



    fn parse_attr_value(&mut self) -> Spanned<AttrValue> {

        if let TokenKind::StringLit(s) = self.current().kind.clone() {

            let span = self.current().span;

            self.advance();

            return Spanned::new(AttrValue::String(s), span);

        }

        if self.match_kind(TokenKind::LBrace) {

            let expr = self.parse_expr();

            self.expect(TokenKind::RBrace, "expected `}` in attribute expression");

            return Spanned::new(AttrValue::Expr(expr.clone()), expr.span);

        }

        let expr = self.parse_expr();

        Spanned::new(AttrValue::Expr(expr.clone()), expr.span)

    }



    fn parse_expr_in_braces(&mut self) -> Spanned<Expr> {

        self.expect(TokenKind::LBrace, "expected `{` for event handler");

        let expr = self.parse_expr();

        self.expect(TokenKind::RBrace, "expected `}` after event handler");

        expr

    }



    pub(super) fn parse_style_block(&mut self) -> Spanned<nasaq_ast::StyleBlock> {
        let start = self.span_start();
        self.expect(TokenKind::Style, "expected `style`");
        let scoped = self.match_kind(TokenKind::Scoped);
        self.expect(TokenKind::LBrace, "expected `{` after style declaration");
        let css_start = self.previous().span.end as usize;
        let mut depth = 1usize;
        while depth > 0 && !self.is_at_end() {
            if self.check(TokenKind::RBrace) {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            } else if self.check(TokenKind::LBrace) {
                depth += 1;
            }
            self.advance();
        }
        let css_end = self.current().span.start as usize;
        let css = self.source[css_start..css_end].trim().to_string();
        self.expect(TokenKind::RBrace, "expected `}` to close style block");
        let css_span = Span::new(css_start as u32, css_end as u32);
        Spanned::new(
            nasaq_ast::StyleBlock {
                scoped,
                css: Spanned::new(css, css_span),
                span: Span::new(start, self.previous().span.end),
            },
            Span::new(start, self.previous().span.end),
        )
    }
}


