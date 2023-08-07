use std::fmt::Display;

use crate::bytecode::Symbol;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypeInformation {
    Int,
    String,
    Float,
    Boolean,
    NonLiteral(Symbol),
    Function(Box<TypeInformation>, Vec<TypeInformation>),
    None,
}

impl Display for TypeInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeInformation::Int => write!(f, "Int"),
            TypeInformation::String => write!(f, "String"),
            TypeInformation::Float => write!(f, "Float"),
            TypeInformation::Boolean => write!(f, "Boolean"),
            TypeInformation::NonLiteral(s) => write!(f, "{}", s),
            TypeInformation::None => write!(f, "!"),
            TypeInformation::Function(return_ty, args) => write!(f, "Fn({args:?}) -> {return_ty}"),
        }
    }
}