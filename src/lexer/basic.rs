use std::iter::Peekable;

use super::*;

pub struct BasicLexer<'a> {
    chars: Peekable<&'a mut dyn Iterator<Item = char>>,

    start: usize,
    current: usize,
    line_num: usize,
    comment_level: usize,
}

impl<'a> BasicLexer<'a> {
    pub fn new(chars: &'a mut dyn Iterator<Item = char>) -> BasicLexer<'a> {
        BasicLexer {
            chars: chars.peekable(),

            start: 0,
            current: 0,
            line_num: 1,
            comment_level: 0,
        }
    }

    fn string(&mut self) -> Option<token::Token> {

        let mut buffer = String::new();

        while self.peek().map(|opt| opt != '"').unwrap_or(false) && !self.is_at_end() {
            if self.peek().map(|opt| opt == '\n').unwrap_or(false) {
                self.line_num += 1;
            }

            if let Some(c) = self.advance() {
                buffer.push(c)
            }
        }

        if self.is_at_end() {
            panic!("Unterminated string");
        }

        self.advance();

        Some(self.generate_token(token::TokenType::Str(buffer)))
    }

    fn number(&mut self, c: char) -> Option<token::Token> {

        let mut buffer = String::new();
        buffer.push(c);
        let mut is_float = false;

        while self.peek().map(|opt| opt.is_ascii_digit()).unwrap_or(false) {
            if let Some(c) = self.advance() {
                buffer.push(c)
            }
        }

        if self.peek().map(|opt| opt == '.').unwrap_or(false) {
            if let Some(c) = self.advance() {
                buffer.push(c)
            }
            is_float = true;

            while self.peek().map(|opt| opt.is_ascii_digit()).unwrap_or(false) {
                if let Some(c) = self.advance() {
                    buffer.push(c)
                }
            }
        }

        Some(match is_float {
            true => self.generate_token(token::TokenType::Float(buffer.parse().unwrap())),
            false => self.generate_token(token::TokenType::Integer(buffer.parse().unwrap())),
        })
    }

    fn identifier(&mut self, c: char) -> Option<token::Token> {

        let mut buffer = String::new();
        buffer.push(c);

        while self.peek().map(|opt| opt.is_alphabetic()).unwrap_or(false) {
            if let Some(c) = self.advance() {
                buffer.push(c)
            }
        }

        let tok_typ: token::TokenType = token::IDENTIFIERS.get::<str>(&buffer).cloned().unwrap_or(token::TokenType::Identifier(buffer));

        Some(self.generate_token(tok_typ))
    }

    fn is_at_end(&mut self) -> bool {
        return self.chars.peek().is_none()
    }

    fn peek(&mut self) -> Option<char> {
        return self.chars.peek().copied();
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c.is_some() {
            self.current += 1;
        }
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        match self.peek() {
            None => false,
            Some(c) if c == expected => {
                self.advance();
                true
            },
            Some(_) => false,
        }
    }

    fn generate_token(&self, tok: token::TokenType) -> token::Token {
        token::Token::new(self.line_num, self.start, self.current, tok)
    }

    fn inc_comment_level(&mut self) {
        self.comment_level += 1;
    }

    fn dec_comment_level(&mut self) {
        if self.comment_level == 0 {
            panic!("Cannot reduce comment level below 0");
        } else {
            self.comment_level -= 1;
        }
    }

}

impl<'a> Lexer for BasicLexer<'a> {

    type Out = Vec<token::Token>;

    fn scan_token(&mut self) -> Option<token::Token> {
        match self.advance() {
            None => { 
                Some(self.generate_token(token::TokenType::Eof))
            },

            Some(' ') | Some('\r') | Some('\t') => None,
            Some('\n') => {
                self.line_num += 1; 
                None
            },

            Some('*') if self.comment_level > 0 => {
                if self.match_char('/') {
                    self.dec_comment_level();
                }
                None
            },
            Some(_) if self.comment_level > 0 => None,

            Some('(') => Some(self.generate_token(token::TokenType::LeftParen)),
            Some(')') => Some(self.generate_token(token::TokenType::RightParen)),
            Some('[') => Some(self.generate_token(token::TokenType::LeftBrace)),
            Some(']') => Some(self.generate_token(token::TokenType::RightBrace)),
            Some('{') => Some(self.generate_token(token::TokenType::LeftBrace)),
            Some('}') => Some(self.generate_token(token::TokenType::RightBrace)),
            Some(',') => Some(self.generate_token(token::TokenType::Comma)),
            Some('.') => Some(self.generate_token(token::TokenType::Dot)),
            Some('+') => Some(self.generate_token(token::TokenType::Plus)),
            Some(';') => Some(self.generate_token(token::TokenType::Semicolon)),
            Some(':') => Some(self.generate_token(token::TokenType::Colon)),
            Some('*') => Some(self.generate_token(token::TokenType::Star)),

            Some('-') => {
                if self.match_char('>') {
                    Some(self.generate_token(token::TokenType::Arrow))
                } else {
                    Some(self.generate_token(token::TokenType::Minus)) 
                }
            }

            Some('!') => {
                if self.match_char('=') {
                    Some(self.generate_token(token::TokenType::BangEqual))
                } else {
                    Some(self.generate_token(token::TokenType::Bang))
                }
            }

            Some('=') => {
                if self.match_char('=') {
                    Some(self.generate_token(token::TokenType::EqualEqual))
                } else {
                    Some(self.generate_token(token::TokenType::Equal))
                }
            }

            Some('<') => {
                if self.match_char('=') {
                    Some(self.generate_token(token::TokenType::LessEqual))
                } else {
                    Some(self.generate_token(token::TokenType::Less))
                }
            }

            Some('>') => {
                if self.match_char('=') {
                    Some(self.generate_token(token::TokenType::GreaterEqual))
                } else {
                    Some(self.generate_token(token::TokenType::Greater))
                }
            }

            Some('/') => {
                if self.match_char('/') {
                    while self.peek() != Some('\n') && self.peek().is_some() {
                        let _ = self.advance();
                    }
                    None
                } else if self.match_char('*') {
                    self.inc_comment_level();
                    None
                } else {
                    Some(self.generate_token(token::TokenType::Slash))
                }
            }

            Some('"') => self.string(),

            Some(c) if c.is_ascii_digit() => {
                self.number(c)
            },

            Some(c) if c.is_alphabetic() => {
                self.identifier(c)
            }

            _ => panic!("Bad input, remove later!"),
        }
    }

    fn lex(mut self) -> Vec<token::Token> {

        let mut tokens = Vec::new();

        while !self.is_at_end() {
            let tok = self.scan_token();
            if let Some(t) = tok {
                tokens.push(t)
            }
            self.start = self.current;
        }

        tokens.push(self.generate_token(token::TokenType::Eof));

        tokens
    }
}