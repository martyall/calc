use std::collections::HashMap;

use crate::ast::declaration::Declaration;
use crate::ast::expression::{Expr, Ident};
use crate::ast::program::Program;

pub struct Context<A> {
    context: HashMap<Ident, Expr<A>>,
}

impl<A> Context<A> {
    fn new() -> Self {
        Context {
            context: HashMap::new(),
        }
    }

    fn insert(&mut self, name: Ident, expr: Expr<A>) {
        self.context.insert(name, expr);
    }

    fn get(&self, name: &Ident) -> Option<&Expr<A>> {
        self.context.get(name)
    }
}

// inline all variables in the expression using the context.
fn inline_expr<A: Clone>(context: &mut Context<A>, expr: Expr<A>) -> Expr<A> {
    match expr {
        Expr::Literal { ann, value } => Expr::Literal { ann, value },
        Expr::UnaryOp { ann, op, expr } => {
            let expr = inline_expr(context, *expr);
            Expr::UnaryOp {
                ann,
                op,
                expr: Box::new(expr),
            }
        }
        Expr::BinOp { ann, lhs, op, rhs } => {
            let lhs = inline_expr(context, *lhs);
            let rhs = inline_expr(context, *rhs);
            Expr::BinOp {
                ann,
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        }
        Expr::Variable { ann, value } => {
            let maybe_existing = context.get(&value).cloned();
            let new_expr = if let Some(existing) = maybe_existing {
                inline_expr(context, existing)
            } else {
                return Expr::Variable {
                    ann,
                    value: value.clone(),
                };
            };
            context.insert(value.clone(), new_expr.clone());
            new_expr
        }
        Expr::IfThenElse {
            ann,
            cond,
            _then,
            _else,
        } => {
            let cond = inline_expr(context, *cond);
            let _then = inline_expr(context, *_then);
            let _else = inline_expr(context, *_else);
            Expr::IfThenElse {
                ann,
                cond: Box::new(cond),
                _then: Box::new(_then),
                _else: Box::new(_else),
            }
        }
    }
}

fn inline_decl<A: Clone>(mut context: Context<A>, decl: Declaration<A>) -> Context<A> {
    match decl {
        Declaration::VarAssignment { binder, expr } => {
            let expr = inline_expr(&mut context, expr);
            context.insert(binder.var, expr);
            context
        }
        Declaration::PublicVar { .. } => context,
    }
}

pub fn inline<A: Clone>(program: Program<A>) -> Expr<A> {
    let context = Context::new();
    let Program { decls, expr } = program;
    let mut context = decls.into_iter().fold(context, inline_decl);
    inline_expr(&mut context, expr)
}

#[cfg(test)]
mod inliner_tests {
    use super::*;
    use crate::ast::declaration::Binder;
    use crate::ast::expression::*;
    use crate::ast::program::Program;

    #[test]
    fn inliner_basic_test() {
        let expr1: Expr<()> =
            Expr::binary_op_default(Expr::field_default(1), Opcode::Add, Expr::field_default(2));
        let decls = vec![Declaration::VarAssignment {
            binder: Binder::default(Ident::new("x")),
            expr: expr1.clone(),
        }];
        let expr2 = Expr::binary_op_default(
            Expr::variable_default(Ident::new("x")),
            Opcode::Add,
            Expr::field_default(3),
        );
        let inlined = Expr::binary_op_default(
            Expr::binary_op_default(Expr::field_default(1), Opcode::Add, Expr::field_default(2)),
            Opcode::Add,
            Expr::field_default(3),
        );
        let program = Program::new(decls, expr2).unwrap();
        assert_eq!(inline(program), inlined);
    }
}
