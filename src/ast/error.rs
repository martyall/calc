use crate::ast::expression::Ident;
use err_derive::Error;

#[derive(Debug, Error)]
pub enum ASTError {
    #[error(display = "Cyclic dependency for binding: {}", _0)]
    CyclicDependency(Ident),
    #[error(display = "Duplicate identifier: {}", _0)]
    DuplicateIdentifier(Ident),
    #[error(display = "Unbound identifier: {}", _0)]
    UnboundIdentifier(Ident),
}
