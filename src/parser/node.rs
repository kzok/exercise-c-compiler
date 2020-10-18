#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    // +
    Add,
    // -
    Sub,
    // *
    Mul,
    // /
    Div,
    // ==
    Equal,
    // !=
    NotEqual,
    // <
    LessThan,
    // <=
    LessThanEqual,
    // =
    Assign,
}

#[derive(Debug, PartialEq)]
pub enum Node {
    // 整数
    Number(u32),
    // ローカル変数
    LocalVar {
        offset: u32,
    },
    // 二項演算子
    Binary {
        op: BinaryOperator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}
