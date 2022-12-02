use crate::vm::{chunk::Chunk, instruction::OpCode};

use super::{Assembler, AssembleError};

pub struct BasicAssembler<'a> {
    output_file: &'a mut Box<dyn std::io::Write>,
}

impl<'a> BasicAssembler<'a> {
    pub fn new(file_out: &'a mut Box<dyn std::io::Write>) -> BasicAssembler {
        BasicAssembler { 
            output_file: file_out
        }
    }
}

impl<'a> Assembler for BasicAssembler<'a> {
    type Out = Result<(), AssembleError>;

    fn assemble(&mut self, chunks: Vec<Chunk>) -> Result<(), AssembleError> {
        for chunk in chunks {
            let instructions_in_chunk = chunk.instructions();
            for opcode in instructions_in_chunk {
                match opcode {
                    OpCode::RETURN => self.output_file.write(&[0u8])?,
                };
            }
        }
        Ok(())
    }
}