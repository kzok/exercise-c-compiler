use crate::parser::{BinaryOperator, Node};

macro_rules! p {
  ($($arg:tt)*) => ({println!($($arg)*);})
}

macro_rules! emit {
  ($($arg:tt)*) => ({print!("\t");p!($($arg)*);})
}

fn gen(node: &Node) {
    match node {
        Node::Number(n) => {
            emit!("push {}", n);
        }
        Node::Binary { op, lhs, rhs } => {
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
            }
            emit!("push rax");
        }
    }
}

pub fn codegen(node: &Node) {
    p!(".intel_syntax noprefix");
    p!(".global main");
    p!("main:");

    gen(node);

    emit!("pop rax");
    emit!("ret");
}
