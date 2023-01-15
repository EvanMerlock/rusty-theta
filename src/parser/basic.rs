use std::{iter::Peekable, rc::Rc, cell::RefCell};
use log::{debug, error};

use crate::{ast::{symbol::{SymbolTable, SymbolData, ExtSymbolTable}, transformers::{typeck::TypeInformation}, Statement, Expression, AbstractTree}, bytecode::Symbol};
use super::{Parser, ParseInfo, ParseError};
use crate::lexer::token::{Token, TokenType};

pub struct BasicParser<'a> {
    tokens: Peekable<&'a mut dyn Iterator<Item = Token>>,
    symbol_tbl: ExtSymbolTable,
    root_symbol_tbl: ExtSymbolTable,
    // used for functions
    // NOT NECESSARY as sym table follows AST tree now and is in RC
    // symbol_tables: Vec<ExtSymbolTable>,

    // TODO: we need some form of phased memory allocation for the compiler
    // but we need to allow some entities, like SymbolTables, to transcend phases.
}

// TODO: split up tokens that are operators into operator type by expression type.
impl<'a> BasicParser<'a> {
    pub fn new(token_stream: &'a mut dyn Iterator<Item = Token>) -> BasicParser<'a> {
        let symbol_table = Rc::new(RefCell::new(SymbolTable::default()));

        BasicParser {
            tokens: token_stream.peekable(),
            symbol_tbl: symbol_table.clone(),
            root_symbol_tbl: symbol_table,
        }
    }

    pub fn new_sym(token_stream: &'a mut dyn Iterator<Item = Token>, sym: ExtSymbolTable) -> BasicParser<'a> {
        BasicParser { 
            tokens: token_stream.peekable(), 
            symbol_tbl: sym.clone(),
            root_symbol_tbl: sym,
        }
    }

    fn begin_scope(&mut self) {
        self.symbol_tbl.borrow_mut().inc_scope_depth();
    }

    fn end_scope(&mut self) -> Result<(), ParseError> {
        self.symbol_tbl.borrow_mut().dec_scope_depth()
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

    fn consume(&mut self, tt: TokenType, msg: &'static str) -> Result<Token, ParseError> {
        if self.check(&tt) {
            self.advance().ok_or_else(|| ParseError::from_other("Unexpected EOS"))
        } else {
            match self.peek() {
                Some(tok) => Err(ParseError::from_token(tok.clone(), msg)),
                None => Err(ParseError::from_other("Unexpected EOS"))
            }
        }
    }

    fn consume_if(&mut self, cond: impl Fn(TokenType) -> bool, msg: &'static str) -> Result<Token, ParseError> {
        match self.peek() {
            Some(tok) if cond(tok.ty()) => self.advance().ok_or_else(|| ParseError::from_other("Unexpected EOS")),
            Some(tok) =>  Err(ParseError::from_token(tok.clone(), msg)),
            None => Err(ParseError::from_other("Unexpected EOS")),
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

    fn declaration(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
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

    fn var_declaration(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
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
            return Err(ParseError::from_other("Expected expression and variable type"));
        }

        let ident = Symbol::new(name)?;
        let ty_ident = Symbol::new(ty.expect("big issue; ty existed prev but not now"))?;

        let ty_info = match self.symbol_tbl.borrow().get_symbol_data(&ty_ident, self.symbol_tbl.borrow().scope_depth()) {
            Some(SymbolData::Type { ty }) => ty,
            Some(_) => return Err(ParseError::from_other("ident is being used by something else")),
            // assume forward declaration here. if the type continues to not be defined via ID, we will error on compilation.
            None => TypeInformation::NonLiteral(ty_ident.clone()),
        };
        self.consume(TokenType::Semicolon, "Expected ';' after statement")?;

        // TODO
        // if double `let` we should fail here, since that isn't valid (yet)
        // once a binding is created it can only be undone by exiting the scope the binding is valid in.
        // note that in a language with better bind semantics a rebind would be valid and could change the type of the variable
        let scope_depth = {
            // hacky work around because we're using a RefCell here
            self.symbol_tbl.borrow().scope_depth()
        };
        match scope_depth {
            0 => {
                self.symbol_tbl.borrow_mut().insert_symbol(ident.clone(), SymbolData::GlobalVariable { ty: ty_info });
            },
            sd => {
                // symbol table does not do debouncing of symbols yet. need a combination of scope_depth and identifier to ensure variables cannot collide.
                let li = self.symbol_tbl.borrow_mut().new_local();
                // slot determined by # of seen variables
                self.symbol_tbl.borrow_mut().insert_symbol(ident.clone(), SymbolData::LocalVariable { ty: ty_info, scope_level: sd, slot: li });
            }
        };

        Ok(Statement::VarStatement { ident, init: init.expect("big issue; init existed prev but now now"), information: ParseInfo::new(scope_depth, self.symbol_tbl.clone()) })
    }

    fn statement(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
        debug!("read statement");
        if let Some(_print_tok) = self.match_token([TokenType::Identifier(String::from("print"))]) {
            self.print_statement()
        } else if let Some(_block_ty) = self.match_token([TokenType::LeftBrace]) {
            // block
            self.begin_scope();
            let bs = self.block_statement();
            self.end_scope()?;
            bs
        } else {
            self.expression_statement()
        }
    }

    fn block_statement(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
        debug!("read block");
        let mut decls = Vec::new();
        while self.match_token([TokenType::RightBrace, TokenType::Eof]).is_none() {
            let decl = self.declaration()?;
            decls.push(decl);
        }

        Ok(Statement::BlockStatement { statements: decls, information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone()) })
    }

    fn print_statement(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
        debug!("read print statement");
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::PrintStatement { expression, information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone()) })
    }

    fn expression_statement(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
        debug!("read expression statement");
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::ExpressionStatement { expression, information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone()) })
    }

