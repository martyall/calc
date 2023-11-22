use crate::ast::{Expr, Ident};
use anyhow::Result;
use plonky2::field::extension::Extendable;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use std::collections::HashMap;

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

struct CircuitBackend {
    builder: CircuitBuilder<F, D>,
}

impl CircuitBackend {
    fn new() -> Self {
        let config = CircuitConfig::standard_recursion_config();
        let builder = CircuitBuilder::new(config);
        Self { builder }
    }
}

fn add_public_inputs(inputs: Vec<Ident>, backend: &mut CircuitBackend) -> HashMap<Ident, Target> {
    let mut public_inputs = HashMap::new();
    for input in inputs {
        let t = backend.builder.add_virtual_target();
        backend.builder.register_public_input(t);
        public_inputs.insert(input, t);
    }
    public_inputs
}
