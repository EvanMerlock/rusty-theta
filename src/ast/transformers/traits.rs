use std::{fmt::{self, Debug}, error::Error};

use crate::ast::{AbstractTree, Expression, Statement};

use super::{typeck::TypeCkError, to_bytecode::ToByteCodeError};

pub trait ASTTransformer<T> where T: Debug + PartialEq {

    type Out;

    fn transform(&self, tree: &AbstractTree<T>) -> Result<Self::Out, TransformError>;

}

pub trait ASTVisitor<T> where T: Debug + PartialEq {

    type InfoOut: Debug + PartialEq;

    fn visit_expression(&self, expr: &Expression<T>) -> Result<Expression<Self::InfoOut>, TransformError>;
    fn visit_statement(&self, stmt: &Statement<T>) -> Result<Statement<Self::InfoOut>, TransformError>;
}

pub trait ASTTerminator<T> where T: Debug + PartialEq {

    type Out;

    fn visit_expression(&self, expr: &Expression<T>) -> Result<Self::Out, TransformError>;
    fn visit_statement(&self, stmt: &Statement<T>) -> Result<Self::Out, TransformError>;


}

#[derive(Debug)]
pub enum TransformError {
    TypeCkError(TypeCkError),
    ToByteCodeError(ToByteCodeError),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::TypeCkError(type_ck_err) => write!(f, "An error occured during type checking: {}", type_ck_err),
            TransformError::ToByteCodeError(to_bytecode_err) => write!(f, "An error occured during bytecode generation: {}", to_bytecode_err),
        }
    }
}

impl Error for TransformError {}

impl From<TypeCkError> for TransformError {
    fn from(ck_error: TypeCkError) -> Self {
        TransformError::TypeCkError(ck_error)
    }
}

impl From<ToByteCodeError> for TransformError {
    fn from(err: ToByteCodeError) -> Self {
        TransformError::ToByteCodeError(err)
    }
}