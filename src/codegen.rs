use crate::parser::{BinaryOperator, Node, Program};

macro_rules! p {
  ($($arg:tt)*) => ({println!($($arg)*);})
}

macro_rules! emit {
  ($($arg:tt)*) => ({print!("\t");p!($($arg)*);})
}

const ARGREG: &'static [&str] = &["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

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

    fn gen_addr(&mut self, node: &Node) {
        match node {
            Node::LocalVar(local) => {
                emit!("mov rax, rbp");
                emit!("sub rax, {}", local.offset);
                emit!("push rax");
            }
            Node::Deref(node) => {
                self.gen(node);
            }
            _ => panic!("代入の左辺値が変数ではありません"),
        }
    }

    fn gen_binary_operator(&mut self, op: &BinaryOperator, lhs: &Node, rhs: &Node) {
        match op {
            BinaryOperator::Assign => {
                self.gen_addr(lhs);
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
                self.gen_addr(node);
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
                self.gen(cond);
                emit!("pop rax");
                emit!("cmp rax, 0");
                match els {
                    Some(els) => {
                        emit!("je  .L.else.{}", id);
                        self.gen(then);
                        emit!("jmp .L.end.{}", id);
                        p!(".L.else.{}:", id);
                        self.gen(els);
                    }
                    None => {
                        emit!("je  .L.end.{}", id);
                        self.gen(then);
                    }
                }
                p!(".L.end.{}:", id);
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
            Node::FunCall { name, args } => {
                for arg in args {
                    self.gen(arg);
                }
                for i in (0..args.len()).rev() {
                    emit!("pop {}", ARGREG[i]);
                }

                // NOTE: 関数呼び出しをする前にRSPが 16 の倍数でなければならないため
                let id = self.generate_id();
                emit!("mov rax, rsp");
                emit!("and rax, 15");
                emit!("jnz .L.call.{}", id);
                emit!("mov rax, 0");
                emit!("call {}", name);
                emit!("jmp .L.end.{}", id);
                p!(".L.call.{}:", id);
                emit!("sub rsp, 8");
                emit!("mov rax, 0");
                emit!("call {}", name);
                emit!("add rsp, 8");
                p!(".L.end.{}:", id);
                emit!("push rax");
            }
            Node::Addr(node) => {
                self.gen_addr(node);
            }
            Node::Deref(node) => {
                self.gen(node);
                emit!("pop rax");
                emit!("mov rax, [rax]");
                emit!("push rax");
            }
        }
    }
}

pub fn codegen(program: &Program) {
    let mut ctx = CodegenContext::new();

    p!(".intel_syntax noprefix");
    for function in &program.functions {
        p!(".global {}", function.name);
        p!("{}:", function.name);

        // 変数分の領域を確保する
        emit!("push rbp");
        emit!("mov rbp, rsp");
        emit!("sub rsp, {}", function.stack_size);

        // 引数をスタックに移動
        for i in 0..function.params.len() {
            emit!("mov [rbp-{}], {}", function.params[i].offset, ARGREG[i]);
        }

        for node in &function.nodes {
            ctx.gen(node);
        }

        // 式の評価結果としてスタックに一つの値が残っている
        // はずなので、スタックが溢れないようにポップしておく
        emit!("pop rax");

        // 最後の式の結果がRAXに残っているのでそれが返り値になる
        emit!("mov rsp, rbp");
        emit!("pop rbp");
        emit!("ret");
    }
}
