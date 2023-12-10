use self::parse::{
    ast::{BinaryExpr, Expr, UnaryExpr},
    Parser,
};

mod parse;

pub fn eval(source: &str) -> Option<isize> {
    let mut parser = Parser::new(source);

    let expr = parser.parse();
    expr.map(|expr| eval_expr(&expr))
}

fn eval_expr(expr: &Expr) -> isize {
    match expr {
        Expr::Interger(i) => *i,
        Expr::BinExpr(expr) => eval_binary_expr(expr),
        Expr::UnExpr(expr) => eval_unary_expr(expr),
    }
}

fn eval_binary_expr(expr: &BinaryExpr) -> isize {
    let left: isize = eval_expr(&expr.left);

    let right = eval_expr(&expr.right);

    match expr.op {
        parse::ast::Op::Plus => left + right,
    }
}

fn eval_unary_expr(expr: &UnaryExpr) -> isize {
    let number = eval_expr(&expr.right);

    match expr.op {
        parse::ast::Op::Plus => number,
    }
}
