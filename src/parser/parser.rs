use pest_derive::Parser;

use crate::ast::{Declaration, Expr, Ident, Opcode, Program, UOpcode};
use lazy_static::lazy_static;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/calculator.pest"]
pub struct CalcParser;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(add, Left) | Op::infix(sub, Left))
            .op(Op::infix(mul, Left))
            .op(Op::prefix(unary_minus))
            .op(Op::infix(pow, Right))
    };
}

fn infix_rule(lhs: Expr, pair: Pair<Rule>, rhs: Expr) -> Expr {
    match pair.as_rule() {
        Rule::add => Expr::BinOp(Box::new(lhs), Opcode::Add, Box::new(rhs)),
        Rule::sub => Expr::BinOp(Box::new(lhs), Opcode::Sub, Box::new(rhs)),
        Rule::mul => Expr::BinOp(Box::new(lhs), Opcode::Mul, Box::new(rhs)),
        Rule::pow => Expr::BinOp(Box::new(lhs), Opcode::Pow, Box::new(rhs)),
        rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
    }
}

fn primary_rule(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::integer => Expr::Number(pair.as_str().parse::<i32>().unwrap()),
        Rule::expression => parse_expr(pair.into_inner()),
        Rule::identifier => Expr::Variable(Ident::new(pair.as_str())),
        rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
    }
}

fn prefix_rule(pair: Pair<Rule>, expr: Expr) -> Expr {
    match pair.as_rule() {
        Rule::unary_minus => Expr::UnaryOp(UOpcode::Neg, Box::new(expr)),
        rule => unreachable!("Expr::parse expected prefix operation, found {:?}", rule),
    }
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(primary_rule)
        .map_infix(infix_rule)
        .map_prefix(prefix_rule)
        .parse(pairs)
}

fn parse_assignment(pairs: Pair<Rule>) -> Declaration {
    match pairs.as_rule() {
        Rule::assignment => {
            let mut pairs = pairs.into_inner();
            let name = pairs.next().expect("Expected identifier").as_str();
            let expr = parse_expr(pairs.next().expect("Expected expression").into_inner());
            Declaration::VarAssignment(Ident::new(name), expr)
        }
        rule => unreachable!("Declaration::parse expected assignment, found {:?}", rule),
    }
}

fn parse_public_var(pairs: Pair<Rule>) -> Declaration {
    match pairs.as_rule() {
        Rule::public_var => {
            let name = Ident::new(pairs.into_inner().as_str());
            Declaration::PublicVar(name)
        }
        rule => unreachable!("Declaration::parse expected public var, found {:?}", rule),
    }
}

fn parse_decls(pairs: &mut Pairs<Rule>) -> Vec<Declaration> {
    let mut declarations = Vec::new();
    while let Some(pair) = pairs.peek() {
        match pair.as_rule() {
            Rule::public_var => {
                declarations.push(parse_public_var(pair));
                pairs.next();
            }
            Rule::assignment => {
                declarations.push(parse_assignment(pair));
                pairs.next();
            }
            _ => break,
        }
    }
    declarations
}

pub fn parse(input: &str) -> Result<Program, Error<Rule>> {
    let mut pairs = CalcParser::parse(Rule::program, input)?;
    let decls_pair = pairs.next().unwrap();
    let decls = parse_decls(&mut decls_pair.into_inner());
    let expr_pair = pairs.next().unwrap();
    let expr = parse_expr(expr_pair.into_inner());
    Ok(Program::new(decls, expr).unwrap())
}

pub fn parse_single_expression(input: &str) -> Result<Expr, Error<Rule>> {
    let mut pairs = CalcParser::parse(Rule::expression, input)?;
    let pair = pairs.next().unwrap();
    Ok(parse_expr(pair.into_inner()))
}

#[cfg(test)]
mod parser_tests {

    use super::*;
    use crate::ast::Expr;

    #[test]
    fn no_parens_test() {
        let input = "22 * 44 + 66";
        let expr = parse_single_expression(input).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(
                Box::new(Expr::BinOp(
                    Box::new(Expr::Number(22)),
                    Opcode::Mul,
                    Box::new(Expr::Number(44))
                )),
                Opcode::Add,
                Box::new(Expr::Number(66))
            )
        );
    }

    #[test]
    fn parens_test() {
        let input = "22 * (44 + 66)";
        let expr = parse_single_expression(input).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(
                Box::new(Expr::Number(22)),
                Opcode::Mul,
                Box::new(Expr::BinOp(
                    Box::new(Expr::Number(44)),
                    Opcode::Add,
                    Box::new(Expr::Number(66))
                ))
            )
        );
    }

    #[test]
    fn unary_minus_test() {
        let input = "-22 * 44";
        let expr = parse_single_expression(input).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(
                Box::new(Expr::UnaryOp(UOpcode::Neg, Box::new(Expr::Number(22)))),
                Opcode::Mul,
                Box::new(Expr::Number(44))
            )
        );
    }

    #[test]
    fn program_test() {
        let input = r#"
            pub x;
            pub y;
            let a = 22 * (x - b);
            let b = 1 - y;
            a * b - 2
          "#;
        let program = parse(input).unwrap();
        assert_eq!(
            program,
            Program::new(
                vec![
                    Declaration::PublicVar(Ident::new("x")),
                    Declaration::PublicVar(Ident::new("y")),
                    Declaration::VarAssignment(
                        Ident::new("a"),
                        Expr::BinOp(
                            Box::new(Expr::Number(22)),
                            Opcode::Mul,
                            Box::new(Expr::BinOp(
                                Box::new(Expr::Variable(Ident::new("x"))),
                                Opcode::Sub,
                                Box::new(Expr::Variable(Ident::new("b")))
                            ))
                        )
                    ),
                    Declaration::VarAssignment(
                        Ident::new("b"),
                        Expr::BinOp(
                            Box::new(Expr::Number(1)),
                            Opcode::Sub,
                            Box::new(Expr::Variable(Ident::new("y")))
                        )
                    ),
                ],
                Expr::BinOp(
                    Box::new(Expr::BinOp(
                        Box::new(Expr::Variable(Ident::new("a"))),
                        Opcode::Mul,
                        Box::new(Expr::Variable(Ident::new("b")))
                    )),
                    Opcode::Sub,
                    Box::new(Expr::Number(2))
                )
            )
            .unwrap()
        );
    }
}
