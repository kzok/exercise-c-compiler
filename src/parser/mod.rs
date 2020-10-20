mod node;
mod token_cursor;

use crate::tokenizer::Token;
pub use node::{BinaryOperator, Node};
use std::vec::Vec;
use token_cursor::TokenCursor;

struct ParserContext<'a> {
    cursor: TokenCursor<'a>,
}

impl<'a> ParserContext<'a> {
    fn new(tokens: &'a Vec<Token<'a>>) -> ParserContext<'a> {
        return ParserContext {
            cursor: TokenCursor::new(tokens),
        };
    }

    fn primary(&mut self) -> Node {
        if self.cursor.consume("(") {
            let node = self.expr();
            self.cursor.expect(")");
            return node;
        }
        if let Some(ident) = self.cursor.consume_ident() {
            return Node::LocalVar {
                offset: (ident as u32 - 'a' as u32 + 1) * 8,
            };
        }
        return Node::Number(self.cursor.expect_number());
    }

    fn unary(&mut self) -> Node {
        if self.cursor.consume("+") {
            return self.primary();
        }
        if self.cursor.consume("-") {
            return Node::Binary {
                op: BinaryOperator::Sub,
                lhs: Box::new(Node::Number(0)),
                rhs: Box::new(self.primary()),
            };
        }
        return self.primary();
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();

        loop {
            if self.cursor.consume("*") {
                node = Node::Binary {
                    op: BinaryOperator::Mul,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                };
            } else if self.cursor.consume("/") {
                node = Node::Binary {
                    op: BinaryOperator::Div,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                };
            } else {
                return node;
            }
        }
    }

    fn add(&mut self) -> Node {
        let mut node = self.mul();

        loop {
            if self.cursor.consume("+") {
                node = Node::Binary {
                    op: BinaryOperator::Add,
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
                };
            } else if self.cursor.consume("-") {
                node = Node::Binary {
                    op: BinaryOperator::Sub,
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
                };
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Node {
        let mut node = self.add();

        loop {
            if self.cursor.consume("<") {
                node = Node::Binary {
                    op: BinaryOperator::LessThan,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                };
            } else if self.cursor.consume("<=") {
                node = Node::Binary {
                    op: BinaryOperator::LessThanEqual,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                };
            } else if self.cursor.consume(">") {
                node = Node::Binary {
                    op: BinaryOperator::LessThan,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                };
            } else if self.cursor.consume(">=") {
                node = Node::Binary {
                    op: BinaryOperator::LessThanEqual,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                };
            } else {
                return node;
            }
        }
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();

        loop {
            if self.cursor.consume("==") {
                node = Node::Binary {
                    op: BinaryOperator::Equal,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                };
            } else if self.cursor.consume("!=") {
                node = Node::Binary {
                    op: BinaryOperator::NotEqual,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                };
            } else {
                return node;
            }
        }
    }

    fn assign(&mut self) -> Node {
        let mut node = self.equality();
        if self.cursor.consume("=") {
            node = Node::Binary {
                op: BinaryOperator::Assign,
                lhs: Box::new(node),
                rhs: Box::new(self.assign()),
            };
        }
        return node;
    }

    fn expr(&mut self) -> Node {
        return self.assign();
    }

    pub fn stmt(&mut self) -> Node {
        let node = self.expr();
        self.cursor.expect(";");
        return node;
    }

    pub fn remains(&mut self) -> bool {
        return self.cursor.remains();
    }
}

pub fn parse(tokens: &Vec<Token>) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut ctx = ParserContext::new(&tokens);

    while ctx.remains() {
        nodes.push(ctx.stmt());
    }

    return nodes;
}
