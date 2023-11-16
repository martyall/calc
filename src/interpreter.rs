use std::collections::HashMap;

use crate::ast::{Declaration, Expr, Opcode, Program, UOpcode};

pub fn interpret_expr(context: &HashMap<String, Expr>, expr: &Expr) -> i32 {
    match expr {
        Expr::Number(n) => n.clone(),
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
        Expr::Variable(name) => {
            let expr = match context.get(name) {
                Some(expr) => expr,
                None => panic!("Variable {} not found in context", name),
            };
            interpret_expr(context, expr)
        }
    }
}

pub fn interpret(program: &Program) -> i32 {
    let mut context = HashMap::new();
    for decl in &program.decls {
        match decl {
            Declaration::VarAssignment(name, expr) => {
                context.insert(name.clone(), expr.clone());
            }
        }
    }
    let expr = program.expr.inline(&mut context);
    interpret_expr(&context, &expr)
}

#[cfg(test)]
mod interpreter_tests {
    use super::*;
    use crate::ast::Expr;
    use crate::parser;
    use pest::error::Error;
    use pest::Parser;

    pub fn parse_single_expression(input: &str) -> Result<Expr, Error<parser::Rule>> {
        let mut pairs = parser::CalcParser::parse(parser::Rule::expression, input)?;
        let pair = pairs.next().unwrap();
        Ok(parser::parse_expr(pair.into_inner()))
    }

    #[test]
    fn no_parens_test() {
        let input = "22 * 44 + 66";
        let expr = parse_single_expression(input).unwrap();
        let mut context = HashMap::new();
        assert_eq!(interpret_expr(&mut context, &expr), 1034);
    }

    #[test]
    fn parens_test() {
        let input = "22 * (44 + 66)";
        let expr = parse_single_expression(input).unwrap();
        let context = HashMap::new();
        assert_eq!(interpret_expr(&context, &expr), 2420);
    }

    #[test]
    fn pow_test() {
        let input = "2^4 + 1";
        let expr = parse_single_expression(input).unwrap();
        let context = HashMap::new();
        assert_eq!(interpret_expr(&context, &expr), 17);
    }

    #[test]
    fn complex_test() {
        let input = "2^(4 +1 )  *  3+ (  2 + 1)^2";
        let expr = parse_single_expression(input).unwrap();
        let context = HashMap::new();
        assert_eq!(interpret_expr(&context, &expr), 32 * 3 + 9);
    }
}
