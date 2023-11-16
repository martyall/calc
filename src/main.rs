pub mod ast;
pub mod interpreter;
pub mod parser;

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
    context: String,

    #[arg(short, long)]
    serialize: bool,
}

fn read_context(file_path: &str) -> io::Result<HashMap<String, i32>> {
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

    if args.serialize {
        let serialized = serde_json::to_string(&program).unwrap();
        println!("{}", serialized);
    } else {
        let initial_context = read_context(&args.context).unwrap_or(HashMap::new());
        let res = interpreter::interpret(&initial_context, &program);
        println!("Your result is: {:?}", res);
    }
}
