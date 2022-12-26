use super::ASTTransformer;
use crate::ast::{AbstractTree, Expression};

pub struct TypeCk;

impl ASTTransformer for TypeCk {

    type Out = ();

    fn transform(tree: AbstractTree) -> Result<Self::Out, super::TransformError> {
        todo!()
    }

}