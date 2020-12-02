mod types;

use std::vec::Vec;
pub use types::{Keyword, Token, TokenKind};

const SIGNES: &'static [&str] = &[
    "==", "!=", "<=", ">=", "<", ">", "(", ")", "+", "-", "*", "/", "&", "=", ";", "{", "}", ",",
    "[", "]",
];

fn is_alpha(c: &char) -> bool {
    return ('a' <= *c && *c <= 'z') || ('A' <= *c && *c <= 'Z') || (*c == '_');
}

fn is_alnum(c: &char) -> bool {
    return is_alpha(c) || c.is_digit(10);
}

struct TokenizerContext<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> TokenizerContext<'a> {
    fn new(input: &'a str) -> TokenizerContext {
        TokenizerContext { input, index: 0 }
    }

    fn rest_input(&self) -> &'a str {
        return &self.input[self.index..];
    }

    fn seek(&mut self, steps: usize) {
        self.index += steps;
    }

    pub fn remains(&self) -> bool {
        return self.input.len() > self.index;
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

    pub fn consume_sign(&mut self) -> Option<Token<'a>> {
        let rest_input = self.rest_input();

        for sign in SIGNES {
            if rest_input.starts_with(sign) {
                let token = Token {
                    kind: TokenKind::Sign(sign),
                    line_of_code: self.input,
                    index: self.index,
                };
                self.seek(sign.len());
                return Some(token);
            }
        }
        return None;
    }

    pub fn consume_keyword(&mut self) -> Option<Token<'a>> {
        let rest_input = self.rest_input();

        for (keyword, value) in Keyword::PAIRS {
            let trailing = rest_input.chars().nth(keyword.len());

            if rest_input.starts_with(keyword) && trailing.filter(is_alnum).is_none() {
                let token = Token {
                    kind: TokenKind::Keyword(*value),
                    line_of_code: self.input,
                    index: self.index,
                };
                self.seek(keyword.len());
                return Some(token);
            }
        }
        return None;
    }

    pub fn consume_number(&mut self) -> Option<Token<'a>> {
        let rest_input = self.rest_input();
        let mut num: Option<u32> = None;
        let mut i: usize = 0;
        while let Some(n) = rest_input.chars().nth(i).and_then(|c| c.to_digit(10)) {
            num = num.map(|num| num * 10 + n).or_else(|| Some(n));
            i += 1;
        }

        match num {
            Some(n) => {
                let token = Token {
                    kind: TokenKind::Number(n),
                    line_of_code: &self.input,
                    index: self.index,
                };
                self.seek(i);
                return Some(token);
            }
            None => return None,
        }
    }

    pub fn consume_ident(&mut self) -> Option<Token<'a>> {
        let rest_input = self.rest_input();

        let mut i: usize = 0;
        let mut iter = rest_input.chars();
        if iter.next().filter(is_alpha).is_some() {
            i += 1;
            while iter.next().filter(is_alnum).is_some() {
                i += 1;
            }
        }

        if i > 0 {
            let token = Token {
                kind: TokenKind::Ident(&rest_input[0..i]),
                line_of_code: &self.input,
                index: self.index,
            };
            self.seek(i);
            return Some(token);
        }

        return None;
    }

    pub fn report_error(&self, msg: &str) -> ! {
        let loc = self.input;
        let i = self.index + 1;
        panic!("\n{0}\n{1:>2$} {3}\n", loc, '^', i, msg);
    }
}

pub fn tokenize<'a>(input: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();
    let mut ctx = TokenizerContext::new(&input);

    while ctx.remains() {
        if ctx.skip_whitespace() {
            continue;
        }
        if let Some(token) = ctx.consume_number() {
            tokens.push(token);
            continue;
        }
        if let Some(token) = ctx.consume_sign() {
            tokens.push(token);
            continue;
        }
        if let Some(token) = ctx.consume_keyword() {
            tokens.push(token);
            continue;
        }
        if let Some(token) = ctx.consume_ident() {
            tokens.push(token);
            continue;
        }
        ctx.report_error("トークナイズ出来ません。");
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        line_of_code: input,
        index: input.len(),
    });
    return tokens;
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
    fn consume_sign() {
        let mut ctx = TokenizerContext::new("123");
        assert!(ctx.consume_sign().is_none());
        assert_eq!(ctx.rest_input(), "123");

        let mut ctx = TokenizerContext::new("+-+123");
        assert!(ctx.consume_sign().is_some());
        assert_eq!(ctx.rest_input(), "-+123");
    }

    #[test]
    fn test_consume_keyword() {
        let mut ctx = TokenizerContext::new("returna");
        assert!(ctx.consume_keyword().is_none());
        assert_eq!(ctx.rest_input(), "returna");

        let mut ctx = TokenizerContext::new("return");
        assert!(ctx.consume_keyword().is_some());
        assert_eq!(ctx.rest_input(), "");

        let mut ctx = TokenizerContext::new("return;");
        assert!(ctx.consume_keyword().is_some());
        assert_eq!(ctx.rest_input(), ";");
    }

    #[test]
    fn test_consume_number() {
        let mut ctx = TokenizerContext::new("");
        assert!(ctx.consume_number().is_none());
        assert_eq!(ctx.rest_input(), "");

        let mut ctx = TokenizerContext::new("123");
        assert_eq!(ctx.consume_number().unwrap().kind, TokenKind::Number(123));
        assert_eq!(ctx.rest_input(), "");

        let mut ctx = TokenizerContext::new("12+3");
        assert_eq!(ctx.consume_number().unwrap().kind, TokenKind::Number(12));
        assert_eq!(ctx.rest_input(), "+3");

        let mut ctx = TokenizerContext::new("nan");
        assert!(ctx.consume_number().is_none());
        assert_eq!(ctx.rest_input(), "nan");
    }

    #[test]
    fn test_consume_ident() {
        let mut ctx = TokenizerContext::new("1abc");
        assert!(ctx.consume_ident().is_none());
        assert_eq!(ctx.rest_input(), "1abc");

        let mut ctx = TokenizerContext::new("a1bc");
        assert_eq!(ctx.consume_ident().unwrap().kind, TokenKind::Ident("a1bc"));
        assert_eq!(ctx.rest_input(), "");

        let mut ctx = TokenizerContext::new("ab1c+2");
        assert_eq!(ctx.consume_ident().unwrap().kind, TokenKind::Ident("ab1c"));
        assert_eq!(ctx.rest_input(), "+2");
    }
}
