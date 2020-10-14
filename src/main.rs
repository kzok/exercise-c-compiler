mod codegen;
mod parser;
mod tokenizer;

use codegen::codegen;
use parser::parse;
use std::vec::Vec;
use tokenizer::tokenize;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("引数の個数が正しくありません");
    }

    let input: &String = &args[1];
    let tokens = tokenize(&input);
    let node = parse(&tokens);

    codegen(&node);
    std::process::exit(0);
}
