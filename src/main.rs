pub mod ast;
pub mod interpreter;
pub mod parser;

use clap::Parser;
use std::fs::File;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,

    #[arg(short, long)]
    serialize: bool,
}

fn main() {
    let args = Args::parse();
    let mut file = File::open(args.input_file).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let expr = parser::parse(&contents).unwrap();

    if args.serialize {
        let serialized = serde_json::to_string(&expr).unwrap();
        println!("{}", serialized);
    } else {
        let result = interpreter::interpret(&expr);
        println!("Your result is: {}", result);
    }
}
