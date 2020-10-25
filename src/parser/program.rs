use std::rc::Rc;

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
pub struct Variable<'a> {
    pub name: &'a str,
    pub offset: u32,
}

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    // 整数
    Number(u32),
    // ローカル変数
    LocalVar(Rc<Variable<'a>>),
    // 二項演算子
    Binary {
        op: BinaryOperator,
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Program<'a> {
    pub stack_size: u32,
    pub locals: Vec<Rc<Variable<'a>>>,
    pub nodes: Vec<Node<'a>>,
}
