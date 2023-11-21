pub mod declaration;
pub mod error;
pub mod expression;
pub mod inliner;
pub mod optimizer;
pub mod program;

pub use declaration::Declaration;
pub use expression::{Expr, Ident, Opcode, UOpcode};
pub use inliner::inline;
pub use optimizer::optimize;
pub use program::Program;
