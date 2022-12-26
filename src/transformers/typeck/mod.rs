use super::ASTTransformer;

pub struct TypeCk;

impl ASTTransformer for TypeCk {

    type Out = ();

    fn transform(tree: crate::parser::tree::AbstractTree) -> Result<Self::Out, super::TransformError> {
        todo!()
    }

}