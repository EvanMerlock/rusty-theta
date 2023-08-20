use std::{rc::Rc, collections::HashMap, io::Write};

use log::{debug, error};
use theta_types::bytecode::{ThetaString, ThetaHeapValue, ThetaCompiledBitstream, ThetaCompiledFunction, ThetaValue, DisassembleError, CHUNK_HEADER, Chunk};

use super::{call_frame::ThetaStack, ThetaCallFrame};

// TODO: can we snapshot the VM using CoW?
// probably not, but what happens if an instruction fails due to bad input data?
pub struct VM {
    current_offset: usize,
    current_chunk: Rc<Vec<u8>>,
    stdout: Box<dyn Write>,
    stack: ThetaStack,
    strings: HashMap<ThetaString, Rc<ThetaHeapValue>>,
    heap: Vec<Rc<ThetaHeapValue>>,
    loaded_bitstreams: Vec<Rc<ThetaCompiledBitstream>>,
    function_table: HashMap<ThetaString, (ThetaCompiledFunction, Rc<ThetaCompiledBitstream>)>,
}

impl VM {
    pub fn new(stdout: Box<dyn Write>) -> VM {
        VM {
            current_offset: 0,
            current_chunk: Rc::default(),
            stdout,
            stack: ThetaStack::new(),
            strings: HashMap::new(),
            heap: Vec::new(),
            loaded_bitstreams: vec![],
            function_table: HashMap::new(),
        }
    }

    pub fn strings(&self) -> &HashMap<ThetaString, Rc<ThetaHeapValue>> {
        &self.strings
    }

    pub fn stack(&self) -> &ThetaStack {
        &self.stack
    }

    pub fn heap(&self) -> &Vec<Rc<ThetaHeapValue>> {
        &self.heap
    }

    pub fn globals(&self) -> &HashMap<String, ThetaValue> {
        self.stack.globals()
    }

    pub fn constants(&self) -> &Vec<ThetaValue> {
        &self.stack.curr_frame().expect("stack should always have frame").bitstream.constants
    }

    pub fn functions(&self) -> &HashMap<ThetaString, (ThetaCompiledFunction, Rc<ThetaCompiledBitstream>)> {
        &self.function_table
    }

    pub fn bitstreams(&self) -> &Vec<Rc<ThetaCompiledBitstream>> {
        &self.loaded_bitstreams
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
        ThetaValue::Pointer(hv)
    }

    pub fn load_bitstream(&mut self, bs: ThetaCompiledBitstream) -> Rc<ThetaCompiledBitstream> {
        let loaded_bs = Rc::new(bs);
        self.loaded_bitstreams.push(loaded_bs.clone());

        // TODO: this should not copy the functions.
        for func in loaded_bs.functions() {
            self.function_table.insert(func.name.clone(), (func.clone(), loaded_bs.clone()));
        }

        // self.stack.set_bitstream(loaded_bs.clone());
        loaded_bs
    }

    pub fn push_frame(&mut self, sf: ThetaCallFrame) {
        self.stack.push_raw_frame(sf);
    }
}

impl VM {
    pub fn execute_code(&mut self) -> Result<(), DisassembleError> {
        let (chunk, new_offset) = self.page_chunk();
        let mut cont = true;
        self.current_offset = new_offset;
        self.current_chunk = chunk;

        while self.current_offset < self.current_chunk.len() && cont {
            // read into chunk
            cont = self.execute_line()?;
        }

        Ok(())
    }

