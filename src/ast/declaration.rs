use crate::ast::annotation::{HasSourceLoc, Span};
use crate::ast::expression::{Expr, Ident};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Binder<A> {
    pub ann: A,
    pub var: Ident,
}

impl<A> Binder<A> {
    pub fn clear_annotations(self) -> Binder<()> {
        Binder {
            ann: (),
            var: self.var,
        }
    }
}

impl<A: Default> Binder<A> {
    pub fn default(ident: Ident) -> Self {
        Binder {
            ann: A::default(),
            var: ident,
        }
    }
}

impl<A: Clone> Clone for Binder<A> {
    fn clone(&self) -> Self {
        match self {
            Binder { ann, var } => Binder {
                ann: ann.clone(),
                var: var.clone(),
            },
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Declaration<A> {
    VarAssignment { binder: Binder<A>, expr: Expr<A> },
    PublicVar { binder: Binder<A> },
}

impl<A> Clone for Declaration<A>
where
    A: Clone,
    Binder<A>: Clone,
    Expr<A>: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Declaration::VarAssignment { binder, expr } => Declaration::VarAssignment {
                binder: binder.clone(),
                expr: expr.clone(),
            },
            Declaration::PublicVar { binder } => Declaration::PublicVar {
                binder: binder.clone(),
            },
        }
    }
}

impl<A> Declaration<A> {
    pub fn binder(&self) -> &Binder<A> {
        match self {
            Declaration::VarAssignment { binder, .. } => binder,
            Declaration::PublicVar { binder } => binder,
        }
    }
}

impl<A: HasSourceLoc> HasSourceLoc for Declaration<A> {
    fn source_loc(&self) -> Span {
        match self.binder() {
            Binder { ann, .. } => ann.source_loc(),
        }
    }
}

impl<A: Clone> Declaration<A> {
    // get the variable name bound in this declaration
    pub fn get_identifier(&self) -> Binder<A> {
        match self {
            Declaration::VarAssignment { binder, .. } => binder.clone(),
            Declaration::PublicVar { binder } => binder.clone(),
        }
    }

    pub fn clear_annotations(self) -> Declaration<()> {
        match self {
            Declaration::VarAssignment { binder, expr } => Declaration::VarAssignment {
                binder: binder.clear_annotations(),
                expr: expr.clear_annotations(),
            },
            Declaration::PublicVar { binder } => Declaration::PublicVar {
                binder: binder.clear_annotations(),
            },
        }
    }
}

impl<A: Clone + PartialEq> Declaration<A> {
    // get all the free variables in the expression bound in this declaration
    // (none for public variables)
    pub fn get_dependencies(&self) -> Vec<(Ident, A)> {
        match self {
            Declaration::VarAssignment { expr, .. } => {
                let mut vars = expr.variables();
                vars.dedup();
                vars
            }
            Declaration::PublicVar { .. } => vec![],
        }
    }
}
