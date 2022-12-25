use std::{fmt, error::Error};

use crate::parser::tree::{AbstractTree, Expression};

pub mod to_bytecode;

pub trait ASTTransformer {

    type Out;

    fn transform(tree: AbstractTree) -> Result<Self::Out, TransformError>;

}

pub trait ASTVisitor {

    type Out;

    fn visit_expression(expr: Expression) -> Result<Self::Out, TransformError>;
}

#[derive(Debug)]
pub enum TransformError {

}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Error for TransformError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}