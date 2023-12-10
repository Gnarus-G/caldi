use self::parse::{
    ast::{BinaryExpr, Expr, UnaryExpr},
    Parser,
};

mod parse;

pub fn eval(source: &str) -> parse::Result<isize> {
    let mut parser = Parser::new(source);

    let expr = parser.parse()?;

    eprintln!("[DEBUG] ast: {expr:?}");

    Ok(eval_expr(&expr))
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
        parse::ast::BinOp::Plus => left + right,
        parse::ast::BinOp::Minus => left - right,
        parse::ast::BinOp::Times => left * right,
        parse::ast::BinOp::Over => left / right,
    }
}

fn eval_unary_expr(expr: &UnaryExpr) -> isize {
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
        ($expr:literal, $ans:literal) => {
            assert_eq!(eval($expr).unwrap(), $ans)
        };
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
        assert_evals!("3 - 2", 1);
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
        assert_evals!("9 / 2", 4);
        assert_evals!("256 * 9 / 2", 1152);
    }

    #[test]
    fn pemdas() {
        assert_evals!("9 * 2 / 3 + 6 - 4 + 2", 10)
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
