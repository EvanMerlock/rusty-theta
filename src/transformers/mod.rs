use std::{fmt, error::Error};

use crate::parser::tree::{AbstractTree, Expression};

mod to_bytecode;

trait ASTTransformer {

    type Out;

    fn transform(tree: AbstractTree) -> Result<Self::Out, TransformError>;

}

trait ASTVisitor {

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