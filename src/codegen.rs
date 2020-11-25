use crate::parser::{Node, NodeKind, Program};

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

    fn generate_label_id(&mut self) -> u32 {
        let id = self.label_id;
        self.label_id += 1;
        return id;
    }

    fn gen_addr(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::LocalVar(local) => {
                emit!("mov rax, rbp");
                emit!("sub rax, {}", local.offset);
                emit!("push rax");
            }
            NodeKind::Deref(node) => {
                self.gen(&node);
            }
            _ => panic!("代入の左辺値が変数ではありません"),
        }
    }

    pub fn gen_binary_ops(&mut self, lhs: &Box<Node>, rhs: &Box<Node>, f: fn()) {
        self.gen(lhs);
        self.gen(rhs);
        emit!("pop rdi");
        emit!("pop rax");
        f();
        emit!("push rax");
    }

    pub fn gen(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::Null => {}
            NodeKind::Number(n) => {
                emit!("push {}", n);
            }
            NodeKind::Add { lhs, rhs } => self.gen_binary_ops(lhs, rhs, || emit!("add rax, rdi")),
            NodeKind::Sub { lhs, rhs } => {
                self.gen_binary_ops(lhs, rhs, || emit!("sub rax, rdi"));
            }
            NodeKind::Mul { lhs, rhs } => {
                self.gen_binary_ops(lhs, rhs, || emit!("imul rax, rdi"));
            }
            NodeKind::Div { lhs, rhs } => self.gen_binary_ops(lhs, rhs, || {
                emit!("cqo");
                emit!("idiv rdi");
            }),
            NodeKind::Equal { lhs, rhs } => self.gen_binary_ops(lhs, rhs, || {
                emit!("cmp rax, rdi");
                emit!("sete al");
                emit!("movzb rax, al");
            }),
            NodeKind::NotEqual { lhs, rhs } => self.gen_binary_ops(lhs, rhs, || {
                emit!("cmp rax, rdi");
                emit!("setne al");
                emit!("movzb rax, al");
            }),
            NodeKind::LessThan { lhs, rhs } => self.gen_binary_ops(lhs, rhs, || {
                emit!("cmp rax, rdi");
                emit!("setl al");
                emit!("movzb rax, al");
            }),
            NodeKind::LessThanEqual { lhs, rhs } => self.gen_binary_ops(lhs, rhs, || {
                emit!("cmp rax, rdi");
                emit!("setle al");
                emit!("movzb rax, al");
            }),
            NodeKind::Assign { lhs, rhs } => {
                self.gen_addr(lhs);
                self.gen(rhs);
                emit!("pop rdi");
                emit!("pop rax");
                emit!("mov [rax], rdi");
                emit!("push rdi");
                return;
            }
            NodeKind::LocalVar(_) => {
                self.gen_addr(node);
                emit!("pop rax");
                emit!("mov rax, [rax]");
                emit!("push rax");
            }
            NodeKind::Return(node) => {
                self.gen(&node);
                emit!("pop rax");
                emit!("mov rsp, rbp");
                emit!("pop rbp");
                emit!("ret");
            }
            NodeKind::If { cond, then, els } => {
                let label_id = self.generate_label_id();
                self.gen(&cond);
                emit!("pop rax");
                emit!("cmp rax, 0");
                match els {
                    Some(els) => {
                        emit!("je  .L.else.{}", label_id);
                        self.gen(&then);
                        emit!("jmp .L.end.{}", label_id);
                        p!(".L.else.{}:", label_id);
                        self.gen(&els);
                    }
                    None => {
                        emit!("je  .L.end.{}", label_id);
                        self.gen(&then);
                    }
                }
                p!(".L.end.{}:", label_id);
            }
            NodeKind::While { cond, then } => {
                let label_id = self.generate_label_id();
                p!(".L.begin.{}:", label_id);
                self.gen(&cond);
                emit!("pop rax");
                emit!("cmp rax, 0");
                emit!("je  .L.end.{}", label_id);
                self.gen(&then);
                emit!("jmp .L.begin.{}", label_id);
                p!(".L.end.{}:", label_id);
            }
            NodeKind::For {
                init,
                cond,
                inc,
                then,
            } => {
                let label_id = self.generate_label_id();
                if let Some(init) = init {
                    self.gen(&init);
                }
                p!(".L.begin.{}:", label_id);
                if let Some(cond) = cond {
                    self.gen(&cond);
                    emit!("pop rax");
                    emit!("cmp rax, 0");
                    emit!("je  .L.end.{}", label_id);
                }
                self.gen(&then);
                if let Some(inc) = inc {
                    self.gen(&inc);
                }
                emit!("jmp .L.begin.{}", label_id);
                p!(".L.end.{}:", label_id);
            }
            NodeKind::Block(nodes) => {
                for node in nodes {
                    self.gen(&node);
                }
            }
            NodeKind::FunCall { name, args } => {
                for arg in args {
                    self.gen(&arg);
                }
                for i in (0..args.len()).rev() {
                    emit!("pop {}", ARGREG[i]);
                }

                // NOTE: 関数呼び出しをする前にRSPが 16 の倍数でなければならないため
                let label_id = self.generate_label_id();
                emit!("mov rax, rsp");
                emit!("and rax, 15");
                emit!("jnz .L.call.{}", label_id);
                emit!("mov rax, 0");
                emit!("call {}", name);
                emit!("jmp .L.end.{}", label_id);
                p!(".L.call.{}:", label_id);
                emit!("sub rsp, 8");
                emit!("mov rax, 0");
                emit!("call {}", name);
                emit!("add rsp, 8");
                p!(".L.end.{}:", label_id);
                emit!("push rax");
            }
            NodeKind::Addr(node) => {
                self.gen_addr(&node);
            }
            NodeKind::Deref(node) => {
                self.gen(&node);
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
