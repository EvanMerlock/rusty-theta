use crate::vm::{chunk::Chunk, instruction::OpCode};

use super::{Assembler, AssembleError};

pub struct PlainTextAssembler<'a> {
    output_file: &'a mut Box<dyn std::io::Write>,
}

impl<'a> PlainTextAssembler<'a> {
    pub fn new(file_out: &'a mut Box<dyn std::io::Write>) -> PlainTextAssembler {
        PlainTextAssembler { 
            output_file: file_out
        }
    }
}

impl<'a> Assembler for PlainTextAssembler<'a> {
    type Out = Result<(), AssembleError>;

    fn assemble(&mut self, chunks: Vec<Chunk>) -> Result<(), AssembleError> {
        for chunk in chunks {
            writeln!(self.output_file, "=== CHUNK BEGIN ===")?;
            writeln!(self.output_file, "-- CONSTANT POOL --")?;
            let constants = chunk.constants();

            for constant in constants {
                writeln!(self.output_file, "Constant: {:?}", constant)?;
            }

            writeln!(self.output_file, "-- INSTRUCTIONS --")?;

            let instructions_in_chunk = chunk.instructions();
            for opcode in instructions_in_chunk {
                match opcode {
                    OpCode::RETURN => writeln!(self.output_file, "Op: Return (0x0)")?,
                    OpCode::CONSTANT { offset } => writeln!(self.output_file, "Op: Constant (0x1), with constant {:?}", chunk.constants()[*offset])?,
                    OpCode::ADD => todo!(),
                    OpCode::SUBTRACT => todo!(),
                    OpCode::MULTIPLY => todo!(),
                    OpCode::DIVIDE => todo!(),
                };
            }
        }
        Ok(())
    }
}