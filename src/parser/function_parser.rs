use super::token_cursor::TokenCursor;
use super::types::{Function, Node, NodeKind, Type, Variable};
use crate::tokenizer::{Keyword, TokenKind};
use std::rc::Rc;

fn detect_type(kind: &NodeKind) -> Option<Type> {
    match kind {
        NodeKind::Mul { lhs: _, rhs: _ }
        | NodeKind::Div { lhs: _, rhs: _ }
        | NodeKind::Equal { lhs: _, rhs: _ }
        | NodeKind::NotEqual { lhs: _, rhs: _ }
        | NodeKind::LessThan { lhs: _, rhs: _ }
        | NodeKind::LessThanEqual { lhs: _, rhs: _ }
        | NodeKind::FunCall { name: _, args: _ }
        | NodeKind::Number(_) => Some(Type::Int),
        NodeKind::Add { lhs, rhs } => match rhs.ty {
            Some(Type::Pointer(_)) => panic!("ポインタを加算の右辺値に指定できません"),
            Some(Type::Array(_, _)) => panic!("配列を加算の右辺値に指定できません"),
            _ => lhs.ty.clone(),
        },
        NodeKind::Sub { lhs, rhs } => match rhs.ty {
            Some(Type::Pointer(_)) => panic!("ポインタを減算の右辺値に指定できません"),
            Some(Type::Array(_, _)) => panic!("配列を減算の右辺値に指定できません"),
            _ => lhs.ty.clone(),
        },
        NodeKind::LocalVar(var) => Some((*var).ty.clone()),
        NodeKind::Assign { lhs, rhs: _ } => lhs.ty.clone(),
        NodeKind::Addr(target) => match &target.ty {
            Some(Type::Array(base, _)) => Some(Type::Pointer(Box::new(*base.clone()))),
            Some(ty) => Some(Type::Pointer(Box::new(ty.clone()))),
            _ => panic!("アドレス参照先の型が不明です"),
        },
        NodeKind::Deref(target) => match &target.ty {
            Some(Type::Pointer(base)) | Some(Type::Array(base, _)) => Some(*base.clone()),
            _ => panic!("デリファレンスできない型です"),
        },
        _ => None,
    }
}

fn make_node<'a>(mut kind: NodeKind<'a>) -> Node<'a> {
    match &mut kind {
        // NOTE: 加算の右辺値がポインタ型や配列型の場合は左辺値と入れ替える
        NodeKind::Add { lhs, rhs } => match rhs.ty {
            Some(Type::Pointer(_)) | Some(Type::Array(_, _)) => std::mem::swap(lhs, rhs),
            _ => {}
        },
        _ => {}
    }
    let ty = detect_type(&kind);
    return Node { kind, ty };
}

struct FunctionVariables<'a> {
    vars: Vec<Rc<Variable<'a>>>,
}
impl<'a> FunctionVariables<'a> {
    pub fn new() -> FunctionVariables<'a> {
        return FunctionVariables { vars: Vec::new() };
    }

    pub fn stack_size(&self) -> u32 {
        return self.vars.iter().fold(0, |p, var| p + var.ty.size());
    }

    pub fn new_var(&mut self, name: &'a str, ty: Type) -> Rc<Variable<'a>> {
        let stack_size = self.stack_size();
        let var = Rc::new(Variable {
            name,
            offset: stack_size + ty.size(),
            ty,
            is_local: true,
        });
        self.vars.push(var.clone());
        return var;
    }

    pub fn find(&self, name: &str) -> Option<Rc<Variable<'a>>> {
        for var in &self.vars {
            if var.name == name {
                return Some(var.clone());
            }
        }
        return None;
    }

    pub fn dump_to_vec(self) -> Vec<Rc<Variable<'a>>> {
        return self.vars;
    }
}

