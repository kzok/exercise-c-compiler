mod tokenizer;

use std::vec::Vec;
use tokenizer::{tokenize, Token, TokenKind};

fn report_parser_error(token: &Token, msg: &str) -> ! {
    let loc = token.line_of_code;
    let i = token.index + 1;
    panic!("\n{0}\n{1:>2$} {3}\n", loc, '^', i, msg);
}

fn consume(token: &Token, c: char) -> bool {
    match token.kind {
        TokenKind::Reserved(t) => return t == c.to_string(),
        _ => return false,
    }
}

fn expect(token: &Token, c: char) {
    if consume(&token, c) {
        return;
    }
    report_parser_error(&token, &format!("'{}' ではありません", c));
}

fn expect_number(token: &Token) -> u32 {
    match token.kind {
        TokenKind::Number(n) => return n,
        _ => {}
    }
    report_parser_error(&token, "数ではありません");
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
