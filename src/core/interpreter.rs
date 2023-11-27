use crate::ast::{Ident, Literal, Opcode, UOpcode};
use crate::core::expression::Expr;
use anyhow::Result;
use core::ops::{Add, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Context {
    pub context: HashMap<Ident, Expr>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            context: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Ident) -> Option<Expr> {
        self.context.get(name).cloned()
    }
}

impl From<HashMap<Ident, Literal>> for Context {
    fn from(initial_context: HashMap<Ident, Literal>) -> Self {
        let context = initial_context
            .iter()
            .map(|(k, v)| (k.clone(), Expr::Literal { value: v.clone() }))
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
    fn and(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs && rhs),
            _ => unreachable!("Only Booleans can be and-ed"),
        }
    }
    fn or(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs || rhs),
            _ => unreachable!("Only Booleans can be or-ed"),
        }
    }
}

pub fn interpret(context: &mut Context, expr: &Expr) -> Result<Value> {
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
                Opcode::And => Ok(lhs.and(rhs)),
                Opcode::Or => Ok(lhs.or(rhs)),
                Opcode::Eq => Ok(Value::Boolean(lhs == rhs)),
            }
        }
        Expr::Variable { value, .. } => match context.get(value) {
            Some(expr) => interpret(context, &expr),
            None => panic!("Unbound variable in interpreter: {}", value),
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
        _ => unreachable!("Only literals and variables can be interpreted"),
    }
}

#[cfg(test)]
mod core_interpreter_tests {
    use super::*;
    use crate::{core::expression::from_ast, parser};

    #[test]
    fn no_parens_test() {
        let input = "22 * 44 + 66";
        let expr = from_ast(parser::parse_single_expression(input).unwrap());
        let mut context = Context::new();
        assert_eq!(interpret(&mut context, &expr).unwrap(), Value::Field(1034));
    }

    #[test]
    fn parens_test() {
        let input = "22 * (44 + 66)";
        let expr = from_ast(parser::parse_single_expression(input).unwrap());
        let mut context = Context::new();
        assert_eq!(interpret(&mut context, &expr).unwrap(), Value::Field(2420));
    }

    #[test]
    fn pow_test() {
        let input = "2^4 + 1";
        let expr = from_ast(parser::parse_single_expression(input).unwrap());
        let mut context = Context::new();
        assert_eq!(interpret(&mut context, &expr).unwrap(), Value::Field(17));
    }
}
