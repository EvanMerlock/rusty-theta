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
                    OpCode::PUSH => writeln!(self.output_file, "Op: Push (0x2)")?,
                    OpCode::POP => writeln!(self.output_file, "Op: Pop (0x3)")?,

                    OpCode::ADD => writeln!(self.output_file, "Op: Add (0x4)")?,
                    OpCode::SUBTRACT => writeln!(self.output_file, "Op: Sub (0x5)")?,
                    OpCode::MULTIPLY => writeln!(self.output_file, "Op: Mul (0x6)")?,
                    OpCode::DIVIDE => writeln!(self.output_file, "Op: Div (0x7)")?,
                    OpCode::NEGATE => writeln!(self.output_file, "Op: Neg (0x8)")?,
                    OpCode::EQ => writeln!(self.output_file, "Op: Equal (0x9)")?,
                    OpCode::GT => writeln!(self.output_file, "Op: Greater Than (0xA)")?,
                    OpCode::LT => writeln!(self.output_file, "Op: Less Than (0xB)")?,

                    OpCode::DEFINE_GLOBAL { offset } => writeln!(self.output_file, "Op: Define Global with global name {:?} (0xC0)", chunk.constants()[*offset])?,
                    OpCode::GET_GLOBAL { offset } => writeln!(self.output_file, "Op: Get Global with global name {:?} (0xC1)", chunk.constants()[*offset])?,

                    OpCode::DEBUG_PRINT => writeln!(self.output_file, "Op: DebugPrint (0xFF)")?,
                };
            }
        }
        Ok(())
    }
}