use std::collections::{HashMap, hash_map::Entry};

use log::debug;

use crate::{parser::Identifier};

use super::transformers::typeck::TypeInformation;

#[derive(Debug, Clone)]
pub struct SymbolTable {
    entries: HashMap<Identifier, SymbolData>,
    children: Option<HashMap<Identifier, SymbolTable>>
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { entries: HashMap::new(), children: None }
    }

    pub fn insert_symbol(&mut self, tk: Identifier, data: SymbolData) {
        self.entries.insert(tk, data);
    }

    pub fn modify_symbol_data(&mut self, tk: Identifier) -> Entry<Identifier, SymbolData> {
        self.entries.entry(tk)
    }

    pub fn get_symbol_data(&self, tk: &Identifier) -> Option<&SymbolData> {
        self.entries.get(tk)
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

#[derive(Debug, Clone)]
pub enum SymbolData {
    Type {
        ty: TypeInformation,
    },
    Variable {
        ty: TypeInformation
    }
}

impl SymbolData {
    pub fn ty(&self) -> &TypeInformation {
        match self {
            SymbolData::Type { ty } => ty,
            SymbolData::Variable { ty } => ty,
        }
    }
}