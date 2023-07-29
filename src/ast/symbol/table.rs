use std::{collections::{HashMap, hash_map::Entry}, rc::Rc, cell::RefCell};

use crate::{bytecode::Symbol};

use super::super::transformers::typeck::TypeInformation;


pub type ExtSymbolTable = Rc<RefCell<SymbolTable>>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct SymbolKey(usize, Symbol);

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolTable {
    entries: HashMap<SymbolKey, SymbolData>,
    enclosing: Option<ExtSymbolTable>,
    scope_depth: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { entries: HashMap::new(), enclosing: None, scope_depth: 0 }
    }

    pub fn new_enclosed(enclosing: ExtSymbolTable) -> SymbolTable {
        SymbolTable { entries: HashMap::new(), enclosing: Some(enclosing), scope_depth: 0 }
    }

    pub fn insert_symbol(&mut self, tk: Symbol, data: SymbolData) -> usize {
        self.entries.insert(SymbolKey(self.scope_depth, tk), data);
        self.scope_depth
    }

    pub fn modify_symbol_data(&mut self, key: SymbolKey) -> Entry<SymbolKey, SymbolData> {
        self.entries.entry(key)
    }

    pub fn scope_depth(&self) -> usize {
        let mut sd = 0;
        let mut enc = self.enclosing.clone();

        while let Some(enc_tbl) = enc {
            sd += 1;
            enc = enc_tbl.borrow().enclosing.clone();
        };

        sd
    }

    pub fn enclosing(&self) -> Option<ExtSymbolTable> {
        self.enclosing.clone()
    }

    pub fn get_symbol_data(&self, tk: &Symbol, sd: usize) -> Option<SymbolData> {
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
        symbol_table.insert_symbol(Symbol::from("Int"), SymbolData::Type { ty: TypeInformation::Int });
        symbol_table.insert_symbol(Symbol::from("String"), SymbolData::Type { ty: TypeInformation::String });
        symbol_table.insert_symbol(Symbol::from("Bool"), SymbolData::Type { ty: TypeInformation::Boolean });
        symbol_table.insert_symbol(Symbol::from("Float"), SymbolData::Type { ty: TypeInformation::Float });
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
    },
    Function {
        return_ty: TypeInformation,
        args: Vec<TypeInformation>,
        fn_ty: TypeInformation
    }
}

impl SymbolData {
    pub fn ty(&self) -> &TypeInformation {
        match self {
            SymbolData::Type { ty } => ty,
            SymbolData::GlobalVariable { ty } => ty,
            SymbolData::LocalVariable { ty, slot: _, scope_level: _ } => ty,
            SymbolData::Function { return_ty: _, args: _, fn_ty } => fn_ty,
        }
    }
}