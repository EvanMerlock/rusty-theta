use core::fmt;
use std::error::Error;

pub trait Disassembler {
    type Out;

    fn disassemble(&mut self) -> Self::Out;
    fn disassemble_chunk(&self, chunk: &[u8]) -> Self::Out;
}

#[derive(Debug)]
pub enum DisassembleError {
    IOError(std::io::Error),
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

pub struct StringDisassembler<'a> {
    reader: &'a mut Box<dyn std::io::BufRead>
}

impl<'a> StringDisassembler<'a> {
    pub fn new(in_read: &'a mut Box<dyn std::io::BufRead>) -> StringDisassembler<'a> {
        StringDisassembler { 
            reader: in_read
        }
    }
}

impl<'a> Disassembler for StringDisassembler<'a> {
    type Out = Result<String, DisassembleError>;

    fn disassemble_chunk(&self, chunk: &[u8]) -> Result<String, DisassembleError> {
        let mut offset = 0;
        let mut readout = String::new();
        readout.push_str("=== chunk ===\r\n");

        while offset < chunk.len() {
            // read into chunk
            match chunk[offset] {
                0 => readout.push_str("Op: Return (0x0)\r\n"),
                code => readout.push_str(&format!("Op: Unknown ({:#x})\r\n", code))
            }

            offset += 1;
        }

        Ok(readout)
    }

    fn disassemble(&mut self) -> Result<String, DisassembleError> {
        let mut input: Vec<u8> = Vec::new();
        self.reader.read_to_end(&mut input)?;

        let mut readout = String::new();

        // TOOD: this only handles 1 chunk as that's all we're passing it right now.
        let chunk_disassembly = self.disassemble_chunk(&input)?;
        readout.push_str(&chunk_disassembly);

        Ok(readout)
    }
}