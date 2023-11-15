use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Pow,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum UOpcode {
    Neg,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Number(i32),
    Variable(String),
    UnaryOp(UOpcode, Box<Expr>),
    BinOp(Box<Expr>, Opcode, Box<Expr>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    VarAssignment(String, Expr),
}
