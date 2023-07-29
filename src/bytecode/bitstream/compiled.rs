use crate::bytecode::{ThetaValue, ThetaCompiledFunction};

#[derive(Debug, Clone)]
pub struct ThetaCompiledBitstream {
    pub constants: Vec<ThetaValue>,
    pub functions: Vec<ThetaCompiledFunction>,
}

impl ThetaCompiledBitstream {
    pub fn new() -> ThetaCompiledBitstream {
        ThetaCompiledBitstream { constants: vec![], functions: vec![] }
    }

    pub fn new_filled(constants: Vec<ThetaValue>, functions: Vec<ThetaCompiledFunction>) -> ThetaCompiledBitstream {
        ThetaCompiledBitstream { constants, functions }
    }

    pub fn write_function(&mut self, func: ThetaCompiledFunction) {
        self.functions.push(func);
    }

    pub fn functions(&self) -> &Vec<ThetaCompiledFunction> {
        &self.functions
    }

    pub fn write_constant(&mut self, constant: ThetaValue) {
        self.constants.push(constant);
    }

    pub fn constants(&self) -> &Vec<ThetaValue> {
        &self.constants
    }

}

impl Default for ThetaCompiledBitstream {
    fn default() -> Self {
        Self::new()
    }
}