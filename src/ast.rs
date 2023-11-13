#[derive(Debug, PartialEq)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(i32),
    BinOp(Box<Expr>, Opcode, Box<Expr>),
}
