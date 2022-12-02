use super::instruction::OpCode;

#[derive(Debug)]
pub struct Chunk {
    instructions: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk { instructions: Vec::new() }
    }

    pub fn write_to_chunk(&mut self, instruction: OpCode) {
        self.instructions.push(instruction);
    }

    pub fn instructions(&self) -> &Vec<OpCode> {
        &self.instructions
    }
}

#[macro_export]
macro_rules! build_chunk {
    ($($opcode:expr)+) => {
        {
            use $crate::vm::chunk::Chunk;
            let mut temp_chunk = Chunk::new();
            $(
                temp_chunk.write_to_chunk($opcode);
            )+
            temp_chunk
        }
    };
}