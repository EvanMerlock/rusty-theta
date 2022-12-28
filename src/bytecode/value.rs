pub const CONSTANT_POOL_HEADER: [u8; 8] = [84, 104, 101, 67, 111, 110, 115, 116];

pub const DOUBLE_MARKER: &[u8] = &[0xF, 0xF];
pub const INT_MARKER: &[u8] = &[0xA, 0xA];
pub const BOOL_MARKER: &[u8] = &[0xB, 0xB];

#[derive(Debug, Clone)]
pub enum ThetaValue {
    Double(f64),
    Int(i64),
    Bool(bool),
}