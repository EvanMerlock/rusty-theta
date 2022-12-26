use std::iter::Peekable;
use log::debug;

use crate::ast::{AbstractTree, Expression};
use super::Parser;
use crate::lexer::token::{Token, TokenType};

pub struct BasicParser<'a> {
    tokens: Peekable<&'a mut dyn Iterator<Item = Token>>
}

impl<'a> BasicParser<'a> {
    pub fn new(token_stream: &'a mut dyn Iterator<Item = Token>) -> BasicParser<'a> {
        BasicParser {
            tokens: token_stream.peekable()
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn is_at_end(&mut self) -> bool {
        match self.peek() {
            Some(tok) => tok.ty() == TokenType::Eof,
            None => true
        }
    }

    fn consume(&mut self, tt: TokenType, msg: &'static str) -> Result<Token, super::ParseError> {
        if self.check(&tt) {
            self.advance().ok_or_else(|| super::ParseError::from_other("Unexpected EOS"))
        } else {
            match self.peek() {
                Some(tok) => Err(super::ParseError::from_token(tok.clone(), msg)),
                None => Err(super::ParseError::from_other("Unexpected EOS"))
            }
        }
    }

    fn check(&mut self, t_ty: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().map(|tt| tt.ty() == *t_ty).unwrap_or(false)
        }
    }

    fn match_token<const U: usize>(&mut self, tokens: [TokenType; U]) -> Option<Token> {
        for tok in &tokens {
            if self.check(tok) {
                return self.advance();
            }
        }

        None
    }

    fn synchronize(&mut self) {
        let mut tok = self.advance();

        while !self.is_at_end() {
            match tok {
                Some(ref inner) => {
                    if inner.ty() == TokenType::Semicolon {
                        return;
                    }
                }
                None => return
            }

            match self.peek().map(|t| t.ty()) {
                Some(TokenType::Class) => return,
                Some(TokenType::Fun) => return,
                Some(TokenType::Let) => return,
                Some(TokenType::For) => return,
                Some(TokenType::If) => return,
                Some(TokenType::While) => return,
                Some(TokenType::Return) => return,
                Some(_) => {},
                None => return,
            }

            tok = self.advance();
        }
    }

    fn expression(&mut self) -> Result<Expression, super::ParseError> {
        debug!("read expression");
        self.equality() 
    }

    fn equality(&mut self) -> Result<Expression, super::ParseError> {
        debug!("read equality");
        let mut lhs = self.comparison()?;

        while let Some(oper) = self.match_token([TokenType::BangEqual, TokenType::EqualEqual]) {
            let rhs = self.comparison()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
            };
        };
        
        Ok(lhs)
    }

    fn comparison(&mut self) -> Result<Expression, super::ParseError> {
        debug!("read comparison");
        let mut lhs = self.term()?;

        while let Some(oper) = self.match_token([TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let rhs = self.term()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn term(&mut self) -> Result<Expression, super::ParseError> {
        debug!("read term");
        let mut lhs = self.factor()?;

        while let Some(oper) = self.match_token([TokenType::Minus, TokenType::Plus]) {
            let rhs = self.factor()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs)
            };
        }

        Ok(lhs)
    }

    fn factor(&mut self) -> Result<Expression, super::ParseError> {
        debug!("read factor");
        let mut lhs = self.unary()?;

        while let Some(oper) = self.match_token([TokenType::Star, TokenType::Slash]) {
            let rhs = self.unary()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs)
            };
        }

        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expression, super::ParseError> {
        debug!("read unary");
        if let Some(oper) = self.match_token([TokenType::Bang, TokenType::Minus]) {
            self.unary().map(|rhs| Expression::Unary {
                operator: oper,
                right: Box::new(rhs)
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expression, super::ParseError> {
        debug!("read primary");
        if self.match_token([TokenType::LeftParen]).is_some() {
            debug!("read seq");
            let mut seq_expressions = vec![];
            let mut inner = self.expression()?;

            seq_expressions.push(inner);

            while self.match_token([TokenType::Semicolon]).is_some() {
                inner = self.expression()?;
                seq_expressions.push(inner);
            };

            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            
            Ok(Expression::Sequence(seq_expressions))
        } else {
            // needs to match literals only
            self
                .advance()
                .filter(|tk| tk.ty().is_literal())
                .map(Expression::Literal)
                .ok_or_else(|| super::ParseError::from_other("Unexpected EOS"))
        }
    }
}

impl<'a> Parser for BasicParser<'a> {
    type Out = Result<AbstractTree, super::ParseError>;

    fn parse(mut self) -> Result<AbstractTree, super::ParseError> {
        self.expression().map(AbstractTree::new)
    }
}