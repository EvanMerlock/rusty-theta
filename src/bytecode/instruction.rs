// convert from OpCode representation
// to bytes via an Assembler
// since in Rust we can represent OpCode sequences using an enumeration
// rather than just a simple u8 seq in a chunk
#[derive(Debug)]
pub enum OpCode {
    Return,
    Constant { offset: usize },
    Push { size: usize },
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

    Noop,
    DebugPrint,
}

impl OpCode {
    /// The size of the OpCode in bytes when assembled to disk, in bytes
    pub fn size(&self) -> usize {
        match self {
            OpCode::Return => 1,
            OpCode::Constant { offset: _ } => 2,
            OpCode::Push { size: _ } => 1 + std::mem::size_of::<usize>(),
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
            OpCode::Noop => 1,
        }
    }

    pub fn human_readable(&self) -> String {
        match self {
            OpCode::Return => "Return".to_string(),
            OpCode::Constant { offset } => format!("Constant with offset {offset:#X}"),
            OpCode::Push { size } => format!("Push with size {size:#X}"),
            OpCode::Pop => "Pop".to_string(),
            OpCode::Add => "Add".to_string(),
            OpCode::Subtract => "Subtract".to_string(),
            OpCode::Multiply => "Multiply".to_string(),
            OpCode::Divide => "Divide".to_string(),
            OpCode::Negate => "Negate".to_string(),
            OpCode::Equal => "Equal".to_string(),
            OpCode::GreaterThan => "Greater Than".to_string(),
            OpCode::LessThan => "Less Than".to_string(),
            OpCode::JumpLocal { offset } => format!("Jump (local) unconditional with offset {offset:#X}"),
            OpCode::JumpLocalIfFalse { offset } => format!("Jump (local) if false with offset {offset:#X}"),
            OpCode::JumpFar { offset } => format!("Jump (far) unconditional with offset {offset:#X}"),
            OpCode::JumpFarIfFalse { offset } => format!("Jump (far) if false with offset {offset:#X}"),
            OpCode::DefineGlobal { offset } => format!("Define global variable with offset {offset:#X}"),
            OpCode::GetGlobal { offset } => format!("Retrieve global variable with offset {offset:#X}"),
            OpCode::DefineLocal { offset } => format!("Define local variable with offset {offset:#X}"),
            OpCode::GetLocal { offset } => format!("Get local variable with offset {offset:#X}"),
            OpCode::DebugPrint => "Debug print".to_string(),
            OpCode::Noop => "Noop".to_string(),
        }
    }

    pub fn as_hexcode(&self) -> usize {
        match self {
            OpCode::Return => 0x0,
            OpCode::Constant { offset: _ } => 0x1,
            OpCode::Push { size: _ } => 0x2,
            OpCode::Pop => 0x3,
            OpCode::Add => 0x4,
            OpCode::Subtract => 0x5,
            OpCode::Multiply => 0x6,
            OpCode::Divide => 0x7,
            OpCode::Negate => 0x8,
            OpCode::Equal => 0x9,
            OpCode::GreaterThan => 0xA,
            OpCode::LessThan => 0xB,
            OpCode::JumpLocal { offset: _ } => 0xD0,
            OpCode::JumpLocalIfFalse { offset: _ } => 0xD1,
            OpCode::JumpFar { offset: _ } => 0xD2,
            OpCode::JumpFarIfFalse { offset: _ } => 0xD3,
            OpCode::DefineGlobal { offset: _ } => 0xC0,
            OpCode::GetGlobal { offset: _ } => 0xC1,
            OpCode::DefineLocal { offset: _ } => 0xC2,
            OpCode::GetLocal { offset: _ } => 0xC3,
            OpCode::DebugPrint => 0xFF,
            OpCode::Noop => 0xFD,
        }
    }

    pub fn relocate_constants(self, new_base: usize) -> OpCode {
        match self {
            OpCode::Constant { offset } => OpCode::Constant { offset: offset + new_base },
            OpCode::DefineGlobal { offset } => OpCode::DefineGlobal { offset: offset + new_base },
            OpCode::GetGlobal { offset } => OpCode::GetGlobal { offset: offset + new_base },
            _ => self,
        }
    }
}