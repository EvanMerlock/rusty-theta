use std::fmt::Debug;

mod expression;
mod statement;
mod function;

pub use self::expression::*;
pub use self::statement::*;
pub use self::function::*;

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

    pub fn strip_token_information(self) -> AbstractTree<T> {
        match self.inner {
            InnerAbstractTree::Expression((exp, info)) => AbstractTree::expression(exp.strip_token_information(), info),
            InnerAbstractTree::Statement((stmt, info)) => AbstractTree::statement(stmt.strip_token_information(), info),
        }
    }
}


#[derive(Debug)]
pub enum Item<T> where T: Debug + PartialEq {
    Function(Function<T>),
}

impl<T> Item<T> where T: Debug + PartialEq {
    pub fn information(&self) -> &T {
        match self {
            Item::Function(func) => &func.information,
        }
    }
}