    #[inline(always)]
    pub fn execute_line(&mut self) -> Result<bool, DisassembleError> {
        match self.current_chunk[self.current_offset] {
            0x0 => { 
                debug!("Op: Void Return (0x0)");
                // correct offset and load chunk
                self.current_offset = self.stack.pop_frame().expect("expected stack frame").rip;
                // end control
                match self.stack().curr_frame() {
                    Some(frame) => self.current_chunk = frame.chunk.clone(),
                    None => return Ok(false),
                };
            },
            0xF0 => {
                debug!("Op: Return (0xF0)");
                let sv = self.stack.pop().expect("expected value on top of stack for return");
                debug!("{:?}", sv);
                // correct offset and load chunk
                self.current_offset = self.stack.pop_frame().expect("expected stack frame").rip;
                match self.stack().curr_frame() {
                    Some(frame) => self.current_chunk = frame.chunk.clone(),
                    None => return Ok(false),
                };
                // load return val onto the stack
                self.stack.curr_frame_mut().expect("expected stack frame").locals.push(Some(sv));
            }
            0x1 => { 
                debug!("Op: Constant (0x1) with offset: {:#X}", &self.current_chunk[self.current_offset+1]); 
                let constant = self.stack.curr_frame().expect("expected stack frame").bitstream.constants[self.current_chunk[self.current_offset+1] as usize].clone();
                self.stack.push(constant); 
                self.current_offset += 2 
            },
            0x2 => { 
                debug!("Op: Push (0x2) with inc size {:#X}", self.current_chunk[self.current_offset+1]);
                let stack_inc_size = usize::from_le_bytes(self.current_chunk[self.current_offset+1..self.current_offset+9].try_into().expect("8 ele slice not converted"));
                self.stack.alloc_framespace(stack_inc_size);
                self.current_offset += 1 + std::mem::size_of::<usize>()
            },
            0x3 => { 
                debug!("Op: Pop (0x3)"); 
                let pot = self.stack.pop();
                debug!("Popped from top of stack: {pot:?}");
                self.current_offset += 1 
            },
            0x4 => {
                debug!("Op: Add (0x4)");
                let right = self.stack.pop().expect("failed to grab value off stack");
                let left = self.stack.pop().expect("failed to grab value off stack");

                match (left, right) {
                    (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Double(l+r)),
                    (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Int(l+r)),
                    (ThetaValue::Pointer(l), ThetaValue::Pointer(r)) => {
                        match (&*l, &*r) {
                            (ThetaHeapValue::Str(ls), ThetaHeapValue::Str(ref rs)) => {
                                let s_val = ls.clone() + rs;
                                let tv = self.intern_string(s_val);                              
                                self.stack.push(tv);
                            },
                        }
                    }
                    _ => panic!("invalid operands"),
                };
                self.current_offset += 1
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
                self.current_offset += 1
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
                self.current_offset += 1
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
                self.current_offset += 1
            },
            0x8 => {
                debug!("Op: Neg (0x8)");
                let left = self.stack.pop().expect("failed to grab value off stack");

                match left {
                    ThetaValue::Double(l) => self.stack.push(ThetaValue::Double(-l)),
                    ThetaValue::Int(_) => todo!(),
                    _ => panic!("invalid operands")
                };
                self.current_offset += 1
            },
            0x9 => {
                debug!("Op: Equal (0x9)");
                let right = self.stack.pop().expect("failed to grab value off stack");
                let left = self.stack.pop().expect("failed to grab value off stack");

                match (left, right) {
                    (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l==r)),
                    (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l==r)),
                    (ThetaValue::Bool(l), ThetaValue::Bool(r)) => self.stack.push(ThetaValue::Bool(l==r)),
                    (ThetaValue::Pointer(l), ThetaValue::Pointer(r)) => {
                        match (&*l, &*r) {
                            (ThetaHeapValue::Str(ls), ThetaHeapValue::Str(rs)) => self.stack.push(ThetaValue::Bool(ls==rs)),
                        }
                    }
                    _ => panic!("invalid operands"),
                };
                self.current_offset += 1
            },
            0xA => {
                debug!("Op: GT (0xA)");
                let right = self.stack.pop().expect("failed to grab value off stack");
                let left = self.stack.pop().expect("failed to grab value off stack");

                match (left, right) {
                    (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l>r)),
                    (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l>r)),
                    (ThetaValue::Pointer(l), ThetaValue::Pointer(r)) => {
                        match (&*l, &*r) {
                            (ThetaHeapValue::Str(ls), ThetaHeapValue::Str(rs)) => self.stack.push(ThetaValue::Bool(ls>rs)),
                        }
                    }
                    _ => panic!("invalid operands"),
                };
                self.current_offset += 1
            },
            0xA1 => {
                debug!("Op: GTE (0xA1)");
                let right = self.stack.pop().expect("failed to grab value off stack");
                let left = self.stack.pop().expect("failed to grab value off stack");

                match (left, right) {
                    (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l>=r)),
                    (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l>=r)),
                    (ThetaValue::Pointer(l), ThetaValue::Pointer(r)) => {
                        match (&*l, &*r) {
                            (ThetaHeapValue::Str(ls), ThetaHeapValue::Str(rs)) => self.stack.push(ThetaValue::Bool(ls>rs)),
                        }
                    }
                    _ => panic!("invalid operands"),
                };
                self.current_offset += 1
            },
            0xB => {
                debug!("Op: LT (0xB)");
                let right = self.stack.pop().expect("failed to grab value off stack");
                let left = self.stack.pop().expect("failed to grab value off stack");

                match (left, right) {
                    (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l<r)),
                    (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l<r)),
                    (ThetaValue::Pointer(l), ThetaValue::Pointer(r)) => {
                        match (&*l, &*r) {
                            (ThetaHeapValue::Str(ls), ThetaHeapValue::Str(rs)) => self.stack.push(ThetaValue::Bool(ls<rs)),
                        }
                    }
                    _ => panic!("invalid operands"),
                };
                self.current_offset += 1
            },
            0xB1 => {
                debug!("Op: LTE (0xB1)");
                let right = self.stack.pop().expect("failed to grab value off stack");
                let left = self.stack.pop().expect("failed to grab value off stack");

                match (left, right) {
                    (ThetaValue::Double(l), ThetaValue::Double(r)) => self.stack.push(ThetaValue::Bool(l<=r)),
                    (ThetaValue::Int(l), ThetaValue::Int(r)) => self.stack.push(ThetaValue::Bool(l<=r)),
                    (ThetaValue::Pointer(l), ThetaValue::Pointer(r)) => {
                        match (&*l, &*r) {
                            (ThetaHeapValue::Str(ls), ThetaHeapValue::Str(rs)) => self.stack.push(ThetaValue::Bool(ls<rs)),
                        }
                    }
                    _ => panic!("invalid operands"),
                };
                self.current_offset += 1
            },
            0xC0 => { 
                debug!("Op: Define Global (0xC0) with offset: {:#X}", self.current_chunk[self.current_offset+1] as usize);
                let glob = self.stack.curr_frame().expect("expected stack frame").bitstream.constants[self.current_chunk[self.current_offset+1] as usize].clone();
                match glob {
                    ThetaValue::Pointer(hv) => {
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
                self.current_offset += 2
            },
            0xC1 => { 
                debug!("Op: Read Global (0xC1)");
                let glob = self.stack.curr_frame().expect("expected stack frame").bitstream.constants[self.current_chunk[self.current_offset+1] as usize].clone();
                match glob {
                    ThetaValue::Pointer(hv) => {
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
                self.current_offset += 2
            },
            0xC2 => { 
                debug!("Op: Define Local (0xC2) with offset: {:#X}", self.current_chunk[self.current_offset+1] as usize);
                let li = self.current_chunk[self.current_offset+1] as usize;
                self.stack.set_local(li);
                self.current_offset += 2
            },
            0xC3 => { 
                debug!("Op: Read Local (0xC3) with offset: {:#X}", self.current_chunk[self.current_offset+1] as usize);
                let li = self.current_chunk[self.current_offset+1] as usize;
                self.stack.push(self.stack.get_local(li).expect("local does not exist when it should").clone());
                self.current_offset += 2
            },
            0xD0 => {
                debug!("Op: Jump Unconditional (0xD0) with offset: {:#X}", self.current_chunk[self.current_offset+1] as usize);
                let local_jump_point = self.current_chunk[self.current_offset+1] as i8;
                let (new_off, overflow) = self.current_offset.overflowing_add_signed(local_jump_point as isize);
                if overflow {
                    panic!()
                }
                self.current_offset = new_off;
            },
            0xD1 => {
                debug!("Op: Jump If False Local (0xD1) with offset: {:#X}", self.current_chunk[self.current_offset+1] as usize);
                let local_jump_point = self.current_chunk[self.current_offset+1] as i8;

                // this op should not pop off the stack, we should instead emit an instruction to do that.
                match self.stack.peek() {
                    Some(ThetaValue::Bool(false)) => {
                        debug!("jumping because top of stack is false");
                        let (new_off, overflow) = self.current_offset.overflowing_add_signed(local_jump_point as isize);
                        if overflow {
                            panic!()
                        }
                        self.current_offset = new_off;
                    },
                    Some(ThetaValue::Bool(_)) => {
                        debug!("not jumping, top of stack is not false");
                        self.current_offset += 2;
                    },
                    _ => {
                        error!("top of stack non-existent on JMPIFF instruction");
                        panic!()
                    }
                }
            },
            0xD2 => {
                debug!("Op: Jump Unconditional Far (0xD2) with offset: {:#X}", self.current_chunk[self.current_offset+1] as usize);
                let local_jump_point = isize::from_le_bytes(self.current_chunk[self.current_offset+1..self.current_offset+9].try_into().expect("8 ele slice not converted"));
                let (new_off, overflow) = self.current_offset.overflowing_add_signed(local_jump_point);
                if overflow {
                    panic!()
                }
                self.current_offset = new_off;                
            },
            0xD3 => {
                debug!("Op: Jump If False Far (0xD3) with offset: {:#X}", self.current_chunk[self.current_offset+1] as usize);
                let local_jump_point = isize::from_le_bytes(self.current_chunk[self.current_offset+1..self.current_offset+9].try_into().expect("8 ele slice not converted"));

                // this op should not pop off the stack, we should instead emit an instruction to do that.
                match self.stack.peek() {
                    Some(ThetaValue::Bool(false)) => {
                        debug!("jumping because top of stack is false");
                        let (new_off, overflow) = self.current_offset.overflowing_add_signed(local_jump_point);
                        if overflow {
                            panic!()
                        }
                        self.current_offset = new_off;                        
                    },
                    Some(ThetaValue::Bool(_)) => {
                        debug!("not jumping, top of stack is not false");
                        self.current_offset += 2;
                    },
                    _ => {
                        error!("top of stack non-existent on JMPIFF instruction");
                        panic!()
                    }
                }
            },
            0xE0 => {
                debug!("Op: Call Direct (0xE0) with offset: {:#X}", &self.current_chunk[self.current_offset+1]);
                // on top of the stack should be either a function object or a symbol reference
                let stack_top = self.stack.pop().expect("expected stack item");
                // let constant = self.stack.curr_frame().expect("expected stack frame").bitstream.constants[chunk[offset+1] as usize].clone();

                // TODO: function object
                let func_name = match stack_top {
                    ThetaValue::Pointer(hv) => match hv.as_ref() {
                        ThetaHeapValue::Str(func_name) => func_name.clone(),
                    },
                    _ => panic!("non-string found at constant for func call")
                };

                // TODO: this should throw a runtime error
                let func = self.function_table.get(&func_name).expect("function is not loaded");

                let stack_size: usize = func.0.args.len();
                let locals = &mut self.stack.curr_frame_mut().expect("need call frame").locals;
                // cut out the params from the current stack
                let params = locals.split_off(locals.len()-stack_size);

                self.current_offset += 2;
                self.stack.push_opt_frame(self.current_offset, func.1.clone(), func.0.chunk.clone(), params);
                // let ck = func.0.chunk.clone();
                // self.execute_code(&ck)?;
                // self.stack.pop_frame();
                (self.current_chunk, self.current_offset) = self.page_chunk();
            }
            0xFD => {
                debug!("Op: Noop (0xFD)");
                self.current_offset += 1
            }
            0xFF => { 
                debug!("Op: Print (0xFF)"); 
                writeln!(self.stdout, "{:?}", self.stack.pop())?; 
                self.current_offset += 1
            },
            code => { 
                debug!("Op: Unknown ({:#x})", code); 
                panic!("unknown");
                self.current_offset += 1
            }
        };

        Ok(true)
    }

    #[inline(always)]
    fn page_chunk(&mut self) -> (Rc<Vec<u8>>, usize) {
        let chunk = self.stack.curr_frame().expect("no frame").chunk.clone();
        let mut offset = 0;

        debug!("chunk: {:X?}", chunk);

        // assert chunk header
        assert!(chunk[0..8] == CHUNK_HEADER);

        offset += 8;
        
        debug!("=== BEGIN CHUNK ===");

        let chunk_size: usize = usize::from_le_bytes(chunk[offset..offset+8].try_into().expect("could not get chunk size"));
        debug!("chunk size: {chunk_size}");

        offset += 8;

        debug!("-- BEGIN INSTRUCTIONS --");

        (chunk, offset)
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(Box::new(std::io::stdout()))
    }
}