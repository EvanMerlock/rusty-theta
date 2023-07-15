use super::{ThetaFunction, ThetaValue};


pub const BITSTREAM_HEADER: [u8; 8] = [0xD, 0xE, 0xA, 0xD, 0xC, 0xA, 0xF, 0xE];

#[derive(Debug)]
pub struct ThetaBitstream {
    constants: Vec<ThetaValue>,
    functions: Vec<ThetaFunction>,
}

impl ThetaBitstream {
    pub fn new() -> ThetaBitstream {
        ThetaBitstream { constants: vec![], functions: vec![] }
    }
}