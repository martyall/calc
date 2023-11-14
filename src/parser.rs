use pest_derive::Parser;

use crate::ast::{Expr, Opcode, UOpcode};
use lazy_static::lazy_static;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest::Parser;

#[derive(Parser)]
#[grammar = "calculator.pest"]
struct CalcParser;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(add, Left) | Op::infix(sub, Left))
            .op(Op::infix(mul, Left) | Op::infix(div, Left))
            .op(Op::prefix(unary_minus))
            .op(Op::infix(pow, Right))
    };
}

fn infix_rule(lhs: Expr, pair: Pair<Rule>, rhs: Expr) -> Expr {
    match pair.as_rule() {
        Rule::add => Expr::BinOp(Box::new(lhs), Opcode::Add, Box::new(rhs)),
        Rule::sub => Expr::BinOp(Box::new(lhs), Opcode::Sub, Box::new(rhs)),
        Rule::mul => Expr::BinOp(Box::new(lhs), Opcode::Mul, Box::new(rhs)),
        Rule::div => Expr::BinOp(Box::new(lhs), Opcode::Div, Box::new(rhs)),
        Rule::pow => Expr::BinOp(Box::new(lhs), Opcode::Pow, Box::new(rhs)),
        rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
    }
}

fn primary_rule(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::integer => Expr::Number(pair.as_str().parse::<i32>().unwrap()),
        Rule::expr => parse_expr(pair.into_inner()),
        rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
    }
}

fn prefix_rule(pair: Pair<Rule>, expr: Expr) -> Expr {
    match pair.as_rule() {
        Rule::unary_minus => Expr::UnaryOp(UOpcode::Neg, Box::new(expr)),
        rule => unreachable!("Expr::parse expected prefix operation, found {:?}", rule),
    }
}

fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(primary_rule)
        .map_infix(infix_rule)
        .map_prefix(prefix_rule)
        .parse(pairs)
}

pub fn parse(input: &str) -> Result<Expr, Error<Rule>> {
    let mut pairs = CalcParser::parse(Rule::equation, input)?;
    Ok(parse_expr(pairs.next().unwrap().into_inner()))
}
