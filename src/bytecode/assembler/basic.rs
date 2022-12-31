use std::io::Write;

use crate::bytecode::{Chunk, CHUNK_HEADER, OpCode, CONSTANT_POOL_HEADER, ThetaValue, DOUBLE_MARKER, INT_MARKER, BOOL_MARKER, ThetaHeapValue, STRING_MARKER};

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
                    ThetaValue::Int(i) => {
                        self.output_file.write_all(INT_MARKER)?;
                        self.output_file.write_all(&i.to_le_bytes())?;
                    },
                    ThetaValue::Bool(b) => {
                        self.output_file.write_all(BOOL_MARKER)?;
                        self.output_file.write_all(&([(*b) as u8]))?;
                    },
                    ThetaValue::HeapValue(b) => {
                        // ??? the box really screws stuff up here. we will most likely want to move to raw heap pointers
                        // and enter the realm of... UNSAFE RUST!!!!!!!
                        match &**b {
                            ThetaHeapValue::Str(s) => {
                                self.output_file.write_all(STRING_MARKER)?;
                                let length = s.len();
                                self.output_file.write_all(&length.to_le_bytes())?;
                                self.output_file.write_all(s.as_bytes())?;
                            },
                        }                        
                        // we will need to care about certain values on the heap to persist them outwards to the constant pool
                        // this involves any objects that can be stored in the constant pool

                    },
                };
            }

            let instructions_in_chunk = chunk.instructions();
            for opcode in instructions_in_chunk {
                match opcode {
                    OpCode::RETURN => self.output_file.write(&[0u8])?,
                    OpCode::CONSTANT { offset } => self.output_file.write(&[1u8, *offset as u8])?,
                    OpCode::PUSH => self.output_file.write(&[2u8])?,
                    OpCode::POP => self.output_file.write(&[3u8])?,

                    OpCode::ADD => self.output_file.write(&[4u8])?,
                    OpCode::SUBTRACT => self.output_file.write(&[5u8])?,
                    OpCode::MULTIPLY => self.output_file.write(&[6u8])?,
                    OpCode::DIVIDE => self.output_file.write(&[7u8])?,
                    OpCode::NEGATE => self.output_file.write(&[8u8])?,
                    OpCode::EQ => self.output_file.write(&[9u8])?,
                    OpCode::GT => self.output_file.write(&[0xAu8])?,
                    OpCode::LT => self.output_file.write(&[0xBu8])?,

                    OpCode::DEFINE_GLOBAL { offset } => self.output_file.write(&[0xC0u8, *offset as u8])?,
                    OpCode::GET_GLOBAL { offset } => self.output_file.write(&[0xC1u8, *offset as u8])?,

                    OpCode::DEBUG_PRINT => self.output_file.write(&[0xFFu8])?,
                };
            }
        }
        Ok(())
    }
}