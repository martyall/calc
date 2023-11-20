use std::collections::HashMap;

use crate::ast::expression::{Expr, Ident};

// inline all variables in the expression using the context.
pub fn inline(context: &mut HashMap<Ident, Expr>, expr: Expr) -> Expr {
    match expr {
        Expr::Number(n) => Expr::Number(n),
        Expr::UnaryOp(op, expr) => {
            let expr = inline(context, *expr);
            Expr::UnaryOp(op, Box::new(expr))
        }
        Expr::BinOp(lhs, op, rhs) => {
            let lhs = inline(context, *lhs);
            let rhs = inline(context, *rhs);
            Expr::BinOp(Box::new(lhs), op, Box::new(rhs))
        }
        Expr::Variable(name) => {
            let maybe_existing = context.get(&name).cloned();
            let new_expr = if let Some(existing) = maybe_existing {
                inline(context, existing)
            } else {
                return Expr::Variable(name.clone());
            };
            context.insert(name.clone(), new_expr.clone());
            new_expr
        }
    }
}
