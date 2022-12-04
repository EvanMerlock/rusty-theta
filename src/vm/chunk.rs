use super::{instruction::OpCode, value::ThetaValue};

pub const CHUNK_HEADER: [u8; 8] = [84, 104, 101, 67, 104, 117, 110, 107];

#[derive(Debug)]
pub struct Chunk {
    instructions: Vec<OpCode>,
    constants: Vec<ThetaValue>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk { instructions: Vec::new(), constants: Vec::new() }
    }

    pub fn write_to_chunk(&mut self, instruction: OpCode) {
        self.instructions.push(instruction);
    }

    pub fn instructions(&self) -> &Vec<OpCode> {
        &self.instructions
    }

    pub fn write_constant(&mut self, constant: ThetaValue) {
        self.constants.push(constant);
    }

    pub fn constants(&self) -> &Vec<ThetaValue> {
        &self.constants
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! build_chunk {
    ($($opcode:expr),+) => {
        {
            use $crate::vm::chunk::Chunk;
            let mut temp_chunk = Chunk::new();
            $(
                temp_chunk.write_to_chunk($opcode);
            )+
            temp_chunk
        }
    };
    ($($opcode:expr),+;$($constants:expr),+) => {
        {
            use $crate::vm::chunk::Chunk;
            let mut temp_chunk = Chunk::new();
            $(
                temp_chunk.write_to_chunk($opcode);
            )+
            $(
                temp_chunk.write_constant($constants);
            )+
            temp_chunk
        }
    };
}