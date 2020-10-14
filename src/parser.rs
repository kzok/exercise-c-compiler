use crate::tokenizer::{Token, TokenKind};
use std::iter::Peekable;
use std::slice::Iter;
use std::vec::Vec;

#[derive(Debug, PartialEq)]
pub enum Node {
    Number(u32),                            // 整数
    Add { lhs: Box<Node>, rhs: Box<Node> }, // +
    Sub { lhs: Box<Node>, rhs: Box<Node> }, // -
    Mul { lhs: Box<Node>, rhs: Box<Node> }, // *
    Div { lhs: Box<Node>, rhs: Box<Node> }, // /
}

struct ParserContext<'a> {
    token_iter: Peekable<Iter<'a, Token<'a>>>,
}

impl<'a> ParserContext<'a> {
    pub fn new(token_iter: Peekable<Iter<'a, Token<'a>>>) -> ParserContext<'a> {
        return ParserContext { token_iter };
    }

    pub fn consume(&mut self, op: &str) -> bool {
        if let Some(token) = self.token_iter.peek() {
            match token.kind {
                TokenKind::Reserved(token_op) if token_op == op => {
                    self.token_iter.next();
                    return true;
                }
                _ => {}
            }
        }
        return false;
    }

    pub fn expect(&mut self, op: &str) {
        if self.consume(op) {
            return;
        }
        let token = self.token_iter.peek().unwrap();
        token.report_error(&format!("'{}' ではありません", op));
    }

    pub fn expect_number(&mut self) -> u32 {
        let token = self.token_iter.next().unwrap();
        match token.kind {
            TokenKind::Number(n) => return n,
            _ => token.report_error("数ではありません"),
        }
    }
}

fn primary(ctx: &mut ParserContext) -> Node {
    if ctx.consume("(") {
        let node = expr(ctx);
        ctx.expect(")");
        return node;
    }
    return Node::Number(ctx.expect_number());
}

fn mul(ctx: &mut ParserContext) -> Node {
    let mut node = primary(ctx);

    loop {
        if ctx.consume("*") {
            node = Node::Mul {
                lhs: Box::new(node),
                rhs: Box::new(primary(ctx)),
            };
        } else if ctx.consume("/") {
            node = Node::Div {
                lhs: Box::new(node),
                rhs: Box::new(primary(ctx)),
            };
        } else {
            return node;
        }
    }
}

fn expr(ctx: &mut ParserContext) -> Node {
    let mut node = mul(ctx);

    loop {
        if ctx.consume("+") {
            node = Node::Add {
                lhs: Box::new(node),
                rhs: Box::new(mul(ctx)),
            };
        } else if ctx.consume("-") {
            node = Node::Sub {
                lhs: Box::new(node),
                rhs: Box::new(mul(ctx)),
            };
        } else {
            return node;
        }
    }
}

pub fn parse(tokens: &Vec<Token>) -> Node {
    let mut ctx = ParserContext::new(tokens.iter().peekable());
    return expr(&mut ctx);
}
