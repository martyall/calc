use crate::ast::annotation::{HasSourceLoc, Span};
use crate::ast::expression::{Expr, Ident};
use crate::ast::typechecker::{Ty, TypeContext};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Binder<A> {
    VarBinder { ann: A, var: Ident },
    TypedBinder { ann: A, var: Ident, _type: Ty },
}

impl<A> Binder<A> {
    pub fn clear_annotations(self) -> Binder<()> {
        match self {
            Binder::VarBinder { var, .. } => Binder::VarBinder { ann: (), var },
            Binder::TypedBinder { var, _type, .. } => Binder::TypedBinder {
                ann: (),
                var,
                _type,
            },
        }
    }
    pub fn var(&self) -> &Ident {
        match self {
            Binder::VarBinder { var, .. } => var,
            Binder::TypedBinder { var, .. } => var,
        }
    }
    pub fn ann(&self) -> &A {
        match self {
            Binder::VarBinder { ann, .. } => ann,
            Binder::TypedBinder { ann, .. } => ann,
        }
    }
}

impl<A: Default> Binder<A> {
    pub fn default(ident: Ident, ty: Option<Ty>) -> Self {
        match ty {
            Some(ty) => Binder::TypedBinder {
                ann: A::default(),
                var: ident,
                _type: ty,
            },
            None => Binder::VarBinder {
                ann: A::default(),
                var: ident,
            },
        }
    }
}

impl<A: HasSourceLoc> HasSourceLoc for Binder<A> {
    fn source_loc(&self) -> Span {
        match self {
            Binder::VarBinder { ann, .. } => ann.source_loc(),
            Binder::TypedBinder { ann, .. } => ann.source_loc(),
        }
    }
}

impl<A: Clone> Clone for Binder<A> {
    fn clone(&self) -> Self {
        match self {
            Binder::VarBinder { ann, var } => Binder::VarBinder {
                ann: ann.clone(),
                var: var.clone(),
            },
            Binder::TypedBinder { ann, var, _type } => Binder::TypedBinder {
                ann: ann.clone(),
                var: var.clone(),
                _type: _type.clone(),
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
            Declaration::PublicVar { binder, .. } => binder,
        }
    }
}

impl<A: HasSourceLoc> HasSourceLoc for Declaration<A> {
    fn source_loc(&self) -> Span {
        match self.binder() {
            Binder::VarBinder { ann, .. } => ann.source_loc(),
            Binder::TypedBinder { ann, .. } => ann.source_loc(),
        }
    }
}

impl<A: Clone> Declaration<A> {
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

impl<A: Clone + HasSourceLoc> Declaration<A> {
    pub fn typecheck(&self, context: &mut TypeContext) -> Result<()> {
        match self {
            Declaration::VarAssignment { binder, expr } => {
                let expr_ty = expr.typecheck(context)?;
                context.context.insert(binder.var().clone(), expr_ty);
                Ok(())
            }
            Declaration::PublicVar { binder } => match binder {
                Binder::TypedBinder { var, _type, .. } => {
                    context.context.insert(var.clone(), _type.clone());
                    Ok(())
                }
                _ => Ok(()),
            },
        }
    }
}
