use nasaq_syntax::Span;

use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

pub struct Lexer<'src> {
    source: &'src str,
    chars: std::str::CharIndices<'src>,
    current: Option<(usize, char)>,
    offset: u32,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        let mut chars = source.char_indices();
        let current = chars.next();
        Self {
            source,
            chars,
            current,
            offset: 0,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace_and_comments()?;
        let start = self.offset;
        let ch = match self.current {
            Some((_, c)) => c,
            None => return Ok(Token::new(TokenKind::Eof, Span::new(start, start))),
        };

        let kind = match ch {
            '(' => {
                self.bump();
                TokenKind::LParen
            }
            ')' => {
                self.bump();
                TokenKind::RParen
            }
            '{' => {
                self.bump();
                TokenKind::LBrace
            }
            '}' => {
                self.bump();
                TokenKind::RBrace
            }
            '[' => {
                self.bump();
                TokenKind::LBracket
            }
            ']' => {
                self.bump();
                TokenKind::RBracket
            }
            ',' => {
                self.bump();
                TokenKind::Comma
            }
            ';' => {
                self.bump();
                TokenKind::Semicolon
            }
            ':' => {
                self.bump();
                if self.current_char() == Some(':') {
                    self.bump();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Colon
                }
            }
            '.' => {
                self.bump();
                TokenKind::Dot
            }
            '?' => {
                self.bump();
                TokenKind::Question
            }
            '@' => {
                self.bump();
                TokenKind::At
            }
            '#' => {
                self.bump();
                let mut text = String::from("#");
                while self.current_char().is_some_and(|c| c.is_ascii_hexdigit()) {
                    text.push(self.current_char().unwrap());
                    self.bump();
                }
                TokenKind::ViewText(text)
            }
            '|' => {
                self.bump();
                TokenKind::Pipe
            }
            '&' => {
                self.bump();
                if self.next_char() == Some('&') {
                    self.bump();
                    TokenKind::AndAnd
                } else {
                    TokenKind::Amp
                }
            }
            '+' => {
                self.bump();
                if self.current_char() == Some('=') {
                    self.bump();
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                self.bump();
                if self.current_char() == Some('>') {
                    self.bump();
                    TokenKind::Arrow
                } else if self.current_char() == Some('=') {
                    self.bump();
                    TokenKind::MinusEq
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                self.bump();
                TokenKind::Star
            }
            '/' => {
                self.bump();
                TokenKind::Slash
            }
            '%' => {
                self.bump();
                TokenKind::Percent
            }
            '!' => {
                self.bump();
                if self.current_char() == Some('=') {
                    self.bump();
                    TokenKind::Ne
                } else {
                    TokenKind::Bang
                }
            }
            '=' => {
                self.bump();
                if self.current_char() == Some('=') {
                    self.bump();
                    TokenKind::EqEq
                } else if self.current_char() == Some('>') {
                    self.bump();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Eq
                }
            }
            '<' => {
                self.bump();
                if self.current_char() == Some('=') {
                    self.bump();
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                self.bump();
                if self.current_char() == Some('=') {
                    self.bump();
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '"' => self.string_literal()?,
            '\'' => self.char_literal()?,
            c if c.is_ascii_digit() => self.number_literal()?,
            c if is_ident_start(c) => self.ident_or_keyword()?,
            c if is_view_text_start(c) => self.view_text_run()?,
            _ => {
                return Err(LexError {
                    message: format!("unexpected character `{ch}`"),
                    span: Span::new(start, self.offset),
                });
            }
        };

        Ok(Token::new(kind, Span::new(start, self.offset)))
    }

    fn string_literal(&mut self) -> Result<TokenKind, LexError> {
        let start = self.offset;
        self.bump(); // opening quote
        let mut value = String::new();
        loop {
            match self.current {
                None => {
                    return Err(LexError {
                        message: "unterminated string literal".into(),
                        span: Span::new(start, self.offset),
                    });
                }
                Some((_, '"')) => {
                    self.bump();
                    break;
                }
                Some((_, '\\')) => {
                    self.bump();
                    let escaped = match self.current {
                        Some((_, 'n')) => {
                            self.bump();
                            '\n'
                        }
                        Some((_, 't')) => {
                            self.bump();
                            '\t'
                        }
                        Some((_, 'r')) => {
                            self.bump();
                            '\r'
                        }
                        Some((_, '\\')) => {
                            self.bump();
                            '\\'
                        }
                        Some((_, '"')) => {
                            self.bump();
                            '"'
                        }
                        Some((_, c)) => {
                            return Err(LexError {
                                message: format!("invalid escape sequence `\\{c}`"),
                                span: Span::new(self.offset - 1, self.offset + 1),
                            });
                        }
                        None => {
                            return Err(LexError {
                                message: "unterminated escape sequence".into(),
                                span: Span::new(start, self.offset),
                            });
                        }
                    };
                    value.push(escaped);
                }
                Some((_, c)) => {
                    value.push(c);
                    self.bump();
                }
            }
        }
        Ok(TokenKind::StringLit(value))
    }

    fn char_literal(&mut self) -> Result<TokenKind, LexError> {
        let start = self.offset;
        self.bump();
        let ch = match self.current {
            Some((_, c)) if c != '\'' => {
                self.bump();
                c
            }
            _ => {
                return Err(LexError {
                    message: "invalid character literal".into(),
                    span: Span::new(start, self.offset),
                });
            }
        };
        if self.current.map(|(_, c)| c) != Some('\'') {
            return Err(LexError {
                message: "character literal must contain exactly one character".into(),
                span: Span::new(start, self.offset),
            });
        }
        self.bump();
        Ok(TokenKind::CharLit(ch))
    }

    fn number_literal(&mut self) -> Result<TokenKind, LexError> {
        let start = self.offset;
        let mut int_part = String::new();
        while matches!(self.current, Some((_, c)) if c.is_ascii_digit()) {
            int_part.push(self.current.unwrap().1);
            self.bump();
        }

        if self.current_char() == Some('.')
            && self.next_char().map(|c| c.is_ascii_digit()).unwrap_or(false)
        {
            self.bump();
            let mut frac = String::new();
            while matches!(self.current, Some((_, c)) if c.is_ascii_digit()) {
                frac.push(self.current.unwrap().1);
                self.bump();
            }
            Ok(TokenKind::FloatLit(format!("{int_part}.{frac}")))
        } else {
            let value: i64 = int_part.parse().map_err(|_| LexError {
                message: format!("invalid integer literal `{int_part}`"),
                span: Span::new(start, self.offset),
            })?;
            Ok(TokenKind::IntLit(value))
        }
    }

    fn view_text_run(&mut self) -> Result<TokenKind, LexError> {
        let mut text = String::new();
        while let Some((_, ch)) = self.current {
            if ch == '<' || ch == '{' {
                break;
            }
            text.push(ch);
            self.bump();
        }
        Ok(TokenKind::ViewText(text))
    }

    fn ident_or_keyword(&mut self) -> Result<TokenKind, LexError> {
        let start = self.offset;
        let mut text = String::new();
        while matches!(self.current, Some((_, c)) if is_ident_continue(c)) {
            text.push(self.current.unwrap().1);
            self.bump();
        }
        Ok(keyword_or_ident(&text))
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexError> {
        loop {
            match self.current {
                Some((_, c)) if c.is_whitespace() => {
                    self.bump();
                }
                Some((idx, '/')) if self.next_char() == Some('/') => {
                    self.bump();
                    self.bump();
                    while matches!(self.current, Some((_, c)) if c != '\n') {
                        self.bump();
                    }
                }
                Some((idx, '/')) if self.next_char() == Some('*') => {
                    let start = self.offset;
                    self.bump();
                    self.bump();
                    let mut closed = false;
                    while self.current.is_some() {
                        if self.current_char() == Some('*')
                            && self.next_char() == Some('/')
                        {
                            self.bump();
                            self.bump();
                            closed = true;
                            break;
                        }
                        self.bump();
                    }
                    if !closed {
                        return Err(LexError {
                            message: "unterminated block comment".into(),
                            span: Span::new(start, self.offset),
                        });
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn bump(&mut self) {
        if let Some((idx, ch)) = self.current {
            self.offset = (idx + ch.len_utf8()) as u32;
            self.current = self.chars.next();
        } else {
            self.offset = self.source.len() as u32;
        }
    }

    fn current_char(&self) -> Option<char> {
        self.current.map(|(_, c)| c)
    }

    fn next_char(&self) -> Option<char> {
        if let Some((idx, _)) = self.current {
            self.source[idx..].chars().nth(1)
        } else {
            None
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_view_text_start(c: char) -> bool {
    if c.is_ascii() {
        return matches!(c, '·' | '—' | '⚡' | '🧩' | '📦' | '🌍' | '+' | '|');
    }
    !c.is_whitespace() && !c.is_control()
}

fn keyword_or_ident(text: &str) -> TokenKind {
    match text {
        "fn" => TokenKind::Fn,
        "let" => TokenKind::Let,
        "mut" => TokenKind::Mut,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "return" => TokenKind::Return,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "module" => TokenKind::Module,
        "import" => TokenKind::Import,
        "export" => TokenKind::Export,
        "extern" => TokenKind::Extern,
        "struct" => TokenKind::Struct,
        "enum" => TokenKind::Enum,
        "match" => TokenKind::Match,
        "async" => TokenKind::Async,
        "await" => TokenKind::Await,
        "pub" => TokenKind::Pub,
        "as" => TokenKind::As,
        "in" => TokenKind::In,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "component" => TokenKind::Component,
        "state" => TokenKind::State,
        "view" => TokenKind::View,
        "style" => TokenKind::Style,
        "scoped" => TokenKind::Scoped,
        "type" => TokenKind::Type,
        "interface" => TokenKind::Interface,
        "trait" => TokenKind::Trait,
        "impl" => TokenKind::Impl,
        "for" => TokenKind::For,
        "Int" => TokenKind::Int,
        "Float" => TokenKind::Float,
        "Bool" => TokenKind::Bool,
        "String" => TokenKind::String,
        "Char" => TokenKind::Char,
        "Void" => TokenKind::Void,
        _ => TokenKind::Ident(text.to_string()),
    }
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(source).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    #[test]
    fn lexes_arrow_token() {
        let tokens = lex("->").unwrap();
        assert_eq!(tokens.len(), 2); // Arrow + Eof
        assert!(matches!(tokens[0].kind, TokenKind::Arrow));
    }

    #[test]
    fn lexes_hello_world_tokens() {
        let tokens = lex(r#"module hello export fn main() { let x = 42 }"#).unwrap();
        assert!(tokens.iter().any(|t| matches!(&t.kind, TokenKind::Ident(s) if s == "hello")));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::IntLit(42))));
    }

    #[test]
    fn rejects_unterminated_string() {
        let err = lex(r#""hello"#).unwrap_err();
        assert!(err.message.contains("unterminated"));
    }
}
