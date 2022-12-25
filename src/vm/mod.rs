#[macro_use]
pub mod chunk;
pub mod instruction;
pub mod bytecode;
pub mod value;

mod machine;
pub use self::machine::*;