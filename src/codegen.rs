use crate::parser::{BinaryOperator, Node, Program};

macro_rules! p {
  ($($arg:tt)*) => ({println!($($arg)*);})
}

macro_rules! emit {
  ($($arg:tt)*) => ({print!("\t");p!($($arg)*);})
}

fn gen_lvar(node: &Node) {
    match node {
        Node::LocalVar(local) => {
            emit!("mov rax, rbp");
            emit!("sub rax, {}", local.offset);
            emit!("push rax");
        }
        _ => panic!("代入の左辺値が変数ではありません"),
    }
}

struct CodegenContext {
    label_id: u32,
}

impl CodegenContext {
    fn new() -> CodegenContext {
        return CodegenContext { label_id: 0 };
    }

    fn generate_id(&mut self) -> u32 {
        let id = self.label_id;
        self.label_id += 1;
        return id;
    }

    fn gen_binary_operator(&mut self, op: &BinaryOperator, lhs: &Node, rhs: &Node) {
        match op {
            BinaryOperator::Assign => {
                gen_lvar(lhs);
                self.gen(rhs);
                emit!("pop rdi");
                emit!("pop rax");
                emit!("mov [rax], rdi");
                emit!("push rdi");
                return;
            }
            _ => (),
        }

        self.gen(lhs);
        self.gen(rhs);
        emit!("pop rdi");
        emit!("pop rax");
        match op {
            BinaryOperator::Add => emit!("add rax, rdi"),
            BinaryOperator::Sub => emit!("sub rax, rdi"),
            BinaryOperator::Mul => emit!("imul rax, rdi"),
            BinaryOperator::Div => {
                emit!("cqo");
                emit!("idiv rdi");
            }
            BinaryOperator::Equal => {
                emit!("cmp rax, rdi");
                emit!("sete al");
                emit!("movzb rax, al");
            }
            BinaryOperator::NotEqual => {
                emit!("cmp rax, rdi");
                emit!("setne al");
                emit!("movzb rax, al");
            }
            BinaryOperator::LessThan => {
                emit!("cmp rax, rdi");
                emit!("setl al");
                emit!("movzb rax, al");
            }
            BinaryOperator::LessThanEqual => {
                emit!("cmp rax, rdi");
                emit!("setle al");
                emit!("movzb rax, al");
            }
            _ => panic!("\"{:?}\" must not be here.", op),
        }
        emit!("push rax");
    }

    pub fn gen(&mut self, node: &Node) {
        match node {
            Node::Number(n) => {
                emit!("push {}", n);
            }
            Node::LocalVar(_) => {
                gen_lvar(node);
                emit!("pop rax");
                emit!("mov rax, [rax]");
                emit!("push rax");
            }
            Node::Return(node) => {
                self.gen(node);
                emit!("pop rax");
                emit!("mov rsp, rbp");
                emit!("pop rbp");
                emit!("ret");
            }
            Node::Binary { op, lhs, rhs } => {
                self.gen_binary_operator(op, lhs, rhs);
            }
            Node::If { cond, then, els } => {
                let id = self.generate_id();
                match els {
                    Some(els) => {
                        self.gen(cond);
                        emit!("pop rax");
                        emit!("cmp rax, 0");
                        emit!("je  .L.else.{}", id);
                        self.gen(then);
                        emit!("jmp .L.end.{}", id);
                        p!(".L.else.{}:", id);
                        self.gen(els);
                        p!(".L.end.{}:", id);
                    }
                    None => {
                        self.gen(cond);
                        emit!("pop rax");
                        emit!("cmp rax, 0");
                        emit!("je  .L.end.{}", id);
                        self.gen(then);
                        p!(".L.end.{}:", id);
                    }
                }
            }
            Node::While { cond, then } => {
                let id = self.generate_id();
                p!(".L.begin.{}:", id);
                self.gen(cond);
                emit!("pop rax");
                emit!("cmp rax, 0");
                emit!("je  .L.end.{}", id);
                self.gen(then);
                emit!("jmp .L.begin.{}", id);
                p!(".L.end.{}:", id);
            }
            Node::For {
                init,
                cond,
                inc,
                then,
            } => {
                let id = self.generate_id();
                if let Some(init) = init {
                    self.gen(init);
                }
                p!(".L.begin.{}:", id);
                if let Some(cond) = cond {
                    self.gen(cond);
                    emit!("pop rax");
                    emit!("cmp rax, 0");
                    emit!("je  .L.end.{}", id);
                }
                self.gen(then);
                if let Some(inc) = inc {
                    self.gen(inc);
                }
                emit!("jmp .L.begin.{}", id);
                p!(".L.end.{}:", id);
            }
            Node::Block(nodes) => {
                for node in nodes {
                    self.gen(node);
                }
            }
            Node::FunCall { name } => {
                emit!("call {}", name);
                emit!("push rax");
            }
        }
    }
}

pub fn codegen(program: &Program) {
    let mut ctx = CodegenContext::new();

    p!(".intel_syntax noprefix");
    p!(".global main");
    p!("main:");

    // 変数分の領域を確保する
    emit!("push rbp");
    emit!("mov rbp, rsp");
    emit!("sub rsp, {}", program.stack_size);

    for node in &program.nodes {
        ctx.gen(node);
        // 式の評価結果としてスタックに一つの値が残っている
        // はずなので、スタックが溢れないようにポップしておく
        emit!("pop rax");
    }

    // 最後の式の結果がRAXに残っているのでそれが返り値になる
    emit!("mov rsp, rbp");
    emit!("pop rbp");
    emit!("ret");
}
