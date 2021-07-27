use super::global_holder::GlobalHolder;
use super::token_cursor::TokenCursor;
use super::types::*;
use crate::tokenizer::Keyword;
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

fn detect_type(kind: &NodeKind) -> Option<Type> {
    match kind {
        NodeKind::Mul { .. }
        | NodeKind::Div { .. }
        | NodeKind::Equal { .. }
        | NodeKind::NotEqual { .. }
        | NodeKind::LessThan { .. }
        | NodeKind::LessThanEqual { .. }
        | NodeKind::FunCall { .. }
        | NodeKind::Number(_) => Some(Type::Int),
        NodeKind::Add { lhs, rhs } => match rhs.ty {
            Some(Type::Pointer(_)) => panic!("ポインタを加算の右辺値に指定できません"),
            Some(Type::Array(..)) => panic!("配列を加算の右辺値に指定できません"),
            _ => lhs.ty.clone(),
        },
        NodeKind::Sub { lhs, rhs } => match rhs.ty {
            Some(Type::Pointer(_)) => panic!("ポインタを減算の右辺値に指定できません"),
            Some(Type::Array(..)) => panic!("配列を減算の右辺値に指定できません"),
            _ => lhs.ty.clone(),
        },
        NodeKind::Variable(var) => Some((*var).ty.clone()),
        NodeKind::Assign { lhs, .. } => lhs.ty.clone(),
        NodeKind::Addr(target) => match &target.ty {
            Some(Type::Array(base, ..)) => Some(Type::Pointer(Box::new(*base.clone()))),
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
            Some(Type::Pointer(_)) | Some(Type::Array(..)) => std::mem::swap(lhs, rhs),
            _ => {}
        },
        _ => {}
    }
    let ty = detect_type(&kind);
    return Node { kind, ty };
}

fn align_to(n: u32, align: u32) -> u32 {
    return (n + align - 1) & !(align - 1);
}

struct LocalHolder<'a> {
    locals: Vec<Rc<Variable<'a>>>,
}
impl<'a> LocalHolder<'a> {
    pub fn new() -> LocalHolder<'a> {
        return LocalHolder { locals: Vec::new() };
    }

    fn total_variable_size(&self) -> u32 {
        return self.locals.iter().fold(0, |acc, v| acc + v.ty.size());
    }

    pub fn stack_size(&self) -> u32 {
        return align_to(self.total_variable_size(), 8);
    }

    pub fn new_var(&mut self, name: &'a str, ty: Type) -> Rc<Variable<'a>> {
        let var = Rc::new(Variable {
            name: String::from(name),
            offset: self.total_variable_size() + ty.size(),
            ty,
            is_local: true,
            content: None,
        });
        self.locals.push(var.clone());
        return var;
    }

    pub fn find(&self, name: &str) -> Option<Rc<Variable<'a>>> {
        for var in &self.locals {
            if var.name == name {
                return Some(var.clone());
            }
        }
        return None;
    }

    pub fn dump_to_vec(self) -> Vec<Rc<Variable<'a>>> {
        return self.locals;
    }
}

pub struct FunctionParser<'local, 'outer: 'local> {
    globals: &'local mut GlobalHolder<'outer>,
    locals: LocalHolder<'outer>,
    cursor: &'local mut TokenCursor<'outer>,
}
impl<'local, 'outer: 'local> FunctionParser<'local, 'outer> {
    fn new(
        cursor: &'local mut TokenCursor<'outer>,
        globals: &'local mut GlobalHolder<'outer>,
    ) -> FunctionParser<'local, 'outer> {
        return FunctionParser {
            locals: LocalHolder::new(),
            cursor,
            globals,
        };
    }

    fn find_var(&self, name: &str) -> Option<Rc<Variable<'outer>>> {
        return self.locals.find(name).or_else(|| {
            return self.globals.find_var(name);
        });
    }

    fn read_func_params(&mut self) -> Vec<Rc<Variable<'outer>>> {
        let mut params = Vec::new();
        if self.cursor.consume_sign(")") {
            return params;
        }
        let ty = self.cursor.read_base_type();
        let name = self.cursor.expect_ident();
        let ty = self.cursor.read_type_suffix(ty);
        let var = self.locals.new_var(name, ty);
        params.push(var);

        while !self.cursor.consume_sign(")") {
            self.cursor.expect_sign(",");
            let ty = self.cursor.read_base_type();
            let name = self.cursor.expect_ident();
            let ty = self.cursor.read_type_suffix(ty);
            let var = self.locals.new_var(name, ty);
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
        let ty = self.cursor.read_base_type();
        let name = self.cursor.expect_ident();
        let ty = self.cursor.read_type_suffix(ty);
        let var = self.locals.new_var(name, ty);
        if self.cursor.consume_sign(";") {
            return make_node(NodeKind::Null);
        }
        self.cursor.expect_sign("=");
        let lhs = Box::new(make_node(NodeKind::Variable(var)));
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
            if let Some(local) = self.find_var(name) {
                return make_node(NodeKind::Variable(local));
            }

            self.cursor
                .previous()
                .report_error(&format!("未定義の変数 \"{}\" を参照しました。", name));
        }
        // String literal
        if let Some(s) = self.cursor.consume_str() {
            let var = self.globals.string_literal(s);
            return make_node(NodeKind::Variable(var));
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
        if self.cursor.is_typename() {
            return self.declaretion();
        }

        let node = self.expr();
        self.cursor.expect_sign(";");
        return node;
    }

    pub fn parse(
        ident: &'outer str,
        cursor: &'local mut TokenCursor<'outer>,
        globals: &'local mut GlobalHolder<'outer>,
    ) -> Option<Function<'outer>> {
        if !cursor.consume_sign("(") {
            return None;
        }
        let mut ctx = FunctionParser::new(cursor, globals);
        let mut nodes = Vec::new();

        let params = ctx.read_func_params();
        ctx.cursor.expect_sign("{");
        while !ctx.cursor.consume_sign("}") {
            nodes.push(ctx.stmt());
        }

        let stack_size = ctx.locals.stack_size();
        return Some(Function {
            name: ident,
            params,
            locals: ctx.locals.dump_to_vec(),
            nodes,
            stack_size,
        });
    }
}