    fn expression(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        debug!("read expression");
        self.assignment() 
    }

    fn assignment(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        debug!("read assignment");
        let lhs = self.equality()?;

        if let Some(eq) = self.match_token([TokenType::Equal]) {
            // we have assignment
            let rhs = self.assignment()?;

            return match lhs {                
                Expression::Literal { literal, information: _ } => {
                    if let TokenType::Identifier(s) = literal.ty() {
                        Ok(Expression::Assignment { name: Symbol::from(s), value: Box::new(rhs), information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone()) })
                    } else {
                        Err(ParseError::from_token(eq, "Invalid assignment target"))
                    }
                },
                _ => Err(ParseError::from_token(eq, "Invalid assignment target")),
            }
        }

        Ok(lhs)
    }

    fn equality(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        debug!("read equality");
        let mut lhs = self.comparison()?;

        while let Some(oper) = self.match_token([TokenType::BangEqual, TokenType::EqualEqual]) {
            let rhs = self.comparison()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone()),
            };
        };
        
        Ok(lhs)
    }

    fn comparison(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        debug!("read comparison");
        let mut lhs = self.term()?;

        while let Some(oper) = self.match_token([TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let rhs = self.term()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone()),
            };
        }

        Ok(lhs)
    }

    fn term(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        debug!("read term");
        let mut lhs = self.factor()?;

        while let Some(oper) = self.match_token([TokenType::Minus, TokenType::Plus]) {
            let rhs = self.factor()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone()),
            };
        }

        Ok(lhs)
    }

    fn factor(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        debug!("read factor");
        let mut lhs = self.unary()?;

        while let Some(oper) = self.match_token([TokenType::Star, TokenType::Slash]) {
            let rhs = self.unary()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone())
            };
        }

        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        debug!("read unary");
        if let Some(oper) = self.match_token([TokenType::Bang, TokenType::Minus]) {
            self.unary().map(|rhs| Expression::Unary {
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone())
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
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
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone()),
            })
        } else {
            // needs to match literals only
            self
                .advance()
                .filter(|tk| tk.ty().is_literal())
                .map(|tk| Expression::Literal { literal: tk, information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone()) })
                .ok_or_else(|| ParseError::from_other("Unexpected EOS"))
        }
    }
}

impl<'a> Parser for BasicParser<'a> {
    type Out = Result<(AbstractTree<ParseInfo>, ExtSymbolTable), ParseError>;

    // TODO: this should parse a full file with multiple top-level statements.
    fn parse(mut self) -> Result<(AbstractTree<ParseInfo>, ExtSymbolTable), ParseError> {
        self.declaration().map(|stmt| (AbstractTree::statement(stmt, ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone())), self.root_symbol_tbl))
    }
}