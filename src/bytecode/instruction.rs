// convert from OpCode representation
// to bytes via an Assembler
// since in Rust we can represent OpCode sequences using an enumeration
// rather than just a simple u8 seq in a chunk
#[derive(Debug)]
pub enum OpCode {
    Return,
    Constant { offset: usize },
    Push,
    Pop,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Equal,
    GreaterThan,
    LessThan,

    JumpLocal { offset: i8 },
    JumpLocalIfFalse { offset: i8 },

    JumpFar { offset: isize },
    JumpFarIfFalse { offset: isize },

    DefineGlobal { offset: usize },
    GetGlobal { offset: usize },

    DefineLocal { offset: usize },
    GetLocal { offset: usize },

    DebugPrint,
}

impl OpCode {
    /// The size of the OpCode in bytes when assembled to disk, in bytes
    pub fn size(&self) -> usize {
        match self {
            OpCode::Return => 1,
            OpCode::Constant { offset: _ } => 2,
            OpCode::Push => 1,
            OpCode::Pop => 1,
            OpCode::Add => 1,
            OpCode::Subtract => 1,
            OpCode::Multiply => 1,
            OpCode::Divide => 1,
            OpCode::Negate => 1,
            OpCode::Equal => 1,
            OpCode::GreaterThan => 1,
            OpCode::LessThan => 1,
            OpCode::JumpLocal { offset: _ } => 2,
            OpCode::JumpLocalIfFalse { offset: _ } => 2,
            OpCode::DefineGlobal { offset: _ } => 2,
            OpCode::GetGlobal { offset: _ } => 2,
            OpCode::DefineLocal { offset: _ } => 2,
            OpCode::GetLocal { offset: _ } => 2,
            OpCode::DebugPrint => 2,
            OpCode::JumpFar { offset: _ } => 1 + std::mem::size_of::<isize>(),
            OpCode::JumpFarIfFalse { offset: _ } => 1 + std::mem::size_of::<isize>(),
        }
    }
}