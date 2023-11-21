use std::collections::HashMap;

use crate::ast::declaration::Declaration;
use crate::ast::expression::{Expr, Ident};
use crate::ast::program::Program;

pub struct Context {
    context: HashMap<Ident, Expr>,
}

impl Context {
    fn new() -> Self {
        Context {
            context: HashMap::new(),
        }
    }

    fn insert(&mut self, name: Ident, expr: Expr) {
        self.context.insert(name, expr);
    }

    fn get(&self, name: &Ident) -> Option<&Expr> {
        self.context.get(name)
    }
}

// inline all variables in the expression using the context.
fn inline_expr(context: &mut Context, expr: Expr) -> Expr {
    match expr {
        Expr::Number(n) => Expr::Number(n),
        Expr::UnaryOp(op, expr) => {
            let expr = inline_expr(context, *expr);
            Expr::UnaryOp(op, Box::new(expr))
        }
        Expr::BinOp(lhs, op, rhs) => {
            let lhs = inline_expr(context, *lhs);
            let rhs = inline_expr(context, *rhs);
            Expr::BinOp(Box::new(lhs), op, Box::new(rhs))
        }
        Expr::Variable(name) => {
            let maybe_existing = context.get(&name).cloned();
            let new_expr = if let Some(existing) = maybe_existing {
                inline_expr(context, existing)
            } else {
                return Expr::Variable(name.clone());
            };
            context.insert(name.clone(), new_expr.clone());
            new_expr
        }
    }
}

fn inline_decl(mut context: Context, decl: Declaration) -> Context {
    match decl {
        Declaration::VarAssignment(v, expr) => {
            let expr = inline_expr(&mut context, expr);
            context.insert(v, expr);
            context
        }
        Declaration::PublicVar(_) => context,
    }
}

pub fn inline(program: Program) -> Expr {
    let context = Context::new();
    let Program { decls, expr } = program;
    let mut context = decls.into_iter().fold(context, inline_decl);
    inline_expr(&mut context, expr)
}

#[cfg(test)]
mod inliner_tests {
    use super::*;
    use crate::ast::expression::*;
    use crate::ast::program::Program;

    #[test]
    fn inliner_basic_test() {
        let expr1 = Expr::BinOp(
            Box::new(Expr::Number(1)),
            Opcode::Add,
            Box::new(Expr::Number(2)),
        );
        let decls = vec![Declaration::VarAssignment(Ident::new("x"), expr1.clone())];
        let expr2 = Expr::BinOp(
            Box::new(Expr::Variable(Ident::new("x"))),
            Opcode::Add,
            Box::new(Expr::Number(3)),
        );
        let inlined = Expr::BinOp(Box::new(expr1), Opcode::Add, Box::new(Expr::Number(3)));
        let program = Program::new(decls, expr2).unwrap();
        assert_eq!(inline(program), inlined);
    }
}
