#[cfg(test)]
mod tests;

pub mod token;
mod basic;

use std::error::Error;


pub use self::basic::*;

pub trait Lexer {

    type Out;
    type Error: Error;

    fn lex(self) -> Result<Self::Out, Self::Error>;
    fn scan_token(&mut self) -> Result<Option<token::Token>, Self::Error>;
}