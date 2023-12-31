#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Ident,
    Integer,
    Float,
    Minus,
    Times,
    Over,
    Plus,
    Eof,
    Illegal,
}

#[derive(Debug)]
pub struct Token<'s> {
    pub start: usize,
    pub kind: TokenKind,
    pub text: &'s str,
}

pub struct Lexer<'s> {
    input: &'s str,
    input_bytes: &'s [u8],
    position: usize,
    read_position: usize,
    tokens: Vec<Token<'s>>,
}

impl<'s> Lexer<'s> {
    pub fn new(input: &'s str) -> Self {
        Self {
            input,
            input_bytes: input.as_bytes(),
            position: 0,
            read_position: 1,
            tokens: vec![],
        }
    }

    pub fn tokenize(mut self) -> Vec<Token<'s>> {
        loop {
            if let Some(token) = self.tokens.last() {
                if token.kind == TokenKind::Eof {
                    break;
                }
            }

            self.next_token();
        }

        return self.tokens;
    }

    fn char(&self) -> Option<char> {
        self.input_bytes
            .get(self.position)
            .map(|&byte| byte as char)
    }

    fn peek_char(&self) -> Option<char> {
        self.input_bytes
            .get(self.read_position)
            .map(|&byte| byte as char)
    }

    fn advance(&mut self) {
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.char().unwrap_or('\0').is_ascii_whitespace() {
            self.advance();
        }
    }

    fn next_token(&mut self) {
        self.skip_whitespace();

        let c = match self.char() {
            Some(c) => c,
            None => {
                self.tokens.push(self.char_token(TokenKind::Eof));
                return;
            }
        };

        let token = match c {
            '+' => self.char_token(TokenKind::Plus),

            '-' => self.char_token(TokenKind::Minus),

            '*' => self.char_token(TokenKind::Times),

            '/' => self.char_token(TokenKind::Over),

            c if c.is_ascii_digit() => {
                let start = self.position;

                let mut is_float = false;

                while self.peek_char().unwrap_or('\0').is_ascii_digit() {
                    self.advance();
                }

                if self.peek_char().unwrap_or('\0') == '.' {
                    is_float = true;
                    self.advance();
                }

                while self.peek_char().unwrap_or('\0').is_ascii_digit() {
                    self.advance();
                }

                let end = self.position;

                let string = &self.input[start..=end];

                if is_float {
                    Token {
                        start,
                        kind: TokenKind::Float,
                        text: string,
                    }
                } else {
                    Token {
                        start,
                        kind: TokenKind::Integer,
                        text: string,
                    }
                }
            }

            c if c.is_alphabetic() => {
                let start = self.position;

                while self
                    .peek_char()
                    .map(|c| c.is_alphabetic() || c == ' ')
                    .unwrap_or(false)
                {
                    self.advance();
                }

                let end = self.position;

                let string = &self.input[start..=end];

                let kind = match string.trim() {
                    "plus" => TokenKind::Plus,
                    "minus" | "negative" => TokenKind::Minus,
                    "times" | "x" | "multiplied by" => TokenKind::Times,
                    "over" | "divided by" => TokenKind::Over,
                    _ => {
                        string.split_whitespace().for_each(|ident| {
                            let kind = match ident {
                                "plus" => TokenKind::Plus,
                                "minus" | "negative" => TokenKind::Minus,
                                "times" | "x" => TokenKind::Times,
                                "over" => TokenKind::Over,
                                _ => TokenKind::Ident,
                            };

                            let token = Token {
                                start,
                                kind,
                                text: ident,
                            };
                            self.tokens.push(token);
                        });
                        self.advance();
                        return;
                    }
                };

                Token {
                    start,
                    kind,
                    text: string,
                }
            }

            _ => self.char_token(TokenKind::Illegal),
        };

        self.tokens.push(token);

        self.advance();
    }

    fn char_token(&self, kind: TokenKind) -> Token<'s> {
        return Token {
            start: self.position,
            kind,
            text: self
                .input
                .get(self.position..self.position + 1)
                .unwrap_or_default(),
        };
    }
}
