use core::{fmt, panic};
use std::{error::Error};

use crate::bytecode::{CHUNK_HEADER, CONSTANT_POOL_HEADER, DOUBLE_MARKER, INT_MARKER, BOOL_MARKER};

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

pub struct StringDisassembler;

impl StringDisassembler {
    pub fn new() -> StringDisassembler {
        StringDisassembler{}
    }
}

impl Default for StringDisassembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Disassembler for StringDisassembler {
    type Out = Result<String, DisassembleError>;

    fn disassemble_chunk(&mut self, chunk: &[u8]) -> Result<String, DisassembleError> {
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
                sli if sli == DOUBLE_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = chunk[offset..offset+8].try_into()?;                        
                    readout.push_str(&format!("Constant: {}\r\n", f64::from_le_bytes(dbl)));
                    offset += 8;
                },
                sli if sli == INT_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = chunk[offset..offset+8].try_into()?;
                    let int = i64::from_le_bytes(dbl);
                    readout.push_str(&format!("Constant: {}\r\n", int));
                    offset += 8;
                },
                sli if sli == BOOL_MARKER => {
                    offset += 2;
                    let bol: [u8; 1] = chunk[offset..offset+1].try_into()?;
                    let bol = bol == [1u8];
                    readout.push_str(&format!("Constant: {}\r\n", bol));
                    offset += 1;
                }
                _ => panic!("invalid marker found in chunk"),
            }
        }
        

        while offset < chunk.len() {
            // read into chunk
            match chunk[offset] {
                0x0 => { readout.push_str("Op: Return (0x0)\r\n"); offset += 1 },
                0x1 => { readout.push_str(&format!("Op: Constant (0x1) with offset: {}\r\n", &chunk[offset+1])); offset += 2 },
                0x2 => { readout.push_str("Op: Add (0x2)\r\n"); offset += 1 },
                0x3 => { readout.push_str("Op: Sub (0x3)\r\n"); offset += 1 },
                0x4 => { readout.push_str("Op: Mul (0x4)\r\n"); offset += 1 },
                0x5 => { readout.push_str("Op: Div (0x5)\r\n"); offset += 1 },
                0x6 => { readout.push_str("Op: Neg (0x6)\r\n"); offset += 1 },
                code => { readout.push_str(&format!("Op: Unknown ({:#x})\r\n", code)); offset += 1 }
            }
        }

        Ok(readout)
    }

    fn disassemble(&mut self, input: &dyn AsRef<[u8]>) -> Result<String, DisassembleError> {
        let mut readout = String::new();

        // TOOD: this only handles 1 chunk as that's all we're passing it right now.
        let chunk_disassembly = self.disassemble_chunk(input.as_ref())?;
        readout.push_str(&chunk_disassembly);

        Ok(readout)
    }
}