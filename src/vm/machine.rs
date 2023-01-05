use std::{rc::Rc, collections::{HashMap, HashSet}};

use log::debug;

use crate::bytecode::{CHUNK_HEADER, CONSTANT_POOL_HEADER, INT_MARKER, DOUBLE_MARKER, BOOL_MARKER, ThetaValue, Disassembler, DisassembleError, ThetaHeapValue, STRING_MARKER, ThetaString};

use super::call_frame::ThetaStack;

// TODO: can we snapshot the VM using CoW?
// probably not, but what happens if an instruction fails due to bad input data?
pub struct VM {
    stack: ThetaStack,
    constants: Vec<ThetaValue>,
    strings: HashMap<ThetaString, Rc<ThetaHeapValue>>,
    heap: Vec<Rc<ThetaHeapValue>>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: ThetaStack::new(),
            strings: HashMap::new(),
            constants: Vec::new(),
            heap: Vec::new(),
        }
    }

    pub fn strings(&self) -> &HashMap<ThetaString, Rc<ThetaHeapValue>> {
        &self.strings
    }

    pub fn stack(&self) -> &ThetaStack {
        &self.stack
    }

    pub fn constants(&self) -> &Vec<ThetaValue> {
        &self.constants
    }

    pub fn heap(&self) -> &Vec<Rc<ThetaHeapValue>> {
        &self.heap
    }

    pub fn globals(&self) -> &HashMap<String, ThetaValue> {
        self.stack.globals()
    }

    pub fn clear_const_pool(&mut self) {
        self.constants.clear()
    }

    pub fn intern_string(&mut self, s_val: ThetaString) -> ThetaValue {
        let hv = match self.strings.get(&s_val) {
            Some(rc) => rc.clone(),
            None => { 
                let rc = Rc::new(ThetaHeapValue::Str(s_val.clone()));
                self.strings.insert(s_val, rc.clone());
                self.heap.push(rc.clone());
                rc
            },
        };
        ThetaValue::HeapValue(hv)
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
                },
                sli if sli == STRING_MARKER => {
                    offset += 2;
                    let len_bytes: [u8; 8] = chunk[offset..offset+8].try_into()?;
                    let len = usize::from_le_bytes(len_bytes);
                    offset += 8;
                    let in_str = &chunk[offset..offset+len];
                    let mut bytes = Vec::new();
                    bytes.extend_from_slice(in_str);
                    let read_str = String::from_utf8(bytes)?;
                    debug!("str found in constant pool: {}", read_str);
                    debug!("checking for memoized string");
                    let s_val = ThetaString::new(read_str);
                    let tv = self.intern_string(s_val);
                    offset += len;
                    self.constants.push(tv);
                }
                _ => return Err(DisassembleError::InvalidMarkerInChunk(marker.to_vec())),
            }
        }

        debug!("-- BEGIN INSTRUCTIONS --");
        

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
                    debug!("Op: Push (0x2)"); 
                    todo!();
                    offset += 1 
                },
                0x3 => { 
                    debug!("Op: Pop (0x3)"); 
                    self.stack.pop(); 
                    offset += 1 
                },
                0x4 => {
                    debug!("Op: Add (0x4)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l+r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Int(l+r)),
                        (ThetaValue::HeapValue(l), ThetaValue::HeapValue(r)) => {
                            match (&*l, &*r) {
                                (&ThetaHeapValue::Str(ref ls), &ThetaHeapValue::Str(ref rs)) => {
                                    let s_val = ls.clone() + rs;
                                    let tv = self.intern_string(s_val);                              
                                    self.stack.push(tv);
                                },
                            }
                        }
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x5 => {
                    debug!("Op: Sub (0x5)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l-r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Int(l-r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x6 => {
                    debug!("Op: Mul (0x6)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l*r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Int(l*r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x7 => {
                    debug!("Op: Div (0x7)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l/r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Int(l/r)),
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0x8 => {
                    debug!("Op: Neg (0x8)");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match left {
                        ThetaValue::Double(l) => self.stack.push(ThetaValue::Double(-l)),
                        ThetaValue::Int(_) => todo!(),
                        _ => panic!("invalid operands")
                    };
                    offset += 1
                },
                0x9 => {
                    debug!("Op: Equal (0x9)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l==r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l==r)),
                        (ThetaValue::Bool(l), ThetaValue::Bool(r)) => self.stack.push(ThetaValue::Bool(l==r)),
                        (ThetaValue::HeapValue(l), ThetaValue::HeapValue(r)) => {
                            match (&*l, &*r) {
                                (&ThetaHeapValue::Str(ref ls), &ThetaHeapValue::Str(ref rs)) => self.stack.push(ThetaValue::Bool(ls==rs)),
                            }
                        }
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0xA => {
                    debug!("Op: GT (0xA)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l>r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l>r)),
                        (ThetaValue::HeapValue(l), ThetaValue::HeapValue(r)) => {
                            match (&*l, &*r) {
                                (&ThetaHeapValue::Str(ref ls), &ThetaHeapValue::Str(ref rs)) => self.stack.push(ThetaValue::Bool(ls>rs)),
                            }
                        }
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0xB => {
                    debug!("Op: LT (0xB)");
                    let right = self.stack.pop().expect("failed to grab value off stack");
                    let left = self.stack.pop().expect("failed to grab value off stack");

                    match (left, right) {
                        (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l<r)),
                        (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l<r)),
                        (ThetaValue::HeapValue(l), ThetaValue::HeapValue(r)) => {
                            match (&*l, &*r) {
                                (&ThetaHeapValue::Str(ref ls), &ThetaHeapValue::Str(ref rs)) => self.stack.push(ThetaValue::Bool(ls<rs)),
                            }
                        }
                        _ => panic!("invalid operands"),
                    };
                    offset += 1
                },
                0xC0 => { 
                    debug!("Op: Define Global (0xC0) with offset: {}", chunk[offset+1] as usize);
                    let glob = self.constants[chunk[offset+1] as usize].clone();
                    match glob {
                        ThetaValue::HeapValue(hv) => {
                            match &*hv {
                                ThetaHeapValue::Str(s) => {
                                    let sv = self.stack.peek().expect("no value on stack").clone();
                                    self.stack.globals_mut().insert(s.to_string(), sv);
                                    self.stack.pop();
                                },
                            }
                        },
                        _ => panic!("Define Global with no HV")
                    }
                    offset += 2
                },
                0xC1 => { 
                    debug!("Op: Read Global (0xC1)");
                    let glob = self.constants[chunk[offset+1] as usize].clone();
                    match glob {
                        ThetaValue::HeapValue(hv) => {
                            match &*hv {
                                ThetaHeapValue::Str(s) => {
                                    let v = self.stack.globals_mut().get(s.internal().as_str());
                                    let v2 = v.expect("no such constant").clone();
                                    self.stack.push(v2);
                                },
                            }
                        },
                        _ => panic!("Read Global with no HV")
                    }
                    offset += 2
                },
                0xC2 => { 
                    debug!("Op: Define Local (0xC0) with offset: {}", chunk[offset+1] as usize);
                    let li = chunk[offset+1] as usize;
                    self.stack.set_local(li);
                    offset += 2
                },
                0xC3 => { 
                    debug!("Op: Read Local (0xC1) with offset: {}", chunk[offset+1] as usize);
                    let li = chunk[offset+1] as usize;
                    self.stack.push(self.stack.get_local(li).expect("local does not exist when it should").clone());
                    offset += 2
                },
                0xFF => { 
                    debug!("Op: Print (0xFF)"); 
                    println!("{:?}", self.stack.pop()); 
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