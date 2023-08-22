#[cfg(test)]
mod tests;
mod basic;

use std::error::Error;


use theta_types::bytecode::Token;

pub use self::basic::*;

pub trait Lexer {

    type Out;
    type Error: Error;

    fn lex(self) -> Result<LexerResult<Self::Out>, Self::Error>;
    fn scan_token(&mut self) -> Result<Option<Token>, Self::Error>;
}

pub struct LexerResult<T> {
    line_mapping: Vec<usize>,
    output: T
}

impl<T> LexerResult<T> {
    pub fn new(out: T, mapping: Vec<usize>) -> LexerResult<T> {
        LexerResult { line_mapping: mapping, output: out }
    }

    pub fn output(&self) -> &T {
        &self.output
    }

    pub fn line_mapping(&self) -> &Vec<usize> {
        &self.line_mapping
    }
}