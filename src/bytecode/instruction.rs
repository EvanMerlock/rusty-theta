// convert from OpCode representation
// to bytes via an Assembler
// since in Rust we can represent OpCode sequences using an enumeration
// rather than just a simple u8 seq in a chunk
#[derive(Debug)]
pub enum OpCode {
    RETURN,
    CONSTANT { offset: usize },
    PUSH,
    POP,
    ADD,
    SUBTRACT,
    MULTIPLY,
    DIVIDE,
    NEGATE,
    EQ,
    GT,
    LT,

    DEFINE_GLOBAL { offset: usize },
    GET_GLOBAL { offset: usize },

    DEBUG_PRINT,
}