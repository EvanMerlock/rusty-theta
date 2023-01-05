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
    Utf8Error(std::string::FromUtf8Error),
    InvalidMarkerInChunk(Vec<u8>),
}

impl fmt::Display for DisassembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisassembleError::IOError(io) => write!(f, "I/O error: {}", io),
            DisassembleError::TryFromSliceError(tfs) => write!(f, "Could not get item from slice: {}", tfs),
            DisassembleError::Utf8Error(utf) => write!(f, "UTF-8 error: {}", utf),
            DisassembleError::InvalidMarkerInChunk(marker) => write!(f, "invalid marker: [{}, {}]", marker[0], marker[1]),
        }
    }
}

impl Error for DisassembleError {}

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

impl From<std::string::FromUtf8Error> for DisassembleError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        DisassembleError::Utf8Error(e)
    }
}