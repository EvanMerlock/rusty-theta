use std::{cell::RefCell, rc::Rc};

use crate::ast::transformers::typeck::TypeInformation;


pub type ExtFrameData = Rc<RefCell<FrameData>>;


#[derive(Debug, Clone, PartialEq)]
pub struct FrameData {
    total_params: usize,
    total_locals: usize,

    /// We use this parameter to propagate return type information
    /// So that return expressions can check
    pub return_ty: Option<TypeInformation>,
}

impl FrameData {

    pub fn new() -> FrameData {
        FrameData { total_params: 0, total_locals: 0, return_ty: None }
    }

    pub fn new_local(&mut self) -> usize {
        // accounts for total params as the first slots
        let li = self.total_locals + self.total_params;
        self.total_locals += 1;
        li
    }

    pub fn total_locals(&self) -> usize {
        self.total_locals
    }

    pub fn new_function_variable(&mut self) -> usize {
        let li = self.total_params;
        self.total_params += 1;
        li
    }

    pub fn total_params(&self) -> usize {
        self.total_params
    }
}

impl Default for FrameData {
    fn default() -> Self {
        Self::new()
    }
}