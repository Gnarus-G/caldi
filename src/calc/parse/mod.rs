use self::{
    ast::{BinaryExpr, Expr, UnaryExpr},
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
            read_position: 0,
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
        self.advance();

        match self.token() {
            Some(token) => match token.kind {
                TokenKind::Ident => self.parse(),
                TokenKind::Integer
                    if self.peek_token().map(|t| t.kind).unwrap_or(TokenKind::Eof)
                        == TokenKind::Plus =>
                {
                    self.parse_binary_expr(ast::BinOp::Plus)
                }
                TokenKind::Integer
                    if self.peek_token().map(|t| t.kind).unwrap_or(TokenKind::Eof)
                        == TokenKind::Minus =>
                {
                    self.parse_binary_expr(ast::BinOp::Minus)
                }
                TokenKind::Integer
                    if self.peek_token().map(|t| t.kind).unwrap_or(TokenKind::Eof)
                        == TokenKind::Times =>
                {
                    self.parse_binary_expr(ast::BinOp::Times)
                }
                TokenKind::Integer
                    if self.peek_token().map(|t| t.kind).unwrap_or(TokenKind::Eof)
                        == TokenKind::Over =>
                {
                    self.parse_binary_expr(ast::BinOp::Over)
                }
                TokenKind::Integer => Ok(self.parse_number()),
                TokenKind::Plus => self.parse_unary_expr(ast::UnOp::Plus),
                TokenKind::Minus => self.parse_unary_expr(ast::UnOp::Minus),
                TokenKind::Times => Err(error::ErrorKind::UnexpectedToken {
                    token: token.into(),
                }),
                TokenKind::Over => Err(error::ErrorKind::UnexpectedToken {
                    token: token.into(),
                }),
                TokenKind::Eof => Err(error::ErrorKind::UnexpectedEnd { at: 0 }),
                TokenKind::Illegal => self.parse(),
            },
            None => Err(error::ErrorKind::UnexpectedEnd { at: 0 }),
        }
    }

    fn parse_number(&self) -> Expr {
        let token = self.token().unwrap();

        Expr::Interger(token.text.parse().expect(
            "failed to parse an ostensibly properly tokenized interger (should not happen)",
        ))
    }

    fn parse_unary_expr(&mut self, op: ast::UnOp) -> Result<Expr> {
        let number = self.parse()?;
        Ok(Expr::UnExpr(Box::new(UnaryExpr { op, right: number })))
    }

    fn parse_binary_expr(&mut self, op: ast::BinOp) -> Result<Expr> {
        let left = self.parse_number();
        self.advance();
        let right = self.parse()?;

        Ok(Expr::BinExpr(Box::new(BinaryExpr { left, right, op })))
    }
}

pub mod ast {
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

    #[derive(Debug)]
    pub enum BinOp {
        Plus,
        Minus,
        Times,
        Over,
    }

    #[derive(Debug)]
    pub enum UnOp {
        Plus,
        Minus,
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
                position: 0,
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
