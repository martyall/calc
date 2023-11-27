use crate::ast::expression::Ident;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Display, Copy, Serialize, Deserialize, Eq)]
pub enum Ty {
    Field,
    Boolean,
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
