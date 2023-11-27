use crate::ast::{self, Ident, Literal};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Expr {
    Literal {
        value: Literal,
    },
    Variable {
        value: Ident,
    },
    App {
        fun: Box<Expr>,
        arg: Box<Expr>,
    },
    Abs {
        param: Ident,
        body: Box<Expr>,
    },
    IfThenElse {
        cond: Box<Expr>,
        _then: Box<Expr>,
        _else: Box<Expr>,
    },
    UnaryOp {
        op: ast::UOpcode,
        expr: Box<Expr>,
    },
    BinOp {
        op: ast::Opcode,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

impl Expr {
    pub fn free_variables(&self) -> Vec<Ident> {
        match self {
            Expr::Literal { .. } => vec![],
            Expr::Variable { value } => vec![value.clone()],
            Expr::App { fun, arg } => {
                let mut free = fun.free_variables();
                free.append(&mut arg.free_variables());
                free
            }
            Expr::Abs { param, body } => {
                let mut free = body.free_variables();
                free.retain(|x| x != param);
                free
            }
            Expr::IfThenElse { cond, _then, _else } => {
                let mut free = cond.free_variables();
                free.append(&mut _then.free_variables());
                free.append(&mut _else.free_variables());
                free
            }
            Expr::UnaryOp { expr, .. } => expr.free_variables(),
            Expr::BinOp { lhs, rhs, .. } => {
                let mut free = lhs.free_variables();
                free.append(&mut rhs.free_variables());
                free
            }
        }
    }
}

#[cfg(test)]
mod test_free_variables {
    use super::*;

    #[test]
    fn test_free_variables_abs() {
        let expr = Expr::Abs {
            param: Ident::new("x"),
            body: Box::new(Expr::Variable {
                value: Ident::new("y"),
            }),
        };
        assert_eq!(expr.free_variables(), vec![Ident::new("y")]);
    }

    #[test]
    fn test_free_variables_app() {
        let body = Expr::App {
            fun: Box::new(Expr::Variable {
                value: Ident::new("x"),
            }),
            arg: Box::new(Expr::Variable {
                value: Ident::new("y"),
            }),
        };
        let fun = Expr::Abs {
            param: Ident::new("x"),
            body: Box::new(body),
        };
        assert_eq!(fun.free_variables(), vec![Ident::new("y")]);
    }
}

pub fn from_ast<A>(ast_expr: ast::Expr<A>) -> Expr {
    match ast_expr {
        ast::Expr::Literal { value, .. } => Expr::Literal { value },
        ast::Expr::Variable { value, .. } => Expr::Variable { value },
        ast::Expr::IfThenElse {
            cond, _then, _else, ..
        } => Expr::IfThenElse {
            cond: Box::new(from_ast(*cond)),
            _then: Box::new(from_ast(*_then)),
            _else: Box::new(from_ast(*_else)),
        },
        ast::Expr::UnaryOp { op, expr, .. } => Expr::UnaryOp {
            op,
            expr: Box::new(from_ast(*expr)),
        },
        ast::Expr::BinOp { op, lhs, rhs, .. } => Expr::BinOp {
            op,
            lhs: Box::new(from_ast(*lhs)),
            rhs: Box::new(from_ast(*rhs)),
        },
    }
}
