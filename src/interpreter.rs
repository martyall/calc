use crate::ast::{error::ASTError, inline, Expr, Ident, Opcode, Program, UOpcode};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct Context {
    context: HashMap<Ident, Expr>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            context: HashMap::new(),
        }
    }

    fn get(&self, name: &Ident) -> Option<Expr> {
        self.context.get(name).cloned()
    }
}

impl From<HashMap<Ident, i32>> for Context {
    fn from(initial_context: HashMap<Ident, i32>) -> Self {
        let context = initial_context
            .iter()
            .map(|(k, v)| (k.clone(), Expr::Number(*v)))
            .collect();
        Context { context }
    }
}

pub fn interpret_expr(context: &mut Context, expr: &Expr) -> Result<i32> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::UnaryOp(op, expr) => {
            let expr = interpret_expr(context, expr)?;
            match op {
                UOpcode::Neg => Ok(-expr),
            }
        }
        Expr::BinOp(lhs, op, rhs) => {
            let lhs = interpret_expr(context, lhs)?;
            let rhs = interpret_expr(context, rhs)?;
            match op {
                Opcode::Add => Ok(lhs + rhs),
                Opcode::Sub => Ok(lhs - rhs),
                Opcode::Mul => Ok(lhs * rhs),
                Opcode::Pow => Ok(lhs.pow(rhs as u32)),
            }
        }
        Expr::Variable(name) => match context.get(name) {
            Some(expr) => interpret_expr(context, &expr),
            None => return Err(anyhow!(ASTError::UnboundIdentifier(name.clone()))),
        },
    }
}

pub fn interpret(initial_context: HashMap<Ident, i32>, program: Program) -> Result<i32> {
    let mut context: Context = initial_context.into();
    let expr = inline(program);
    interpret_expr(&mut context, &expr)
}

#[cfg(test)]
mod interpreter_tests {
    use super::*;
    use crate::parser;

    #[test]
    fn no_parens_test() {
        let input = "22 * 44 + 66";
        let expr = parser::parse_single_expression(input).unwrap();
        let mut context = Context::new();
        assert_eq!(interpret_expr(&mut context, &expr).unwrap(), 1034);
    }

    #[test]
    fn parens_test() {
        let input = "22 * (44 + 66)";
        let expr = parser::parse_single_expression(input).unwrap();
        let mut context = Context::new();
        assert_eq!(interpret_expr(&mut context, &expr).unwrap(), 2420);
    }

    #[test]
    fn pow_test() {
        let input = "2^4 + 1";
        let expr = parser::parse_single_expression(input).unwrap();
        let mut context = Context::new();
        assert_eq!(interpret_expr(&mut context, &expr).unwrap(), 17);
    }
}
