use super::function_parser::FunctionParser;
use super::global_holder::GlobalHolder;
use super::token_cursor::TokenCursor;
use super::types::*;
use crate::tokenizer::Token;
use std::string::String;
use std::vec::Vec;

fn program<'a>(tokens: &'a Vec<Token>) -> Program<'a> {
    let mut globals: GlobalHolder<'a> = GlobalHolder::new();
    let mut functions: Vec<Function> = Vec::new();
    let mut cursor = TokenCursor::new(&tokens);

    while cursor.remains() {
        let ty = cursor.read_base_type();
        let ident = cursor.expect_ident();
        // function
        if let Some(f) = FunctionParser::parse(&ident, &mut cursor, &mut globals) {
            functions.push(f);
            continue;
        }
        // global-var
        let ty = cursor.read_type_suffix(ty);
        cursor.expect_sign(";");
        globals.push(Variable {
            name: String::from(ident),
            offset: 0,
            ty,
            is_local: false,
            content: None,
        });
    }

    return Program {
        functions,
        globals: globals.dump_to_vec(),
    };
}

pub fn parse<'a>(tokens: &'a Vec<Token>) -> Program<'a> {
    return program(tokens);
}
