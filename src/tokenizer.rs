use std::vec::Vec;

#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {
    Reserved(&'a str),
    Number(u32),
}

pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line_of_code: &'a str,
    pub index: usize,
}

struct TokenizerContext<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> TokenizerContext<'a> {
    pub fn new(input: &'a str) -> TokenizerContext {
        TokenizerContext { input, index: 0 }
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

    pub fn consume(&mut self, s: &'a str) -> Option<Token<'a>> {
        let rest_input = self.rest_input();

        if rest_input.starts_with(s) {
            let result = Some(Token {
                kind: TokenKind::Reserved(s),
                line_of_code: self.input,
                index: self.index,
            });
            self.seek(s.len());
            return result;
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

        let result = num.map(|n| Token {
            kind: TokenKind::Number(n),
            line_of_code: &self.input,
            index: self.index,
        });
        self.seek(i);
        return result;
    }
}

fn report_error(ctx: &TokenizerContext, msg: &str) -> ! {
    let loc = ctx.input;
    let i = ctx.index + 1;
    panic!("\n{0}\n{1:>2$} {3}\n", loc, '^', i, msg);
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

        if let Some(token) = ctx.consume("+") {
            tokens.push(token);
            continue;
        }

        if let Some(token) = ctx.consume("-") {
            tokens.push(token);
            continue;
        }

        report_error(&ctx, "トークナイズ出来ません。");
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
    fn test_consume_number() {
        let mut ctx = TokenizerContext::new("");
        assert_eq!(ctx.consume_number().is_none(), true);

        let mut ctx = TokenizerContext::new("123");
        assert_eq!(
            ctx.consume_number().map(|t| t.kind).unwrap(),
            TokenKind::Number(123)
        );

        let mut ctx = TokenizerContext::new("12+3");
        assert_eq!(
            ctx.consume_number().map(|t| t.kind).unwrap(),
            TokenKind::Number(12)
        );

        let mut ctx = TokenizerContext::new("nan");
        assert_eq!(ctx.consume_number().is_none(), true);
    }
}
