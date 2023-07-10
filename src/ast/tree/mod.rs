use std::fmt::Debug;

mod expression;
mod statement;

pub use self::expression::*;
pub use self::statement::*;

#[derive(Debug, PartialEq, Clone)]
pub struct AbstractTree<T> where T: Debug + PartialEq {
    inner: InnerAbstractTree<T>
}

#[derive(Debug, PartialEq, Clone)]
pub enum InnerAbstractTree<T> where T: Debug + PartialEq {
    Expression((Expression<T>, T)),
    Statement((Statement<T>, T)),
}

impl<T> AbstractTree<T> where T: Debug + PartialEq {
    pub fn information(&self) -> &T {
        match &self.inner {
            InnerAbstractTree::Expression((_, info)) => info,
            InnerAbstractTree::Statement((_, info)) => info,
        }
    }

    pub fn expression(expression: Expression<T>, information: T) -> AbstractTree<T> {
        AbstractTree {
            inner: InnerAbstractTree::Expression((expression, information))
        }
    }

    pub fn statement(statement: Statement<T>, information: T) -> AbstractTree<T> {
        AbstractTree {
            inner: InnerAbstractTree::Statement((statement, information))
        }
    }

    pub fn inner(&self) -> &InnerAbstractTree<T> {
        &self.inner
    }

    pub fn strip_information(self) -> AbstractTree<()> {
        match self.inner {
            InnerAbstractTree::Expression((exp, _)) => AbstractTree::expression(exp.strip_information(), ()),
            InnerAbstractTree::Statement((stmt, _)) => AbstractTree::statement(stmt.strip_information(), ()),
        }
    }
}