mod token_cursor;
mod types;

use crate::tokenizer::{Keyword, Token};
use std::rc::Rc;
use std::vec::Vec;
use token_cursor::TokenCursor;
pub use types::{BinaryOperator, Node, Program, Variable};

struct ParserContext<'a> {
    locals: Vec<Rc<Variable<'a>>>,
    cursor: TokenCursor<'a>,
}

impl<'a> ParserContext<'a> {
    fn new(tokens: &'a Vec<Token<'a>>) -> ParserContext<'a> {
        return ParserContext {
            locals: Vec::new(),
            cursor: TokenCursor::new(tokens),
        };
    }

    fn find_lvar(&self, name: &str) -> Option<Rc<Variable<'a>>> {
        for local in &self.locals {
            if local.name == name {
                return Some(local.clone());
            }
        }
        return None;
    }

    fn primary(&mut self) -> Node<'a> {
        if self.cursor.consume_sign("(") {
            let node = self.expr();
            self.cursor.expect_sign(")");
            return node;
        }
        if let Some(name) = self.cursor.consume_ident() {
            // funcall
            if self.cursor.consume_sign("(") {
                self.cursor.expect_sign(")");
                return Node::FunCall { name };
            }

            // known variable
            if let Some(local) = self.find_lvar(name) {
                return Node::LocalVar(local);
            }

            // new variable
            let previous_offset = self
                .locals
                .iter()
                .fold(0, |p, local| std::cmp::max(p, local.offset));
            let local = Rc::new(Variable {
                name: name,
                offset: previous_offset + 8,
            });
            self.locals.push(local.clone());
            return Node::LocalVar(local);
        }
        return Node::Number(self.cursor.expect_number());
    }

    fn unary(&mut self) -> Node<'a> {
        if self.cursor.consume_sign("+") {
            return self.primary();
        }
        if self.cursor.consume_sign("-") {
            return Node::Binary {
                op: BinaryOperator::Sub,
                lhs: Box::new(Node::Number(0)),
                rhs: Box::new(self.primary()),
            };
        }
        return self.primary();
    }

    fn mul(&mut self) -> Node<'a> {
        let mut node = self.unary();

        loop {
            if self.cursor.consume_sign("*") {
                node = Node::Binary {
                    op: BinaryOperator::Mul,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                };
            } else if self.cursor.consume_sign("/") {
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

    fn add(&mut self) -> Node<'a> {
        let mut node = self.mul();

        loop {
            if self.cursor.consume_sign("+") {
                node = Node::Binary {
                    op: BinaryOperator::Add,
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
                };
            } else if self.cursor.consume_sign("-") {
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

    fn relational(&mut self) -> Node<'a> {
        let mut node = self.add();

        loop {
            if self.cursor.consume_sign("<") {
                node = Node::Binary {
                    op: BinaryOperator::LessThan,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                };
            } else if self.cursor.consume_sign("<=") {
                node = Node::Binary {
                    op: BinaryOperator::LessThanEqual,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                };
            } else if self.cursor.consume_sign(">") {
                node = Node::Binary {
                    op: BinaryOperator::LessThan,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                };
            } else if self.cursor.consume_sign(">=") {
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

    fn equality(&mut self) -> Node<'a> {
        let mut node = self.relational();

        loop {
            if self.cursor.consume_sign("==") {
                node = Node::Binary {
                    op: BinaryOperator::Equal,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                };
            } else if self.cursor.consume_sign("!=") {
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

    fn assign(&mut self) -> Node<'a> {
        let mut node = self.equality();
        if self.cursor.consume_sign("=") {
            node = Node::Binary {
                op: BinaryOperator::Assign,
                lhs: Box::new(node),
                rhs: Box::new(self.assign()),
            };
        }
        return node;
    }

    fn expr(&mut self) -> Node<'a> {
        return self.assign();
    }

    pub fn stmt(&mut self) -> Node<'a> {
        // block
        if self.cursor.consume_sign("{") {
            let mut nodes: Vec<Box<Node<'a>>> = Vec::new();
            while !self.cursor.consume_sign("}") {
                nodes.push(Box::new(self.stmt()));
            }
            return Node::Block(nodes);
        }

        // if
        if self.cursor.consume_keyword(Keyword::If) {
            self.cursor.expect_sign("(");
            let cond = Box::new(self.expr());
            self.cursor.expect_sign(")");
            let then = Box::new(self.stmt());
            let els = if self.cursor.consume_keyword(Keyword::Else) {
                Some(Box::new(self.stmt()))
            } else {
                None
            };
            return Node::If { cond, then, els };
        }

        // while
        if self.cursor.consume_keyword(Keyword::While) {
            self.cursor.expect_sign("(");
            let cond = Box::new(self.expr());
            self.cursor.expect_sign(")");
            let then = Box::new(self.stmt());
            return Node::While { cond, then };
        }

        // for
        if self.cursor.consume_keyword(Keyword::For) {
            self.cursor.expect_sign("(");
            let init = if !self.cursor.consume_sign(";") {
                let node = self.expr();
                self.cursor.expect_sign(";");
                Some(Box::new(node))
            } else {
                None
            };
            let cond = if !self.cursor.consume_sign(";") {
                let node = self.expr();
                self.cursor.expect_sign(";");
                Some(Box::new(node))
            } else {
                None
            };
            let inc = if !self.cursor.consume_sign(")") {
                let node = self.expr();
                self.cursor.expect_sign(")");
                Some(Box::new(node))
            } else {
                None
            };
            let then = Box::new(self.stmt());
            return Node::For {
                init,
                cond,
                inc,
                then,
            };
        }

        // return
        if self.cursor.consume_keyword(Keyword::Return) {
            let node = Node::Return(Box::new(self.expr()));
            self.cursor.expect_sign(";");
            return node;
        }

        let node = self.expr();
        self.cursor.expect_sign(";");
        return node;
    }

    pub fn remains(&self) -> bool {
        return self.cursor.remains();
    }
}

pub fn parse<'a>(tokens: &'a Vec<Token>) -> Program<'a> {
    let mut nodes = Vec::new();
    let mut ctx = ParserContext::new(&tokens);

    while ctx.remains() {
        nodes.push(ctx.stmt());
    }

    let stack_size = ctx
        .locals
        .iter()
        .fold(0, |p, local| std::cmp::max(p, local.offset));
    return Program {
        locals: ctx.locals,
        nodes,
        stack_size,
    };
}