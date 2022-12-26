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

            let instructions_in_chunk = chunk.instructions();
            for opcode in instructions_in_chunk {
                match opcode {
                    OpCode::RETURN => writeln!(self.output_file, "Op: Return (0x0)")?,
                    OpCode::CONSTANT { offset } => writeln!(self.output_file, "Op: Constant (0x1), with constant {:?}", chunk.constants()[*offset])?,
                    OpCode::ADD => writeln!(self.output_file, "Op: Add (0x2)")?,
                    OpCode::SUBTRACT => writeln!(self.output_file, "Op: Sub (0x3)")?,
                    OpCode::MULTIPLY => writeln!(self.output_file, "Op: Mul (0x4)")?,
                    OpCode::DIVIDE => writeln!(self.output_file, "Op: Div (0x5)")?,
                    OpCode::NEGATE => writeln!(self.output_file, "Op: Neg (0x6)")?,
                    OpCode::EQ => writeln!(self.output_file, "Op: Equal (0x7)")?,
                    OpCode::GT => writeln!(self.output_file, "Op: Greater Than (0x8)")?,
                    OpCode::LT => writeln!(self.output_file, "Op: Less Than (0x9)")?,
                };
            }
        }
        Ok(())
    }
}