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

    DefineGlobal { offset: usize },
    GetGlobal { offset: usize },

    DefineLocal { offset: usize },
    GetLocal { offset: usize },

    DebugPrint,
}