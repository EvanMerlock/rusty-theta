#[cfg(test)]
mod tests;

pub(crate) mod token;
mod basic;

use std::error::Error;

use crate::parser::ParseError;

pub use self::basic::*;
use self::token::Token;

pub trait Lexer {

    type Out;
    type Error: Error;

    fn lex(self) -> Result<Self::Out, Self::Error>;
    fn scan_token(&mut self) -> Result<Option<token::Token>, Self::Error>;
}