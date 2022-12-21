use core::{fmt, panic};
use std::{error::Error};

use crate::vm::{chunk::CHUNK_HEADER, value::{CONSTANT_POOL_HEADER, DOUBLE_MARKER}};

pub trait Disassembler {
    type Out;

    fn disassemble(&mut self) -> Self::Out;
    fn disassemble_chunk(&self, chunk: &[u8]) -> Self::Out;
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
        let mut offset = 18;
        let mut readout = String::new();

        println!("chunk: {:?}", chunk);

        // assert chunk header
        assert!(chunk[0..8] == CHUNK_HEADER);
        
        readout.push_str("=== BEGIN CHUNK ===\r\n");

        // assert constant pool header
        assert!(chunk[8..16] == CONSTANT_POOL_HEADER);

        readout.push_str("-- BEGIN CONSTANT POOL --\r\n");

        // read const pool size
        let const_pool_size = chunk[17];
        for _ in 0..const_pool_size {
            let marker = &chunk[offset..offset+2];
            println!("marker: {:?}", marker);
            match marker {
                DOUBLE_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = chunk[offset..offset+8].try_into()?;                        
                    readout.push_str(&format!("Constant: {}\r\n", f64::from_le_bytes(dbl)));
                    offset += 8;
                },
                _ => panic!("invalid marker found in chunk"),
            }
        }
        

        while offset < chunk.len() {
            // read into chunk
            match chunk[offset] {
                0x0 => { readout.push_str("Op: Return (0x0)\r\n"); offset += 1 },
                0x1 => { readout.push_str(&format!("Op: Constant (0x1) with offset: {}\r\n", &chunk[offset+1])); offset += 2 },
                code => { readout.push_str(&format!("Op: Unknown ({:#x})\r\n", code)); offset += 1 }
            }
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