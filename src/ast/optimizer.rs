use crate::ast::expression::{Expr, Opcode, UOpcode};

pub fn optimize<A: Clone>(expr: Expr<A>) -> Expr<A> {
    fold_constants(expr)
}

// fold constants in the expression in the most naive way possible
fn fold_constants<A: Clone>(expr: Expr<A>) -> Expr<A> {
    match expr {
        Expr::Number { ann, value } => Expr::Number { ann, value },
        Expr::Variable { ann, value } => Expr::Variable { ann, value },
        Expr::UnaryOp { ann, op, expr } => {
            let expr = fold_constants(*expr);
            match (op, expr) {
                (UOpcode::Neg, Expr::Number { value: n, .. }) => Expr::Number { ann, value: -n },
                (_, expr) => Expr::UnaryOp {
                    ann,
                    op,
                    expr: Box::new(expr),
                },
            }
        }
        Expr::BinOp { ann, lhs, op, rhs } => {
            let lhs = fold_constants(*lhs);
            let rhs = fold_constants(*rhs);
            match (lhs, op, rhs) {
                (Expr::Number { value: n1, .. }, Opcode::Add, Expr::Number { value: n2, .. }) => {
                    Expr::Number {
                        ann,
                        value: n1 + n2,
                    }
                }
                (Expr::Number { value: n1, .. }, Opcode::Sub, Expr::Number { value: n2, .. }) => {
                    Expr::Number {
                        ann,
                        value: n1 - n2,
                    }
                }
                (Expr::Number { value: n1, .. }, Opcode::Mul, Expr::Number { value: n2, .. }) => {
                    Expr::Number {
                        ann,
                        value: n1 * n2,
                    }
                }
                (Expr::Number { value: n1, .. }, Opcode::Pow, Expr::Number { value: n2, .. }) => {
                    Expr::Number {
                        ann,
                        value: n1.pow(n2 as u32),
                    }
                }
                (lhs, op, rhs) => Expr::BinOp {
                    ann,
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                },
            }
        }
    }
}

#[cfg(test)]
mod ast_test {
    use super::*;

    #[test]
    fn const_folding_basic_test() {
        let expr1: Expr<()> = Expr::binary_op_default(
            Expr::number_default(1),
            Opcode::Add,
            Expr::number_default(2),
        );

        let expr2 = Expr::binary_op_default(
            Expr::number_default(3),
            Opcode::Sub,
            Expr::number_default(4),
        );

        let expr = Expr::binary_op_default(expr1, Opcode::Mul, expr2);
        assert_eq!(fold_constants(expr), Expr::number_default(-3));
    }
}
