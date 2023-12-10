#[derive(Clone, Copy, PartialEq)]
pub enum TokenKind {
    Ident,
    Integer,
    Plus,
    Eof,
    Illegal,
}

pub struct Token<'s> {
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
            read_position: 0,
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
                self.tokens.push(Token {
                    kind: TokenKind::Eof,
                    text: "",
                });
                return;
            }
        };

        let token = match c {
            '+' => Token {
                kind: TokenKind::Plus,
                text: "+",
            },

            c if c.is_ascii_digit() => {
                let start = self.position;

                while self.peek_char().unwrap_or('\0').is_ascii_digit() {
                    self.advance();
                }

                let end = self.position;

                let string = &self.input[start..=end];

                Token {
                    kind: TokenKind::Integer,
                    text: string,
                }
            }

            c if c.is_alphabetic() => {
                let start = self.position;

                while self.peek_char().unwrap_or('\0').is_alphabetic() {
                    self.advance();
                }

                let end = self.position;

                let string = &self.input[start..=end];

                let kind = match string {
                    "plus" => TokenKind::Plus,
                    _ => TokenKind::Ident,
                };

                Token { kind, text: string }
            }

            _ => Token {
                kind: TokenKind::Illegal,
                text: &self.input[self.position..self.position + 1],
            },
        };

        self.tokens.push(token);

        self.advance();
    }
}
