use std::collections::HashMap;

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

    pub fn push_frame(&mut self, params: Vec<ThetaValue>, locals_required: usize) {
        self.frames.push(ThetaCallFrame::new(params, locals_required));
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
            Some(frame) => frame.locals.last(),
            None => self.top_level_locals.last()
        }
    }

    pub fn set_local(&mut self, li: usize) {
        // this can set a local into a slot with no local in it. we need to allocate empty space in the vec on the fly
        let tv = self.peek().expect("value not on stack").clone();
        match self.curr_frame_mut() {
            Some(frame) => frame.locals[li] = tv,
            None => self.top_level_locals[li] = tv,
        }
    }

    pub fn get_local(&self, li: usize) -> Option<&ThetaValue> {
        match self.curr_frame() {
            Some(frame) => frame.locals.get(li),
            None => self.top_level_locals.get(li)
        }
    }

    pub fn globals(&self) -> &HashMap<String, ThetaValue> {
        &self.globals
    }

    pub fn globals_mut(&mut self) -> &mut HashMap<String, ThetaValue> {
        &mut self.globals
    }
}

impl Default for ThetaStack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ThetaCallFrame {
    // In THEORY the size of params and locals is always well known by the compiler in advanced
    // We would need a custom allocating collection in order to do this.
    // This has extra redirection
    params: Vec<ThetaValue>,
    rip: usize,
    // locals are uninitalized when created. we might need to make this an option and panic on fail
    locals: Vec<ThetaValue>,
}

impl ThetaCallFrame {
    pub fn new(params: Vec<ThetaValue>, locals_required: usize) -> ThetaCallFrame {
        ThetaCallFrame { params, rip: 0, locals: Vec::with_capacity(locals_required) }
    }
}