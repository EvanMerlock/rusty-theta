use crate::{lexer::token::Token, bytecode::Symbol};
use std::fmt::Debug;

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
            InnerAbstractTree::Expression((_, info)) => &info,
            InnerAbstractTree::Statement((_, info)) => &info,
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

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<T> where T: Debug + PartialEq {
    Binary {
        left: Box<Expression<T>>,
        operator: Token,
        right: Box<Expression<T>>,
        information: T
    },
    Unary {
        operator: Token,
        right: Box<Expression<T>>,
        information: T
    },
    Literal {
        literal: Token,
        information: T
    },
    Sequence {
        seq: Vec<Expression<T>>,
        information: T
    },
    // TODO: name will become an lvalue
    Assignment {
        name: Symbol,
        value: Box<Expression<T>>,
        information: T
    },
    If {
        check_expression: Box<Expression<T>>,
        body: Box<Expression<T>>,
        else_body: Option<Box<Expression<T>>>,
        information: T,
    },
    BlockExpression {
        statements: Vec<Statement<T>>,
        information: T,
    }
}

impl<T: Debug + PartialEq> Expression<T> {
    pub fn information(&self) -> &T {
        match self {
            Expression::Binary { left: _, operator: _, right: _, information } => information,
            Expression::Unary { operator: _, right: _, information } => information,
            Expression::Literal { literal: _, information } => information,
            Expression::Sequence { seq: _, information } => information,
            Expression::Assignment { name: _, value: _, information } => information,
            Expression::If { check_expression: _, body: _, else_body: _, information } => information,
            Expression::BlockExpression { statements: _, information } => information,
        }
    }

    pub fn strip_information(self) -> Expression<()> {
        match self {
            Expression::Binary { left, operator, right, information: _ } => Expression::Binary { left: Box::new(left.strip_information()), operator, right: Box::new(right.strip_information()), information: () },
            Expression::Unary { operator, right, information: _ } => Expression::Unary { operator, right: Box::new(right.strip_information()), information: () },
            Expression::Literal { literal, information: _ } => Expression::Literal { literal, information: () },
            Expression::Sequence { seq, information: _ } => Expression::Sequence { seq: seq.into_iter().map(|x| x.strip_information()).collect(), information: () },
            Expression::Assignment { name, value, information: _ } => Expression::Assignment { name, value: Box::new(value.strip_information()), information: () },
            Expression::If { check_expression, body, else_body, information: _ } => Expression::If { check_expression: Box::new(check_expression.strip_information()), body: Box::new(body.strip_information()), else_body: else_body.map(|stmt| Box::new(stmt.strip_information())), information: () },
            Expression::BlockExpression { statements, information: _ } => { Expression::BlockExpression { statements: statements.into_iter().map(|x| x.strip_information()).collect(), information: () } },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<T> where T: Debug + PartialEq {
    ExpressionStatement {
        expression: Expression<T>,
        information: T
    },
    PrintStatement {
        expression: Expression<T>,
        information: T
    },
    VarStatement {
        ident: Symbol,
        init: Expression<T>,
        information: T,
    },
}

impl<T: Debug + PartialEq> Statement<T> {
    pub fn information(&self) -> &T {
        match self {
            Statement::ExpressionStatement { expression: _, information } => information,
            Statement::PrintStatement { information, expression: _ } => information,
            Statement::VarStatement { ident: _, init: _, information } => information,
        }
    }

    pub fn strip_information(self) -> Statement<()> {
        match self {
            Statement::ExpressionStatement { expression, information: _ } => Statement::ExpressionStatement { expression: expression.strip_information(), information: () },
            Statement::PrintStatement { expression, information: _ } => Statement::PrintStatement { expression: expression.strip_information(), information: () },
            Statement::VarStatement { ident, init, information: _ } => Statement::VarStatement { ident, init: init.strip_information(), information: () },
        }
    }
}