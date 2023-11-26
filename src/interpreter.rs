use crate::ast::{
    annotation::HasSourceLoc, error::ASTError, Expr, Ident, Literal, Opcode, UOpcode,
};
use anyhow::{anyhow, Result};
use core::ops::{Add, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};
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

impl<A: Clone + Default> From<HashMap<Ident, Literal>> for Context<A> {
    fn from(initial_context: HashMap<Ident, Literal>) -> Self {
        let context = initial_context
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    Expr::Literal {
                        ann: A::default(),
                        value: v.clone(),
                    },
                )
            })
            .collect();
        Context { context }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Value {
    Field(i32),
    Boolean(bool),
}

impl Neg for Value {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Value::Field(n) => Value::Field(-n),
            _ => unreachable!("Only Fields can be negated"),
        }
    }
}

impl Add for Value {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Field(lhs), Value::Field(rhs)) => Value::Field(lhs + rhs),
            _ => unreachable!("Only Fields can be added"),
        }
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Field(lhs), Value::Field(rhs)) => Value::Field(lhs - rhs),
            _ => unreachable!("Only Fields can be subtracted"),
        }
    }
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Field(lhs), Value::Field(rhs)) => Value::Field(lhs * rhs),
            _ => unreachable!("Only Fields can be multiplied"),
        }
    }
}

impl Value {
    pub fn pow(self, rhs: Value) -> Self {
        match (self, rhs) {
            (Value::Field(n), Value::Field(m)) => Value::Field(n.pow(m as u32)),
            _ => unreachable!("Only Fields can be raised to a power"),
        }
    }
}

pub fn interpret<A: Clone + HasSourceLoc>(
    context: &mut Context<A>,
    expr: &Expr<A>,
) -> Result<Value> {
    match expr {
        Expr::Literal {
            value: Literal::Field(n),
            ..
        } => Ok(Value::Field(*n)),
        Expr::Literal {
            value: Literal::Boolean(b),
            ..
        } => Ok(Value::Boolean(*b)),
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
                Opcode::Pow => Ok(lhs.pow(rhs)),
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
        Expr::IfThenElse {
            cond, _then, _else, ..
        } => {
            let cond = interpret(context, cond)?;
            match cond {
                Value::Boolean(true) => interpret(context, _then),
                Value::Boolean(false) => interpret(context, _else),
                _ => unreachable!("Only booleans can be used as conditions"),
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
        let expr = parser::parse_single_expression(input).unwrap();
        let mut context = Context::new();
        assert_eq!(interpret(&mut context, &expr).unwrap(), Value::Field(1034));
    }

    #[test]
    fn parens_test() {
        let input = "22 * (44 + 66)";
        let expr = parser::parse_single_expression(input).unwrap();
        let mut context = Context::new();
        assert_eq!(interpret(&mut context, &expr).unwrap(), Value::Field(2420));
    }

    #[test]
    fn pow_test() {
        let input = "2^4 + 1";
        let expr = parser::parse_single_expression(input).unwrap();
        let mut context = Context::new();
        assert_eq!(interpret(&mut context, &expr).unwrap(), Value::Field(17));
    }
}
