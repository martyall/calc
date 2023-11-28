pub mod expression;
pub mod inliner;
pub mod interpreter;
pub mod prim;

pub use expression::{ast_expr_to_lambda_expr, LambdaExpr};
pub use inliner::inline;
pub use interpreter::evaluate;
pub use prim::prim;
