use crate::parser::Node;

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
        Node::Add { lhs, rhs } => {
            gen(lhs);
            gen(rhs);
            emit!("pop rdi");
            emit!("pop rax");
            emit!("add rax, rdi");
            emit!("push rax");
        }
        Node::Sub { lhs, rhs } => {
            gen(lhs);
            gen(rhs);
            emit!("pop rdi");
            emit!("pop rax");
            emit!("sub rax, rdi");
            emit!("push rax");
        }
        Node::Mul { lhs, rhs } => {
            gen(lhs);
            gen(rhs);
            emit!("pop rdi");
            emit!("pop rax");
            emit!("imul rax, rdi");
            emit!("push rax");
        }
        Node::Div { lhs, rhs } => {
            gen(lhs);
            gen(rhs);
            emit!("pop rdi");
            emit!("pop rax");
            emit!("cqo");
            emit!("idiv rdi");
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
