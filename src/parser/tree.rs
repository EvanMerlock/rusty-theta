use crate::lexer::token::Token;

#[derive(Debug)]
pub struct AbstractTree(Expression);

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
    Grouping(Box<Expression>),
}