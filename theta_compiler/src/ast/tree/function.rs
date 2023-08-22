use std::fmt::Debug;

use theta_types::{bytecode::Symbol, types::TypeInformation};

use super::AbstractTree;



#[derive(Debug, PartialEq, Clone)]
pub struct Function<T> where T: Debug + PartialEq {
    pub args: Vec<FunctionArg>,
    pub chunk: AbstractTree<T>,
    pub name: Symbol,
    pub return_ty: TypeInformation,
    pub information: T,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct FunctionArg {
    pub name: Symbol,
    pub ty: TypeInformation,
}

impl <T: Debug + PartialEq> Function<T> {
    pub fn information(&self) -> &T {
        &self.information
    }

    pub fn strip_information(self) -> Function<()> {
        Function { args: self.args, chunk: self.chunk.strip_information(), name: self.name, return_ty: self.return_ty, information: () }
    }

    pub fn strip_token_information(self) -> Function<T> {
        Function { args: self.args, chunk: self.chunk.strip_token_information(), name: self.name, return_ty: self.return_ty, information: self.information }
    }

    pub fn map_information<V: Debug + PartialEq>(self, map_fn: &dyn Fn(T) -> V) -> Function<V> {
        Function { args: self.args, chunk: self.chunk.map_information(map_fn), name: self.name, return_ty: self.return_ty, information: map_fn(self.information) }
    }
}