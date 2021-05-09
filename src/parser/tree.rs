use crate::lexer::token::Token;

#[derive(Debug)]
pub struct AbstractTree(Expression);

impl AbstractTree {
    pub fn new(expression: Expression) -> AbstractTree {
        AbstractTree(expression)
    }
}

#[derive(Debug)]
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
    Sequence(Vec<Box<Expression>>),
}