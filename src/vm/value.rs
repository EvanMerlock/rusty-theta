pub const CONSTANT_POOL_HEADER: [u8; 8] = [84, 104, 101, 67, 111, 110, 115, 116];

pub const DOUBLE_MARKER: &[u8] = &[0xF, 0xF];

#[derive(Debug)]
pub enum ThetaValue {
    Double(f64),
}