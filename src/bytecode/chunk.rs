use super::{OpCode, ThetaConstant};

pub const CHUNK_HEADER: [u8; 8] = [84, 104, 101, 67, 104, 117, 110, 107];

#[derive(Debug)]
pub struct Chunk {
    instructions: Vec<OpCode>,
    constants: Vec<ThetaConstant>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk { instructions: Vec::new(), constants: Vec::new() }
    }

    /// Derives the size of the individual instructions inside the chunk.
    /// This can be used, for example, to derive jump information for if-statements and loops.
    /// We do not necessarily need constant pool or chunk header sizes included in the instruction size.
    pub fn instruction_size(&self) -> usize {
        let mut size: usize = 0;
        for instruction in &self.instructions {
            size += instruction.size();
        } 
        size
    }

    pub fn write_to_chunk(&mut self, instruction: OpCode) {
        self.instructions.push(instruction);
    }

    pub fn instructions(&self) -> &Vec<OpCode> {
        &self.instructions
    }

    pub fn write_constant(&mut self, constant: ThetaConstant) {
        self.constants.push(constant);
    }

    pub fn constants(&self) -> &Vec<ThetaConstant> {
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
            let new_opcode = opcode.relocate_constants(offset_size);
            new_chunk.write_to_chunk(new_opcode);
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
            use $crate::bytecode::Chunk;
            let mut temp_chunk = Chunk::new();
            $(
                temp_chunk.write_to_chunk($opcode);
            )+
            temp_chunk
        }
    };
    ($($opcode:expr),+;$($constants:expr),+) => {
        {
            use $crate::bytecode::Chunk;
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