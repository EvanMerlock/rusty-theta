use std::{fmt, error::Error};

use crate::bytecode::{Token, TokenType};

#[derive(Debug)]
pub enum ParseError {
    TokenError {
        token: Token,
        msg: &'static str
    },
    Other {
        msg: &'static str
    }
}

impl ParseError {
    pub fn from_token(token: Token, msg: &'static str) -> ParseError {
        ParseError::TokenError {
            token,
            msg
        }
    }

    pub fn from_other(msg: &'static str) -> ParseError {
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
                TokenType::Eof => write!(f, "[Parse] Error: {} at end of file", msg),
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
