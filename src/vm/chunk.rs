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

    pub fn merge_chunk(self, other: Chunk) -> Chunk {
        let offset_size = self.constants.len();
        let mut new_chunk = Chunk::new();
        for constant in self.constants {
            new_chunk.write_constant(constant);
        }
        for opcode in self.instructions {
            new_chunk.write_to_chunk(opcode);
        }
        for constant in other.constants {
            new_chunk.write_constant(constant);
        }
        for opcode in other.instructions {
            match opcode {
                OpCode::CONSTANT { offset } => new_chunk.write_to_chunk(OpCode::CONSTANT { offset: offset + offset_size }),
                _ => new_chunk.write_to_chunk(opcode),
            }
        }
        new_chunk
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