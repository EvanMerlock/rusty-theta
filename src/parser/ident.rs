use std::rc::Rc;

use crate::lexer::token::{Token, TokenType};

// TODO: PartialEq/Hash def. needs to only take into account raw token ident data.
// Maybe have ID contain String ref and LocData 
// TODO: Rename to "Symbol" and move to it's own package.
// prior art: Ruby's symbols: https://ruby-doc.org/core-2.5.0/Symbol.html
// this makes more logical sense. We can also intern IDs like we would intern strings in the VM.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Identifier {
    tk: Rc<String>
}

impl Identifier {
    pub fn new(tk: Token) -> Result<Identifier, super::ParseError> {
        match tk.ty() {
            TokenType::Identifier(s) => Ok(Identifier { tk: Rc::new(s) }),
            _ => Err(super::ParseError::from_other("Failed to assemble ident from token"))
        }
    }

    pub fn id(&self) -> &String {
        &self.tk
    }
}

impl From<&'static str> for Identifier {
    fn from(s: &'static str) -> Self {
        Identifier { tk: Rc::new(String::from(s)) }
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Identifier { tk: Rc::new(s) }
    }
}