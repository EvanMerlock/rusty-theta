use crate::{lexer::token::Token, bytecode::Symbol, parser::ParseInfo};
use std::fmt::Debug;

use super::Statement;


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
        final_expression: Option<Box<Expression<T>>>,
        information: T,
    },
    LoopExpression {
        predicate: Option<Box<Expression<T>>>,
        body: Box<Expression<T>>,
        information: T,
    },
    // TODO: name will become an lvalue
    Call {
        callee: Box<Expression<T>>,
        args: Vec<Expression<T>>,
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
            Expression::BlockExpression { statements: _, information, final_expression } => information,
            Expression::LoopExpression { predicate: _, body: _, information } => information,
            Expression::Call { callee: _, args: _, information } => information,
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
            Expression::BlockExpression { statements, information: _, final_expression } => { Expression::BlockExpression { statements: statements.into_iter().map(|x| x.strip_information()).collect(), information: (), final_expression: final_expression.map(|x| Box::new(x.strip_information())) } },
            Expression::LoopExpression { predicate, body, information: _ } => Expression::LoopExpression { predicate: predicate.map(|x| Box::new(x.strip_information())), body: Box::new(body.strip_information()), information: () },
            Expression::Call { callee: function, args, information: _ } => Expression::Call { callee: Box::new(function.strip_information()), args: args.into_iter().map(|x| x.strip_information()).collect(), information: () },
        }
    }

    pub fn strip_token_information(self) -> Expression<T> {
        match self {
            Expression::Binary { left, operator, right, information } => Expression::Binary { left: Box::new(left.strip_token_information()), operator: operator.strip_information(), right: Box::new(right.strip_token_information()), information },
            Expression::Unary { operator, right, information } => Expression::Unary { operator: operator.strip_information(), right: Box::new(right.strip_token_information()), information },
            Expression::Literal { literal, information } => Expression::Literal { literal: literal.strip_information(), information },
            Expression::Sequence { seq, information } => Expression::Sequence { seq: seq.into_iter().map(|x| x.strip_token_information()).collect(), information },
            Expression::Assignment { name, value, information } => Expression::Assignment { name, value: Box::new(value.strip_token_information()), information },
            Expression::If { check_expression, body, else_body, information } => Expression::If { check_expression: Box::new(check_expression.strip_token_information()), body: Box::new(body.strip_token_information()), else_body: else_body.map(|stmt| Box::new(stmt.strip_token_information())), information },
            Expression::BlockExpression { statements, information, final_expression } => Expression::BlockExpression { statements: statements.into_iter().map(|x| x.strip_token_information()).collect(), information, final_expression: final_expression.map(|x| Box::new(x.strip_token_information())) },
            Expression::LoopExpression { predicate, body, information } => Expression::LoopExpression { predicate: predicate.map(|x| Box::new(x.strip_token_information())), body: Box::new(body.strip_token_information()), information },
            Expression::Call { callee: function, args, information } => Expression::Call { callee: function, args: args.into_iter().map(|x| x.strip_token_information()).collect(), information },
        }
    }
}