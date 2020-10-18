mod node;

pub use node::{BinaryOperator, Node};

use crate::tokenizer::{Token, TokenKind};
use std::iter::Peekable;
use std::slice::Iter;
use std::vec::Vec;

struct ParserContext<'a> {
    token_iter: Peekable<Iter<'a, Token<'a>>>,
}

impl<'a> ParserContext<'a> {
    pub fn new(token_iter: Peekable<Iter<'a, Token<'a>>>) -> ParserContext<'a> {
        return ParserContext { token_iter };
    }

    pub fn remains(&mut self) -> bool {
        return self.token_iter.peek().is_some();
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

    pub fn consume_ident(&mut self) -> Option<char> {
        if let Some(token) = self.token_iter.peek() {
            match token.kind {
                TokenKind::Ident(c) => {
                    self.token_iter.next();
                    return Some(c);
                }
                _ => (),
            }
        }
        return None;
    }

    pub fn expect(&mut self, op: &str) {
        if self.consume(op) {
            return;
        }
        let error_message = format!("'{}' ではありません", op);
        if let Some(token) = self.token_iter.peek() {
            token.report_error(&error_message);
        }
        panic!(error_message);
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
    if let Some(ident) = ctx.consume_ident() {
        return Node::LocalVar {
            offset: (ident as u32 - 'a' as u32 + 1) * 8,
        };
    }
    return Node::Number(ctx.expect_number());
}

fn unary(ctx: &mut ParserContext) -> Node {
    if ctx.consume("+") {
        return primary(ctx);
    }
    if ctx.consume("-") {
        return Node::Binary {
            op: BinaryOperator::Sub,
            lhs: Box::new(Node::Number(0)),
            rhs: Box::new(primary(ctx)),
        };
    }
    return primary(ctx);
}

fn mul(ctx: &mut ParserContext) -> Node {
    let mut node = unary(ctx);

    loop {
        if ctx.consume("*") {
            node = Node::Binary {
                op: BinaryOperator::Mul,
                lhs: Box::new(node),
                rhs: Box::new(unary(ctx)),
            };
        } else if ctx.consume("/") {
            node = Node::Binary {
                op: BinaryOperator::Div,
                lhs: Box::new(node),
                rhs: Box::new(unary(ctx)),
            };
        } else {
            return node;
        }
    }
}

fn add(ctx: &mut ParserContext) -> Node {
    let mut node = mul(ctx);

    loop {
        if ctx.consume("+") {
            node = Node::Binary {
                op: BinaryOperator::Add,
                lhs: Box::new(node),
                rhs: Box::new(mul(ctx)),
            };
        } else if ctx.consume("-") {
            node = Node::Binary {
                op: BinaryOperator::Sub,
                lhs: Box::new(node),
                rhs: Box::new(mul(ctx)),
            };
        } else {
            return node;
        }
    }
}

fn relational(ctx: &mut ParserContext) -> Node {
    let mut node = add(ctx);

    loop {
        if ctx.consume("<") {
            node = Node::Binary {
                op: BinaryOperator::LessThan,
                lhs: Box::new(node),
                rhs: Box::new(add(ctx)),
            };
        } else if ctx.consume("<=") {
            node = Node::Binary {
                op: BinaryOperator::LessThanEqual,
                lhs: Box::new(node),
                rhs: Box::new(add(ctx)),
            };
        } else if ctx.consume(">") {
            node = Node::Binary {
                op: BinaryOperator::LessThan,
                lhs: Box::new(add(ctx)),
                rhs: Box::new(node),
            };
        } else if ctx.consume(">=") {
            node = Node::Binary {
                op: BinaryOperator::LessThanEqual,
                lhs: Box::new(add(ctx)),
                rhs: Box::new(node),
            };
        } else {
            return node;
        }
    }
}

fn equality(ctx: &mut ParserContext) -> Node {
    let mut node = relational(ctx);

    loop {
        if ctx.consume("==") {
            node = Node::Binary {
                op: BinaryOperator::Equal,
                lhs: Box::new(node),
                rhs: Box::new(relational(ctx)),
            };
        } else if ctx.consume("!=") {
            node = Node::Binary {
                op: BinaryOperator::NotEqual,
                lhs: Box::new(node),
                rhs: Box::new(relational(ctx)),
            };
        } else {
            return node;
        }
    }
}

fn assign(ctx: &mut ParserContext) -> Node {
    let mut node = equality(ctx);
    if ctx.consume("=") {
        node = Node::Binary {
            op: BinaryOperator::Assign,
            lhs: Box::new(node),
            rhs: Box::new(assign(ctx)),
        };
    }
    return node;
}

fn expr(ctx: &mut ParserContext) -> Node {
    return assign(ctx);
}

fn stmt(ctx: &mut ParserContext) -> Node {
    let node = expr(ctx);
    ctx.expect(";");
    return node;
}

pub fn parse(tokens: &Vec<Token>) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut ctx = ParserContext::new(tokens.iter().peekable());

    while ctx.remains() {
        nodes.push(stmt(&mut ctx));
    }

    return nodes;
}
