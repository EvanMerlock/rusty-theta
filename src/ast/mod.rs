use crate::lexer::token::Token;

pub mod transformers;

#[derive(Debug, PartialEq)]
pub enum AbstractTree {
    Expression(Expression),
}

impl AbstractTree {
    pub fn new(expression: Expression) -> AbstractTree {
        AbstractTree::Expression(expression)
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Unary {
        operator: Token,
        right: Box<Expression>,
    },
    Literal(Token),
    Sequence(Vec<Expression>),
}