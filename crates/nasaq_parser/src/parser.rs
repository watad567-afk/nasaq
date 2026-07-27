use nasaq_ast::{
    AssignOp, BinOp, Block, Expr, ExternFn, FieldInit, FnDecl, ImportDecl, Item, Literal, ModuleDecl,
    Param, Pattern, Program, Stmt, StructDecl, StructField, Type, UnaryOp,
};
use nasaq_diagnostics::{Diagnostic, DiagnosticBag};
use nasaq_lexer::{Token, TokenKind, lex};
use nasaq_syntax::{SourceFile, Span, Spanned};

pub struct ParseResult {
    pub program: Option<Program>,
    pub diagnostics: DiagnosticBag,
}

pub struct Parser<'src> {
    pub(super) source: &'src str,
    pub(super) tokens: Vec<Token>,
    pub(super) current: usize,
    pub(super) diagnostics: DiagnosticBag,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            current: 0,
            diagnostics: DiagnosticBag::new(),
        }
    }

    pub fn parse_program(mut self) -> ParseResult {
        let start = self.span_start();
        let module = if self.check(TokenKind::Module) {
            Some(self.parse_module_decl())
        } else {
            None
        };

        let mut items = Vec::new();
        while !self.is_at_end() {
            match self.parse_item() {
                Some(item) => items.push(item),
                None => self.synchronize(),
            }
        }

        let end = if self.is_at_end() {
            self.previous().span.end
        } else {
            self.current().span.end
        };

        ParseResult {
            program: Some(Program {
                module,
                items,
                span: Span::new(start, end),
            }),
            diagnostics: self.diagnostics,
        }
    }

    fn parse_module_decl(&mut self) -> Spanned<ModuleDecl> {
        let start = self.span_start();
        self.expect(TokenKind::Module, "expected `module` declaration");
        let name = self.parse_ident("expected module name");
        let end = name.span.end;
        Spanned::new(ModuleDecl { name }, Span::new(start, end))
    }

    fn parse_item(&mut self) -> Option<Spanned<Item>> {
        let start = self.span_start();
        let exported = if self.match_kind(TokenKind::Export) {
            true
        } else {
            false
        };

        let item = if self.match_kind(TokenKind::Fn) {
            Item::Function(self.parse_fn_decl(exported))
        } else if self.match_kind(TokenKind::Extern) {
            if exported {
                self.push_error("extern functions cannot be exported", self.previous().span);
            }
            self.expect(TokenKind::Fn, "expected `fn` after `extern`");
            Item::Extern(self.parse_extern_fn())
        } else if self.match_kind(TokenKind::Struct) {
            Item::Struct(self.parse_struct_decl(exported))
        } else if self.match_kind(TokenKind::Import) {
            if exported {
                self.push_error("import declarations cannot be exported", self.previous().span);
            }
            Item::Import(self.parse_import())
        } else if self.match_kind(TokenKind::Component) {
            Item::Component(self.parse_component_decl(exported))
        } else {
            self.push_error(
                format!("unexpected token `{}`", self.token_text(self.current())),
                self.current().span,
            );
            return None;
        };

        let end = self.previous().span.end;
        Some(Spanned::new(item, Span::new(start, end)))
    }

    fn parse_fn_decl(&mut self, exported: bool) -> FnDecl {
        let name = self.parse_ident("expected function name");
        self.expect(TokenKind::LParen, "expected `(` after function name");
        let params = self.parse_params();
        self.expect(TokenKind::RParen, "expected `)` after parameters");
        let return_type = if self.match_kind(TokenKind::Arrow) {
            Some(self.parse_type())
        } else {
            None
        };
        let body = self.parse_block();
        FnDecl {
            name,
            params,
            return_type,
            body,
            exported,
        }
    }

    fn parse_extern_fn(&mut self) -> ExternFn {
        let name = self.parse_ident("expected extern function name");
        self.expect(TokenKind::LParen, "expected `(` after function name");
        let params = self.parse_params();
        self.expect(TokenKind::RParen, "expected `)` after parameters");
        let return_type = if self.match_kind(TokenKind::Arrow) {
            Some(self.parse_type())
        } else {
            None
        };
        if !self.check_item_start() {
            self.expect(TokenKind::Semicolon, "expected `;` after extern declaration");
        }
        ExternFn {
            name,
            params,
            return_type,
        }
    }

    fn parse_struct_decl(&mut self, exported: bool) -> StructDecl {
        let name = self.parse_ident("expected struct name");
        self.expect(TokenKind::LBrace, "expected `{` after struct name");
        let mut fields = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let field_start = self.span_start();
            let field_name = self.parse_ident("expected field name");
            self.expect(TokenKind::Colon, "expected `:` after field name");
            let ty = self.parse_type();
            let field_end = self.previous().span.end;
            fields.push(Spanned::new(
                StructField {
                    name: field_name,
                    ty,
                },
                Span::new(field_start, field_end),
            ));
            if self.check(TokenKind::RBrace) {
                break;
            }
            if !self.match_kind(TokenKind::Comma) && !matches!(self.current().kind, TokenKind::Ident(_)) {
                break;
            }
        }
        self.expect(TokenKind::RBrace, "expected `}` after struct fields");
        StructDecl {
            name,
            fields,
            exported,
        }
    }

    fn parse_import(&mut self) -> ImportDecl {
        let path = self.parse_string_or_ident("expected import path");
        let alias = if self.match_kind(TokenKind::As) {
            Some(self.parse_ident("expected import alias"))
        } else {
            None
        };
        self.expect(TokenKind::Semicolon, "expected `;` after import");
        ImportDecl { path, alias }
    }

    pub(super) fn parse_params(&mut self) -> Vec<Spanned<Param>> {
        let mut params = Vec::new();
        if self.check(TokenKind::RParen) {
            return params;
        }
        loop {
            let start = self.span_start();
            let name = self.parse_ident("expected parameter name");
            self.expect(TokenKind::Colon, "expected type annotation after parameter name");
            let ty = self.parse_type();
            let default = if self.match_kind(TokenKind::Eq) {
                Some(self.parse_expr())
            } else {
                None
            };
            let end = self.previous().span.end;
            params.push(Spanned::new(
                Param {
                    name,
                    ty,
                    default,
                },
                Span::new(start, end),
            ));
            if !self.match_kind(TokenKind::Comma) {
                break;
            }
        }
        params
    }

    pub(super) fn parse_type(&mut self) -> Spanned<Type> {
        let start = self.span_start();
        let ty = match self.current().kind.clone() {
            TokenKind::Int => {
                self.advance();
                Type::Int
            }
            TokenKind::Float => {
                self.advance();
                Type::Float
            }
            TokenKind::Bool => {
                self.advance();
                Type::Bool
            }
            TokenKind::String => {
                self.advance();
                Type::String
            }
            TokenKind::Char => {
                self.advance();
                Type::Char
            }
            TokenKind::Void => {
                self.advance();
                Type::Void
            }
            TokenKind::Ident(name) => {
                self.advance();
                let mut base = Spanned::new(
                    Type::Named(Spanned::new(name.clone(), self.previous().span)),
                    self.previous().span,
                );
                if self.match_kind(TokenKind::Lt) {
                    let mut args = vec![self.parse_type()];
                    while self.match_kind(TokenKind::Comma) {
                        args.push(self.parse_type());
                    }
                    self.expect(TokenKind::Gt, "expected `>` to close generic arguments");
                    base = Spanned::new(
                        Type::Generic {
                            base: Box::new(base),
                            args,
                        },
                        Span::new(start, self.previous().span.end),
                    );
                }
                return base;
            }
            TokenKind::LParen => {
                self.advance();
                let mut types = vec![self.parse_type()];
                while self.match_kind(TokenKind::Comma) {
                    types.push(self.parse_type());
                }
                self.expect(TokenKind::RParen, "expected `)` after tuple type");
                if self.match_kind(TokenKind::Arrow) {
                    let ret = self.parse_type();
                    Type::Function {
                        params: types,
                        return_type: Box::new(ret),
                    }
                } else {
                    Type::Tuple(types)
                }
            }
            _ => {
                self.push_error(
                    format!("expected type, found `{}`", self.token_text(self.current())),
                    self.current().span,
                );
                Type::Void
            }
        };
        Spanned::new(ty, Span::new(start, self.previous().span.end))
    }

    fn parse_block(&mut self) -> Block {
        let start = self.span_start();
        self.expect(TokenKind::LBrace, "expected `{` to start block");
        let mut stmts = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                self.synchronize();
            }
        }
        self.expect(TokenKind::RBrace, "expected `}` to close block");
        Block {
            stmts,
            span: Span::new(start, self.previous().span.end),
        }
    }

    fn parse_stmt(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.span_start();
        if self.match_kind(TokenKind::Let) {
            let mutable = self.match_kind(TokenKind::Mut);
            let name = self.parse_ident("expected binding name");
            let ty = if self.match_kind(TokenKind::Colon) {
                Some(self.parse_type())
            } else {
                None
            };
            self.expect(TokenKind::Eq, "expected `=` in let binding");
            let init = self.parse_expr();
            if !self.match_kind(TokenKind::Semicolon) {
                if !(self.check(TokenKind::RBrace) || self.check_stmt_start()) {
                    self.expect(TokenKind::Semicolon, "expected `;` after let binding");
                }
            }
            return Some(Spanned::new(
                Stmt::Let {
                    mutable,
                    name,
                    ty,
                    init,
                },
                Span::new(start, self.previous().span.end),
            ));
        }
        if self.match_kind(TokenKind::Return) {
            let value = if self.check_stmt_end() {
                None
            } else {
                Some(self.parse_expr())
            };
            if !self.check(TokenKind::RBrace) {
                self.expect(TokenKind::Semicolon, "expected `;` after return");
            }
            return Some(Spanned::new(
                Stmt::Return {
                    value,
                    span: Span::new(start, self.previous().span.end),
                },
                Span::new(start, self.previous().span.end),
            ));
        }
        if self.match_kind(TokenKind::If) {
            let cond = self.parse_expr();
            let then_block = self.parse_block();
            let else_block = if self.match_kind(TokenKind::Else) {
                Some(self.parse_block())
            } else {
                None
            };
            return Some(Spanned::new(
                Stmt::If {
                    cond,
                    then_block,
                    else_block,
                    span: Span::new(start, self.previous().span.end),
                },
                Span::new(start, self.previous().span.end),
            ));
        }
        if self.match_kind(TokenKind::While) {
            let cond = self.parse_expr();
            let body = self.parse_block();
            return Some(Spanned::new(
                Stmt::While {
                    cond,
                    body,
                    span: Span::new(start, self.previous().span.end),
                },
                Span::new(start, self.previous().span.end),
            ));
        }
        if self.match_kind(TokenKind::Break) {
            self.expect(TokenKind::Semicolon, "expected `;` after break");
            return Some(Spanned::new(
                Stmt::Break {
                    span: Span::new(start, self.previous().span.end),
                },
                Span::new(start, self.previous().span.end),
            ));
        }
        if self.match_kind(TokenKind::Continue) {
            self.expect(TokenKind::Semicolon, "expected `;` after continue");
            return Some(Spanned::new(
                Stmt::Continue {
                    span: Span::new(start, self.previous().span.end),
                },
                Span::new(start, self.previous().span.end),
            ));
        }

        let expr = self.parse_expr();
        let end = expr.span.end;
        if self.match_kind(TokenKind::Semicolon) {
            Some(Spanned::new(
                Stmt::Expr(expr),
                Span::new(start, self.previous().span.end),
            ))
        } else if self.check(TokenKind::RBrace) || self.check_stmt_start() || matches!(
            expr.node,
            Expr::If { .. } | Expr::Block(_) | Expr::Match { .. }
        ) {
            Some(Spanned::new(Stmt::Expr(expr), Span::new(start, end)))
        } else if self.check_item_start() {
            Some(Spanned::new(Stmt::Expr(expr), Span::new(start, end)))
        } else {
            self.push_error("expected `;` after expression", self.current().span);
            None
        }
    }

    pub(super) fn parse_expr(&mut self) -> Spanned<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Spanned<Expr> {
        let expr = self.parse_or();
        if self.match_any(&[TokenKind::Eq, TokenKind::PlusEq, TokenKind::MinusEq]) {
            let op = match self.previous().kind {
                TokenKind::Eq => AssignOp::Assign,
                TokenKind::PlusEq => AssignOp::AddAssign,
                TokenKind::MinusEq => AssignOp::SubAssign,
                _ => unreachable!(),
            };
            let value = self.parse_assignment();
            let span = expr.span.merge(value.span);
            return Spanned::new(
                Expr::Assign {
                    target: Box::new(expr),
                    op,
                    value: Box::new(value),
                    span,
                },
                span,
            );
        }
        expr
    }

    fn parse_or(&mut self) -> Spanned<Expr> {
        let mut expr = self.parse_and();
        while self.match_kind(TokenKind::OrOr) {
            let right = self.parse_and();
            let span = expr.span.merge(right.span);
            expr = Spanned::new(
                Expr::Binary {
                    op: BinOp::Or,
                    left: Box::new(expr),
                    right: Box::new(right),
                    span,
                },
                span,
            );
        }
        expr
    }

    fn parse_and(&mut self) -> Spanned<Expr> {
        let mut expr = self.parse_equality();
        while self.match_kind(TokenKind::AndAnd) {
            let right = self.parse_equality();
            let span = expr.span.merge(right.span);
            expr = Spanned::new(
                Expr::Binary {
                    op: BinOp::And,
                    left: Box::new(expr),
                    right: Box::new(right),
                    span,
                },
                span,
            );
        }
        expr
    }

    fn parse_equality(&mut self) -> Spanned<Expr> {
        let mut expr = self.parse_comparison();
        while self.match_any(&[TokenKind::EqEq, TokenKind::Ne]) {
            let op = match self.previous().kind {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::Ne => BinOp::Ne,
                _ => unreachable!(),
            };
            let right = self.parse_comparison();
            let span = expr.span.merge(right.span);
            expr = Spanned::new(
                Expr::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                    span,
                },
                span,
            );
        }
        expr
    }

    fn parse_comparison(&mut self) -> Spanned<Expr> {
        let mut expr = self.parse_term();
        while self.match_any(&[
            TokenKind::Lt,
            TokenKind::Le,
            TokenKind::Gt,
            TokenKind::Ge,
        ]) {
            let op = match self.previous().kind {
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Ge => BinOp::Ge,
                _ => unreachable!(),
            };
            let right = self.parse_term();
            let span = expr.span.merge(right.span);
            expr = Spanned::new(
                Expr::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                    span,
                },
                span,
            );
        }
        expr
    }

    fn parse_term(&mut self) -> Spanned<Expr> {
        let mut expr = self.parse_factor();
        while self.match_any(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = match self.previous().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_factor();
            let span = expr.span.merge(right.span);
            expr = Spanned::new(
                Expr::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                    span,
                },
                span,
            );
        }
        expr
    }

    fn parse_factor(&mut self) -> Spanned<Expr> {
        let mut expr = self.parse_unary();
        while self.match_any(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op = match self.previous().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_unary();
            let span = expr.span.merge(right.span);
            expr = Spanned::new(
                Expr::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                    span,
                },
                span,
            );
        }
        expr
    }

    fn parse_unary(&mut self) -> Spanned<Expr> {
        if self.match_any(&[TokenKind::Bang, TokenKind::Minus]) {
            let op = match self.previous().kind {
                TokenKind::Bang => UnaryOp::Not,
                TokenKind::Minus => UnaryOp::Neg,
                _ => unreachable!(),
            };
            let start = self.previous().span.start;
            let expr = self.parse_unary();
            let span = Span::new(start, expr.span.end);
            return Spanned::new(
                Expr::Unary {
                    op,
                    expr: Box::new(expr),
                    span,
                },
                span,
            );
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Spanned<Expr> {
        let mut expr = self.parse_primary();
        loop {
            if self.match_kind(TokenKind::LParen) {
                let mut args = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr());
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen, "expected `)` after arguments");
                let span = expr.span.merge(self.previous().span);
                expr = Spanned::new(
                    Expr::Call {
                        callee: Box::new(expr),
                        args,
                        span,
                    },
                    span,
                );
            } else if self.match_kind(TokenKind::Dot) {
                let field = self.parse_ident("expected field name after `.`");
                let span = expr.span.merge(field.span);
                expr = Spanned::new(
                    Expr::Field {
                        object: Box::new(expr),
                        field,
                        span,
                    },
                    span,
                );
            } else if self.match_kind(TokenKind::LBracket) {
                let index = self.parse_expr();
                self.expect(TokenKind::RBracket, "expected `]` after index");
                let span = expr.span.merge(self.previous().span);
                expr = Spanned::new(
                    Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                        span,
                    },
                    span,
                );
            } else {
                break;
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Spanned<Expr> {
        let start = self.span_start();
        match self.current().kind.clone() {
            TokenKind::True => {
                self.advance();
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(
                    Expr::Literal(Spanned::new(Literal::Bool(true), span)),
                    span,
                )
            }
            TokenKind::False => {
                self.advance();
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(
                    Expr::Literal(Spanned::new(Literal::Bool(false), span)),
                    span,
                )
            }
            TokenKind::IntLit(v) => {
                self.advance();
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(
                    Expr::Literal(Spanned::new(Literal::Int(v), span)),
                    span,
                )
            }
            TokenKind::FloatLit(v) => {
                self.advance();
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(
                    Expr::Literal(Spanned::new(Literal::Float(v), span)),
                    span,
                )
            }
            TokenKind::StringLit(v) => {
                self.advance();
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(
                    Expr::Literal(Spanned::new(Literal::String(v), span)),
                    span,
                )
            }
            TokenKind::CharLit(c) => {
                self.advance();
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(
                    Expr::Literal(Spanned::new(Literal::Char(c), span)),
                    span,
                )
            }
            TokenKind::Ident(name) => {
                self.advance();
                if name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_uppercase())
                    && self.match_kind(TokenKind::LBrace)
                {
                    let mut fields = Vec::new();
                    while !self.check(TokenKind::RBrace) && !self.is_at_end() {
                        let field_start = self.span_start();
                        let field_name = self.parse_ident("expected struct field name");
                        self.expect(TokenKind::Colon, "expected `:` in struct literal");
                        let value = self.parse_expr();
                        fields.push(Spanned::new(
                            FieldInit {
                                name: field_name,
                                value,
                            },
                            Span::new(field_start, self.previous().span.end),
                        ));
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RBrace, "expected `}` in struct literal");
                    let span = Span::new(start, self.previous().span.end);
                    return Spanned::new(
                        Expr::StructLit {
                            name: Spanned::new(name.clone(), Span::new(start, start + name.len() as u32)),
                            fields,
                            span,
                        },
                        span,
                    );
                }
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(Expr::Ident(Spanned::new(name, span)), span)
            }
            TokenKind::If => {
                self.advance();
                let cond = self.parse_expr();
                let then_block = self.parse_block();
                let else_expr = if self.match_kind(TokenKind::Else) {
                    Some(Box::new(if self.check(TokenKind::If) {
                        self.parse_expr()
                    } else {
                        Spanned::new(Expr::Block(self.parse_block()), self.previous().span)
                    }))
                } else {
                    None
                };
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(
                    Expr::If {
                        cond: Box::new(cond),
                        then_block,
                        else_expr,
                        span,
                    },
                    span,
                )
            }
            TokenKind::Match => {
                self.advance();
                let scrutinee = self.parse_expr();
                self.expect(TokenKind::LBrace, "expected `{` after match scrutinee");
                let mut arms = Vec::new();
                while !self.check(TokenKind::RBrace) && !self.is_at_end() {
                    let arm_start = self.span_start();
                    let pattern = self.parse_pattern();
                    let guard = if self.match_kind(TokenKind::If) {
                        Some(self.parse_expr())
                    } else {
                        None
                    };
                    self.expect(TokenKind::FatArrow, "expected `=>` in match arm");
                    let body = self.parse_expr();
                    self.expect(TokenKind::Comma, "expected `,` after match arm");
                    arms.push(Spanned::new(
                        nasaq_ast::MatchArm {
                            pattern,
                            guard,
                            body,
                        },
                        Span::new(arm_start, self.previous().span.end),
                    ));
                }
                self.expect(TokenKind::RBrace, "expected `}` after match arms");
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(
                    Expr::Match {
                        scrutinee: Box::new(scrutinee),
                        arms,
                        span,
                    },
                    span,
                )
            }
            TokenKind::LParen => {
                self.advance();
                if self.check(TokenKind::RParen) {
                    self.advance();
                    let span = Span::new(start, self.previous().span.end);
                    return Spanned::new(Expr::Tuple(Vec::new(), span), span);
                }
                let first = self.parse_expr();
                if self.match_kind(TokenKind::Comma) {
                    let mut elems = vec![first];
                    while !self.check(TokenKind::RParen) {
                        elems.push(self.parse_expr());
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen, "expected `)` after tuple");
                    let span = Span::new(start, self.previous().span.end);
                    Spanned::new(Expr::Tuple(elems, span), span)
                } else {
                    self.expect(TokenKind::RParen, "expected `)` after grouped expression");
                    Spanned::new(Expr::Group(Box::new(first), self.previous().span), self.previous().span)
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elems = Vec::new();
                if !self.check(TokenKind::RBracket) {
                    loop {
                        elems.push(self.parse_expr());
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBracket, "expected `]` after array literal");
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(Expr::Array(elems, span), span)
            }
            TokenKind::LBrace => {
                let block = self.parse_block();
                Spanned::new(Expr::Block(block.clone()), block.span)
            }
            _ => {
                self.push_error(
                    format!("expected expression, found `{}`", self.token_text(self.current())),
                    self.current().span,
                );
                self.advance();
                let span = Span::new(start, self.previous().span.end);
                Spanned::new(
                    Expr::Literal(Spanned::new(Literal::Int(0), span)),
                    span,
                )
            }
        }
    }

    fn parse_pattern(&mut self) -> Spanned<Pattern> {
        let start = self.span_start();
        if matches!(self.current().kind, TokenKind::Ident(_)) {
            if self.current_token_text() == "_" {
                self.advance();
                return Spanned::new(Pattern::Wildcard, self.previous().span);
            }
            let name = self.parse_ident("expected pattern identifier");
            let mutable = self.match_kind(TokenKind::Mut);
            if self.match_kind(TokenKind::LBrace) {
                let mut fields = Vec::new();
                while !self.check(TokenKind::RBrace) && !self.is_at_end() {
                    let field_name = self.parse_ident("expected field name in pattern");
                    let pat = if self.match_kind(TokenKind::Colon) {
                        self.parse_pattern()
                    } else {
                        Spanned::new(
                            Pattern::Ident {
                                name: field_name.clone(),
                                mutable: false,
                            },
                            field_name.span,
                        )
                    };
                    fields.push((field_name, pat));
                    if !self.match_kind(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBrace, "expected `}` in struct pattern");
                return Spanned::new(
                    Pattern::Struct {
                        name,
                        fields,
                    },
                    Span::new(start, self.previous().span.end),
                );
            }
            return Spanned::new(Pattern::Ident { name, mutable }, self.previous().span);
        }
        if matches!(
            self.current().kind,
            TokenKind::IntLit(_)
                | TokenKind::StringLit(_)
                | TokenKind::True
                | TokenKind::False
        ) {
            let lit = self.parse_literal();
            return Spanned::new(Pattern::Literal(lit.clone()), lit.span);
        }
        self.push_error("expected pattern", self.current().span);
        Spanned::new(Pattern::Wildcard, self.current().span)
    }

    fn parse_literal(&mut self) -> Spanned<Literal> {
        let span = self.current().span;
        let lit = match self.current().kind.clone() {
            TokenKind::IntLit(v) => {
                self.advance();
                Literal::Int(v)
            }
            TokenKind::FloatLit(v) => {
                self.advance();
                Literal::Float(v)
            }
            TokenKind::StringLit(v) => {
                self.advance();
                Literal::String(v)
            }
            TokenKind::True => {
                self.advance();
                Literal::Bool(true)
            }
            TokenKind::False => {
                self.advance();
                Literal::Bool(false)
            }
            _ => Literal::Int(0),
        };
        Spanned::new(lit, span)
    }

    pub(super) fn parse_ident(&mut self, message: &str) -> Spanned<String> {
        if let TokenKind::Ident(name) = self.current().kind.clone() {
            let span = self.current().span;
            self.advance();
            Spanned::new(name, span)
        } else {
            self.push_error(message, self.current().span);
            Spanned::new(String::new(), self.current().span)
        }
    }

    fn parse_string_or_ident(&mut self, message: &str) -> Spanned<String> {
        match self.current().kind.clone() {
            TokenKind::StringLit(s) => {
                let span = self.current().span;
                self.advance();
                Spanned::new(s, span)
            }
            TokenKind::Ident(name) => self.parse_ident(message),
            _ => {
                self.push_error(message, self.current().span);
                Spanned::new(String::new(), self.current().span)
            }
        }
    }

    pub(super) fn expect(&mut self, kind: TokenKind, message: &str) {
        if self.check(kind.clone()) {
            self.advance();
        } else {
            self.push_error(
                format!("{message}, found `{}`", self.token_text(self.current())),
                self.current().span,
            );
            if self.is_at_end() {
                return;
            }
            self.advance();
        }
    }

    pub(super) fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(super) fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.match_kind(kind.clone()) {
                return true;
            }
        }
        false
    }

    pub(super) fn check_ident(&self, name: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Ident(text) if text == name)
    }

    pub(super) fn peek(&self, offset: usize) -> Option<&Token> {
        let idx = self.current.saturating_add(offset);
        self.tokens.get(idx)
    }

    pub(super) fn check(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && std::mem::discriminant(&self.current().kind) == std::mem::discriminant(&kind)
            && matches!(
                (&self.current().kind, &kind),
                (TokenKind::Ident(_), TokenKind::Ident(_))
                    | (TokenKind::IntLit(_), TokenKind::IntLit(_))
                    | (TokenKind::FloatLit(_), TokenKind::FloatLit(_))
                    | (TokenKind::StringLit(_), TokenKind::StringLit(_))
                    | (TokenKind::CharLit(_), TokenKind::CharLit(_))
                    | _ if self.current().kind == kind
            )
    }

    pub(super) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    pub(super) fn is_at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    pub(super) fn current(&self) -> &Token {
        &self.tokens[self.current]
    }

    pub(super) fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    pub(super) fn span_start(&self) -> u32 {
        self.current().span.start
    }

    fn check_stmt_end(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof
        ) || self.check_item_start()
    }

    fn check_item_start(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Export
                | TokenKind::Fn
                | TokenKind::Extern
                | TokenKind::Struct
                | TokenKind::Import
                | TokenKind::Module
                | TokenKind::Component
        )
    }

    fn check_stmt_start(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Let
                | TokenKind::Return
                | TokenKind::If
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Match
                | TokenKind::Ident(_)
        )
    }

    pub(super) fn token_text(&self, token: &Token) -> String {
        self.source[token.span.range()].to_string()
    }

    pub(super) fn current_token_text(&self) -> &str {
        &self.source[self.current().span.range()]
    }

    pub(super) fn next_char(&self) -> Option<char> {
        let start = self.current().span.end as usize;
        self.source[start..].chars().next()
    }
}

pub fn parse_program(source: &str) -> ParseResult {
    let tokens = match lex(source) {
        Ok(tokens) => tokens,
        Err(err) => {
            let mut diagnostics = DiagnosticBag::new();
            diagnostics.push(Diagnostic::error(err.message, err.span));
            return ParseResult {
                program: None,
                diagnostics,
            };
        }
    };
    Parser::new(source, tokens).parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_arrow_return_type() {
        let result = parse_program("export fn fib(n: Int) -> Int { return n }");
        if result.diagnostics.has_errors() {
            eprintln!("{:?}", result.diagnostics.diagnostics);
        }
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn parses_simple_function() {
        let result = parse_program(
            r#"
            module hello
            export fn main() -> Void {
                let message = "Hello, Nasaq!"
                return
            }
            "#,
        );
        assert!(!result.diagnostics.has_errors());
        let program = result.program.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn parses_fibonacci() {
        let result = parse_program(
            r#"
            export fn fib(n: Int) -> Int {
                if n <= 1 {
                    return n
                }
                return fib(n - 1) + fib(n - 2)
            }
            "#,
        );
        assert!(!result.diagnostics.has_errors());
    }
}
