use std::{cell::RefCell, rc::Rc};


pub type ExtFrameData = Rc<RefCell<FrameData>>;


#[derive(Debug, Clone, PartialEq)]
pub struct FrameData {
    total_locals: usize,
}

impl FrameData {

    pub fn new() -> FrameData {
        FrameData { total_locals: 0 }
    }

    pub fn new_local(&mut self) -> usize {
        let li = self.total_locals;
        self.total_locals += 1;
        li
    }

    pub fn total_locals(&self) -> usize {
        self.total_locals
    }
}

impl Default for FrameData {
    fn default() -> Self {
        Self::new()
    }
}