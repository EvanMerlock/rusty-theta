mod basic;
mod plaintext;

pub use self::basic::*;
pub use self::plaintext::*;

use core::fmt;
use std::error::Error;

use crate::bytecode::{Chunk};

pub trait Assembler {
    type Out;

    fn assemble(&mut self, chunks: Vec<Chunk>) -> Self::Out;
}

#[derive(Debug)]
pub enum AssembleError {
    IOError(std::io::Error),
}

impl fmt::Display for AssembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssembleError::IOError(e) => write!(f, "AssembleError: {:?}", e),
        }
    }
}

impl Error for AssembleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl From<std::io::Error> for AssembleError {
    fn from(err: std::io::Error) -> Self {
        AssembleError::IOError(err)
    }
}