mod basic;
mod parseinfo;

#[cfg(test)]
mod tests;

use theta_types::errors::parse::ParseError;

pub use self::basic::*;
pub use self::parseinfo::*;

pub trait Parser {

    type Out;

    fn next(&mut self) -> Result<Self::Out, ParseError>;
    fn parse(self) -> Result<Vec<Self::Out>, ParseError>;
    
}