use crate::bytecode::{Chunk, OpCode};

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

            let mut code_offset: usize = 0;
            let instructions_in_chunk = chunk.instructions();
            for opcode in instructions_in_chunk {
                write!(self.output_file, "{code_offset:#X} | Op: {} ({:#X})", opcode.human_readable(), opcode.as_hexcode())?;

                match opcode {
                    OpCode::JumpFar { offset } => write!(self.output_file, " -> {:#X}", code_offset as isize + offset)?,
                    OpCode::JumpLocal { offset } => write!(self.output_file, " -> {:#X}", code_offset as isize + *offset as isize)?,
                    OpCode::JumpFarIfFalse { offset } => write!(self.output_file, " -> {:#X}", code_offset as isize + offset)?,
                    OpCode::JumpLocalIfFalse { offset } => write!(self.output_file, " -> {:#X}", code_offset as isize + *offset as isize)?,
                    _ => {},
                }

                writeln!(self.output_file)?;

                code_offset += opcode.size();
            }
            writeln!(self.output_file, "=== CHUNK END @ {code_offset:#X} ===")?;
        }
        Ok(())
    }
}