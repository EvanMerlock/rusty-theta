use core::{fmt};
use std::{error::Error};

mod string;
pub use self::string::*;

pub trait Disassembler {
    type Out;

    fn disassemble(&mut self, code: &dyn AsRef<[u8]>) -> Self::Out;
    fn disassemble_chunk(&mut self, chunk: &[u8]) -> Self::Out;
}

#[derive(Debug)]
pub enum DisassembleError {
    IOError(std::io::Error),
    TryFromSliceError(std::array::TryFromSliceError),
}

impl fmt::Display for DisassembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Error for DisassembleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl From<std::io::Error> for DisassembleError {
    fn from(err: std::io::Error) -> Self {
        DisassembleError::IOError(err)
    }
}

impl From<std::array::TryFromSliceError> for DisassembleError {
    fn from(err: std::array::TryFromSliceError) -> Self {
        DisassembleError::TryFromSliceError(err)
    }
}