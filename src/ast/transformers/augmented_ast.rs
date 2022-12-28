use crate::lexer::token::Token;
use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct AugmentedAbstractTree<T> where T: Debug + PartialEq {
    inner: InnerAbstractTree<T>
}

#[derive(Debug, PartialEq)]
pub enum InnerAbstractTree<T> where T: Debug + PartialEq {
    Expression((AugmentedExpression<T>, T)),
}

impl<T> AugmentedAbstractTree<T> where T: Debug + PartialEq {
    pub fn new(expression: AugmentedExpression<T>, information: T) -> AugmentedAbstractTree<T> {
        AugmentedAbstractTree {
            inner: InnerAbstractTree::Expression((expression, information))
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
    pub fn information(&self) ->&T {
        match self {
            AugmentedExpression::Binary { left, operator, right, information } => information,
            AugmentedExpression::Unary { operator, right, information } => information,
            AugmentedExpression::Literal { literal, information } => information,
            AugmentedExpression::Sequence { seq, information } => information,
        }
    }
}