pub struct FunctionParser<'local, 'outer: 'local> {
    variables: FunctionVariables<'outer>,
    cursor: &'local mut TokenCursor<'outer>,
}
impl<'local, 'outer: 'local> FunctionParser<'local, 'outer> {
    pub fn new(cursor: &'local mut TokenCursor<'outer>) -> FunctionParser<'local, 'outer> {
        return FunctionParser {
            variables: FunctionVariables::new(),
            cursor,
        };
    }

    fn read_type_suffix(&mut self, ty: Type) -> Type {
        if !self.cursor.consume_sign("[") {
            return ty;
        }
        let size = self.cursor.expect_number();
        self.cursor.expect_sign("]");
        let ty = self.read_type_suffix(ty);
        return Type::Array(Box::new(ty), size);
    }

    fn base_type(&mut self) -> Type {
        self.cursor.expect_keyword(Keyword::Int);
        let mut ty = Type::Int;
        loop {
            if !self.cursor.consume_sign("*") {
                break;
            }
            ty = Type::Pointer(Box::new(ty));
        }
        return ty;
    }

    fn read_func_params(&mut self) -> Vec<Rc<Variable<'outer>>> {
        let mut params = Vec::new();
        if self.cursor.consume_sign(")") {
            return params;
        }
        let ty = self.base_type();
        let name = self.cursor.expect_ident();
        let ty = self.read_type_suffix(ty);
        let var = self.variables.new_var(name, ty);
        params.push(var);

        while !self.cursor.consume_sign(")") {
            self.cursor.expect_sign(",");
            let ty = self.base_type();
            let name = self.cursor.expect_ident();
            let ty = self.read_type_suffix(ty);
            let var = self.variables.new_var(name, ty);
            params.push(var);
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
        let ty = self.base_type();
        let name = self.cursor.expect_ident();
        let ty = self.read_type_suffix(ty);
        let var = self.variables.new_var(name, ty);
        if self.cursor.consume_sign(";") {
            return make_node(NodeKind::Null);
        }
        self.cursor.expect_sign("=");
        let lhs = Box::new(make_node(NodeKind::LocalVar(var)));
        let rhs = Box::new(self.expr());
        self.cursor.expect_sign(";");
        return make_node(NodeKind::Assign { lhs, rhs });
    }

    fn primary(&mut self) -> Node<'outer> {
        if self.cursor.consume_keyword(Keyword::SizeOf) {
            let target = Box::new(self.unary());
            let size = target.ty.unwrap().size();
            return make_node(NodeKind::Number(size));
        }
        if self.cursor.consume_sign("(") {
            let node = self.expr();
            self.cursor.expect_sign(")");
            return node;
        }
        if let Some(name) = self.cursor.consume_ident() {
            // funcall
            if self.cursor.consume_sign("(") {
                let args = self.func_args();
                return make_node(NodeKind::FunCall { name, args });
            }

            // known variable
            if let Some(local) = self.variables.find(name) {
                return make_node(NodeKind::LocalVar(local));
            }

            self.cursor
                .previous()
                .report_error(&format!("未定義の変数 \"{}\" を参照しました。", name));
        }
        return make_node(NodeKind::Number(self.cursor.expect_number()));
    }

    fn postfix(&mut self) -> Node<'outer> {
        let mut node = self.primary();

        while self.cursor.consume_sign("[") {
            let exp = make_node(NodeKind::Add {
                lhs: Box::new(node),
                rhs: Box::new(self.expr()),
            });
            self.cursor.expect_sign("]");
            node = make_node(NodeKind::Deref(Box::new(exp)));
        }
        return node;
    }

    fn unary(&mut self) -> Node<'outer> {
        if self.cursor.consume_sign("+") {
            return self.primary();
        }
        if self.cursor.consume_sign("-") {
            return make_node(NodeKind::Sub {
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
        return self.postfix();
    }

    fn mul(&mut self) -> Node<'outer> {
        let mut node = self.unary();

        loop {
            if self.cursor.consume_sign("*") {
                node = make_node(NodeKind::Mul {
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                });
            } else if self.cursor.consume_sign("/") {
                node = make_node(NodeKind::Div {
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
                node = make_node(NodeKind::Add {
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
                });
            } else if self.cursor.consume_sign("-") {
                node = make_node(NodeKind::Sub {
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
                node = make_node(NodeKind::LessThan {
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                });
            } else if self.cursor.consume_sign("<=") {
                node = make_node(NodeKind::LessThanEqual {
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                });
            } else if self.cursor.consume_sign(">") {
                node = make_node(NodeKind::LessThan {
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                });
            } else if self.cursor.consume_sign(">=") {
                node = make_node(NodeKind::LessThanEqual {
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
                node = make_node(NodeKind::Equal {
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                });
            } else if self.cursor.consume_sign("!=") {
                node = make_node(NodeKind::NotEqual {
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
            node = make_node(NodeKind::Assign {
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

            let stack_size = ctx.variables.stack_size();
            return Some(Function {
                name,
                params,
                locals: ctx.variables.dump_to_vec(),
                nodes,
                stack_size,
            });
        });
    }
}
