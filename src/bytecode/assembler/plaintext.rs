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
                    OpCode::Return => writeln!(self.output_file, "Op: Return (0x0)")?,
                    OpCode::Constant { offset } => writeln!(self.output_file, "Op: Constant (0x1), with constant {:?}", chunk.constants()[*offset])?,
                    OpCode::Push => writeln!(self.output_file, "Op: Push (0x2)")?,
                    OpCode::Pop => writeln!(self.output_file, "Op: Pop (0x3)")?,

                    OpCode::Add => writeln!(self.output_file, "Op: Add (0x4)")?,
                    OpCode::Subtract => writeln!(self.output_file, "Op: Sub (0x5)")?,
                    OpCode::Multiply => writeln!(self.output_file, "Op: Mul (0x6)")?,
                    OpCode::Divide => writeln!(self.output_file, "Op: Div (0x7)")?,
                    OpCode::Negate => writeln!(self.output_file, "Op: Neg (0x8)")?,
                    OpCode::Equal => writeln!(self.output_file, "Op: Equal (0x9)")?,
                    OpCode::GreaterThan => writeln!(self.output_file, "Op: Greater Than (0xA)")?,
                    OpCode::LessThan => writeln!(self.output_file, "Op: Less Than (0xB)")?,

                    OpCode::DefineGlobal { offset } => writeln!(self.output_file, "Op: Define Global with global name {:?} (0xC0)", chunk.constants()[*offset])?,
                    OpCode::GetGlobal { offset } => writeln!(self.output_file, "Op: Get Global with global name {:?} (0xC1)", chunk.constants()[*offset])?,
                    OpCode::DefineLocal { offset } => writeln!(self.output_file, "Op: Define Local with offset {:?} (0xC2)", offset)?,
                    OpCode::GetLocal { offset } => writeln!(self.output_file, "Op: Get Local wih offset {:?} (0xC3)", offset)?,

                    OpCode::JumpLocal { offset } => writeln!(self.output_file, "Op: Jump Unconditional with offset {:?} (0xD0)", offset)?,
                    OpCode::JumpLocalIfFalse { offset } => writeln!(self.output_file, "Op: Jump If False with offset {:?} (OxD1)", offset)?,

                    OpCode::JumpFar { offset } => writeln!(self.output_file, "Op: Jump Unconditional Far with offset {:?} (0xD2)", offset)?,
                    OpCode::JumpFarIfFalse { offset } => writeln!(self.output_file, "Op: Jump Far If False with offset {:?} (OxD3)", offset)?,


                    OpCode::DebugPrint => writeln!(self.output_file, "Op: DebugPrint (0xFF)")?,
                };
            }
        }
        Ok(())
    }
}