use crate::{lexer::token::Token, parser::Identifier};
use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct AugmentedAbstractTree<T> where T: Debug + PartialEq {
    inner: InnerAbstractTree<T>
}

#[derive(Debug, PartialEq)]
pub enum InnerAbstractTree<T> where T: Debug + PartialEq {
    Expression((AugmentedExpression<T>, T)),
    Statement((AugmentedStatement<T>, T)),
}

impl<T> AugmentedAbstractTree<T> where T: Debug + PartialEq {
    pub fn expression(expression: AugmentedExpression<T>, information: T) -> AugmentedAbstractTree<T> {
        AugmentedAbstractTree {
            inner: InnerAbstractTree::Expression((expression, information))
        }
    }

    pub fn statement(statement: AugmentedStatement<T>, information: T) -> AugmentedAbstractTree<T> {
        AugmentedAbstractTree {
            inner: InnerAbstractTree::Statement((statement, information))
        }
    }

    pub fn inner(&self) -> &InnerAbstractTree<T> {
        &self.inner
    }
}

#[derive(Debug, PartialEq)]
pub enum AugmentedExpression<T> where T: Debug + PartialEq {
    Binary {
        left: Box<AugmentedExpression<T>>,
        operator: Token,
        right: Box<AugmentedExpression<T>>,
        information: T
    },
    Unary {
        operator: Token,
        right: Box<AugmentedExpression<T>>,
        information: T
    },
    Literal {
        literal: Token,
        information: T
    },
    Sequence {
        seq: Vec<AugmentedExpression<T>>,
        information: T
    },
}

impl<T: Debug + PartialEq> AugmentedExpression<T> {
    pub fn information(&self) -> &T {
        match self {
            AugmentedExpression::Binary { left, operator, right, information } => information,
            AugmentedExpression::Unary { operator, right, information } => information,
            AugmentedExpression::Literal { literal, information } => information,
            AugmentedExpression::Sequence { seq, information } => information,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AugmentedStatement<T> where T: Debug + PartialEq {
    ExpressionStatement {
        expression: AugmentedExpression<T>,
        information: T
    },
    PrintStatement {
        expression: AugmentedExpression<T>,
        information: T
    },
    VarStatement {
        ident: Identifier,
        init: AugmentedExpression<T>,
        information: T,
    }
}

impl<T: Debug + PartialEq> AugmentedStatement<T> {
    pub fn information(&self) -> &T {
        match self {
            AugmentedStatement::ExpressionStatement { expression, information } => information,
            AugmentedStatement::PrintStatement { information, expression } => information,
            AugmentedStatement::VarStatement { ident, init, information } => information,
        }
    }
}