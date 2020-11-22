use crate::tokenizer::{Keyword, Token, TokenKind};
use std::vec::Vec;

pub struct TokenCursor<'a> {
    tokens: &'a Vec<Token<'a>>,
    index: usize,
}

impl<'a> TokenCursor<'a> {
    pub fn new(tokens: &'a Vec<Token<'a>>) -> TokenCursor<'a> {
        return TokenCursor { tokens, index: 0 };
    }

    pub fn current(&self) -> &Token<'a> {
        return &self.tokens[self.index];
    }

    fn seek(&mut self) {
        self.index += 1;
    }

    pub fn remains(&self) -> bool {
        match self.current().kind {
            TokenKind::Eof => return false,
            _ => return true,
        };
    }

    pub fn report_error(&self, msg: &str) -> ! {
        self.current().report_error(msg);
    }

    pub fn consume_sign(&mut self, sign: &str) -> bool {
        match self.current().kind {
            TokenKind::Sign(s) if s == sign => {
                self.seek();
                return true;
            }
            _ => return false,
        }
    }

    pub fn consume_keyword(&mut self, keyword: Keyword) -> bool {
        match self.current().kind {
            TokenKind::Keyword(s) if s == keyword => {
                self.seek();
                return true;
            }
            _ => return false,
        }
    }

    pub fn consume_ident(&mut self) -> Option<&'a str> {
        match self.current().kind {
            TokenKind::Ident(c) => {
                self.seek();
                return Some(c);
            }
            _ => return None,
        }
    }

    pub fn expect_keyword(&mut self, keyword: Keyword) {
        if self.consume_keyword(keyword) {
            return;
        }
        self.report_error(&format!("'{}' ではありません", keyword));
    }

    pub fn expect_sign(&mut self, op: &str) {
        if self.consume_sign(op) {
            return;
        }
        self.report_error(&format!("'{}' ではありません", op));
    }

    pub fn expect_number(&mut self) -> u32 {
        match self.current().kind {
            TokenKind::Number(n) => {
                self.seek();
                return n;
            }
            _ => self.report_error("数ではありません"),
        }
    }

    pub fn expect_ident(&mut self) -> &'a str {
        match self.current().kind {
            TokenKind::Ident(ident) => {
                self.seek();
                return ident;
            }
            _ => self.report_error("数ではありません"),
        }
    }
}
