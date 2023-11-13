pub mod ast;
pub mod calculator;
pub mod interpreter;

use clap::Parser;
use std::fs::File;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,
}

fn main() {
    let args = Args::parse();
    let mut file = File::open(args.input_file).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let expr = calculator::ExprParser::new().parse(&contents).unwrap();
    let result = interpreter::interpret(&expr);
    println!("Your result is: {}", result);
}
