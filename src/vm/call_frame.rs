use std::{collections::HashMap, hash::Hash};

use crate::bytecode::ThetaValue;

#[derive(Debug)]
pub struct ThetaStack {
    globals: HashMap<String, ThetaValue>,
    top_level_locals: Vec<ThetaValue>,
    // These frames should be colocated near each other to provide spatial locality.
    // However, that's not as easy as it sounds in Rust. I think a Vec<> should provide spatial locality
    // _within_ the Vec but I don't know the defined behavior.
    frames: Vec<ThetaCallFrame>,
}

impl ThetaStack {
    pub fn new() -> ThetaStack {
        ThetaStack { globals: HashMap::new(), top_level_locals: Vec::new(), frames: Vec::new() }
    }

    pub fn curr_frame(&self) -> Option<&ThetaCallFrame> {
        self.frames.last()
    }

    fn curr_frame_mut(&mut self) -> Option<&mut ThetaCallFrame> {
        self.frames.last_mut()
    }

    pub fn push_frame(&mut self, params: Vec<ThetaValue>) {
        self.frames.push(ThetaCallFrame::new(params));
    }

    pub fn pop_frame(&mut self) -> Option<ThetaCallFrame> {
        self.frames.pop()
    }

    pub fn push(&mut self, loc: ThetaValue) {
        match self.curr_frame_mut() {
            Some(frame) => frame.locals.push(loc),
            None => self.top_level_locals.push(loc),
        }
    }

    pub fn pop(&mut self) -> Option<ThetaValue> {
        match self.curr_frame_mut() {
            Some(frame) => frame.locals.pop(),
            None => self.top_level_locals.pop(),
        }
    }

    pub fn peek(&self) -> Option<&ThetaValue> {
        match self.curr_frame() {
            Some(frame) => frame.locals.get(frame.locals.len()-1),
            None => self.top_level_locals.get(self.top_level_locals.len()-1)
        }
    }

    pub fn globals(&self) -> &HashMap<String, ThetaValue> {
        &self.globals
    }

    pub fn globals_mut(&mut self) -> &mut HashMap<String, ThetaValue> {
        &mut self.globals
    }
}

#[derive(Debug)]
pub struct ThetaCallFrame {
    // In THEORY the size of params and locals is always well known by the compiler in advanced
    // We would need a custom allocating collection in order to do this.
    // This has extra redirection
    params: Vec<ThetaValue>,
    rip: usize,
    locals: Vec<ThetaValue>,
}

impl ThetaCallFrame {
    pub fn new(params: Vec<ThetaValue>) -> ThetaCallFrame {
        ThetaCallFrame { params, rip: 0, locals: Vec::new() }
    }
}