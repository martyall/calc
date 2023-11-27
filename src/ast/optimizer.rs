use crate::ast::expression::{Expr, Literal, Opcode, UOpcode};

pub fn optimize<A: Clone>(expr: Expr<A>) -> Expr<A> {
    fold_constants(expr)
}

// fold constants in the expression in the most naive way possible
fn fold_constants<A: Clone>(expr: Expr<A>) -> Expr<A> {
    match expr {
        Expr::Literal { ann, value } => Expr::Literal { ann, value },
        Expr::Variable { ann, value } => Expr::Variable { ann, value },
        Expr::UnaryOp { ann, op, expr } => {
            let expr = fold_constants(*expr);
            match (op, expr) {
                (
                    UOpcode::Neg,
                    Expr::Literal {
                        value: Literal::Field(n),
                        ..
                    },
                ) => Expr::Literal {
                    ann,
                    value: Literal::Field(-n),
                },
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
                (
                    Expr::Literal {
                        value: Literal::Field(n1),
                        ..
                    },
                    Opcode::Add,
                    Expr::Literal {
                        value: Literal::Field(n2),
                        ..
                    },
                ) => Expr::Literal {
                    ann,
                    value: Literal::Field(n1 + n2),
                },
                (
                    Expr::Literal {
                        value: Literal::Field(n1),
                        ..
                    },
                    Opcode::Sub,
                    Expr::Literal {
                        value: Literal::Field(n2),
                        ..
                    },
                ) => Expr::Literal {
                    ann,
                    value: Literal::Field(n1 - n2),
                },
                (
                    Expr::Literal {
                        value: Literal::Field(n1),
                        ..
                    },
                    Opcode::Mul,
                    Expr::Literal {
                        value: Literal::Field(n2),
                        ..
                    },
                ) => Expr::Literal {
                    ann,
                    value: Literal::Field(n1 * n2),
                },
                (
                    Expr::Literal {
                        value: Literal::Field(n1),
                        ..
                    },
                    Opcode::Pow,
                    Expr::Literal {
                        value: Literal::Field(n2),
                        ..
                    },
                ) => Expr::Literal {
                    ann,
                    value: Literal::Field(n1.pow(n2 as u32)),
                },
                (lhs, op, rhs) => Expr::BinOp {
                    ann,
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                },
            }
        }
        Expr::IfThenElse {
            ann,
            cond,
            _then,
            _else,
        } => {
            let cond = fold_constants(*cond);
            let _then = fold_constants(*_then);
            let _else = fold_constants(*_else);
            Expr::IfThenElse {
                ann,
                cond: Box::new(cond),
                _then: Box::new(_then),
                _else: Box::new(_else),
            }
        }
    }
}

#[cfg(test)]
mod ast_test {
    use super::*;

    #[test]
    fn const_folding_basic_test() {
        let expr1: Expr<()> =
            Expr::binary_op_default(Expr::field_default(1), Opcode::Add, Expr::field_default(2));

        let expr2 =
            Expr::binary_op_default(Expr::field_default(3), Opcode::Sub, Expr::field_default(4));

        let expr = Expr::binary_op_default(expr1, Opcode::Mul, expr2);
        assert_eq!(fold_constants(expr), Expr::field_default(-3));
    }
}
