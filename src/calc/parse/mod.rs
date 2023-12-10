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

    pub fn parse(&mut self) -> Option<ast::Expr> {
        self.advance();

        match self.token() {
            Some(token) => match token.kind {
                TokenKind::Ident => self.parse(),
                TokenKind::Integer
                    if self.peek_token().map(|t| t.kind).unwrap_or(TokenKind::Eof)
                        == TokenKind::Plus =>
                {
                    let left = self.parse_number();
                    self.advance();
                    let right = self.parse()?;

                    Some(Expr::BinExpr(Box::new(BinaryExpr {
                        left,
                        right,
                        op: ast::Op::Plus,
                    })))
                }
                TokenKind::Integer => Some(self.parse_number()),
                TokenKind::Plus => {
                    let number = self.parse()?;

                    Some(Expr::UnExpr(Box::new(UnaryExpr {
                        op: ast::Op::Plus,
                        right: number,
                    })))
                }
                TokenKind::Eof => None,
                TokenKind::Illegal => self.parse(),
            },
            None => None,
        }
    }

    fn parse_number(&self) -> Expr {
        let token = self.token().unwrap();

        Expr::Interger(token.text.parse().expect(
            "failed to parse an ostensibly properly tokenized interger (should not happen)",
        ))
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
        pub op: Op,
        pub right: Expr,
    }

    #[derive(Debug)]
    pub struct UnaryExpr {
        pub op: Op,
        pub right: Expr,
    }

    #[derive(Debug)]
    pub enum Op {
        Plus,
    }
}
