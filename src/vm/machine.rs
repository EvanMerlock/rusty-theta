use log::debug;

use crate::bytecode::{CHUNK_HEADER, CONSTANT_POOL_HEADER, INT_MARKER, DOUBLE_MARKER, BOOL_MARKER, ThetaValue, Disassembler, DisassembleError};

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

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl Disassembler for VM {
    type Out = Result<(), DisassembleError>;

    fn disassemble_chunk(&mut self, chunk: &[u8]) -> Result<(), DisassembleError> {
        let mut offset = 18;

        debug!("chunk: {:?}", chunk);

        // assert chunk header
        assert!(chunk[0..8] == CHUNK_HEADER);
        
        debug!("=== BEGIN CHUNK ===");

        // assert constant pool header
        assert!(chunk[8..16] == CONSTANT_POOL_HEADER);

        debug!("-- BEGIN CONSTANT POOL --");

        // read const pool size
        let const_pool_size = chunk[17];
        // TODO: constant pool should not be loaded into the VM.
        // Instead, only global variables should be loaded into the global scope.
        // Constants being loaded into the VM would require relocation upon load time
        // However, it may be possible to do this for string interning.
        // https://stackoverflow.com/questions/10578984/what-is-java-string-interning
        // We should still store the string in the constant pool, but if the string literal exists in the heap already
        // We should reference that instead; live bytecode patching could be a possibility, rather than using the same
        // OP_CONSTANT bytecode fragment
        for _ in 0..const_pool_size {
            let marker = &chunk[offset..offset+2];
            debug!("marker: {:?}", marker);
            match marker {
                sli if sli == DOUBLE_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = chunk[offset..offset+8].try_into()?;
                    let float = f64::from_le_bytes(dbl);
                    debug!("float found in constant pool: {}", float);
                    self.constants.push(ThetaValue::Double(float));              
                    offset += 8;
                },
                sli if sli == INT_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = chunk[offset..offset+8].try_into()?;
                    let int = i64::from_le_bytes(dbl);
                    debug!("i64 found in constant pool: {}", int);
                    self.constants.push(ThetaValue::Int(int));              
                    offset += 8;
                },
                sli if sli == BOOL_MARKER => {
                    offset += 2;
                    let bol: [u8; 1] = chunk[offset..offset+1].try_into()?;
                    let bol = bol == [1u8];
                    debug!("bool found in constant pool: {}", bol);
                    self.constants.push(ThetaValue::Bool(bol));              
                    offset += 1;
                }
                _ => panic!("invalid marker found in chunk"),
            }
        }
        

        while offset < chunk.len() {
            // read into chunk
            match chunk[offset] {
                0x0 => { 
                    debug!("Op: Return (0x0)"); 
                    println!("{:?}", self.stack.pop()); 
                    offset += 1 
                },
                0x1 => { 
                    debug!("Op: Constant (0x1) with offset: {}", &chunk[offset+1]); 
                    self.stack.push(self.constants[chunk[offset+1] as usize].clone()); 
                    offset += 2 
                },
                0x2 => {
                    debug!("Op: Add (0x2)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l+r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Int(l+r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x3 => {
                    debug!("Op: Sub (0x3)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l-r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Int(l-r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x4 => {
                    debug!("Op: Mul (0x3)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l*r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Int(l*r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x5 => {
                    debug!("Op: Div (0x5)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l/r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Int(l/r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x6 => {
                    debug!("Op: Neg (0x6)");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match left {
                        ThetaValue::Double(l) => self.stack.push(ThetaValue::Double(-l)),
                        ThetaValue::Int(_) => todo!(),
                        _ => panic!("invalid operands")
                    };
                    offset += 1
                },
                0x7 => {
                    debug!("Op: Equal (0x7)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l==r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l==r)),
                        (ThetaValue::Bool(l), ThetaValue::Bool(r)) => self.stack.push(ThetaValue::Bool(l==r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x8 => {
                    debug!("Op: GT (0x8)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l>r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l>r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x9 => {
                    debug!("Op: LT (0x9)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l<r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l<r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                code => { 
                    debug!("Op: Unknown ({:#x})", code); 
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