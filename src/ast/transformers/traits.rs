use std::{fmt::{self, Debug}, error::Error};

use crate::{ast::{AbstractTree, Expression, Statement, Function, tree::Item}, bytecode::ThetaFunction};

use super::{typeck::TypeCkError, to_bytecode::ToByteCodeError};

pub trait ASTTransformer<T> where T: Debug + PartialEq {

    type ItemOut;
    type TreeOut;

    fn transform_item(&self, item: &Item<T>) -> Result<Self::ItemOut, TransformError>;
    fn transform_tree(&self, tree: &AbstractTree<T>) -> Result<Self::TreeOut, TransformError>;

}

pub trait ASTVisitor<T> where T: Debug + PartialEq {

    type InfoOut: Debug + PartialEq;

    fn visit_function(&self, func: &Function<T>) -> Result<Function<Self::InfoOut>, TransformError>;
    fn visit_expression(&self, expr: &Expression<T>) -> Result<Expression<Self::InfoOut>, TransformError>;
    fn visit_statement(&self, stmt: &Statement<T>) -> Result<Statement<Self::InfoOut>, TransformError>;
}

pub trait ASTTerminator<T> where T: Debug + PartialEq {

    type ChunkOut;

    fn visit_function(&self, func: &Function<T>) -> Result<ThetaFunction, TransformError>;
    fn visit_expression(&self, expr: &Expression<T>) -> Result<Self::ChunkOut, TransformError>;
    fn visit_statement(&self, stmt: &Statement<T>) -> Result<Self::ChunkOut, TransformError>;


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