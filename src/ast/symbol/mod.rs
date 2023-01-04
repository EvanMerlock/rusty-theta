use std::{collections::{HashMap, hash_map::Entry}, rc::Rc, cell::RefCell};

use log::debug;

use crate::{parser::Identifier};

use super::transformers::typeck::TypeInformation;

pub type ExtSymbolTable = Rc<RefCell<SymbolTable>>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct SymbolKey(usize, Identifier);

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolTable {
    entries: HashMap<SymbolKey, SymbolData>,
    enclosing: Option<ExtSymbolTable>,
    scope_depth: usize,
    total_locals: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { entries: HashMap::new(), enclosing: None, scope_depth: 0, total_locals: 0 }
    }

    pub fn new_enclosed(enclosing: ExtSymbolTable) -> SymbolTable {
        SymbolTable { entries: HashMap::new(), enclosing: Some(enclosing), scope_depth: 0, total_locals: 0 }
    }

    pub fn insert_symbol(&mut self, tk: Identifier, data: SymbolData) -> usize {
        self.entries.insert(SymbolKey(self.scope_depth, tk), data);
        self.scope_depth
    }

    pub fn modify_symbol_data(&mut self, key: SymbolKey) -> Entry<SymbolKey, SymbolData> {
        self.entries.entry(key)
    }

    pub fn inc_scope_depth(&mut self) {
        self.scope_depth += 1
    }

    pub fn scope_depth(&self) -> usize {
        self.scope_depth
    }

    pub fn dec_scope_depth(&mut self) {
        if self.scope_depth > 0 {
            self.scope_depth -= 1;
        } else {
            panic!("bad scope depth dec");
        }
    }

    pub fn new_local(&mut self) -> usize {
        let li = self.total_locals;
        self.total_locals += 1;
        li
    }

    pub fn total_locals(&self) -> usize {
        self.total_locals
    }

    pub fn get_symbol_data(&self, tk: &Identifier, sd: usize) -> Option<SymbolData> {
        // chain upward through the environment as necessary.
        match self.entries.get(&SymbolKey(sd, tk.clone())) {
            Some(ent) => Some(ent.clone()),
            None => {
                if sd == 0 {
                    match &self.enclosing {
                        // THIS WILL NOT WORK IN ALL CASES.
                        // WE NEED TO FIND A BETTER WAY TO HANDLE ENCLOSING ENVIRONMENTS.
                        Some(enc) => enc.borrow().get_symbol_data(tk, sd),
                        None => None,
                    }
                } else {
                    // base case; sd == 0
                    self.get_symbol_data(tk, sd-1)
                }

            }
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        let mut symbol_table = Self::new();
        symbol_table.insert_symbol(Identifier::from("Int"), SymbolData::Type { ty: TypeInformation::Int });
        symbol_table.insert_symbol(Identifier::from("String"), SymbolData::Type { ty: TypeInformation::String });
        symbol_table.insert_symbol(Identifier::from("Bool"), SymbolData::Type { ty: TypeInformation::Boolean });
        symbol_table.insert_symbol(Identifier::from("Float"), SymbolData::Type { ty: TypeInformation::Float });
        symbol_table
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolData {
    Type {
        ty: TypeInformation,
    },
    GlobalVariable {
        ty: TypeInformation
    },
    LocalVariable {
        ty: TypeInformation,
        scope_level: usize,
        slot: usize,
    }
}

impl SymbolData {
    pub fn ty(&self) -> &TypeInformation {
        match self {
            SymbolData::Type { ty } => ty,
            SymbolData::GlobalVariable { ty } => ty,
            SymbolData::LocalVariable { ty, slot: _, scope_level: _ } => ty,
        }
    }
}