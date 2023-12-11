use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
};

use self::parse::{
    ast::{BinaryExpr, Expr, UnaryExpr},
    Parser,
};

mod parse;

#[derive(Debug, PartialEq)]
enum Value {
    Integer(isize),
    Float(f64),
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(l), Value::Integer(r)) => (l + r).into(),
            (Value::Integer(l), Value::Float(r)) => (l as f64 + r).into(),
            (Value::Float(l), Value::Integer(r)) => (l + r as f64).into(),
            (Value::Float(l), Value::Float(r)) => (l + r).into(),
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(l), Value::Integer(r)) => (l - r).into(),
            (Value::Integer(l), Value::Float(r)) => (l as f64 - r).into(),
            (Value::Float(l), Value::Integer(r)) => (l - r as f64).into(),
            (Value::Float(l), Value::Float(r)) => (l - r).into(),
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(l), Value::Integer(r)) => (l * r).into(),
            (Value::Integer(l), Value::Float(r)) => (l as f64 * r).into(),
            (Value::Float(l), Value::Integer(r)) => (l * r as f64).into(),
            (Value::Float(l), Value::Float(r)) => (l * r).into(),
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(l), Value::Integer(r)) => (l as f64 / r as f64).into(),
            (Value::Integer(l), Value::Float(r)) => (l as f64 / r).into(),
            (Value::Float(l), Value::Integer(r)) => (l / r as f64).into(),
            (Value::Float(l), Value::Float(r)) => (l / r).into(),
        }
    }
}

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            Value::Integer(number) => (-number).into(),
            Value::Float(number) => (-number).into(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(number) => write!(f, "{number}"),
            Value::Float(number) => write!(f, "{number}"),
        }
    }
}

impl From<isize> for Value {
    fn from(value: isize) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

pub fn eval(source: &str) -> parse::Result<String> {
    let mut parser = Parser::new(source);

    let expr = parser.parse()?;

    eprintln!("[DEBUG] ast: {expr:?}");

    Ok(eval_expr(&expr).to_string())
}

fn eval_expr(expr: &Expr) -> Value {
    match expr {
        Expr::Integer(i) => (*i).into(),
        Expr::BinExpr(expr) => eval_binary_expr(expr),
        Expr::UnExpr(expr) => eval_unary_expr(expr),
        Expr::Float(f) => (*f).into(),
    }
}

fn eval_binary_expr(expr: &BinaryExpr) -> Value {
    let left: Value = eval_expr(&expr.left);

    let right = eval_expr(&expr.right);

    match expr.op {
        parse::ast::BinOp::Plus => left + right,
        parse::ast::BinOp::Minus => left - right,
        parse::ast::BinOp::Times => left * right,
        parse::ast::BinOp::Over => left / right,
    }
}

fn eval_unary_expr(expr: &UnaryExpr) -> Value {
    let number = eval_expr(&expr.right);

    match expr.op {
        parse::ast::UnOp::Plus => number,
        parse::ast::UnOp::Minus => -number,
    }
}

pub fn render_error(error: parse::error::ErrorKind, source: &str) -> String {
    let mut output = String::new();

    let location = match &error {
        parse::error::ErrorKind::UnexpectedToken { token } => token.position,
        parse::error::ErrorKind::UnexpectedEnd { at } => *at,
    };

    output.push_str(source);

    output.push_str(&" ".repeat(location));
    output.push('\n');
    output.push_str(&" ".repeat(location));

    output.push('↳');
    output.push(' ');

    output.push_str(&error.to_string());

    output
}

#[cfg(test)]
mod tests {
    use insta::assert_display_snapshot;

    use crate::calc::{eval, render_error};

    macro_rules! assert_evals {
        ($expr:literal, $ans:expr) => {
            assert_eq!(eval($expr).unwrap(), ($ans).to_string())
        };
    }

    #[test]
    fn ops_in_letters() {
        assert_evals!("minus 3", -3);
        assert_evals!("negative 3", -3);
        assert_evals!("2 plus 2 times 3 minus 1 over 2 minus negative 5", 12.5);
    }

    #[test]
    fn unaries() {
        assert_evals!("-2", -2);
        assert_evals!("--2", 2);
        assert_evals!("+2", 2);
        assert_evals!("-+-2", 2);
        assert_evals!("--+-2", -2);
    }

    #[test]
    fn sums() {
        assert_evals!("3 + 2", 5);
        assert_evals!("3 + 3.", 6);
        assert_evals!("3 - 2 / 6", 2.6666666666666665);
        assert_evals!("75 + 100", 175);
        assert_evals!("75 - 100", -25);
        assert_evals!("75 + 1000 - 100", 975);
        assert_evals!("1000 + 75 - 100", 975);
    }

    #[test]
    fn products() {
        assert_evals!("3 * 2", 6);
        assert_evals!("89 * 34", 3026);
        assert_evals!("89 * 34 * 23 * 199", 13850002);
        assert_evals!("9 / 2", 4.5);
        assert_evals!("256 * 9 / 2", 1152);
    }

    #[test]
    fn pemdas() {
        assert_evals!("9 * 2 / 3 + 6 - 4 + 2", 10);
        assert_evals!("2 + 2 / 2 - - 2", 5);
        assert_evals!("2 + 2 * 3 - 1 / 2 - - 5", 12.5);
        assert_evals!("3 - 1 / 2 - 5", -2.5);
        assert_evals!("-3 - 1 / 2 - 5", -8.5)
    }

    macro_rules! assert_error {
        ($source:literal) => {
            let source = $source;
            let err = eval(source).unwrap_err();
            let prettied = render_error(err, source);
            insta::with_settings!({ description => source }, {
                assert_display_snapshot!(prettied)
            })
        };
    }

    #[test]
    fn errors() {
        assert_error!("* 2");
        assert_error!("/ 2");
        assert_error!("2 + * 2");
    }
}
