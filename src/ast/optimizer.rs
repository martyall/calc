use crate::ast::expression::{Expr, Opcode, UOpcode};

pub fn optimize(expr: Expr) -> Expr {
    fold_constants(expr)
}

// fold constants in the expression in the most naive way possible
fn fold_constants(expr: Expr) -> Expr {
    match expr {
        Expr::Number(n) => Expr::Number(n),
        Expr::Variable(name) => Expr::Variable(name),
        Expr::UnaryOp(op, expr) => {
            let expr = fold_constants(*expr);
            match (op, expr) {
                (UOpcode::Neg, Expr::Number(n)) => Expr::Number(-n),
                (UOpcode::Neg, expr) => Expr::UnaryOp(UOpcode::Neg, Box::new(expr)),
            }
        }
        Expr::BinOp(lhs, op, rhs) => {
            let lhs = fold_constants(*lhs);
            let rhs = fold_constants(*rhs);
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
        assert_eq!(fold_constants(expr), Expr::Number(-3));
    }
}
