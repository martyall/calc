use std::collections::HashMap;

use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_data::CircuitData;

use crate::ast::Ident;
use crate::compiler::CompiledProgram;
use crate::plonk::circuit_builder::{build_circuit, from_i32};
use crate::plonk::parameters::*;
use anyhow::Result;

use super::ProvableCircuit;

pub fn prove(
    initital_context: HashMap<Ident, i32>,
    program: CompiledProgram,
) -> Result<ProvingData> {
    let mut circuit = build_circuit(program);
    let (pw, inputs) = set_public_inputs(&mut circuit, &initital_context);
    let data = circuit.builder.build::<C>();
    Ok(ProvingData { data, pw, inputs })
}

pub struct ProvingData {
    pub data: CircuitData<F, C, 2>,
    pub pw: PartialWitness<F>,
    pub inputs: Vec<Ident>,
}

// We need to guarantee that the variables delclared initial context are the same as what
// we declared as public inputs in the circuit.
fn set_public_inputs(
    circuit: &mut ProvableCircuit,
    initital_context: &HashMap<Ident, i32>,
) -> (PartialWitness<F>, Vec<Ident>) {
    let mut pw = PartialWitness::<F>::new();
    let mut inputs = Vec::new();
    for (ident, value) in initital_context {
        let target = match circuit.public_inputs.get(ident) {
            Some(target) => *target,
            None => panic!("Public input {} not found in circuit", ident),
        };
        let val = from_i32(*value);
        pw.set_target(target, val);
        inputs.push(ident.clone());
    }
    (pw, inputs)
}
