use crate::ast::{Expr, Literal};
use std::rc::Rc;

//impl Expr {
//    pub fn free_variables(&self) -> Vec<Ident> {
//        match self {
//            Expr::Literal { .. } => vec![],
//            Expr::Variable { value } => vec![value.clone()],
//            Expr::App { fun, arg } => {
//                let mut free = fun.free_variables();
//                free.append(&mut arg.free_variables());
//                free
//            }
//            Expr::Abs { param, body } => {
//                let mut free = body.free_variables();
//                free.retain(|x| x != param);
//                free
//            }
//            Expr::IfThenElse { cond, _then, _else } => {
//                let mut free = cond.free_variables();
//                free.append(&mut _then.free_variables());
//                free.append(&mut _else.free_variables());
//                free
//            }
//            Expr::UnaryOp { expr, .. } => expr.free_variables(),
//            Expr::BinOp { lhs, rhs, .. } => {
//                let mut free = lhs.free_variables();
//                free.append(&mut rhs.free_variables());
//                free
//            }
//        }
//    }
//}
//
//#[cfg(test)]
//mod test_free_variables {
//    use super::*;
//
//    #[test]
//    fn test_free_variables_abs() {
//        let expr = Expr::Abs {
//            param: Ident::new("x"),
//            body: Box::new(Expr::Variable {
//                value: Ident::new("y"),
//            }),
//        };
//        assert_eq!(expr.free_variables(), vec![Ident::new("y")]);
//    }
//
//    #[test]
//    fn test_free_variables_app() {
//        let body = Expr::App {
//            fun: Box::new(Expr::Variable {
//                value: Ident::new("x"),
//            }),
//            arg: Box::new(Expr::Variable {
//                value: Ident::new("y"),
//            }),
//        };
//        let fun = Expr::Abs {
//            param: Ident::new("x"),
//            body: Box::new(body),
//        };
//        assert_eq!(fun.free_variables(), vec![Ident::new("y")]);
//    }
//}

pub enum LambdaExpr {
    Var(String),
    Literal(Literal),
    Abs(Box<dyn Fn(Rc<LambdaExpr>) -> Rc<LambdaExpr>>),
    App(Rc<LambdaExpr>, Rc<LambdaExpr>),
    IfThenElse(Rc<LambdaExpr>, Rc<LambdaExpr>, Rc<LambdaExpr>),
}

impl LambdaExpr {
    pub fn as_literal(&self) -> Option<Literal> {
        match self {
            LambdaExpr::Literal(l) => Some(l.clone()),
            _ => None,
        }
    }
}

pub fn ast_expr_to_lambda_expr(expr: Expr<()>) -> LambdaExpr {
    match expr {
        Expr::Literal { value, .. } => LambdaExpr::Literal(value),
        Expr::Variable { value, .. } => LambdaExpr::Var(value.to_string()),
        Expr::IfThenElse {
            cond, _then, _else, ..
        } => LambdaExpr::IfThenElse(
            Rc::new(ast_expr_to_lambda_expr(*cond)),
            Rc::new(ast_expr_to_lambda_expr(*_then)),
            Rc::new(ast_expr_to_lambda_expr(*_else)),
        ),
        Expr::UnaryOp { op, expr, .. } => {
            let expr = Rc::new(ast_expr_to_lambda_expr(*expr));
            let fun = LambdaExpr::Var(op.to_string());
            LambdaExpr::App(Rc::new(fun), expr)
        }
        Expr::BinOp { op, lhs, rhs, .. } => {
            let lhs = Rc::new(ast_expr_to_lambda_expr(*lhs));
            let rhs = Rc::new(ast_expr_to_lambda_expr(*rhs));
            let fun = LambdaExpr::Var(op.to_string());
            LambdaExpr::App(Rc::new(LambdaExpr::App(Rc::new(fun), lhs)), rhs)
        }
    }
}
