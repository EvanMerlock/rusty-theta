use crate::ast::symbol::{ExtSymbolTable, ExtFrameData};

#[derive(Debug, PartialEq, Clone)]
pub struct ParseInfo {
    pub scope_depth: usize,
    pub current_symbol_table: ExtSymbolTable,
    pub frame_data: ExtFrameData,
}

impl ParseInfo {
    pub fn new(sd: usize, curr_sym: ExtSymbolTable, curr_frame: ExtFrameData) -> ParseInfo {
        ParseInfo { scope_depth: sd, current_symbol_table: curr_sym, frame_data: curr_frame }
    }
}