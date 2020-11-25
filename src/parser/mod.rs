mod function_parser;
mod token_cursor;
mod types;

use self::function_parser::FunctionParser;
use crate::tokenizer::Token;

use std::vec::Vec;
use token_cursor::TokenCursor;
pub use types::{Function, Node, NodeKind, Program, Variable};

pub fn parse<'a>(tokens: &'a Vec<Token>) -> Program<'a> {
    let mut functions: Vec<Function> = Vec::new();
    let mut cursor = TokenCursor::new(&tokens);

    while cursor.remains() {
        if let Some(function) = FunctionParser::parse(&mut cursor) {
            functions.push(function);
            continue;
        }
        cursor.report_error("識別子ではありません");
    }

    return Program { functions };
}
