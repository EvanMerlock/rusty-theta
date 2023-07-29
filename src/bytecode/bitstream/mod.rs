use super::{ThetaFunction, ThetaConstant};


mod compiled;
pub use self::compiled::*;

pub const BITSTREAM_HEADER: [u8; 8] = [0xD, 0xE, 0xA, 0xD, 0xC, 0xA, 0xF, 0xE];

#[derive(Debug, Clone)]
pub struct ThetaBitstream {
    pub constants: Vec<ThetaConstant>,
    pub functions: Vec<ThetaFunction>,
}

impl ThetaBitstream {
    pub fn new() -> ThetaBitstream {
        ThetaBitstream { constants: vec![], functions: vec![] }
    }

    pub fn new_filled(constants: Vec<ThetaConstant>, functions: Vec<ThetaFunction>) -> ThetaBitstream {
        ThetaBitstream { constants, functions }
    }

    pub fn write_function(&mut self, func: ThetaFunction) {
        self.functions.push(func);
    }

    pub fn functions(&self) -> &Vec<ThetaFunction> {
        &self.functions
    }

    pub fn write_constant(&mut self, constant: ThetaConstant) {
        self.constants.push(constant);
    }

    pub fn constants(&self) -> &Vec<ThetaConstant> {
        &self.constants
    }

    pub fn merge(self, other: ThetaBitstream) -> ThetaBitstream {
        let offset_size = self.constants.len();
        let mut new_bitstream  = ThetaBitstream::new();
        for constant in self.constants {
            new_bitstream.write_constant(constant);
        } 

        for func in self.functions {
            new_bitstream.write_function(func);
        }

        for constant in other.constants {
            new_bitstream.write_constant(constant);
        }

        for mut func in other.functions {
            func.chunk = func.chunk.relocate(offset_size);
            new_bitstream.write_function(func);
        }

        new_bitstream
    }

}