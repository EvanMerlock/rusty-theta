use crate::ast::symbol::ExtSymbolTable;

#[derive(Debug, PartialEq, Clone)]
pub struct ParseInfo {
    pub scope_depth: usize,
    pub current_symbol_table: ExtSymbolTable,
}

impl ParseInfo {
    pub fn new(sd: usize, curr_sym: ExtSymbolTable) -> ParseInfo {
        ParseInfo { scope_depth: sd, current_symbol_table: curr_sym }
    }
}