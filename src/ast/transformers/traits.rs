use std::{fmt::{self, Debug}, error::Error};

use super::{AugmentedAbstractTree, AugmentedExpression, typeck::TypeCkError};

pub trait ASTTransformer<T> where T: Debug + PartialEq {

    type Out;

    fn transform(tree: &AugmentedAbstractTree<T>) -> Result<Self::Out, TransformError>;

}

pub trait ASTVisitor<T> where T: Debug + PartialEq {

    type Out;

    fn visit_expression(expr: &AugmentedExpression<T>) -> Result<Self::Out, TransformError>;
}

#[derive(Debug)]
pub enum TransformError {
    TypeCkError(TypeCkError),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::TypeCkError(type_ck_err) => write!(f, "An error occured during type checking: {}", type_ck_err),
        }
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

impl From<TypeCkError> for TransformError {
    fn from(ck_error: TypeCkError) -> Self {
        TransformError::TypeCkError(ck_error)
    }
}