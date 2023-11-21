use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Pow,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum UOpcode {
    Neg,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone, Display)]
pub struct Ident(String);

impl Ident {
    pub fn new(s: &str) -> Self {
        Ident(s.to_string())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Expr {
    Number(i32),
    Variable(Ident),
    UnaryOp(UOpcode, Box<Expr>),
    BinOp(Box<Expr>, Opcode, Box<Expr>),
}

impl Expr {
    // get all of the variables that appear in an expression
    pub fn variables(&self) -> Vec<Ident> {
        match self {
            Expr::Number(_) => vec![],
            Expr::UnaryOp(_, expr) => expr.variables(),
            Expr::BinOp(lhs, _, rhs) => {
                let mut deps = lhs.variables();
                deps.append(&mut rhs.variables());
                deps
            }
            Expr::Variable(name) => vec![name.clone()],
        }
    }
}
