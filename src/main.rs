mod tokenizer;

use std::vec::{Vec};
use tokenizer::{TokenKind, Token, tokenize};

fn consume(token: &Token, c: char) -> bool {
    token.kind == TokenKind::Reserved && token.substr == c.to_string()
}

fn expect(token: &Token, c: char) {
    if consume(&token, c) {
        return;
    }
    panic!("'{}' ではありません", c);
}

fn expect_number(token: &Token) -> u32 {
    match token.kind {
        TokenKind::Number(n) => return n,
        _ => {}
    }
    panic!("数ではありません");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("引数の個数が正しくありません");
    }

    let input: &String = &args[1];
    let tokens = tokenize(&input);
    let mut iter = tokens.iter();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // 式の最初は数でなければならないので、それをチェックして
    // 最初のmov命令を出力
    println!("\tmov rax, {}", expect_number(&iter.next().unwrap()));
    while let Some(token) = iter.next() {
        if consume(token, '+') {
            println!("\tadd rax, {}", expect_number(&iter.next().unwrap()));
            continue;
        }

        expect(token, '-');
        println!("\tsub rax, {}", expect_number(&iter.next().unwrap()));
    }

    println!("\tret");
    std::process::exit(0);
}
