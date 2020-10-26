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

fn gen(node: &Node) {
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
            gen(node);
            emit!("pop rax");
            emit!("mov rsp, rbp");
            emit!("pop rbp");
            emit!("ret");
        }
        Node::Binary { op, lhs, rhs } => {
            match op {
                BinaryOperator::Assign => {
                    gen_lvar(lhs);
                    gen(rhs);
                    emit!("pop rdi");
                    emit!("pop rax");
                    emit!("mov [rax], rdi");
                    emit!("push rdi");
                    return;
                }
                _ => (),
            }

            gen(lhs);
            gen(rhs);
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
    }
}

pub fn codegen(program: &Program) {
    p!(".intel_syntax noprefix");
    p!(".global main");
    p!("main:");

    // 変数分の領域を確保する
    emit!("push rbp");
    emit!("mov rbp, rsp");
    emit!("sub rsp, {}", program.stack_size);

    for node in &program.nodes {
        gen(node);
        // 式の評価結果としてスタックに一つの値が残っている
        // はずなので、スタックが溢れないようにポップしておく
        emit!("pop rax");
    }

    // 最後の式の結果がRAXに残っているのでそれが返り値になる
    emit!("mov rsp, rbp");
    emit!("pop rbp");
    emit!("ret");
}
