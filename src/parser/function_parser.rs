use super::token_cursor::TokenCursor;
use super::types::{BinaryOperator, Function, Node, NodeKind, Variable};
use crate::tokenizer::{Keyword, TokenKind};
use std::rc::Rc;

pub struct FunctionParser<'local, 'outer: 'local> {
    locals: Vec<Rc<Variable<'outer>>>,
    cursor: &'local mut TokenCursor<'outer>,
}

fn make_node<'a>(kind: NodeKind<'a>) -> Node<'a> {
    return Node { kind };
}

impl<'local, 'outer: 'local> FunctionParser<'local, 'outer> {
    pub fn new(cursor: &'local mut TokenCursor<'outer>) -> FunctionParser<'local, 'outer> {
        return FunctionParser {
            locals: Vec::new(),
            cursor,
        };
    }

    fn new_localvar(&mut self, name: &'outer str) -> Rc<Variable<'outer>> {
        let previous_offset = self
            .locals
            .iter()
            .fold(0, |p, local| std::cmp::max(p, local.offset));
        let local = Rc::new(Variable {
            name: name,
            offset: previous_offset + 8,
        });
        self.locals.push(local.clone());
        return local;
    }

    fn find_lvar(&self, name: &str) -> Option<Rc<Variable<'outer>>> {
        for local in &self.locals {
            if local.name == name {
                return Some(local.clone());
            }
        }
        return None;
    }

    fn base_type(&mut self) {
        self.cursor.expect_keyword(Keyword::Int);
    }

    fn read_func_params(&mut self) -> Vec<Rc<Variable<'outer>>> {
        let mut params = Vec::new();
        if self.cursor.consume_sign(")") {
            return params;
        }
        self.base_type();
        let name = self.cursor.expect_ident();
        let local = self.new_localvar(name);
        self.locals.push(local.clone());
        params.push(local);

        while !self.cursor.consume_sign(")") {
            self.cursor.expect_sign(",");
            self.base_type();
            let name = self.cursor.expect_ident();
            let local = self.new_localvar(name);
            self.locals.push(local.clone());
            params.push(local);
        }

        return params;
    }

    fn func_args(&mut self) -> Vec<Box<Node<'outer>>> {
        let mut args = Vec::new();
        if self.cursor.consume_sign(")") {
            return args;
        }
        args.push(Box::new(self.assign()));
        while self.cursor.consume_sign(",") {
            args.push(Box::new(self.assign()));
        }
        self.cursor.expect_sign(")");
        return args;
    }

    fn declaretion(&mut self) -> Node<'outer> {
        self.base_type();
        let name = self.cursor.expect_ident();
        let var = self.new_localvar(name);
        if self.cursor.consume_sign(";") {
            return make_node(NodeKind::Null);
        }
        self.cursor.expect_sign("=");
        let lhs = Box::new(make_node(NodeKind::LocalVar(var)));
        let rhs = Box::new(self.expr());
        self.cursor.expect_sign(";");
        return make_node(NodeKind::Binary {
            op: BinaryOperator::Assign,
            lhs,
            rhs,
        });
    }

    fn primary(&mut self) -> Node<'outer> {
        if self.cursor.consume_sign("(") {
            let node = self.expr();
            self.cursor.expect_sign(")");
            return node;
        }
        let maybe_name = self.cursor.consume_ident();
        if let Some(name) = maybe_name {
            // funcall
            if self.cursor.consume_sign("(") {
                let args = self.func_args();
                return make_node(NodeKind::FunCall { name, args });
            }

            // known variable
            if let Some(local) = self.find_lvar(name) {
                return make_node(NodeKind::LocalVar(local));
            }

            // new variable
            let local = self.new_localvar(name);
            return make_node(NodeKind::LocalVar(local));
        }
        return make_node(NodeKind::Number(self.cursor.expect_number()));
    }

    fn unary(&mut self) -> Node<'outer> {
        if self.cursor.consume_sign("+") {
            return self.primary();
        }
        if self.cursor.consume_sign("-") {
            return make_node(NodeKind::Binary {
                op: BinaryOperator::Sub,
                lhs: Box::new(make_node(NodeKind::Number(0))),
                rhs: Box::new(self.primary()),
            });
        }
        if self.cursor.consume_sign("&") {
            return make_node(NodeKind::Addr(Box::new(self.unary())));
        }
        if self.cursor.consume_sign("*") {
            return make_node(NodeKind::Deref(Box::new(self.unary())));
        }
        return self.primary();
    }

    fn mul(&mut self) -> Node<'outer> {
        let mut node = self.unary();

        loop {
            if self.cursor.consume_sign("*") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::Mul,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                });
            } else if self.cursor.consume_sign("/") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::Div,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                });
            } else {
                return node;
            }
        }
    }

    fn add(&mut self) -> Node<'outer> {
        let mut node = self.mul();

        loop {
            if self.cursor.consume_sign("+") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::Add,
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
                });
            } else if self.cursor.consume_sign("-") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::Sub,
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
                });
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Node<'outer> {
        let mut node = self.add();

        loop {
            if self.cursor.consume_sign("<") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::LessThan,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                });
            } else if self.cursor.consume_sign("<=") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::LessThanEqual,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                });
            } else if self.cursor.consume_sign(">") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::LessThan,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                });
            } else if self.cursor.consume_sign(">=") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::LessThanEqual,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                });
            } else {
                return node;
            }
        }
    }

    fn equality(&mut self) -> Node<'outer> {
        let mut node = self.relational();

        loop {
            if self.cursor.consume_sign("==") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::Equal,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                });
            } else if self.cursor.consume_sign("!=") {
                node = make_node(NodeKind::Binary {
                    op: BinaryOperator::NotEqual,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                });
            } else {
                return node;
            }
        }
    }

    fn assign(&mut self) -> Node<'outer> {
        let mut node = self.equality();
        if self.cursor.consume_sign("=") {
            node = make_node(NodeKind::Binary {
                op: BinaryOperator::Assign,
                lhs: Box::new(node),
                rhs: Box::new(self.assign()),
            });
        }
        return node;
    }

    fn expr(&mut self) -> Node<'outer> {
        return self.assign();
    }

    fn stmt(&mut self) -> Node<'outer> {
        // block
        if self.cursor.consume_sign("{") {
            let mut nodes: Vec<Box<Node<'outer>>> = Vec::new();
            while !self.cursor.consume_sign("}") {
                nodes.push(Box::new(self.stmt()));
            }
            return make_node(NodeKind::Block(nodes));
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
            return make_node(NodeKind::If { cond, then, els });
        }

        // while
        if self.cursor.consume_keyword(Keyword::While) {
            self.cursor.expect_sign("(");
            let cond = Box::new(self.expr());
            self.cursor.expect_sign(")");
            let then = Box::new(self.stmt());
            return make_node(NodeKind::While { cond, then });
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
            return make_node(NodeKind::For {
                init,
                cond,
                inc,
                then,
            });
        }

        // return
        if self.cursor.consume_keyword(Keyword::Return) {
            let node = make_node(NodeKind::Return(Box::new(self.expr())));
            self.cursor.expect_sign(";");
            return node;
        }

        // declaretion
        if self.cursor.current().kind == TokenKind::Keyword(Keyword::Int) {
            return self.declaretion();
        }

        let node = self.expr();
        self.cursor.expect_sign(";");
        return node;
    }

    pub fn parse(cursor: &'local mut TokenCursor<'outer>) -> Option<Function<'outer>> {
        let mut ctx = FunctionParser::new(cursor);
        ctx.base_type();
        return ctx.cursor.consume_ident().and_then(|name| {
            let mut nodes = Vec::new();

            ctx.cursor.expect_sign("(");
            let params = ctx.read_func_params();
            ctx.cursor.expect_sign("{");
            while !ctx.cursor.consume_sign("}") {
                nodes.push(ctx.stmt());
            }

            let stack_size = ctx
                .locals
                .iter()
                .fold(0, |p, local| std::cmp::max(p, local.offset));
            return Some(Function {
                name,
                params,
                locals: ctx.locals,
                nodes,
                stack_size,
            });
        });
    }
}
