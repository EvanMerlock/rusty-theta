use std::{iter::Peekable, rc::Rc};
use log::{debug, error};

use crate::ast::{AbstractTree, Expression, Statement, symbol::{SymbolTable, SymbolData}, transformers::typeck::TypeInformation};
use super::Parser;
use crate::lexer::token::{Token, TokenType};


// TODO: PartialEq/Hash def. needs to only take into account raw token ident data.
// Maybe have ID contain String ref and LocData 
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Identifier {
    tk: Rc<String>
}

impl Identifier {
    pub fn new(tk: Token) -> Result<Identifier, super::ParseError> {
        match tk.ty() {
            TokenType::Identifier(s) => Ok(Identifier { tk: Rc::new(s) }),
            _ => Err(super::ParseError::from_other("Failed to assemble ident from token"))
        }
    }

    pub fn id(&self) -> &String {
        &self.tk
    }
}

impl From<&'static str> for Identifier {
    fn from(s: &'static str) -> Self {
        Identifier { tk: Rc::new(String::from(s)) }
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Identifier { tk: Rc::new(s) }
    }
}

pub struct BasicParser<'a> {
    tokens: Peekable<&'a mut dyn Iterator<Item = Token>>,
    symbol_tbl: SymbolTable,
}

// TODO: split up tokens that are operators into operator type by expression type.
impl<'a> BasicParser<'a> {
    pub fn new(token_stream: &'a mut dyn Iterator<Item = Token>) -> BasicParser<'a> {
        let symbol_table = SymbolTable::default();

        BasicParser {
            tokens: token_stream.peekable(),
            symbol_tbl: symbol_table,
        }
    }

    pub fn new_sym(token_stream: &'a mut dyn Iterator<Item = Token>, sym: SymbolTable) -> BasicParser<'a> {
        BasicParser { tokens: token_stream.peekable(), symbol_tbl: sym }
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

    fn consume_if(&mut self, cond: impl Fn(TokenType) -> bool, msg: &'static str) -> Result<Token, super::ParseError> {
        match self.peek() {
            Some(tok) if cond(tok.ty()) => self.advance().ok_or_else(|| super::ParseError::from_other("Unexpected EOS")),
            Some(tok) =>  Err(super::ParseError::from_token(tok.clone(), msg)),
            None => Err(super::ParseError::from_other("Unexpected EOS")),
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

    fn declaration(&mut self) -> Result<Statement, super::ParseError> {
        debug!("read declaration");
        let stmt = if let Some(_var_tok) = self.match_token([TokenType::Let]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match stmt {
            Ok(s) => Ok(s),
            Err(e) => {
                error!("Error occured during parsing: {}", e);
                debug!("Synchronizing and attempting to parse again");
                self.synchronize();
                Err(e)
            },
        }
    }

    fn var_declaration(&mut self) -> Result<Statement, super::ParseError> {
        debug!("read var declaration");
        let name = self.consume_if(|ty| ty.is_ident(), "Expected variable name")?;

        let mut init = None;
        let mut ty = None;
        if let Some(_colon_tok) = self.match_token([TokenType::Colon]) {
            ty = Some(self.consume_if(|ty| ty.is_ident(), "Expected variable type")?);
        }


        if let Some(_eq_tok) = self.match_token([TokenType::Equal]) {
            init = Some(self.expression()?);
        }
        if init.is_none() || ty.is_none() {
            return Err(super::ParseError::from_other("Expected expression and variable type"));
        }

        let ident = Identifier::new(name)?;
        let ty_ident = Identifier::new(ty.expect("big issue; ty existed prev but not now"))?;

        let ty_info = match self.symbol_tbl.get_symbol_data(&ty_ident) {
            Some(SymbolData::Type { ty }) => ty.clone(),
            Some(_) => return Err(super::ParseError::from_other("ident is being used by something else")),
            // assume forward declaration here. if the type continues to not be defined via ID, we will error on compilation.
            None => TypeInformation::NonLiteral(ty_ident.clone()),
        };
        self.consume(TokenType::Semicolon, "Expected ';' after statement")?;

        // TODO
        // if double `let` we should fail here, since that isn't valid (yet)
        // once a binding is created it can only be undone by exiting the scope the binding is valid in.
        // note that in a language with better bind semantics a rebind would be valid and could change the type of the variable
        self.symbol_tbl.insert_symbol(ident.clone(), SymbolData::Variable { ty: ty_info });

        Ok(Statement::VarStatement { ident, init: init.expect("big issue; init existed prev but now now"), information: () })
    }

    fn statement(&mut self) -> Result<Statement, super::ParseError> {
        debug!("read statement");
        if let Some(_print_tok) = self.match_token([TokenType::Identifier(String::from("print"))]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Statement, super::ParseError> {
        debug!("read print statement");
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::PrintStatement { expression, information: () })
    }

    fn expression_statement(&mut self) -> Result<Statement, super::ParseError> {
        debug!("read expression statement");
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::ExpressionStatement { expression, information: () })
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
                information: (),
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
                information: (),
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
                right: Box::new(rhs),
                information: (),
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
                right: Box::new(rhs),
                information: ()
            };
        }

        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expression, super::ParseError> {
        debug!("read unary");
        if let Some(oper) = self.match_token([TokenType::Bang, TokenType::Minus]) {
            self.unary().map(|rhs| Expression::Unary {
                operator: oper,
                right: Box::new(rhs),
                information: ()
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
            
            Ok(Expression::Sequence {
                seq: seq_expressions,
                information: (),
            })
        } else {
            // needs to match literals only
            self
                .advance()
                .filter(|tk| tk.ty().is_literal())
                .map(|tk| Expression::Literal { literal: tk, information: () })
                .ok_or_else(|| super::ParseError::from_other("Unexpected EOS"))
        }
    }
}

impl<'a> Parser for BasicParser<'a> {
    type Out = Result<(AbstractTree, SymbolTable), super::ParseError>;

    fn parse(mut self) -> Result<(AbstractTree, SymbolTable), super::ParseError> {
        self.declaration().map(|stmt| (AbstractTree::statement(stmt, ()), self.symbol_tbl))
    }
}