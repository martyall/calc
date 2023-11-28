use crate::ast::{Literal, Opcode, UOpcode};
use crate::core::expression::LambdaExpr;
use std::{collections::HashMap, rc::Rc};

pub fn prim() -> HashMap<String, Rc<LambdaExpr>> {
    let mut env = HashMap::new();
    make_bin_op(&mut env, Opcode::Add);
    make_bin_op(&mut env, Opcode::Sub);
    make_bin_op(&mut env, Opcode::Mul);
    make_bin_op(&mut env, Opcode::Pow);
    make_bin_op(&mut env, Opcode::Eq);
    make_bin_op(&mut env, Opcode::And);
    make_bin_op(&mut env, Opcode::Or);
    make_unary_op(UOpcode::Neg);
    env
}

fn make_bin_op(env: &mut HashMap<String, Rc<LambdaExpr>>, op: Opcode) -> () {
    let f = Rc::new(LambdaExpr::Abs(Box::new(move |x| {
        Rc::new(LambdaExpr::Abs(Box::new(move |y| match (&*x, &*y) {
            (
                LambdaExpr::Literal(Literal::Field(x_val)),
                LambdaExpr::Literal(Literal::Field(y_val)),
            ) => match op {
                Opcode::Add => Rc::new(LambdaExpr::Literal(Literal::Field(x_val + y_val))),
                Opcode::Sub => Rc::new(LambdaExpr::Literal(Literal::Field(x_val - y_val))),
                Opcode::Mul => Rc::new(LambdaExpr::Literal(Literal::Field(x_val * y_val))),
                Opcode::Pow => Rc::new(LambdaExpr::Literal(Literal::Field(
                    x_val.pow(*y_val as u32),
                ))),
                Opcode::Eq => Rc::new(LambdaExpr::Literal(Literal::Boolean(x_val == y_val))),
                _ => unreachable!("Invalid binary operation for field elems"),
            },
            (
                LambdaExpr::Literal(Literal::Boolean(x_val)),
                LambdaExpr::Literal(Literal::Boolean(y_val)),
            ) => match op {
                Opcode::And => Rc::new(LambdaExpr::Literal(Literal::Boolean(*x_val && *y_val))),
                Opcode::Or => Rc::new(LambdaExpr::Literal(Literal::Boolean(*x_val || *y_val))),
                _ => unreachable!("Invalid binary operation for booleans"),
            },
            _ => unreachable!("Invalid binary operation"),
        })))
    })));
    env.insert(op.to_string(), f);
}

fn make_unary_op(op: UOpcode) -> Rc<LambdaExpr> {
    Rc::new(LambdaExpr::Abs(Box::new(move |x| match &*x {
        LambdaExpr::Literal(Literal::Field(x_val)) => match op {
            UOpcode::Neg => Rc::new(LambdaExpr::Literal(Literal::Field(-x_val))),
        },
        _ => unreachable!("Invalid unary operation"),
    })))
}
