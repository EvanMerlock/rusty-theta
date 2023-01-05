use std::rc::Rc;

use crate::{lexer::token::{Token, TokenType}, parser::ParseError};

// TODO: PartialEq/Hash def. needs to only take into account raw token ident data.
// Maybe have ID contain String ref and LocData 
// TODO: Rename to "Symbol" and move to it's own package.
// prior art: Ruby's symbols: https://ruby-doc.org/core-2.5.0/Symbol.html
// this makes more logical sense. We can also intern IDs like we would intern strings in the VM.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Symbol {
    tk: Rc<String>
}

impl Symbol {
    pub fn new(tk: Token) -> Result<Symbol, ParseError> {
        match tk.ty() {
            TokenType::Identifier(s) => Ok(Symbol { tk: Rc::new(s) }),
            _ => Err(ParseError::from_other("Failed to assemble ident from token"))
        }
    }

    pub fn id(&self) -> &String {
        &self.tk
    }
}

impl From<&'static str> for Symbol {
    fn from(s: &'static str) -> Self {
        Symbol { tk: Rc::new(String::from(s)) }
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Symbol { tk: Rc::new(s) }
    }
}