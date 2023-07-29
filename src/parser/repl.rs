use crate::{ast::{AbstractTree, Item}, lexer::token::TokenType};

use super::{BasicParser, Parser, ParseInfo, ParseError};

pub enum ReplItem {
    ParserItem(Item<ParseInfo>),
    Declaration(AbstractTree<ParseInfo>),
}


pub struct ReplParser<'a> {
    internal: BasicParser<'a>,
}

impl<'a> ReplParser<'a> {
    pub fn new(bp: BasicParser<'a>) -> ReplParser<'a> {
        ReplParser { internal: bp }
    }
}

impl <'a> Parser for ReplParser<'a> {
    type Out = ReplItem;

    fn parse(mut self) -> Result<Vec<Self::Out>, super::ParseError> {
        let mut trees = Vec::new();

        while !self.internal.is_at_end() {
            let item = self.next()?;
            trees.push(item);
        }

        Ok(trees)
    }

    fn next(&mut self) -> Result<Self::Out, super::ParseError> {
        match self.internal.peek() {
            Some(token) if token.ty() == TokenType::Fun => self.internal.next().map(ReplItem::ParserItem),
            Some(_token) => self.internal.declaration().map(|x| AbstractTree::statement(x.clone(), x.information().clone())).map(ReplItem::Declaration),
            None => Err(ParseError::from_other("no token found"))
        }
    }
    
}