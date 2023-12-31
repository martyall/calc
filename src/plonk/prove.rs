use std::collections::HashMap;

use crate::ast::{Ident, Literal};
use crate::compiler::CompiledProgram;
use crate::plonk::circuit_builder::ProvableCircuit;
use crate::plonk::circuit_builder::{build_circuit, from_literal};
use crate::plonk::parameters::*;
use anyhow::Result;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_data::CircuitData;

pub fn prove<A>(
    initital_context: HashMap<Ident, Literal>,
    program: CompiledProgram<A>,
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
    initital_context: &HashMap<Ident, Literal>,
) -> (PartialWitness<F>, Vec<Ident>) {
    let mut pw = PartialWitness::<F>::new();
    let mut inputs = Vec::new();
    for (ident, value) in initital_context {
        let target = match circuit.public_inputs.get(ident) {
            Some(target) => *target,
            None => panic!("Public input {} not found in circuit", ident),
        };
        let val = from_literal(*value);
        pw.set_target(target, val);
        inputs.push(ident.clone());
    }
    (pw, inputs)
}
