use crate::ast::{inline, optimize, Expr, Ident, Program};
use anyhow::{anyhow, Result};
use err_derive::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error(display = "Unconstrained variables {:?}", _0)]
    UnconstrainedVariable(Vec<Ident>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CompiledProgram {
    pub public_vars: Vec<Ident>,
    pub expr: Expr,
}

pub fn compile(program: Program) -> Result<CompiledProgram> {
    let public_vars: Vec<Ident> = program
        .public_variable_decls()
        .iter()
        .map(|decl| decl.get_identifier())
        .collect();
    let expr = optimize(inline(program));
    assert_normal_form(public_vars.clone(), &expr)?;
    Ok(CompiledProgram { public_vars, expr })
}

// normal form means that every public variable appears in `expr`
fn assert_normal_form(public_vars: Vec<Ident>, expr: &Expr) -> Result<()> {
    let public_vars: HashSet<Ident> = public_vars.into_iter().collect();
    let expr_vars: HashSet<Ident> = expr.variables().into_iter().collect();
    let unconstrained_vars: Vec<Ident> = public_vars.difference(&expr_vars).cloned().collect();
    if !unconstrained_vars.is_empty() {
        Err(anyhow!(CompilerError::UnconstrainedVariable(
            unconstrained_vars
        )))
    } else {
        Ok(())
    }
}
