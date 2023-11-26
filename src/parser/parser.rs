use pest_derive::Parser;

use crate::ast::annotation::{from_pest_span, Span};
use crate::ast::{Binder, Declaration, Expr, Ident, Literal, Opcode, Program, UOpcode};
use anyhow::Result;
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

fn infix_rule(lhs: Expr<Span>, pair: Pair<Rule>, rhs: Expr<Span>) -> Expr<Span> {
    let op = match pair.as_rule() {
        Rule::add => Opcode::Add,
        Rule::sub => Opcode::Sub,
        Rule::mul => Opcode::Mul,
        Rule::pow => Opcode::Pow,
        rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
    };
    let ann = from_pest_span(pair.as_span());
    Expr::BinOp {
        ann,
        lhs: Box::new(lhs),
        op,
        rhs: Box::new(rhs),
    }
}

fn primary_rule(pair: Pair<Rule>) -> Expr<Span> {
    let ann = from_pest_span(pair.as_span());
    match pair.as_rule() {
        Rule::integer => Expr::Literal {
            ann,
            value: Literal::Number(pair.as_str().parse::<i32>().unwrap()),
        },
        Rule::identifier => Expr::Variable {
            ann,
            value: Ident::new(pair.as_str()),
        },
        Rule::expression => parse_expr(pair.into_inner()),
        rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
    }
}

fn prefix_rule(pair: Pair<Rule>, expr: Expr<Span>) -> Expr<Span> {
    let ann = from_pest_span(pair.as_span());
    match pair.as_rule() {
        Rule::unary_minus => Expr::UnaryOp {
            ann,
            op: UOpcode::Neg,
            expr: Box::new(expr),
        },
        rule => unreachable!("Expr::parse expected prefix operation, found {:?}", rule),
    }
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr<Span> {
    PRATT_PARSER
        .map_primary(primary_rule)
        .map_infix(infix_rule)
        .map_prefix(prefix_rule)
        .parse(pairs)
}

fn parse_assignment(pairs: Pair<Rule>) -> Declaration<Span> {
    match pairs.as_rule() {
        Rule::assignment => {
            let mut pairs = pairs.into_inner();
            let name_pair = pairs.next().expect("Expected identifier");
            let binder = Binder {
                ann: from_pest_span(name_pair.as_span()),
                var: Ident::new(name_pair.as_str()),
            };
            let expr = parse_expr(pairs.next().expect("Expected expression").into_inner());
            Declaration::VarAssignment { binder, expr }
        }
        rule => unreachable!("Declaration::parse expected assignment, found {:?}", rule),
    }
}

fn parse_public_var(pairs: Pair<Rule>) -> Declaration<Span> {
    match pairs.as_rule() {
        Rule::public_var => {
            let name_pair = pairs.into_inner().next().expect("Expected identifier");
            let name = Ident::new(name_pair.as_str());
            let binder = Binder {
                ann: from_pest_span(name_pair.as_span()),
                var: name,
            };
            Declaration::PublicVar { binder }
        }
        rule => unreachable!("Declaration::parse expected public var, found {:?}", rule),
    }
}

fn parse_decls(pairs: &mut Pairs<Rule>) -> Vec<Declaration<Span>> {
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

pub fn parse(input: &str) -> Result<Program<Span>> {
    let mut pairs = CalcParser::parse(Rule::program, input)?;
    let decls_pair = pairs.next().unwrap();
    let decls = parse_decls(&mut decls_pair.into_inner());
    let expr_pair = pairs.next().unwrap();
    let expr = parse_expr(expr_pair.into_inner());
    Program::new(decls, expr)
}

pub fn parse_single_expression(input: &str) -> Result<Expr<Span>, Error<Rule>> {
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
        let expr = parse_single_expression(input).unwrap().clear_annotations();
        assert_eq!(
            expr,
            Expr::binary_op_default(
                Expr::binary_op_default(
                    Expr::number_default(22),
                    Opcode::Mul,
                    Expr::number_default(44)
                ),
                Opcode::Add,
                Expr::number_default(66)
            )
        );
    }

    #[test]
    fn parens_test() {
        let input = "22 * (44 + 66)";
        let expr = parse_single_expression(input).unwrap().clear_annotations();
        assert_eq!(
            expr,
            Expr::binary_op_default(
                Expr::number_default(22),
                Opcode::Mul,
                Expr::binary_op_default(
                    Expr::number_default(44),
                    Opcode::Add,
                    Expr::number_default(66)
                )
            )
        );
    }

    #[test]
    fn unary_minus_test() {
        let input = "-22 * 44";
        let expr = parse_single_expression(input).unwrap().clear_annotations();
        assert_eq!(
            expr,
            Expr::binary_op_default(
                Expr::unary_op_default(UOpcode::Neg, Expr::number_default(22)),
                Opcode::Mul,
                Expr::number_default(44)
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
        let parsed_program = parse(input)
            .expect("Expected end of program")
            .clear_annotations();
        let program: Program<()> = Program::new(
            vec![
                Declaration::PublicVar {
                    binder: Binder::default(Ident::new("x")),
                },
                Declaration::PublicVar {
                    binder: Binder::default(Ident::new("y")),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("a")),
                    expr: Expr::binary_op_default(
                        Expr::number_default(22),
                        Opcode::Mul,
                        Expr::binary_op_default(
                            Expr::variable_default(Ident::new("x")),
                            Opcode::Sub,
                            Expr::variable_default(Ident::new("b")),
                        ),
                    ),
                },
                Declaration::VarAssignment {
                    binder: Binder::default(Ident::new("b")),
                    expr: Expr::binary_op_default(
                        Expr::number_default(1),
                        Opcode::Sub,
                        Expr::variable_default(Ident::new("y")),
                    ),
                },
            ],
            Expr::binary_op_default(
                Expr::binary_op_default(
                    Expr::variable_default(Ident::new("a")),
                    Opcode::Mul,
                    Expr::variable_default(Ident::new("b")),
                ),
                Opcode::Sub,
                Expr::number_default(2),
            ),
        )
        .unwrap();

        assert_eq!(parsed_program, program);
    }
}
