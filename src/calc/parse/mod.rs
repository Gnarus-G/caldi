use self::{
    ast::{BinOp, BinaryExpr, Expr, UnaryExpr},
    lexer::{Lexer, Token, TokenKind},
};

mod lexer;

pub struct Parser<'s> {
    tokens: Vec<Token<'s>>,
    position: usize,
    read_position: usize,
}

pub type Result<T> = std::result::Result<T, error::ErrorKind>;

impl<'s> Parser<'s> {
    pub fn new(input: &'s str) -> Self {
        Self {
            tokens: Lexer::new(input).tokenize(),
            position: 0,
            read_position: 1,
        }
    }

    fn token(&self) -> Option<&Token<'s>> {
        if let Some(t) = self.tokens.get(self.position) {
            if t.kind == TokenKind::Eof {
                return None;
            }

            return Some(t);
        }

        return None;
    }

    fn peek_token(&self) -> Option<&Token<'s>> {
        if let Some(t) = self.tokens.get(self.read_position) {
            if t.kind == TokenKind::Eof {
                return None;
            }

            return Some(t);
        }

        return None;
    }

    fn advance(&mut self) {
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn parse(&mut self) -> Result<Expr> {
        self.parse_expr(Precedence::default())
    }

    fn parse_expr(&mut self, curr_precedence: Precedence) -> Result<Expr> {
        let mut exp = match self.token() {
            Some(token) => match token.kind {
                TokenKind::Ident => {
                    self.advance(); // skipping identifiers
                    self.parse()?
                }
                TokenKind::Float => self.parse_fp_number(),
                TokenKind::Integer => self.parse_integer(),
                TokenKind::Plus => self.parse_unary_expr()?,
                TokenKind::Minus => self.parse_unary_expr()?,
                TokenKind::Times => {
                    return Err(error::ErrorKind::UnexpectedToken {
                        token: token.into(),
                    })
                }
                TokenKind::Over => {
                    return Err(error::ErrorKind::UnexpectedToken {
                        token: token.into(),
                    })
                }
                TokenKind::Eof => return Err(error::ErrorKind::UnexpectedEnd { at: token.start }),
                TokenKind::Illegal => {
                    self.advance(); // skipping any illegal characters
                    self.parse()?
                }
            },
            None => return Err(error::ErrorKind::UnexpectedEnd { at: 0 }),
        };

        loop {
            let peek_precedence: Precedence = match self.peek_token().map(|t| t.try_into()) {
                Some(Ok(p)) => p,
                Some(Err(_)) => return Ok(exp),
                None => return Ok(exp),
            };

            if self
                .peek_token()
                .map(|t| t.kind != TokenKind::Eof)
                .unwrap_or(false)
                && curr_precedence < peek_precedence
            {
                self.advance();
                exp = self.parse_binary_expr(exp)?;
            } else {
                break;
            }
        }

        Ok(exp)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr> {
        let op: ast::UnOp = match self.token().map(|t| t.try_into()) {
            Some(Ok(op)) => op,
            Some(Err(token)) => {
                return Err(error::ErrorKind::UnexpectedToken {
                    token: token.into(),
                })
            }
            None => return Err(error::ErrorKind::UnexpectedEnd { at: 0 }),
        };

        self.advance();

        let number = self.parse_expr(Precedence::Prefix)?;
        Ok(Expr::UnExpr(Box::new(UnaryExpr { op, right: number })))
    }

    fn parse_binary_expr(&mut self, left: Expr) -> Result<Expr> {
        let op: BinOp = match self.token().map(|t| t.try_into()) {
            Some(Ok(op)) => op,
            Some(Err(token)) => {
                return Err(error::ErrorKind::UnexpectedToken {
                    token: token.into(),
                })
            }
            None => return Err(error::ErrorKind::UnexpectedEnd { at: 1 }),
        };

        self.advance();

        Ok(Expr::BinExpr(Box::new(BinaryExpr {
            left,
            op,
            right: self.parse_expr(op.into())?,
        })))
    }

    fn parse_fp_number(&self) -> Expr {
        let token = self.token().unwrap();

        Expr::Float(
            token.text.parse().expect(
                "failed to parse an ostensibly properly tokenized integer (should not happen)",
            ),
        )
    }

    fn parse_integer(&self) -> Expr {
        let token = self.token().unwrap();

        Expr::Integer(
            token.text.parse().expect(
                "failed to parse an ostensibly properly tokenized floating point number (should not happen)",
            ),
        )
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
enum Precedence {
    #[default]
    None,
    Sum,
    Product,
    Prefix,
}

impl From<BinOp> for Precedence {
    fn from(value: BinOp) -> Self {
        match value {
            BinOp::Plus => Self::Sum,
            BinOp::Minus => Self::Sum,
            BinOp::Times => Self::Product,
            BinOp::Over => Self::Product,
        }
    }
}

impl<'t, 's> TryFrom<&'t Token<'s>> for Precedence {
    type Error = &'t Token<'s>;

    fn try_from(value: &'t Token<'s>) -> std::prelude::v1::Result<Self, Self::Error> {
        BinOp::try_from(value).map(Precedence::from)
    }
}

pub mod ast {
    use std::fmt::Debug;

    use super::lexer::Token;

    pub enum Expr {
        Integer(isize),
        Float(f64),
        BinExpr(Box<BinaryExpr>),
        UnExpr(Box<UnaryExpr>),
    }

    impl Debug for Expr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Expr::Integer(number) => write!(f, "{number}"),
                Expr::Float(number) => write!(f, "{number}"),
                Expr::BinExpr(expr) => write!(f, "{expr:?}"),
                Expr::UnExpr(expr) => write!(f, "{expr:?}"),
            }
        }
    }

    pub struct BinaryExpr {
        pub left: Expr,
        pub op: BinOp,
        pub right: Expr,
    }

    impl Debug for BinaryExpr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({:?} {:?} {:?})", self.left, self.op, self.right)
        }
    }

    pub struct UnaryExpr {
        pub op: UnOp,
        pub right: Expr,
    }

    impl Debug for UnaryExpr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({:?}{:?})", self.op, self.right)
        }
    }

    #[derive(Clone, Copy)]
    pub enum BinOp {
        Plus,
        Minus,
        Times,
        Over,
    }

    impl Debug for BinOp {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                BinOp::Plus => write!(f, "+"),
                BinOp::Minus => write!(f, "-"),
                BinOp::Times => write!(f, "*"),
                BinOp::Over => write!(f, "/"),
            }
        }
    }

    impl<'t, 's> TryFrom<&'t Token<'s>> for BinOp {
        type Error = &'t Token<'s>;

        fn try_from(value: &'t Token<'s>) -> Result<Self, Self::Error> {
            let r = match value.kind {
                super::lexer::TokenKind::Minus => BinOp::Minus,
                super::lexer::TokenKind::Times => BinOp::Times,
                super::lexer::TokenKind::Over => BinOp::Over,
                super::lexer::TokenKind::Plus => BinOp::Plus,
                _ => return Err(value),
            };

            Ok(r)
        }
    }

    pub enum UnOp {
        Plus,
        Minus,
    }

    impl Debug for UnOp {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                UnOp::Plus => write!(f, "+"),
                UnOp::Minus => write!(f, "-"),
            }
        }
    }

    impl<'t, 's> TryFrom<&'t Token<'s>> for UnOp {
        type Error = &'t Token<'s>;

        fn try_from(value: &'t Token<'s>) -> Result<Self, Self::Error> {
            let r = match value.kind {
                super::lexer::TokenKind::Minus => UnOp::Minus,
                super::lexer::TokenKind::Plus => UnOp::Plus,
                _ => return Err(value),
            };

            Ok(r)
        }
    }
}

pub mod error {
    use std::fmt::Display;

    use super::lexer::{Token, TokenKind};

    #[derive(Debug)]
    pub struct TokenKindAt {
        pub position: usize,
        pub kind: TokenKind,
    }

    impl From<&Token<'_>> for TokenKindAt {
        fn from(value: &Token<'_>) -> Self {
            Self {
                position: value.start,
                kind: value.kind,
            }
        }
    }

    #[derive(Debug)]
    pub enum ErrorKind {
        UnexpectedToken { token: TokenKindAt },
        UnexpectedEnd { at: usize },
    }

    impl std::error::Error for ErrorKind {}

    impl Display for ErrorKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ErrorKind::UnexpectedToken { token } => write!(
                    f,
                    "unexpected token {:?} at position {}",
                    token.kind, token.position
                ),
                ErrorKind::UnexpectedEnd { at } => write!(
                    f,
                    "unexpected end of expression encountered at position {}",
                    at
                ),
            }
        }
    }
}
