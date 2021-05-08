#[cfg(test)]
mod tests;

pub(crate) mod token;
mod basic;

pub use self::basic::*;

pub trait Lexer {

    type Out;

    fn lex(self) -> Self::Out;
    fn scan_token(&mut self) -> Option<token::Token>;
}