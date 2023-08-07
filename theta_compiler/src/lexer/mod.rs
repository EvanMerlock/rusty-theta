#[cfg(test)]
mod tests;
mod basic;

use std::error::Error;


use theta_types::bytecode::Token;

pub use self::basic::*;

pub trait Lexer {

    type Out;
    type Error: Error;

    fn lex(self) -> Result<Self::Out, Self::Error>;
    fn scan_token(&mut self) -> Result<Option<Token>, Self::Error>;
}