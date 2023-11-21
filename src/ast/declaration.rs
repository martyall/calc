use crate::ast::expression::{Expr, Ident};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Declaration {
    VarAssignment(Ident, Expr),
    PublicVar(Ident),
}

impl Declaration {
    // get the variable name bound in this declaration
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

// find the declaration for a given variable name
pub fn find_declaration(name: Ident, decls: Vec<Declaration>) -> Option<Declaration> {
    for decl in decls {
        match decl {
            Declaration::VarAssignment(n, expr) => {
                if n == name {
                    return Some(Declaration::VarAssignment(n.clone(), expr.clone()));
                }
            }
            Declaration::PublicVar(n) => {
                if n == name {
                    return Some(Declaration::PublicVar(n.clone()));
                }
            }
        }
    }
    None
}
