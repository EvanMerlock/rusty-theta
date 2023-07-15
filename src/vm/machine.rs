use std::{rc::Rc, collections::HashMap};

use log::{debug, error};

use crate::bytecode::{CHUNK_HEADER, CONSTANT_POOL_HEADER, INT_MARKER, DOUBLE_MARKER, BOOL_MARKER, ThetaValue, Disassembler, DisassembleError, ThetaHeapValue, STRING_MARKER, ThetaString, ThetaBitstream, BITSTREAM_HEADER, BasicDisassembler};

use super::call_frame::ThetaStack;

// TODO: can we snapshot the VM using CoW?
// probably not, but what happens if an instruction fails due to bad input data?
// TODO: separate disassembler from execution. disassembly only needs to happen once per input
// and thus will take less time than execution except on REPL.
pub struct VM {
    stack: ThetaStack,
    constants: Vec<ThetaValue>,
    strings: HashMap<ThetaString, Rc<ThetaHeapValue>>,
    heap: Vec<Rc<ThetaHeapValue>>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: ThetaStack::new(Rc::new(ThetaBitstream::new())),
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

impl VM {
    pub fn execute_code(&mut self, chunk: &[u8]) -> Result<(), DisassembleError> {
        let mut offset: usize = 18;

        debug!("chunk: {:X?}", chunk);

        // assert chunk header
        assert!(chunk[0..8] == CHUNK_HEADER);
        
        debug!("=== BEGIN CHUNK ===");

        let mut basic = BasicDisassembler::new(&|tv| {
            self.intern_string(tv)
        });

        let constants = basic.disassemble_constant_pool(&chunk[8..])?;
        self.constants = constants;

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
                    debug!("Op: Constant (0x1) with offset: {:#X}", &chunk[offset+1]); 
                    self.stack.push(self.constants[chunk[offset+1] as usize].clone()); 
                    offset += 2 
                },
                0x2 => { 
                    debug!("Op: Push (0x2) with inc size {:#X}", chunk[offset+1]);
                    let stack_inc_size = usize::from_le_bytes(chunk[offset+1..offset+9].try_into().expect("8 ele slice not converted"));
                    // TODO: what the heck is this supposed to push onto the stack?
                    // Should it take in a heap value and push a pointer to that heap value to the stack? 
                    self.stack.alloc_framespace(stack_inc_size);
                    offset += 1 + std::mem::size_of::<usize>()
                },
                0x3 => { 
                    debug!("Op: Pop (0x3)"); 
                    let pot = self.stack.pop();
                    debug!("Popped from top of stack: {pot:?}");
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
                    debug!("Op: Define Global (0xC0) with offset: {:#X}", chunk[offset+1] as usize);
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
                    debug!("Op: Define Local (0xC0) with offset: {:#X}", chunk[offset+1] as usize);
                    let li = chunk[offset+1] as usize;
                    self.stack.set_local(li);
                    offset += 2
                },
                0xC3 => { 
                    debug!("Op: Read Local (0xC1) with offset: {:#X}", chunk[offset+1] as usize);
                    let li = chunk[offset+1] as usize;
                    self.stack.push(self.stack.get_local(li).expect("local does not exist when it should").clone());
                    offset += 2
                },
                0xD0 => {
                    debug!("Op: Jump Unconditional (0xD0) with offset: {:#X}", chunk[offset+1] as usize);
                    let local_jump_point = chunk[offset+1] as i8;
                    let (new_off, overflow) = offset.overflowing_add_signed(local_jump_point as isize);
                    if overflow {
                        panic!()
                    }
                    offset = new_off;
                },
                0xD1 => {
                    debug!("Op: Jump If False Local (0xD1) with offset: {:#X}", chunk[offset+1] as usize);
                    let local_jump_point = chunk[offset+1] as i8;

                    // this op should not pop off the stack, we should instead emit an instruction to do that.
                    match self.stack.peek() {
                        Some(ThetaValue::Bool(false)) => {
                            debug!("jumping because top of stack is false");
                            let (new_off, overflow) = offset.overflowing_add_signed(local_jump_point as isize);
                            if overflow {
                                panic!()
                            }
                            offset = new_off;
                        },
                        Some(ThetaValue::Bool(_)) => {
                            debug!("not jumping, top of stack is not false");
                            offset += 2;
                        },
                        _ => {
                            error!("top of stack non-existent on JMPIFF instruction");
                            panic!()
                        }
                    }
                },
                0xD2 => {
                    debug!("Op: Jump Unconditional Far (0xD2) with offset: {:#X}", chunk[offset+1] as usize);
                    let local_jump_point = isize::from_le_bytes(chunk[offset+1..offset+9].try_into().expect("8 ele slice not converted"));
                    let (new_off, overflow) = offset.overflowing_add_signed(local_jump_point);
                    if overflow {
                        panic!()
                    }
                    offset = new_off;                
                },
                0xD3 => {
                    debug!("Op: Jump If False Far (0xD3) with offset: {:#X}", chunk[offset+1] as usize);
                    let local_jump_point = isize::from_le_bytes(chunk[offset+1..offset+9].try_into().expect("8 ele slice not converted"));

                    // this op should not pop off the stack, we should instead emit an instruction to do that.
                    match self.stack.peek() {
                        Some(ThetaValue::Bool(false)) => {
                            debug!("jumping because top of stack is false");
                            let (new_off, overflow) = offset.overflowing_add_signed(local_jump_point);
                            if overflow {
                                panic!()
                            }
                            offset = new_off;                        
                        },
                        Some(ThetaValue::Bool(_)) => {
                            debug!("not jumping, top of stack is not false");
                            offset += 2;
                        },
                        _ => {
                            error!("top of stack non-existent on JMPIFF instruction");
                            panic!()
                        }
                    }
                },
                0xFD => {
                    debug!("Op: Noop (0xFD)");
                    offset += 1
                }
                0xFF => { 
                    debug!("Op: Print (0xFF)"); 
                    println!("{:?}", self.stack.pop()); 
                    offset += 1
                },
                code => { 
                    debug!("Op: Unknown ({:#x})", code); 
                    panic!("unknown");
                    offset += 1
                }
            }
        }

        Ok(())
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}