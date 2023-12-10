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
                TokenKind::Integer => self.parse_number(),
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

        let peek_precedence: Precedence = match self.peek_token().map(|t| t.try_into()) {
            Some(Ok(p)) => p,
            Some(Err(_)) => return Ok(exp),
            None => return Ok(exp),
        };

        while self
            .peek_token()
            .map(|t| t.kind != TokenKind::Eof)
            .unwrap_or(false)
            && curr_precedence < peek_precedence
        {
            self.advance();
            exp = self.parse_binary_expr(exp)?;
        }

        Ok(exp)
    }

    fn parse_number(&self) -> Expr {
        let token = self.token().unwrap();

        Expr::Interger(token.text.parse().expect(
            "failed to parse an ostensibly properly tokenized interger (should not happen)",
        ))
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

        let number = self.parse()?;
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
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
enum Precedence {
    #[default]
    None,
    Sum,
    Product,
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
    use super::lexer::Token;

    #[derive(Debug)]
    pub enum Expr {
        Interger(isize),
        BinExpr(Box<BinaryExpr>),
        UnExpr(Box<UnaryExpr>),
    }

    #[derive(Debug)]
    pub struct BinaryExpr {
        pub left: Expr,
        pub op: BinOp,
        pub right: Expr,
    }

    #[derive(Debug)]
    pub struct UnaryExpr {
        pub op: UnOp,
        pub right: Expr,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum BinOp {
        Plus,
        Minus,
        Times,
        Over,
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

    #[derive(Debug)]
    pub enum UnOp {
        Plus,
        Minus,
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
