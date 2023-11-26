use crate::ast::annotation::Span;
use crate::ast::Binder;
use crate::ast::{annotation::HasSourceLoc, inline, optimize, Expr, Ident, Program};
use anyhow::{anyhow, Result};
use err_derive::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error(display = "Unconstrained variable: {:?}", _0)]
    UnconstrainedVariable(Vec<(Ident, Span)>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CompiledProgram<A> {
    pub public_vars: Vec<Ident>,
    pub expr: Expr<A>,
}

pub fn compile<A: Clone + HasSourceLoc + Eq + Hash>(
    program: Program<A>,
) -> Result<CompiledProgram<A>> {
    let public_vars: Vec<Binder<A>> = program
        .public_variable_decls()
        .iter()
        .map(|decl| decl.binder().clone())
        .collect();
    let expr = optimize(inline(program));
    assert_normal_form(public_vars.clone(), &expr)?;
    let public_vars = public_vars.into_iter().map(|x| x.var).collect();
    Ok(CompiledProgram { public_vars, expr })
}

// normal form means that every public variable appears in `expr`
fn assert_normal_form<A: Clone + Eq + Hash + HasSourceLoc>(
    public_vars: Vec<Binder<A>>,
    expr: &Expr<A>,
) -> Result<()> {
    let public_vars = public_vars
        .into_iter()
        .map(|x| x.var)
        .collect::<HashSet<Ident>>();
    let unconstrained_vars: Vec<(Ident, A)> = expr
        .variables()
        .into_iter()
        .filter(|x| !public_vars.contains(&x.0))
        .collect();
    if !unconstrained_vars.is_empty() {
        Err(anyhow!(CompilerError::UnconstrainedVariable(
            unconstrained_vars
                .into_iter()
                .map(|(var, ann)| (var, ann.source_loc()))
                .collect()
        )))
    } else {
        Ok(())
    }
}
