use std::vec::Vec;

pub enum TokenKind<'a> {
    Reserved(&'a str),
    Number(u32),
}

pub struct Token<'a> {
    pub kind: TokenKind<'a>,
}

#[derive(Debug)]
struct TokenizerContext<'a> {
    rest_input: &'a str,
}

impl<'a> TokenizerContext<'a> {
    pub const fn new(input: &'a str) -> TokenizerContext<'a> {
        TokenizerContext {
            rest_input: input,
        }
    }

    pub const fn remains(&self) -> bool {
        self.rest_input.len() != 0
    }

    /// returns true if any whitespace skipped.
    pub fn skip_whitespace(&mut self) -> bool {
        let original_length = self.rest_input.len();
        self.rest_input = self.rest_input.trim_start();
        return self.rest_input.len() < original_length;
    }

    pub fn consume(&mut self, s: &str) -> bool {
        if self.rest_input.starts_with(s) {
            self.rest_input = &self.rest_input[s.len()..];
            return true;
        }
        return false;
    }

    pub fn consume_u32(&mut self) -> Option<u32> {
        let mut num: Option<u32> = None;
        let mut i: usize = 0;
        while let Some(n) = self.rest_input.chars().nth(i).and_then(|c| c.to_digit(10)) {
            num = num.map(|num| num * 10 + n).or_else(|| Some(n));
            i += 1;
        }
        self.rest_input = &self.rest_input[i..];
        return num;
    }
}

pub fn tokenize<'a>(input: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();
    let mut ctx = TokenizerContext::new(&input);

    while ctx.remains() {
        if ctx.skip_whitespace() {
            continue;
        }

        if let Some(n) = ctx.consume_u32() {
            tokens.push(Token {
                kind: TokenKind::Number(n),
            });
            continue;
        }

        if ctx.consume("+") {
            tokens.push(Token {
                kind: TokenKind::Reserved("+"),
            });
            continue;
        }

        if ctx.consume("-") {
            tokens.push(Token {
                kind: TokenKind::Reserved("-"),
            });
            continue;
        }

        panic!("トークナイズ出来ません。\n{:?}", ctx);
    }

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
        ctx.skip_whitespace();
        assert_eq!(ctx.rest_input, "123");

        let mut ctx = TokenizerContext::new("   ");
        ctx.skip_whitespace();
        assert_eq!(ctx.rest_input, "");
    }

    #[test]
    fn test_consume_u32() {
        assert_eq!(TokenizerContext::new("").consume_u32(), None);
        assert_eq!(TokenizerContext::new("123").consume_u32(), Some(123));
        assert_eq!(TokenizerContext::new("12+3").consume_u32(), Some(12));
        assert_eq!(TokenizerContext::new("12 3").consume_u32(), Some(12));
        assert_eq!(TokenizerContext::new("nan").consume_u32(), None);
    }
}
