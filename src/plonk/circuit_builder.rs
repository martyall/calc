use crate::ast::{Expr, Ident, Literal, Opcode, UOpcode};
use crate::compiler::CompiledProgram;
use crate::plonk::parameters::*;
use plonky2::field::types::Field;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use std::collections::HashMap;

fn interpret_as_target<A>(
    context: &mut HashMap<Ident, Target>,
    builder: &mut CircuitBuilder<F, D>,
    expr: Expr<A>,
) -> Target {
    match expr {
        Expr::Literal {
            value: Literal::Number(n),
            ..
        } => {
            let n = from_i32(n);
            builder.constant(n)
        }
        Expr::Variable { value: ident, .. } => match context.get(&ident) {
            Some(target) => *target,
            None => {
                let x = builder.add_virtual_target();
                context.insert(ident, x);
                x
            }
        },
        Expr::UnaryOp { op, expr, .. } => {
            let expr = interpret_as_target(context, builder, *expr);
            match op {
                UOpcode::Neg => builder.mul_const(F::NEG_ONE, expr),
            }
        }
        Expr::BinOp { lhs, op, rhs, .. } => {
            let lhs = interpret_as_target(context, builder, *lhs);
            let rhs = interpret_as_target(context, builder, *rhs);
            match op {
                Opcode::Add => builder.add(lhs, rhs),
                Opcode::Sub => builder.sub(lhs, rhs),
                Opcode::Mul => builder.mul(lhs, rhs),
                Opcode::Pow => builder.exp(lhs, rhs, 10),
            }
        }
    }
}

pub fn from_i32(n: i32) -> F {
    let sign = if n < 0 { F::NEG_ONE } else { F::ONE };
    let n = n.abs() as u32;
    sign * F::from_canonical_u32(n)
}

pub struct ProvableCircuit {
    pub public_inputs: HashMap<Ident, Target>,
    pub output: Target,
    pub builder: CircuitBuilder<F, D>,
}

pub fn build_circuit<A>(program: CompiledProgram<A>) -> ProvableCircuit {
    let config = CircuitConfig::standard_recursion_config();
    let mut builder: CircuitBuilder<F, D> = CircuitBuilder::new(config);
    let mut public_inputs = HashMap::new();
    let output = interpret_as_target(&mut public_inputs, &mut builder, program.expr);

    for ident in program.public_vars {
        let target = public_inputs.get(&ident).unwrap().clone();
        builder.register_public_input(target);
    }
    builder.register_public_input(output);
    ProvableCircuit {
        public_inputs,
        output,
        builder,
    }
}
