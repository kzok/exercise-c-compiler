fn consume_u32(iter: &mut std::iter::Peekable<std::str::Chars>) -> Option<u32> {
    let mut num: Option<u32> = None;
    while let Some(n) = iter.peek().and_then(|c| c.to_digit(10)) {
        num = num.map(|num| num * 10 + n).or_else(|| Some(n));
        iter.next();
    }
    return num;
}

fn main()  {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        std::process::exit(1);
    }

    let input: &String = &args[1];
    let mut iter = input.chars().peekable();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
    println!("\tmov rax, {}", consume_u32(&mut iter).unwrap());
    while let Some(c) = iter.next() {
        if c == '+' {
            println!("\tadd rax, {}", consume_u32(&mut iter).unwrap());
            continue;
        }
        if c == '-' {
            println!("\tsub rax, {}", consume_u32(&mut iter).unwrap());
            continue;
        }

        eprintln!("予期していない文字です: {}", c);
        std::process::exit(1);
    }

    println!("\tret");
    std::process::exit(0);
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
