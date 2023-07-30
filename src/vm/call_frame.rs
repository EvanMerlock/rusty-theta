use std::{collections::HashMap, rc::Rc};

use crate::bytecode::{ThetaValue, ThetaCompiledBitstream};

#[derive(Debug)]
pub struct ThetaStack {
    globals: HashMap<String, ThetaValue>,
    // These frames should be colocated near each other to provide spatial locality.
    // However, that's not as easy as it sounds in Rust. I think a Vec<> should provide spatial locality
    // _within_ the Vec but I don't know the defined behavior.
    frames: Vec<ThetaCallFrame>,
}

impl ThetaStack {
    pub fn new(root_bitstream: Rc<ThetaCompiledBitstream>) -> ThetaStack {
        ThetaStack { globals: HashMap::new(), frames: vec![ThetaCallFrame::new(root_bitstream, vec![])] }
    }

    pub fn curr_frame(&self) -> Option<&ThetaCallFrame> {
        self.frames.last()
    }

    fn curr_frame_mut(&mut self) -> Option<&mut ThetaCallFrame> {
        self.frames.last_mut()
    }

    pub fn push_frame(&mut self, bitstream_ref: Rc<ThetaCompiledBitstream>, params: Vec<ThetaValue>) {
        self.frames.push(ThetaCallFrame::new(bitstream_ref, params));
    }

    pub fn push_opt_frame(&mut self, bitstream_ref: Rc<ThetaCompiledBitstream>, params: Vec<Option<ThetaValue>>) {
        self.frames.push(ThetaCallFrame::new_optional(bitstream_ref, params));
    }

    pub fn pop_frame(&mut self) -> Option<ThetaCallFrame> {
        self.frames.pop()
    }

    pub fn alloc_framespace(&mut self, size: usize) {
        match self.curr_frame_mut() {
            Some(frame) => frame.locals.resize(frame.locals.len() + size, None),
            None => todo!()
        }
    }

    pub fn push(&mut self, loc: ThetaValue) {
        // TODO: because this is called when a constant is loaded we need
        // to track all constants to ensure enough stack space is allocated
        match self.curr_frame_mut() {
            Some(frame) => frame.locals.push(Some(loc)),
            None => panic!("all frames gone"),
        }
    }

    pub fn pop(&mut self) -> Option<ThetaValue> {
        match self.curr_frame_mut() {
            Some(frame) => ThetaStack::flatten_stackval(frame.locals.pop()),
            None => panic!("all frames gone"),
        }
    }

    pub fn peek(&self) -> Option<&ThetaValue> {
        match self.curr_frame() {
            Some(frame) => ThetaStack::flatten_refstackval(frame.locals.last()),
            None => panic!("all frames gone")
        }
    }

    pub fn set_local(&mut self, li: usize) {
        // TODO: we should have an instruction as a prefix to allocate stack space
        // this can set a local into a slot with no local in it. we need to allocate empty space in the vec on the fly
        let tv = self.peek().expect("value not on stack").clone();
        match self.curr_frame_mut() {
            Some(frame) => frame.locals[li] = Some(tv),
            None => panic!("all frames gone"),
        }
    }

    pub fn get_local(&self, li: usize) -> Option<&ThetaValue> {
        match self.curr_frame() {
            Some(frame) => ThetaStack::flatten_refstackval(frame.locals.get(li)),
            None => panic!("all frames gone")
        }
    }

    pub fn globals(&self) -> &HashMap<String, ThetaValue> {
        &self.globals
    }

    pub fn globals_mut(&mut self) -> &mut HashMap<String, ThetaValue> {
        &mut self.globals
    }

    fn flatten_stackval(val: Option<Option<ThetaValue>>) -> Option<ThetaValue> {
        val.flatten()
    }

    fn flatten_refstackval(val: Option<&Option<ThetaValue>>) -> Option<&ThetaValue> {
        val.unwrap_or(&None).as_ref()
    }

    pub fn set_bitstream(&mut self, bitstream_ref: Rc<ThetaCompiledBitstream>) {
        self.frames.last_mut().expect("no stack frame").bitstream = bitstream_ref;
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ThetaCallFrame {
    rip: usize,
    // locals are uninitalized when created. we might need to make this an option and panic on fail
    pub locals: Vec<Option<ThetaValue>>,
    pub bitstream: Rc<ThetaCompiledBitstream>,
}

impl ThetaCallFrame {
    pub fn new(bitstream_ref: Rc<ThetaCompiledBitstream>, params: Vec<ThetaValue>) -> ThetaCallFrame {
        ThetaCallFrame { rip: 0, locals: params.into_iter().map(|x| Some(x)).collect(), bitstream: bitstream_ref }
    }

    pub fn new_optional(bitstream_ref: Rc<ThetaCompiledBitstream>, params: Vec<Option<ThetaValue>>) -> ThetaCallFrame {
        ThetaCallFrame { rip: 0, locals: params, bitstream: bitstream_ref }
    }
}