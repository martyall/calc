use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Number(i32),
    BinOp(Box<Expr>, Opcode, Box<Expr>),
}
