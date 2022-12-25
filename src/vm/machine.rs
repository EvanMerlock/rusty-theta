use log::debug;
use tracing::field::debug;

use crate::vm::{chunk::CHUNK_HEADER, value::CONSTANT_POOL_HEADER};

use super::{value::ThetaValue, bytecode::{Disassembler, DisassembleError}};

pub struct VM {
    stack: Vec<ThetaValue>,
    constants: Vec<ThetaValue>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn stack(&self) -> &Vec<ThetaValue> {
        &self.stack
    }

    pub fn constants(&self) -> &Vec<ThetaValue> {
        &self.constants
    }

    pub fn clear_const_pool(&mut self) {
        self.constants.clear()
    }
}

impl Disassembler for VM {
    type Out = Result<(), DisassembleError>;

    fn disassemble_chunk(&mut self, chunk: &[u8]) -> Result<(), DisassembleError> {
        let mut offset = 18;

        debug!("chunk: {:?}", chunk);

        // assert chunk header
        assert!(chunk[0..8] == CHUNK_HEADER);
        
        debug!("=== BEGIN CHUNK ===\r\n");

        // assert constant pool header
        assert!(chunk[8..16] == CONSTANT_POOL_HEADER);

        debug!("-- BEGIN CONSTANT POOL --\r\n");

        // read const pool size
        let const_pool_size = chunk[17];
        for _ in 0..const_pool_size {
            let marker = &chunk[offset..offset+2];
            debug!("marker: {:?}", marker);
            match marker {
                DOUBLE_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = chunk[offset..offset+8].try_into()?;
                    let float = f64::from_le_bytes(dbl);
                    debug!("float found in constant pool: {}", float);
                    self.constants.push(ThetaValue::Double(float));              
                    offset += 8;
                },
                _ => panic!("invalid marker found in chunk"),
            }
        }
        

        while offset < chunk.len() {
            // read into chunk
            match chunk[offset] {
                0x0 => { 
                    debug!("Op: Return (0x0)\r\n"); 
                    println!("{:?}", self.stack.pop()); 
                    offset += 1 
                },
                0x1 => { 
                    debug!("Op: Constant (0x1) with offset: {}\r\n", &chunk[offset+1]); 
                    self.stack.push(self.constants[chunk[offset+1] as usize].clone()); 
                    offset += 2 
                },
                0x2 => {
                    debug!("Op: Add (0x2)\r\n");
                    let left = self.stack.pop().expect("failed to grab value off stack");
                    let right = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l+r)),
                    };
                    offset += 1
                },
                0x3 => {
                    debug!("Op: Sub (0x3)\r\n");
                    let left = self.stack.pop().expect("failed to grab value off stack");
                    let right = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l-r)),
                    };
                    offset += 1
                },
                0x4 => {
                    debug!("Op: Mul (0x3)\r\n");
                    let left = self.stack.pop().expect("failed to grab value off stack");
                    let right = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l*r)),
                    };
                    offset += 1
                },
                0x5 => {
                    debug!("Op: Div (0x5)\r\n");
                    let left = self.stack.pop().expect("failed to grab value off stack");
                    let right = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l/r)),
                    };
                    offset += 1
                },
                0x6 => {
                    debug!("Op: Neg (0x6)\r\n");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match left {
                        ThetaValue::Double(l) => self.stack.push(ThetaValue::Double(-l)),
                    };
                    offset += 1
                },
                code => { 
                    debug!("Op: Unknown ({:#x})\r\n", code); 
                    offset += 1 
                }
            }
        }

        Ok(())
    }

    fn disassemble(&mut self, input: &dyn AsRef<[u8]>) -> Result<(), DisassembleError> {
        // TOOD: this only handles 1 chunk as that's all we're passing it right now.
        self.disassemble_chunk(input.as_ref())?;

        Ok(())
    }
}