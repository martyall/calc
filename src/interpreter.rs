use crate::ast::{Expr, Opcode, UOpcode};

pub fn interpret(expr: &Expr) -> i32 {
    match expr {
        Expr::Number(n) => n.clone(),
        Expr::UnaryOp(op, expr) => {
            let expr = interpret(expr);
            match op {
                UOpcode::Neg => -expr,
            }
        }
        Expr::BinOp(lhs, op, rhs) => {
            let lhs = interpret(lhs);
            let rhs = interpret(rhs);
            match op {
                Opcode::Add => lhs + rhs,
                Opcode::Sub => lhs - rhs,
                Opcode::Mul => lhs * rhs,
                Opcode::Div => lhs / rhs,
                Opcode::Pow => lhs.pow(rhs as u32),
            }
        }
    }
}

#[cfg(test)]
mod interpreter_tests {
    use super::*;
    use crate::parser;

    #[test]
    fn no_parens_test() {
        let input = "22 * 44 + 66";
        let expr = parser::parse(input).unwrap();
        assert_eq!(interpret(&expr), 1034);
    }

    #[test]
    fn parens_test() {
        let input = "22 * (44 + 66)";
        let expr = parser::parse(input).unwrap();
        assert_eq!(interpret(&expr), 2420);
    }

    #[test]
    fn pow_test() {
        let input = "2^4 + 1";
        let expr = parser::parse(input).unwrap();
        assert_eq!(interpret(&expr), 17);
    }

    #[test]
    fn complex_test() {
        let input = "2^(4 +1 )  *  3+ (  2 + 1)^2";
        let expr = parser::parse(input).unwrap();
        assert_eq!(interpret(&expr), 32 * 3 + 9);
    }
}
