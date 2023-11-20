use crate::ast::expression::{Expr, Ident};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Declaration {
    VarAssignment(Ident, Expr),
    PublicVar(Ident),
}

impl Declaration {
    // get the variable name for this declaration
    pub fn get_identifier(&self) -> Ident {
        match self {
            Declaration::VarAssignment(name, _) => name.clone(),
            Declaration::PublicVar(name) => name.clone(),
        }
    }

    // get all the free variables in the expression bound in this declaration
    // (none for public variables)
    pub fn get_dependencies(&self) -> Vec<Ident> {
        match self {
            Declaration::VarAssignment(_, expr) => {
                let mut vars = expr.variables();
                vars.dedup();
                vars
            }
            Declaration::PublicVar(_) => vec![],
        }
    }
}
