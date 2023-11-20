use std::collections::HashMap;

use crate::ast::{inline, Declaration, Expr, Ident, Opcode, Program, UOpcode};

pub fn interpret_expr(context: &HashMap<Ident, Expr>, expr: &Expr) -> i32 {
    match expr {
        Expr::Number(n) => *n,
        Expr::UnaryOp(op, expr) => {
            let expr = interpret_expr(context, expr);
            match op {
                UOpcode::Neg => -expr,
            }
        }
        Expr::BinOp(lhs, op, rhs) => {
            let lhs = interpret_expr(context, lhs);
            let rhs = interpret_expr(context, rhs);
            match op {
                Opcode::Add => lhs + rhs,
                Opcode::Sub => lhs - rhs,
                Opcode::Mul => lhs * rhs,
                Opcode::Pow => lhs.pow(rhs as u32),
            }
        }
        Expr::Variable(name) => match context.get(name) {
            Some(expr) => interpret_expr(context, expr),
            None => panic!("Variable {:?} not found in context", name),
        },
    }
}

pub fn interpret(initial_context: HashMap<Ident, i32>, program: Program) -> i32 {
    let mut context = initial_context
        .iter()
        .map(|(k, v)| (k.clone(), Expr::Number(*v)))
        .collect::<HashMap<Ident, Expr>>();
    for decl in &program.decls {
        match decl {
            Declaration::VarAssignment(name, expr) => {
                context.insert(name.clone(), expr.clone());
            }
            Declaration::PublicVar(name) => {
                if !context.contains_key(name) {
                    panic!("hey! you forgot to give me a value for {:?}", name)
                }
            }
        }
    }
    let expr = inline(&mut context, program.expr);
    interpret_expr(&context, &expr)
}

#[cfg(test)]
mod interpreter_tests {
    use super::*;
    use crate::parser;

    #[test]
    fn no_parens_test() {
        let input = "22 * 44 + 66";
        let expr = parser::parse_single_expression(input).unwrap();
        let mut context = HashMap::new();
        assert_eq!(interpret_expr(&mut context, &expr), 1034);
    }

    #[test]
    fn parens_test() {
        let input = "22 * (44 + 66)";
        let expr = parser::parse_single_expression(input).unwrap();
        let context = HashMap::new();
        assert_eq!(interpret_expr(&context, &expr), 2420);
    }

    #[test]
    fn pow_test() {
        let input = "2^4 + 1";
        let expr = parser::parse_single_expression(input).unwrap();
        let context = HashMap::new();
        assert_eq!(interpret_expr(&context, &expr), 17);
    }
}
