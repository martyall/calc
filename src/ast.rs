#[derive(Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
}

#[derive(Debug, PartialEq)]
pub enum AST {
    Literal(u32),
    Negate(Box<AST>),
    BinOp(Op, Box<AST>, Box<AST>),
}
