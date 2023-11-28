pub mod ast;
pub mod compiler;
pub mod core;
pub mod interpreter;
pub mod parser;
pub mod plonk;

use anyhow::Result;
use ast::{Ident, Literal};
use clap::Parser;
use jemallocator::Jemalloc;
use plonk::{prove, F};
use plonky2::field::types::Field;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,

    #[arg(short, long)]
    context: Option<String>,

    #[arg(short, long)]
    serialize: bool,
}

fn read_context(file_path: &str) -> io::Result<HashMap<Ident, Literal>> {
    let file = File::open(file_path)?;
    let data = serde_json::from_reader(file)?;
    Ok(data)
}

fn default_main() -> Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.input_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let program = parser::parse(&contents)?;
    let program = compiler::compile(program)?;

    if args.serialize {
        let serialized = serde_json::to_string(&program)?;
        Ok(println!("{}", serialized))
    } else {
        let initial_context = match args.context {
            None => HashMap::new(),
            Some(ref file_path) => read_context(&file_path).unwrap(),
        };

        let interpreter_result = {
            let mut context = interpreter::Context::from(initial_context.clone());
            interpreter::interpret(&mut context, &program.expr)
        };

        println!(
            "According to the interpreter, your result is: {:?}",
            interpreter_result.unwrap()
        );

        let program_expr = program.expr.clone();

        let proving_data = prove(initial_context, program).unwrap();
        let proof = proving_data.data.prove(proving_data.pw).unwrap();
        let formatted_input: String = proving_data
            .inputs
            .into_iter()
            .zip(proof.public_inputs.clone().into_iter())
            .map(|(a, b)| format!("{}={}", a, b))
            .collect::<Vec<String>>()
            .join(", ");

        println!(
            "Proof for equation {} = {:?} (mod {:?}),  where {}",
            program_expr.format(),
            proof.public_inputs.last().unwrap(),
            F::order(),
            formatted_input
        );
        proving_data.data.verify(proof).unwrap();
        Ok(println!("Verified!"))
    }
}

fn main() {
    match default_main() {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
