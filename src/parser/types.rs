use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Pointer(Box<Type>),
}

#[derive(Debug, PartialEq)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub offset: u32,
    pub ty: Type,
}

#[derive(Debug, PartialEq)]
pub enum NodeKind<'a> {
    // 何もしないノード
    Null,
    // 整数
    Number(u32),
    // "+"
    Add {
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
    // "-"
    Sub {
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
    // "*"
    Mul {
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
    // "/"
    Div {
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
    // "=="
    Equal {
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
    // "!="
    NotEqual {
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
    // "<"
    LessThan {
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
    // "<="
    LessThanEqual {
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
    // "="
    Assign {
        lhs: Box<Node<'a>>,
        rhs: Box<Node<'a>>,
    },
    // local variable
    LocalVar(Rc<Variable<'a>>),
    // "return"
    Return(Box<Node<'a>>),
    // "if"
    If {
        cond: Box<Node<'a>>,
        then: Box<Node<'a>>,
        els: Option<Box<Node<'a>>>,
    },
    // "while"
    While {
        cond: Box<Node<'a>>,
        then: Box<Node<'a>>,
    },
    // "for"
    For {
        init: Option<Box<Node<'a>>>,
        cond: Option<Box<Node<'a>>>,
        inc: Option<Box<Node<'a>>>,
        then: Box<Node<'a>>,
    },
    // "{" "}"
    Block(Vec<Box<Node<'a>>>),
    // "func()"
    FunCall {
        name: &'a str,
        args: Vec<Box<Node<'a>>>,
    },
    // "&val"
    Addr(Box<Node<'a>>),
    // "*ptr"
    Deref(Box<Node<'a>>),
}

#[derive(Debug, PartialEq)]
pub struct Node<'a> {
    pub kind: NodeKind<'a>,
    pub ty: Option<Type>,
}

#[derive(Debug, PartialEq)]
pub struct Function<'a> {
    pub name: &'a str,
    pub stack_size: u32,
    pub params: Vec<Rc<Variable<'a>>>,
    pub locals: Vec<Rc<Variable<'a>>>,
    pub nodes: Vec<Node<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Program<'a> {
    pub functions: Vec<Function<'a>>,
}
