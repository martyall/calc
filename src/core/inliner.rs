use super::LambdaExpr;
use std::rc::Rc;

pub fn inline(expr: Rc<LambdaExpr>) -> Rc<LambdaExpr> {
    match &*expr {
        LambdaExpr::Var(_) => Rc::clone(&expr),
        LambdaExpr::Literal(_) => Rc::clone(&expr),
        LambdaExpr::Abs(_) => Rc::clone(&expr),
        LambdaExpr::App(fun, arg) => {
            match &**fun {
                LambdaExpr::Abs(f) => {
                    // Apply the function to the argument.
                    inline(f(inline(Rc::clone(arg))))
                }
                _ => {
                    // If it's not an abstraction, return as is or handle as an error.
                    Rc::new(LambdaExpr::App(
                        inline(Rc::clone(fun)),
                        inline(Rc::clone(arg)),
                    ))
                }
            }
        }
        LambdaExpr::IfThenElse(cond, _then, _else) => Rc::new(LambdaExpr::IfThenElse(
            inline(Rc::clone(cond)),
            inline(Rc::clone(_then)),
            inline(Rc::clone(_else)),
        )),
    }
}
