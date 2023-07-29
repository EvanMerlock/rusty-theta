mod basic;
mod plaintext;

pub use self::basic::*;
pub use self::plaintext::*;

use core::fmt;
use std::error::Error;

use crate::bytecode::Chunk;

use super::ThetaBitstream;
use super::ThetaConstant;

use super::ThetaFunction;


pub trait Assembler {
    type Out;

    fn assemble(&mut self, bitstream: ThetaBitstream) -> Self::Out;
    fn assemble_bitstream(&mut self, bitstream: ThetaBitstream) -> Self::Out;
    fn assemble_constant_pool(&mut self, constant_pool: Vec<ThetaConstant>) -> Self::Out;
    fn assemble_function_pool(&mut self, function_pool: Vec<ThetaFunction>) -> Self::Out;
    fn assemble_chunk(&mut self, chunk: Chunk) -> Self::Out;
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

impl Error for AssembleError {}

impl From<std::io::Error> for AssembleError {
    fn from(err: std::io::Error) -> Self {
        AssembleError::IOError(err)
    }
}