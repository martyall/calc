use serde::{Deserialize, Serialize};

use std::collections::HashMap;

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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Expr {
    Number(i32),
    Variable(String),
    UnaryOp(UOpcode, Box<Expr>),
    BinOp(Box<Expr>, Opcode, Box<Expr>),
}

impl Expr {
    pub fn inline(self, context: &mut HashMap<String, Expr>) -> Self {
        match self {
            Expr::Number(n) => Expr::Number(n),
            Expr::UnaryOp(op, expr) => {
                let expr = expr.inline(context);
                Expr::UnaryOp(op, Box::new(expr))
            }
            Expr::BinOp(lhs, op, rhs) => {
                let lhs = lhs.inline(context);
                let rhs = rhs.inline(context);
                Expr::BinOp(Box::new(lhs), op, Box::new(rhs))
            }
            Expr::Variable(name) => {
                let maybe_existing = context.get(&name).cloned();
                let new_expr = if let Some(existing) = maybe_existing {
                    existing.inline(context)
                } else {
                    return Expr::Variable(name.clone());
                };
                context.insert(name.clone(), new_expr.clone());
                new_expr
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
