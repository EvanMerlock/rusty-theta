use self::transformers::{AugmentedAbstractTree, AugmentedExpression, AugmentedStatement};

pub mod transformers;

pub type AbstractTree = AugmentedAbstractTree<()>;
pub type Expression = AugmentedExpression<()>;
pub type Statement = AugmentedStatement<()>;