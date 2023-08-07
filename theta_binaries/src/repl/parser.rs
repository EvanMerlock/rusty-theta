use theta_compiler::{ast::{Item, AbstractTree}, parser::{ParseInfo, BasicParser, Parser}};
use theta_types::{errors::parse::ParseError, bytecode::TokenType};


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

    fn parse(mut self) -> Result<Vec<Self::Out>, ParseError> {
        let mut trees = Vec::new();

        while !self.internal.is_at_end() {
            let item = self.next()?;
            trees.push(item);
        }

        Ok(trees)
    }

    fn next(&mut self) -> Result<Self::Out, ParseError> {
        match self.internal.peek() {
            Some(token) if token.ty() == TokenType::Fun => self.internal.next().map(ReplItem::ParserItem),
            Some(_token) => self.internal.declaration().map(|x| AbstractTree::statement(x.clone(), x.information().clone())).map(ReplItem::Declaration),
            None => Err(ParseError::from_other("no token found"))
        }
    }
    
}