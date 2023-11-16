use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Pow,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum UOpcode {
    Neg,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Expr {
    Number(i32),
    Variable(String),
    UnaryOp(UOpcode, Box<Expr>),
    BinOp(Box<Expr>, Opcode, Box<Expr>),
}

impl Expr {
    pub fn inline(&self, context: &mut HashMap<String, Expr>) -> Self {
        match self {
            Expr::Number(n) => Expr::Number(n.clone()),
            Expr::UnaryOp(op, expr) => {
                let expr = expr.inline(context);
                Expr::UnaryOp(op.clone(), Box::new(expr))
            }
            Expr::BinOp(lhs, op, rhs) => {
                let lhs = lhs.inline(context);
                let rhs = rhs.inline(context);
                Expr::BinOp(Box::new(lhs), op.clone(), Box::new(rhs))
            }
            Expr::Variable(name) => {
                let expr = match context.get(name) {
                    Some(expr) => expr.clone(),
                    None => Expr::Variable(name.clone()),
                };
                let expr = expr.inline(context);
                context.insert(name.clone(), expr.clone());
                expr
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Declaration {
    VarAssignment(String, Expr),
    PublicVar(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Program {
    pub decls: Vec<Declaration>,
    pub expr: Expr,
}
