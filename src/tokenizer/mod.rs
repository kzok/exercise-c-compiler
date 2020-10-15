mod token;

use std::vec::Vec;
pub use token::{Token, TokenKind};

const SIGNES: &'static [&str] = &[
    "==", "!=", "<=", ">=", "<", ">", "(", ")", "+", "-", "*", "/",
];

struct TokenizerContext<'a> {
    input: &'a str,
    index: usize,
    tokens: Vec<Token<'a>>,
}

impl<'a> TokenizerContext<'a> {
    pub fn new(input: &'a str) -> TokenizerContext {
        TokenizerContext {
            input,
            index: 0,
            tokens: Vec::new(),
        }
    }

    pub fn rest_input(&self) -> &'a str {
        return &self.input[self.index..];
    }

    pub fn remains(&self) -> bool {
        return self.input.len() > self.index;
    }

    pub fn seek(&mut self, steps: usize) {
        self.index += steps;
    }

    /// returns true if any whitespace skipped.
    pub fn skip_whitespace(&mut self) -> bool {
        let rest_input = self.rest_input();

        let mut i: usize = 0;
        while let Some(_) = rest_input.chars().nth(i).filter(|c| c.is_whitespace()) {
            i += 1;
        }
        self.seek(i);
        return i > 0;
    }

    pub fn consume(&mut self, s: &'a str) -> bool {
        let rest_input = self.rest_input();

        if rest_input.starts_with(s) {
            self.tokens.push(Token {
                kind: TokenKind::Reserved(s),
                line_of_code: self.input,
                index: self.index,
            });
            self.seek(s.len());
            return true;
        }

        return false;
    }

    pub fn consume_number(&mut self) -> bool {
        let rest_input = self.rest_input();
        let mut num: Option<u32> = None;
        let mut i: usize = 0;
        while let Some(n) = rest_input.chars().nth(i).and_then(|c| c.to_digit(10)) {
            num = num.map(|num| num * 10 + n).or_else(|| Some(n));
            i += 1;
        }

        match num {
            Some(n) => {
                self.tokens.push(Token {
                    kind: TokenKind::Number(n),
                    line_of_code: &self.input,
                    index: self.index,
                });
                self.seek(i);
                return true;
            }
            None => {
                return false;
            }
        }
    }

    pub fn report_error(&self, msg: &str) -> ! {
        let loc = self.input;
        let i = self.index + 1;
        panic!("\n{0}\n{1:>2$} {3}\n", loc, '^', i, msg);
    }
}

fn consume_as_reserved(ctx: &mut TokenizerContext) -> bool {
    for sign in SIGNES {
        if ctx.consume(sign) {
            return true;
        }
    }
    return false;
}

pub fn tokenize<'a>(input: &'a str) -> Vec<Token<'a>> {
    let mut ctx = TokenizerContext::new(&input);

    while ctx.remains() {
        if ctx.skip_whitespace() {
            continue;
        }
        if ctx.consume_number() {
            continue;
        }
        if consume_as_reserved(&mut ctx) {
            continue;
        }
        ctx.report_error("トークナイズ出来ません。");
    }

    return ctx.tokens;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remains() {
        assert_eq!(TokenizerContext::new("").remains(), false);
        assert_eq!(TokenizerContext::new("123").remains(), true);
        assert_eq!(TokenizerContext::new(" ").remains(), true);
    }

    #[test]
    fn test_skip_whitespace() {
        let mut ctx = TokenizerContext::new("   123");
        assert_eq!(ctx.skip_whitespace(), true);
        assert_eq!(ctx.rest_input(), "123");

        let mut ctx = TokenizerContext::new("   ");
        assert_eq!(ctx.skip_whitespace(), true);
        assert_eq!(ctx.rest_input(), "");

        let mut ctx = TokenizerContext::new("123   ");
        assert_eq!(ctx.skip_whitespace(), false);
        assert_eq!(ctx.rest_input(), "123   ");
    }

    #[test]
    fn test_consume() {
        let mut ctx = TokenizerContext::new("123");
        assert_eq!(ctx.consume("+-+"), false);
        assert_eq!(ctx.rest_input(), "123");

        let mut ctx = TokenizerContext::new("+-123");
        assert_eq!(ctx.consume("+-+"), false);
        assert_eq!(ctx.rest_input(), "+-123");

        let mut ctx = TokenizerContext::new("+-+123");
        assert_eq!(ctx.consume("+-+"), true);
        assert_eq!(ctx.rest_input(), "123");
    }

    #[test]
    fn test_consume_number() {
        let mut ctx = TokenizerContext::new("");
        assert_eq!(ctx.consume_number(), false);
        assert_eq!(ctx.rest_input(), "");

        let mut ctx = TokenizerContext::new("123");
        assert_eq!(ctx.consume_number(), true);
        assert_eq!(
            ctx.tokens.iter().next().unwrap().kind,
            TokenKind::Number(123)
        );
        assert_eq!(ctx.rest_input(), "");

        let mut ctx = TokenizerContext::new("12+3");
        assert_eq!(ctx.consume_number(), true);
        assert_eq!(
            ctx.tokens.iter().next().unwrap().kind,
            TokenKind::Number(12)
        );
        assert_eq!(ctx.rest_input(), "+3");

        let mut ctx = TokenizerContext::new("nan");
        assert_eq!(ctx.consume_number(), false);
        assert_eq!(ctx.rest_input(), "nan");
    }
}
