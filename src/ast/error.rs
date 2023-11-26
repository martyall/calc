use crate::ast::annotation::Span;
use crate::ast::expression::Ident;
use err_derive::Error;

#[derive(Debug, Error)]
pub enum ASTError {
    #[error(display = "Cyclic dependency for binding at {}: {}", _0, _1)]
    CyclicDependency(Span, Ident),
    #[error(display = "Duplicate identifier at {}: {}", _0, 1)]
    DuplicateIdentifier(Span, Ident),
    #[error(display = "Unbound identifier at {}: {}", _0, _1)]
    UnboundIdentifier(Span, Ident),
}
