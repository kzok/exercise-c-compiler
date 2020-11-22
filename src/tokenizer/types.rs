#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Keyword {
    Return,
    If,
    Else,
    While,
    For,
    Int,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {
    Number(u32),
    Sign(&'a str),
    Ident(&'a str),
    Keyword(Keyword),
    Eof,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line_of_code: &'a str,
    pub index: usize,
}

impl<'a> Token<'a> {
    pub fn report_error(&self, msg: &str) -> ! {
        let loc = self.line_of_code;
        let i = self.index + 1;
        panic!("\n{0}\n{1:>2$} {3}\n", loc, '^', i, msg);
    }
}
