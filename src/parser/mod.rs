use std::error::Error;
use std::fmt;

mod basic;

#[cfg(test)]
mod tests;

pub use self::basic::*;

pub trait Parser {

    type Out;

    fn parse(self) -> Self::Out;
    
}

#[derive(Debug)]
pub enum ParseError {
    TokenError {
        token: crate::lexer::token::Token,
        msg: &'static str
    },
    Other {
        msg: &'static str
    }
}

impl ParseError {
    fn from_token(token: crate::lexer::token::Token, msg: &'static str) -> ParseError {
        ParseError::TokenError {
            token,
            msg
        }
    }

    fn from_other(msg: &'static str) -> ParseError {
        ParseError::Other {
            msg
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::TokenError {
                token,
                msg
            } => match token.ty() {
                crate::lexer::token::TokenType::Eof => write!(f, "[Parse] Error: {} at end of file", msg),
                _ => write!(f, "[Parse] Error: {} at line {}, character {}", msg, token.line_num(), token.char_loc())
            },
            ParseError::Other { msg } => write!(f, "[Parse] Error: {}", msg)
        }

    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}