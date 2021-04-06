use std::io::prelude::*;
use std::io;

mod token;

pub trait Lexer {
    fn lex<'a>(self, file: &'a mut dyn Read) -> Result<Vec<token::Token>, io::Error>;
}

pub struct BasicLexer {
    start: usize,
    current: usize,
    line_num: usize,
}

impl BasicLexer {
    pub fn new() -> BasicLexer {
        BasicLexer {
            start: 0,
            current: 0,
            line_num: 0,
        }
    }
}

impl Lexer for BasicLexer {
    fn lex<'a>(mut self, file: &'a mut dyn Read) -> Result<Vec<token::Token>, io::Error> {

        let mut tokens = Vec::new();

        let mut source = String::new();
        file.read_to_string(&mut source)?;

        let mut chars = source.chars().peekable();

        let is_at_end = || {
            return self.current >= source.len()
        };

        let advance = || {
            self.current += 1;
            return chars.next();
        };

        let add_token = |tok: token::TokenType| {
            tokens.push(token::Token::new(self.line_num, self.start, self.current, tok));
        };

        let scan_token = || {

        };

        Ok(tokens)
    }
}