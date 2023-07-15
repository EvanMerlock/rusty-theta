use std::io::Write;

use crate::bytecode::{
    Chunk, OpCode, ThetaBitstream, ThetaConstant, BOOL_MARKER, CHUNK_HEADER, CONSTANT_POOL_HEADER,
    DOUBLE_MARKER, INT_MARKER, STRING_MARKER,
};

use super::{AssembleError, Assembler};

pub struct BasicAssembler<'a> {
    output_file: &'a mut dyn Write,
}

impl<'a> BasicAssembler<'a> {
    pub fn new(file_out: &'a mut dyn Write) -> BasicAssembler {
        BasicAssembler {
            output_file: file_out,
        }
    }
}

impl<'a> Assembler for BasicAssembler<'a> {
    type Out = Result<(), AssembleError>;

    fn assemble(&mut self, bitstream: ThetaBitstream) -> Result<(), AssembleError> {
        todo!()
    }

    fn assemble_bitstream(&mut self, bitstream: ThetaBitstream) -> Self::Out {
        todo!()
    }

    fn assemble_chunk(&mut self, chunk: Chunk) -> Self::Out {
        self.output_file.write_all(&CHUNK_HEADER)?;
        self.output_file.write_all(&CONSTANT_POOL_HEADER)?;
        self.output_file
            .write_all(&[0u8, chunk.constants().len() as u8])?;

        let constants_in_chunk = chunk.constants();
        for constant in constants_in_chunk {
            match constant {
                ThetaConstant::Double(d) => {
                    self.output_file.write_all(DOUBLE_MARKER)?;
                    self.output_file.write_all(&d.to_le_bytes())?;
                }
                ThetaConstant::Int(i) => {
                    self.output_file.write_all(INT_MARKER)?;
                    self.output_file.write_all(&i.to_le_bytes())?;
                }
                ThetaConstant::Bool(b) => {
                    self.output_file.write_all(BOOL_MARKER)?;
                    self.output_file.write_all(&([(*b) as u8]))?;
                }
                ThetaConstant::Str(s) => {
                    self.output_file.write_all(STRING_MARKER)?;
                    let length = s.len();
                    self.output_file.write_all(&length.to_le_bytes())?;
                    self.output_file.write_all(s.as_bytes())?;
                }
            };
        }

        let instructions_in_chunk = chunk.instructions();
        for opcode in instructions_in_chunk {
            match opcode {
                OpCode::Return => self.output_file.write(&[0u8])?,
                OpCode::Constant { offset } => self.output_file.write(&[1u8, *offset as u8])?,
                OpCode::Push { size } => {
                    let off_bytes = size.to_le_bytes();
                    self.output_file.write_all(&[2u8])?;
                    self.output_file.write(&off_bytes)?
                }
                OpCode::Pop => self.output_file.write(&[3u8])?,

                OpCode::Add => self.output_file.write(&[4u8])?,
                OpCode::Subtract => self.output_file.write(&[5u8])?,
                OpCode::Multiply => self.output_file.write(&[6u8])?,
                OpCode::Divide => self.output_file.write(&[7u8])?,
                OpCode::Negate => self.output_file.write(&[8u8])?,
                OpCode::Equal => self.output_file.write(&[9u8])?,
                OpCode::GreaterThan => self.output_file.write(&[0xAu8])?,
                OpCode::LessThan => self.output_file.write(&[0xBu8])?,

                OpCode::DefineGlobal { offset } => {
                    self.output_file.write(&[0xC0u8, *offset as u8])?
                }
                OpCode::GetGlobal { offset } => self.output_file.write(&[0xC1u8, *offset as u8])?,

                OpCode::DefineLocal { offset } => {
                    self.output_file.write(&[0xC2u8, *offset as u8])?
                }
                OpCode::GetLocal { offset } => self.output_file.write(&[0xC3u8, *offset as u8])?,

                OpCode::JumpLocal { offset } => self.output_file.write(&[0xD0u8, *offset as u8])?,

                OpCode::JumpLocalIfFalse { offset } => {
                    self.output_file.write(&[0xD1u8, *offset as u8])?
                }

                OpCode::JumpFar { offset } => {
                    let off_bytes = offset.to_le_bytes();

                    self.output_file.write_all(&[0xD2u8])?;
                    self.output_file.write(&off_bytes)?
                }

                OpCode::JumpFarIfFalse { offset } => {
                    let off_bytes = offset.to_le_bytes();

                    self.output_file.write_all(&[0xD3u8])?;
                    self.output_file.write(&off_bytes)?
                }

                OpCode::DebugPrint => self.output_file.write(&[0xFFu8])?,
                OpCode::Noop => self.output_file.write(&[0xFDu8])?,
            };
        }
        Ok(())
    }
}
