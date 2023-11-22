pub mod ast;
pub mod compiler;
pub mod interpreter;
pub mod parser;
pub mod plonk;

use ast::Ident;
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};

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

fn read_context(file_path: &str) -> io::Result<HashMap<Ident, i32>> {
    let file = File::open(file_path)?;
    let data = serde_json::from_reader(file)?;
    Ok(data)
}

fn main() {
    let args = Args::parse();
    let mut file = File::open(args.input_file).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let program = parser::parse(&contents).unwrap();
    let program = compiler::compile(program).unwrap();

    if args.serialize {
        let serialized = serde_json::to_string(&program).unwrap();
        println!("{}", serialized);
    } else {
        let initial_context = match args.context {
            None => HashMap::new(),
            Some(ref file_path) => read_context(&file_path).unwrap_or(HashMap::new()),
        };

        let mut context = interpreter::Context::from(initial_context);

        let res = interpreter::interpret(&mut context, &program.expr);
        println!("Your result is: {:?}", res.unwrap());
    }
}
