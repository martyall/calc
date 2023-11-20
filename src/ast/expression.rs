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

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
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
    pub fn inline(self, context: &mut HashMap<Ident, Expr>) -> Self {
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

    fn fold_constants(self) -> Self {
        match self {
            Expr::Number(n) => Expr::Number(n),
            Expr::Variable(name) => Expr::Variable(name),
            Expr::UnaryOp(op, expr) => {
                let expr = expr.fold_constants();
                match (op, expr) {
                    (UOpcode::Neg, Expr::Number(n)) => Expr::Number(-n),
                    (UOpcode::Neg, expr) => Expr::UnaryOp(UOpcode::Neg, Box::new(expr)),
                }
            }
            Expr::BinOp(lhs, op, rhs) => {
                let lhs = lhs.fold_constants();
                let rhs = rhs.fold_constants();
                match (lhs, op, rhs) {
                    (Expr::Number(n1), Opcode::Add, Expr::Number(n2)) => Expr::Number(n1 + n2),
                    (Expr::Number(n1), Opcode::Sub, Expr::Number(n2)) => Expr::Number(n1 - n2),
                    (Expr::Number(n1), Opcode::Mul, Expr::Number(n2)) => Expr::Number(n1 * n2),
                    (Expr::Number(n1), Opcode::Pow, Expr::Number(n2)) => {
                        Expr::Number(n1.pow(n2 as u32))
                    }
                    (lhs, op, rhs) => Expr::BinOp(Box::new(lhs), op, Box::new(rhs)),
                }
            }
        }
    }
}

#[cfg(test)]
mod ast_test {
    use super::*;

    #[test]
    fn const_folding_basic_test() {
        let expr1 = Expr::BinOp(
            Box::new(Expr::Number(1)),
            Opcode::Add,
            Box::new(Expr::Number(2)),
        );
        let expr2 = Expr::BinOp(
            Box::new(Expr::Number(3)),
            Opcode::Sub,
            Box::new(Expr::Number(4)),
        );
        let expr = Expr::BinOp(Box::new(expr1), Opcode::Mul, Box::new(expr2));
        assert_eq!(expr.fold_constants(), Expr::Number(-3));
    }
}
