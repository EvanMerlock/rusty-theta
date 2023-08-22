use std::{iter::Peekable, error::Error, fmt::Display, collections::HashMap};

use theta_types::bytecode::{Token, TokenType, IDENTIFIERS};

use super::*;

#[derive(Debug)]
pub enum LexerError {
    UnexpectedEof,
    UnexpectedInput(char),
    UnterminatedString(usize, usize),
    ExtraCommentTermination,
}

impl Error for LexerError {}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::UnexpectedEof => write!(f, "Unexpected EOF encountered"),
            LexerError::UnexpectedInput(c) => write!(f, "Unexpected input {}", c),
            LexerError::UnterminatedString(line_num, _) => write!(f, "Unterminated string beginning on line {}", line_num),
            LexerError::ExtraCommentTermination => write!(f, "An additional comment termination was found"),
        }
    }
}

pub struct BasicLexer<'a> {
    chars: Peekable<&'a mut dyn Iterator<Item = char>>,

    start: usize,
    current: usize,
    line_num: usize,
    comment_level: usize,
    line_mapping: Vec<usize>,
}

impl<'a> BasicLexer<'a> {
    pub fn new(chars: &'a mut dyn Iterator<Item = char>) -> BasicLexer<'a> {
        BasicLexer {
            chars: chars.peekable(),

            start: 0,
            current: 0,
            line_num: 1,
            comment_level: 0,
            line_mapping: Vec::new(),
        }
    }

    fn inc_line_number(&mut self) {
        self.line_num += 1;
        self.line_mapping.push(self.current);
    }

    fn string(&mut self) -> Result<Token, LexerError> {

        let mut buffer = String::new();
        let location = (self.line_num, self.current);

        while self.peek().map(|opt| opt != '"').unwrap_or(false) && !self.is_at_end() {
            if self.peek().map(|opt| opt == '\n').unwrap_or(false) {
                self.inc_line_number();
            }

            if let Some(c) = self.advance() {
                buffer.push(c)
            }
        }

        if self.is_at_end() {
            return Err(LexerError::UnterminatedString(location.0, location.1))
        }

        self.advance();

        Ok(self.generate_token(TokenType::Str(buffer)))
    }

    fn number(&mut self, c: char) -> Option<Token> {

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
            true => self.generate_token(TokenType::Float(buffer.parse().unwrap())),
            false => self.generate_token(TokenType::Integer(buffer.parse().unwrap())),
        })
    }

    fn identifier(&mut self, c: char) -> Option<Token> {

        let mut buffer = String::new();
        buffer.push(c);

        while self.peek().map(|opt| opt.is_alphabetic()).unwrap_or(false) {
            if let Some(c) = self.advance() {
                buffer.push(c)
            }
        }

        let tok_typ: TokenType = IDENTIFIERS.get::<str>(&buffer).cloned().unwrap_or(TokenType::Identifier(buffer));

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

    fn generate_token(&self, tok: TokenType) -> Token {
        Token::new(self.start, self.current, tok)
    }

    fn inc_comment_level(&mut self) {
        self.comment_level += 1;
    }

    fn dec_comment_level(&mut self) -> Result<(), LexerError> {
        if self.comment_level == 0 {
            return Err(LexerError::ExtraCommentTermination);
        } else {
            self.comment_level -= 1;
        }
        Ok(())
    }

}

impl<'a> Lexer for BasicLexer<'a> {

    type Out = Vec<Token>;
    type Error = LexerError;

    fn scan_token(&mut self) -> Result<Option<Token>, LexerError> {
        match self.advance() {
            None => { 
                Ok(Some(self.generate_token(TokenType::Eof)))
            },

            Some(' ') | Some('\r') | Some('\t') => Ok(None),
            Some('\n') => {
                self.inc_line_number();
                Ok(None)
            },

            Some('*') if self.comment_level > 0 => {
                if self.match_char('/') {
                    self.dec_comment_level()?;
                }
                Ok(None)
            },
            Some(_) if self.comment_level > 0 => Ok(None),

            Some('(') => Ok(Some(self.generate_token(TokenType::LeftParen))),
            Some(')') => Ok(Some(self.generate_token(TokenType::RightParen))),
            Some('[') => Ok(Some(self.generate_token(TokenType::LeftBrace))),
            Some(']') => Ok(Some(self.generate_token(TokenType::RightBrace))),
            Some('{') => Ok(Some(self.generate_token(TokenType::LeftBrace))),
            Some('}') => Ok(Some(self.generate_token(TokenType::RightBrace))),
            Some(',') => Ok(Some(self.generate_token(TokenType::Comma))),
            Some('.') => Ok(Some(self.generate_token(TokenType::Dot))),
            Some('+') => Ok(Some(self.generate_token(TokenType::Plus))),
            Some(';') => Ok(Some(self.generate_token(TokenType::Semicolon))),
            Some(':') => Ok(Some(self.generate_token(TokenType::Colon))),
            Some('*') => Ok(Some(self.generate_token(TokenType::Star))),

            Some('-') => {
                if self.match_char('>') {
                    Ok(Some(self.generate_token(TokenType::Arrow)))
                } else {
                    Ok(Some(self.generate_token(TokenType::Minus)))
                }
            }

            Some('!') => {
                if self.match_char('=') {
                    Ok(Some(self.generate_token(TokenType::BangEqual)))
                } else {
                    Ok(Some(self.generate_token(TokenType::Bang)))
                }
            }

            Some('=') => {
                if self.match_char('=') {
                    Ok(Some(self.generate_token(TokenType::EqualEqual)))
                } else {
                    Ok(Some(self.generate_token(TokenType::Equal)))
                }
            }

            Some('<') => {
                if self.match_char('=') {
                    Ok(Some(self.generate_token(TokenType::LessEqual)))
                } else {
                    Ok(Some(self.generate_token(TokenType::Less)))
                }
            }

            Some('>') => {
                if self.match_char('=') {
                    Ok(Some(self.generate_token(TokenType::GreaterEqual)))
                } else {
                    Ok(Some(self.generate_token(TokenType::Greater)))
                }
            }

            Some('/') => {
                if self.match_char('/') {
                    while self.peek() != Some('\n') && self.peek().is_some() {
                        let _ = self.advance();
                    }
                    Ok(None)
                } else if self.match_char('*') {
                    self.inc_comment_level();
                    Ok(None)
                } else {
                    Ok(Some(self.generate_token(TokenType::Slash)))
                }
            }

            Some('"') => self.string().map(Some),

            Some(c) if c.is_ascii_digit() => {
                Ok(self.number(c))
            },

            Some(c) if c.is_alphabetic() => {
                Ok(self.identifier(c))
            }

            Some(c) => Err(LexerError::UnexpectedInput(c)),
        }
    }

    fn lex(mut self) -> Result<LexerResult<Vec<Token>>, LexerError> {

        self.line_mapping = Vec::new();
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            let tok = self.scan_token()?;
            if let Some(t) = tok {
                tokens.push(t)
            }
            self.start = self.current;
        }

        tokens.push(self.generate_token(TokenType::Eof));

        Ok(LexerResult::new(tokens, self.line_mapping))
    }
}