use crate::ast::{annotation::HasSourceLoc, typechecker::TypeError};
use anyhow::{anyhow, Result};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::typechecker::{Ty, TypeContext};

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Pow,
    And,
    Or,
    Eq,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum UOpcode {
    Neg,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize, Display)]
pub struct Ident(String);

impl Ident {
    pub fn new(s: &str) -> Self {
        Ident(s.to_string())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Literal {
    Boolean(bool),
    Field(i32),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Field(n) => write!(f, "{}", n),
            Literal::Boolean(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Expr<A> {
    Literal {
        ann: A,
        value: Literal,
    },
    Variable {
        ann: A,
        value: Ident,
    },
    UnaryOp {
        ann: A,
        op: UOpcode,
        expr: Box<Expr<A>>,
    },
    BinOp {
        ann: A,
        lhs: Box<Expr<A>>,
        op: Opcode,
        rhs: Box<Expr<A>>,
    },
    IfThenElse {
        ann: A,
        cond: Box<Expr<A>>,
        _then: Box<Expr<A>>,
        _else: Box<Expr<A>>,
    },
}

impl<A: Clone> Clone for Expr<A> {
    fn clone(&self) -> Self {
        match self {
            Expr::Literal { ann, value } => Expr::Literal {
                ann: ann.clone(),
                value: *value,
            },
            Expr::Variable { ann, value } => Expr::Variable {
                ann: ann.clone(),
                value: value.clone(), // Assuming Ident implements Clone
            },
            Expr::UnaryOp { ann, op, expr } => Expr::UnaryOp {
                ann: ann.clone(),
                op: *op, // Assuming UOpcode is Clone
                expr: Box::new((**expr).clone()),
            },
            Expr::BinOp { ann, lhs, op, rhs } => Expr::BinOp {
                ann: ann.clone(),
                op: *op, // Assuming Opcode is Clone
                lhs: Box::new((**lhs).clone()),
                rhs: Box::new((**rhs).clone()),
            },
            Expr::IfThenElse {
                ann,
                cond,
                _then,
                _else,
            } => Expr::IfThenElse {
                ann: ann.clone(),
                cond: Box::new((**cond).clone()),
                _then: Box::new((**_then).clone()),
                _else: Box::new((**_else).clone()),
            },
        }
    }
}

impl<A> Expr<A> {
    pub fn format(&self) -> String {
        match self {
            Expr::Literal { value, .. } => value.to_string(),
            Expr::UnaryOp { op, expr, .. } => match op {
                UOpcode::Neg => format!("-({})", expr.format()),
            },
            Expr::BinOp { lhs, op, rhs, .. } => match op {
                Opcode::Add => format!("({} + {})", lhs.format(), rhs.format()),
                Opcode::Sub => format!("({} - {})", lhs.format(), rhs.format()),
                Opcode::Mul => format!("({} * {})", lhs.format(), rhs.format()),
                Opcode::Pow => format!("({} ^ {})", lhs.format(), rhs.format()),
                Opcode::And => format!("({} && {})", lhs.format(), rhs.format()),
                Opcode::Or => format!("({} || {})", lhs.format(), rhs.format()),
                Opcode::Eq => format!("({} == {})", lhs.format(), rhs.format()),
            },
            Expr::Variable { value, .. } => value.to_string(),
            Expr::IfThenElse {
                cond, _then, _else, ..
            } => format!(
                "(if {} then {} else {})",
                cond.format(),
                _then.format(),
                _else.format()
            ),
        }
    }
}

impl<A: Clone> Expr<A> {
    // get all of the variables that appear in an expression
    pub fn variables(&self) -> Vec<(Ident, A)> {
        match self {
            Expr::Literal { .. } => vec![],
            Expr::UnaryOp { expr, .. } => expr.variables(),
            Expr::BinOp { lhs, rhs, .. } => {
                let mut deps = lhs.variables();
                deps.append(&mut rhs.variables());
                deps
            }
            Expr::Variable { value, ann } => vec![(value.clone(), ann.clone())],
            Expr::IfThenElse {
                cond, _then, _else, ..
            } => {
                let mut deps = cond.variables();
                deps.append(&mut _then.variables());
                deps.append(&mut _else.variables());
                deps
            }
        }
    }

    pub fn clear_annotations(self) -> Expr<()> {
        match self {
            Expr::Literal { value, .. } => Expr::Literal { ann: (), value },
            Expr::UnaryOp { op, expr, .. } => Expr::UnaryOp {
                ann: (),
                op,
                expr: Box::new(expr.clear_annotations()),
            },
            Expr::BinOp { lhs, op, rhs, .. } => Expr::BinOp {
                ann: (),
                lhs: Box::new(lhs.clear_annotations()),
                op,
                rhs: Box::new(rhs.clear_annotations()),
            },
            Expr::Variable { value, .. } => Expr::Variable { ann: (), value },
            Expr::IfThenElse {
                cond, _then, _else, ..
            } => Expr::IfThenElse {
                ann: (),
                cond: Box::new(cond.clear_annotations()),
                _then: Box::new(_then.clear_annotations()),
                _else: Box::new(_else.clear_annotations()),
            },
        }
    }
}

impl<A: HasSourceLoc> HasSourceLoc for Expr<A> {
    fn source_loc(&self) -> crate::ast::annotation::Span {
        match self {
            Expr::Literal { ann, .. } => ann.source_loc(),
            Expr::UnaryOp { ann, .. } => ann.source_loc(),
            Expr::BinOp { ann, .. } => ann.source_loc(),
            Expr::Variable { ann, .. } => ann.source_loc(),
            Expr::IfThenElse { ann, .. } => ann.source_loc(),
        }
    }
}

impl<A: Default> Expr<A> {
    pub fn field_default(value: i32) -> Self {
        Expr::Literal {
            ann: A::default(),
            value: Literal::Field(value),
        }
    }

    pub fn bool_default(value: bool) -> Self {
        Expr::Literal {
            ann: A::default(),
            value: Literal::Boolean(value),
        }
    }

    pub fn variable_default(value: Ident) -> Self {
        Expr::Variable {
            ann: A::default(),
            value,
        }
    }

    pub fn unary_op_default(op: UOpcode, expr: Expr<A>) -> Self {
        Expr::UnaryOp {
            ann: A::default(),
            op,
            expr: Box::new(expr),
        }
    }

    pub fn binary_op_default(lhs: Expr<A>, op: Opcode, rhs: Expr<A>) -> Self {
        Expr::BinOp {
            ann: A::default(),
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }
}

impl<A: Clone + HasSourceLoc> Expr<A> {
    pub fn typecheck(&self, context: &TypeContext) -> Result<Ty> {
        match self {
            Expr::Literal { value, .. } => match value {
                Literal::Field(_) => Ok(Ty::Field),
                Literal::Boolean(_) => Ok(Ty::Boolean),
            },
            Expr::Variable { ann, value } => match context.context.get(&value) {
                Some(ty) => Ok(ty.clone()),
                None => Err(anyhow!(TypeError::UndefinedVariable(
                    ann.source_loc(),
                    value.clone()
                ))),
            },
            Expr::UnaryOp { ann, op, expr } => {
                let expr_ty = expr.typecheck(context)?;
                match op {
                    UOpcode::Neg => match expr_ty {
                        Ty::Field => Ok(Ty::Field),
                        _ => Err(anyhow!(TypeError::TypeMismatch(
                            ann.source_loc(),
                            Ty::Field,
                            expr.source_loc(),
                            expr_ty,
                        ))),
                    },
                }
            }
            Expr::BinOp { ann, lhs, op, rhs } => {
                let lhs_ty = lhs.typecheck(context)?;
                let rhs_ty = rhs.typecheck(context)?;
                match op {
                    Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Pow => match (lhs_ty, rhs_ty)
                    {
                        (Ty::Field, Ty::Field) => Ok(Ty::Field),
                        (Ty::Field, _) => Err(anyhow!(TypeError::TypeMismatch(
                            ann.source_loc(),
                            Ty::Field,
                            rhs.source_loc(),
                            rhs_ty,
                        ))),
                        _ => Err(anyhow!(TypeError::TypeMismatch(
                            ann.source_loc(),
                            Ty::Field,
                            lhs.source_loc(),
                            lhs_ty,
                        ))),
                    },
                    Opcode::And | Opcode::Or => match (lhs_ty, rhs_ty) {
                        (Ty::Boolean, Ty::Boolean) => Ok(Ty::Boolean),
                        (Ty::Boolean, _) => Err(anyhow!(TypeError::TypeMismatch(
                            ann.source_loc(),
                            Ty::Boolean,
                            rhs.source_loc(),
                            rhs_ty,
                        ))),
                        _ => Err(anyhow!(TypeError::TypeMismatch(
                            ann.source_loc(),
                            Ty::Boolean,
                            lhs.source_loc(),
                            lhs_ty,
                        ))),
                    },
                    Opcode::Eq => match (lhs_ty, rhs_ty) {
                        (Ty::Field, Ty::Field) => Ok(Ty::Boolean),
                        (Ty::Field, _) => Err(anyhow!(TypeError::TypeMismatch(
                            ann.source_loc(),
                            Ty::Field,
                            rhs.source_loc(),
                            rhs_ty,
                        ))),
                        _ => Err(anyhow!(TypeError::TypeMismatch(
                            ann.source_loc(),
                            Ty::Field,
                            lhs.source_loc(),
                            lhs_ty,
                        ))),
                    },
                }
            }
            Expr::IfThenElse {
                ann,
                cond,
                _then,
                _else,
            } => {
                let cond_ty = cond.typecheck(context)?;
                let _then_ty = _then.typecheck(context)?;
                let _else_ty = _else.typecheck(context)?;
                match cond_ty {
                    Ty::Boolean => match (_then_ty, _else_ty) {
                        (Ty::Field, Ty::Field) => Ok(Ty::Field),
                        _ => Err(anyhow!(TypeError::TypeMismatch(
                            ann.source_loc(),
                            Ty::Field,
                            _then.clone().source_loc(),
                            _then_ty,
                        ))),
                    },
                    _ => Err(anyhow!(TypeError::TypeMismatch(
                        ann.source_loc(),
                        Ty::Boolean,
                        cond.source_loc(),
                        cond_ty,
                    ))),
                }
            }
        }
    }
}
