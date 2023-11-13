use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{satisfy, space0},
    combinator::{map, recognize},
    multi::many1,
    sequence::{delimited, preceded},
    IResult, Parser,
};

use crate::ast::{
    Op,
    AST::{self, BinOp, Literal, Negate},
};

fn literal(input: &str) -> IResult<&str, AST> {
    recognize(preceded(space0, many1(satisfy(|c| c.is_ascii_digit()))))(input).map(
        |(next_input, digits)| {
            (
                next_input,
                Literal(digits.parse().expect("digit string to parse as u32")),
            )
        },
    )
}

#[cfg(test)]
mod literal_tests {
    use super::*;

    #[test]
    fn test_literal() {
        assert_eq!(literal("2"), Ok(("", Literal(2))));
    }

    #[test]
    fn test_expr() {
        assert_eq!(literal("8734 + 12"), Ok((" + 12", Literal(8734))));
    }
}

pub fn negate<F>(mut ast_parser: F) -> impl FnMut(&str) -> IResult<&str, AST>
where
    F: FnMut(&str) -> IResult<&str, AST>,
{
    move |input| {
        let (input, _) = tag("-")(input)?;
        let (input, o) = ast_parser.parse(input)?;
        Ok((input, Negate(Box::new(o))))
    }
}

#[cfg(test)]
mod negate_tests {
    use super::*;

    #[test]
    fn test_negate() {
        assert_eq!(
            negate(literal)("-2123"),
            Ok(("", Negate(Box::new(Literal(2123)))))
        );
    }

    #[test]
    fn test_expr() {
        assert_eq!(
            negate(literal)("-12 + 3"),
            Ok((" + 3", Negate(Box::new(Literal(12)))))
        );
    }
}

fn parens<F>(mut ast_parser: F) -> impl FnMut(&str) -> IResult<&str, AST>
where
    F: FnMut(&str) -> IResult<&str, AST>,
{
    move |input| {
        let (input, _) = tag("(")(input)?;
        let (input, o) = ast_parser.parse(input)?;
        let (input, _) = tag(")")(input)?;
        Ok((input, o))
    }
}

#[cfg(test)]
mod parens_tests {
    use super::*;

    #[test]
    fn test_parens() {
        assert_eq!(parens(literal)("(123)"), Ok(("", Literal(123))));
    }

    #[test]
    fn test_expr() {
        assert_eq!(
            parens(negate(literal))("(-456)"),
            Ok(("", Negate(Box::new(Literal(456)))))
        );
    }
}

fn op(input: &str) -> IResult<&str, Op> {
    delimited(
        space0,
        alt((map(tag("+"), |_| Op::Add), map(tag("-"), |_| Op::Sub))),
        space0,
    )(input)
}

#[cfg(test)]
mod op_tests {
    use super::*;

    #[test]
    fn test_op() {
        assert_eq!(op("+"), Ok(("", Op::Add)));
        assert_eq!(op("-"), Ok(("", Op::Sub)));
    }
}

fn binop<F>(mut ast_parser: F) -> impl FnMut(&str) -> IResult<&str, AST>
where
    F: FnMut(&str) -> IResult<&str, AST>,
{
    move |input| {
        let (input, lhs) = ast_parser(input)?;
        let (input, op) = op(input)?;
        let (input, rhs) = ast_parser(input)?;
        Ok((input, BinOp(op, Box::new(lhs), Box::new(rhs))))
    }
}

#[cfg(test)]
mod binop_tests {
    use super::*;

    #[test]
    fn test_binop() {
        assert_eq!(
            binop(literal)("1 + 2"),
            Ok((
                "",
                BinOp(Op::Add, Box::new(Literal(1)), Box::new(Literal(2)))
            ))
        );
    }

    #[test]
    fn test_expr() {
        assert_eq!(
            parens(binop(negate(literal)))("(-1 + -2)"),
            Ok((
                "",
                BinOp(
                    Op::Add,
                    Box::new(Negate(Box::new(Literal(1)))),
                    Box::new(Negate(Box::new(Literal(2))))
                )
            ))
        );
    }
}

fn simple_ast(input: &str) -> IResult<&str, AST> {
    alt((parens(ast), negate(simple_ast), literal))(input)
}

pub fn ast(input: &str) -> IResult<&str, AST> {
    alt((binop(simple_ast), parens(ast), negate(ast), literal))(input)
}

#[cfg(test)]
mod ast_tests {
    use super::*;

    #[test]
    fn test_ast() {
        assert_eq!(
            ast("1 + 2"),
            Ok((
                "",
                BinOp(Op::Add, Box::new(Literal(1)), Box::new(Literal(2)))
            ))
        );
    }
    #[test]
    fn test_expr() {
        assert_eq!(
            ast("(-1 + -2)"),
            Ok((
                "",
                BinOp(
                    Op::Add,
                    Box::new(Negate(Box::new(Literal(1)))),
                    Box::new(Negate(Box::new(Literal(2))))
                )
            ))
        );
    }

    #[test]
    fn test_expr2() {
        assert_eq!(
            ast("1 + ((-1 + -2) - 3)"),
            Ok((
                "",
                BinOp(
                    Op::Add,
                    Box::new(Literal(1)),
                    Box::new(BinOp(
                        Op::Sub,
                        Box::new(BinOp(
                            Op::Add,
                            Box::new(Negate(Box::new(Literal(1)))),
                            Box::new(Negate(Box::new(Literal(2))))
                        )),
                        Box::new(Literal(3))
                    ))
                )
            ))
        );
    }
}
