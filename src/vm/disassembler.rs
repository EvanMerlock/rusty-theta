use core::fmt;
use std::error::Error;

pub trait Disassembler {
    type Out;

    fn disassemble(&mut self) -> Self::Out;
}

#[derive(Debug)]
pub enum DisassembleError {

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

pub struct StringDisassembler {

}

impl StringDisassembler {
    pub fn new(in_read: Box<dyn std::io::BufRead>) -> StringDisassembler {
        StringDisassembler {  }
    }
}

impl Disassembler for StringDisassembler {
    type Out = Result<String, DisassembleError>;

    fn disassemble(&mut self) -> Result<String, DisassembleError> {
        todo!()
    }
}