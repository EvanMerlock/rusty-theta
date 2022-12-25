use std::io::Write;

use crate::vm::{chunk::{Chunk, CHUNK_HEADER}, instruction::OpCode, value::{CONSTANT_POOL_HEADER, ThetaValue, DOUBLE_MARKER}};

use super::{Assembler, AssembleError};

pub struct BasicAssembler<'a> {
    output_file: &'a mut dyn Write,
}

impl<'a> BasicAssembler<'a> {
    pub fn new(file_out: &'a mut dyn Write) -> BasicAssembler {
        BasicAssembler { 
            output_file: file_out
        }
    }
}

impl<'a> Assembler for BasicAssembler<'a> {
    type Out = Result<(), AssembleError>;

    fn assemble(&mut self, chunks: Vec<Chunk>) -> Result<(), AssembleError> {
        for chunk in chunks {
            self.output_file.write_all(&CHUNK_HEADER)?;
            self.output_file.write_all(&CONSTANT_POOL_HEADER)?;
            self.output_file.write_all(&[0u8, chunk.constants().len() as u8])?;

            let constants_in_chunk = chunk.constants();
            for constant in constants_in_chunk {
                match constant {
                    ThetaValue::Double(d) => {
                        self.output_file.write_all(DOUBLE_MARKER)?;
                        self.output_file.write_all(&d.to_le_bytes())?;
                    },
                };
            }

            let instructions_in_chunk = chunk.instructions();
            for opcode in instructions_in_chunk {
                match opcode {
                    OpCode::RETURN => self.output_file.write(&[0u8])?,
                    OpCode::CONSTANT { offset } => self.output_file.write(&[1u8, *offset as u8])?,
                    OpCode::ADD => self.output_file.write(&[2u8])?,
                    OpCode::SUBTRACT => self.output_file.write(&[3u8])?,
                    OpCode::MULTIPLY => self.output_file.write(&[4u8])?,
                    OpCode::DIVIDE => self.output_file.write(&[5u8])?,
                    OpCode::NEGATE => self.output_file.write(&[6u8])?,
                };
            }
        }
        Ok(())
    }
}