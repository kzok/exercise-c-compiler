use crate::tokenizer::{Token, TokenKind};
use std::vec::Vec;

pub struct TokenCursor<'a> {
    tokens: &'a Vec<Token<'a>>,
    index: usize,
}

impl<'a> TokenCursor<'a> {
    pub fn new(tokens: &'a Vec<Token<'a>>) -> TokenCursor<'a> {
        return TokenCursor { tokens, index: 0 };
    }

    fn current(&self) -> &Token<'a> {
        return &self.tokens[self.index];
    }

    fn seek(&mut self) {
        self.index += 1;
    }

    pub fn remains(&mut self) -> bool {
        match self.current().kind {
            TokenKind::Eof => return false,
            _ => return true,
        };
    }

    pub fn consume(&mut self, op: &str) -> bool {
        match self.current().kind {
            TokenKind::Reserved(token_op) if token_op == op => {
                self.seek();
                return true;
            }
            _ => return false,
        }
    }

    pub fn consume_ident(&mut self) -> Option<char> {
        match self.current().kind {
            TokenKind::Ident(c) => {
                self.seek();
                return Some(c);
            }
            _ => return None,
        }
    }

    pub fn expect(&mut self, op: &str) {
        if self.consume(op) {
            return;
        }
        self.current()
            .report_error(&format!("'{}' ではありません", op));
    }

    pub fn expect_number(&mut self) -> u32 {
        match self.current().kind {
            TokenKind::Number(n) => {
                self.seek();
                return n;
            }
            _ => self.current().report_error("数ではありません"),
        }
    }
}
