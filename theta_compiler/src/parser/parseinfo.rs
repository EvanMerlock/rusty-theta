use theta_types::types::LocationData;

use crate::ast::symbol::{ExtSymbolTable, ExtFrameData};

#[derive(Debug, PartialEq, Clone)]
pub struct ParseInfo {
    pub scope_depth: usize,
    pub current_symbol_table: ExtSymbolTable,
    pub frame_data: ExtFrameData,
    pub location_data: LocationData,
}

impl ParseInfo {
    pub fn new(sd: usize, curr_sym: ExtSymbolTable, curr_frame: ExtFrameData, loc: LocationData) -> ParseInfo {
        ParseInfo { scope_depth: sd, current_symbol_table: curr_sym, frame_data: curr_frame, location_data: loc }
    }
}