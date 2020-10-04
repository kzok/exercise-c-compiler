use std::string::String;
use std::vec::Vec;

#[derive(PartialEq)]
pub enum TokenKind {
    Reserved,
    Number(u32),
}

pub struct Token {
    pub kind: TokenKind,
    pub substr: String,
}

fn consume_u32(iter: &mut std::iter::Peekable<std::str::Chars>) -> Option<u32> {
  let mut num: Option<u32> = None;
  while let Some(n) = iter.peek().and_then(|c| c.to_digit(10)) {
      num = num.map(|num| num * 10 + n).or_else(|| Some(n));
      iter.next();
  }
  return num;
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(c) = iter.peek().cloned() {
        if let Some(n) = consume_u32(&mut iter) {
            tokens.push(Token {
                kind: TokenKind::Number(n),
                substr: c.to_string(),
            });
            continue;
        }

        match c {
            _ if c.is_whitespace() => {
                iter.next();
                continue;
            },
            '+' | '-' => {
                tokens.push(Token {
                    kind: TokenKind::Reserved,
                    substr: c.to_string(),
                });
                iter.next();
                continue;
            },
            _ => {
                panic!("トークナイズ出来ません。");
            },
        }
    }

    return tokens;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_u32() {
        assert_eq!(consume_u32(&mut "".chars().peekable()), None);
        assert_eq!(consume_u32(&mut "123".chars().peekable()), Some(123));
        assert_eq!(consume_u32(&mut "12+3".chars().peekable()), Some(12));
        assert_eq!(consume_u32(&mut "nan".chars().peekable()), None);
    }
}
