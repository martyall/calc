pub mod ast;
pub mod calculator;

use ast::{Expr, Opcode};

fn fold_expr(expr: &Expr) -> i32 {
    match expr {
        Expr::Number(n) => n.clone(),
        Expr::BinOp(lhs, op, rhs) => {
            let lhs = fold_expr(lhs);
            let rhs = fold_expr(rhs);
            match op {
                Opcode::Add => lhs + rhs,
                Opcode::Sub => lhs - rhs,
                Opcode::Mul => lhs * rhs,
                Opcode::Div => lhs / rhs,
                Opcode::Pow => lhs.pow(rhs as u32),
            }
        }
    }
}

#[test]
fn calculator() {
    let input = "22 * 44 + 66";
    let expr = calculator::ExprParser::new().parse(input).unwrap();
    assert_eq!(fold_expr(&expr), 1034);
    let input = "22 * (44 + 66)";
    let expr = calculator::ExprParser::new().parse(input).unwrap();
    assert_eq!(fold_expr(&expr), 2420);
    let input = "2^4 + 1";
    let expr = calculator::ExprParser::new().parse(input).unwrap();
    assert_eq!(fold_expr(&expr), 17);
    let input = "2^(4 +1 )  *  3+ (  2 + 1)^2";
    let expr = calculator::ExprParser::new().parse(input).unwrap();
    assert_eq!(fold_expr(&expr), 32 * 3 + 9);
}

#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
}
