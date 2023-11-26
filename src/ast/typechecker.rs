use crate::ast::annotation::Span;
use crate::ast::expression::Ident;
use derive_more::Display;
use err_derive::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Display, Copy, Serialize, Deserialize)]
pub enum Ty {
    Field,
    Boolean,
}

#[derive(Debug, Error)]
pub enum TypeError {
    #[error(display = "Undefined variable at {}: {}", _0, _1)] // TODO: Fix this
    UndefinedVariable(Span, Ident),
    #[error(
        display = "Type Error at {}. Could not match expected type {} with type {}",
        _0,
        _1,
        _3
    )]
    TypeMismatch(Span, Ty, Span, Ty),
}

pub struct TypeContext {
    pub context: HashMap<Ident, Ty>,
}

impl TypeContext {
    pub fn new() -> Self {
        TypeContext {
            context: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Ident) -> Option<Ty> {
        self.context.get(name).cloned()
    }
}
