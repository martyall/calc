use crate::ast::Literal;
use crate::core::expression::LambdaExpr;
use std::{collections::HashMap, rc::Rc};

pub fn evaluate(env: &HashMap<String, Rc<LambdaExpr>>, expr: Rc<LambdaExpr>) -> Rc<LambdaExpr> {
    match &*expr {
        LambdaExpr::Var(x) => {
            // Look up the variable in the environment.
            if let Some(expr) = env.get(x) {
                // If it's found, return the expression.
                Rc::clone(expr)
            } else {
                // If it's not found, return as is or handle as an error.
                Rc::clone(&expr)
            }
        }
        LambdaExpr::Literal(_) => {
            // Literals are returned as is.
            Rc::clone(&expr)
        }
        LambdaExpr::Abs(_) => {
            // Abstractions cannot be evaluated further without an application.
            Rc::clone(&expr)
        }
        LambdaExpr::App(fun, arg) => {
            match &**fun {
                LambdaExpr::Abs(f) => {
                    // Apply the function to the argument.
                    evaluate(env, f(Rc::clone(arg)))
                }
                _ => {
                    // If it's not an abstraction, return as is or handle as an error.
                    evaluate(
                        env,
                        Rc::new(LambdaExpr::App(
                            evaluate(env, Rc::clone(fun)),
                            evaluate(env, Rc::clone(arg)),
                        )),
                    )
                }
            }
        }
        LambdaExpr::IfThenElse(cond, _then, _else) => {
            match &**cond {
                LambdaExpr::Literal(Literal::Boolean(b)) => {
                    if *b {
                        evaluate(env, Rc::clone(_then))
                    } else {
                        evaluate(env, Rc::clone(_else))
                    }
                }
                _ => {
                    // If condition is not a boolean literal, return as is or handle as an error.
                    evaluate(
                        env,
                        Rc::new(LambdaExpr::IfThenElse(
                            evaluate(env, Rc::clone(cond)),
                            Rc::clone(_then),
                            Rc::clone(_else),
                        )),
                    )
                }
            }
        } // ... other variants
    }
}

#[cfg(test)]
mod test_hoas_encoding {
    use super::*;
    use crate::ast::{
        self,
        expression::{Expr, Opcode},
    };
    use crate::core::expression::ast_expr_to_lambda_expr;
    use crate::core::prim::prim;

    #[test]
    fn test_evaluate() {
        let prelude = prim();
        let expr: ast::expression::Expr<()> =
            Expr::binary_op_default(Expr::field_default(1), Opcode::Add, Expr::field_default(2));
        println!("expr: {:?}", expr);
        let expr = ast_expr_to_lambda_expr(expr);
        let expr = evaluate(&prelude, Rc::new(expr));
        assert_eq!(expr.as_literal(), Some(Literal::Field(3)));
    }
}
