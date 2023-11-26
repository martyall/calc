use crate::ast::{
    annotation::HasSourceLoc, error::ASTError, Expr, Ident, Literal, Opcode, UOpcode,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct Context<A> {
    pub context: HashMap<Ident, Expr<A>>,
}

impl<A: Clone> Context<A> {
    pub fn new() -> Self {
        Context {
            context: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Ident) -> Option<Expr<A>> {
        self.context.get(name).cloned()
    }
}

impl<A: Clone + Default> From<HashMap<Ident, i32>> for Context<A> {
    fn from(initial_context: HashMap<Ident, i32>) -> Self {
        let context = initial_context
            .iter()
            .map(|(k, v)| (k.clone(), Expr::number_default(*v)))
            .collect();
        Context { context }
    }
}

pub fn interpret<A: Clone + HasSourceLoc>(context: &mut Context<A>, expr: &Expr<A>) -> Result<i32> {
    match expr {
        Expr::Literal {
            value: Literal::Number(n),
            ..
        } => Ok(*n),
        Expr::UnaryOp { op, expr, .. } => {
            let expr = interpret(context, expr)?;
            match op {
                UOpcode::Neg => Ok(-expr),
            }
        }
        Expr::BinOp { lhs, op, rhs, .. } => {
            let lhs = interpret(context, lhs)?;
            let rhs = interpret(context, rhs)?;
            match op {
                Opcode::Add => Ok(lhs + rhs),
                Opcode::Sub => Ok(lhs - rhs),
                Opcode::Mul => Ok(lhs * rhs),
                Opcode::Pow => Ok(lhs.pow(rhs as u32)),
            }
        }
        Expr::Variable { value, ann } => match context.get(value) {
            Some(expr) => interpret(context, &expr),
            None => {
                return Err(anyhow!(ASTError::UnboundIdentifier(
                    ann.source_loc(),
                    value.clone()
                )))
            }
        },
    }
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
        assert_eq!(interpret(&mut context, &expr).unwrap(), 1034);
    }

    #[test]
    fn parens_test() {
        let input = "22 * (44 + 66)";
        let expr = parser::parse_single_expression(input).unwrap();
        let mut context = Context::new();
        assert_eq!(interpret(&mut context, &expr).unwrap(), 2420);
    }

    #[test]
    fn pow_test() {
        let input = "2^4 + 1";
        let expr = parser::parse_single_expression(input).unwrap();
        let mut context = Context::new();
        assert_eq!(interpret(&mut context, &expr).unwrap(), 17);
    }
}
