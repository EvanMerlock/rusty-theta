use std::{iter::Peekable, rc::Rc, cell::RefCell};
use log::{debug, error, trace};

use crate::{ast::{symbol::{SymbolTable, SymbolData, ExtSymbolTable, ExtFrameData, FrameData}, transformers::{typeck::TypeInformation}, Statement, Expression, AbstractTree, FunctionArg, Function, Item}, bytecode::{Symbol}};
use super::{Parser, ParseInfo, ParseError};
use crate::lexer::token::{Token, TokenType};

pub struct BasicParser<'a> {
    tokens: Peekable<&'a mut dyn Iterator<Item = Token>>,
    symbol_tbl: ExtSymbolTable,
    root_symbol_tbl: ExtSymbolTable,
    frame_data: ExtFrameData,
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
        let frame_data: Rc<RefCell<FrameData>> = Rc::new(RefCell::new(FrameData::new()));

        BasicParser {
            tokens: token_stream.peekable(),
            symbol_tbl: symbol_table.clone(),
            root_symbol_tbl: symbol_table,
            frame_data,
        }
    }

    pub fn new_sym(token_stream: &'a mut dyn Iterator<Item = Token>, sym: ExtSymbolTable) -> BasicParser<'a> {
        let frame_data: Rc<RefCell<FrameData>> = Rc::new(RefCell::new(FrameData::new()));

        BasicParser { 
            tokens: token_stream.peekable(), 
            symbol_tbl: sym.clone(),
            root_symbol_tbl: sym,
            frame_data,
        }
    }

    fn begin_scope(&mut self) {
        self.symbol_tbl = Rc::new(RefCell::new(SymbolTable::new_enclosed(self.symbol_tbl.clone())));
    }

    fn end_scope(&mut self) -> Result<(), ParseError> {
        let enclosing = self.symbol_tbl.borrow().enclosing().ok_or_else(|| ParseError::from_other("failed to end scope"))?;
        self.symbol_tbl = enclosing;
        Ok(())
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    pub fn is_at_end(&mut self) -> bool {
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

    // TODO: This needs to read in a function declaration and then grab the internal block
    // this should be where the parser begins
    // https://doc.rust-lang.org/reference/items.html
    fn item(&mut self) -> Result<Item<ParseInfo>, ParseError> {
        trace!("read parser item");
        if let Some(_func_tok) = self.match_token([TokenType::Fun]) {
            let func_name = self.consume_if(|ty| ty.is_ident(), "Expected function name")?;
            let func_name = Symbol::new(func_name)?;
            self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
            
            let mut func_args = Vec::new();

            // read function args
            while self.match_token([TokenType::RightParen]).is_none() {
                let func_arg_name = self.consume_if(|ty| ty.is_ident(), "could not find function argument name")?;
                let func_arg_name = Symbol::new(func_arg_name)?;

                self.consume(TokenType::Colon, "no colon after arg name")?;

                let func_arg_ty = self.consume_if(|ty| ty.is_ident(), "could not find function argument type")?;

                let ty_ident = Symbol::new(func_arg_ty)?;

                let ty_info = match self.symbol_tbl.borrow().get_symbol_data(&ty_ident, self.symbol_tbl.borrow().scope_depth()) {
                    Some(SymbolData::Type { ty }) => ty,
                    Some(_) => return Err(ParseError::from_other("ident is being used by something else")),
                    // assume forward declaration here. if the type continues to not be defined via ID, we will error on compilation.
                    None => TypeInformation::NonLiteral(ty_ident.clone()),
                };


                let func_arg = FunctionArg {
                    name: func_arg_name,
                    ty: ty_info,
                };

                func_args.push(func_arg);
            }

            // read func return
            let ret_ty = if let Some(_arrow_tok) = self.match_token([TokenType::Arrow]) {
                let func_arg_ty = self.consume_if(|ty| ty.is_ident(), "could not find function argument type")?;

                let ty_ident = Symbol::new(func_arg_ty)?;

                match self.symbol_tbl.borrow().get_symbol_data(&ty_ident, self.symbol_tbl.borrow().scope_depth()) {
                    Some(SymbolData::Type { ty }) => ty,
                    Some(_) => return Err(ParseError::from_other("ident is being used by something else")),
                    // assume forward declaration here. if the type continues to not be defined via ID, we will error on compilation.
                    None => TypeInformation::NonLiteral(ty_ident.clone()),
                }
            } else {
                TypeInformation::None
            };

            // read block
            self.consume(TokenType::LeftBrace, "no block before function")?;
            let block = self.block_expression()?;

            let func = Function {
                args: func_args,
                chunk: AbstractTree::expression(block.clone(), block.information().clone()),
                name: func_name,
                return_ty: ret_ty,
                information: ParseInfo { scope_depth: 0, current_symbol_table: self.symbol_tbl.clone(), frame_data: self.frame_data.clone() },
            };

            Ok(Item::Function(func))
        } else {
            error!("Could not find top level item");
            Err(ParseError::Other { msg: "failed to find top level item" })
        }
    }

    pub fn declaration(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
        trace!("read declaration");
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
        trace!("read var declaration");
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

        // TODO:
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
                // TODO:
                // symbol table does not do debouncing of symbols yet. need a combination of scope_depth and identifier to ensure variables cannot collide.
                // this is because we could have 2 variable declarations with different types in 2 lexical scopes
                // I'm not sure why I didn't split symbol tables by lexical bounds here...
                // maybe because of local totals. fixed with a FrameData struct so maybe we can move back to lexical bounded symbol tables.
                let li = self.frame_data.borrow_mut().new_local();
                // slot determined by # of seen variables
                self.symbol_tbl.borrow_mut().insert_symbol(ident.clone(), SymbolData::LocalVariable { ty: ty_info, scope_level: sd, slot: li });
            }
        };

        Ok(Statement::VarStatement { ident, init: init.expect("big issue; init existed prev but now now"), information: ParseInfo::new(scope_depth, self.symbol_tbl.clone(), self.frame_data.clone()) })
    }

    fn statement(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
        trace!("read statement");
        if let Some(_print_tok) = self.match_token([TokenType::Identifier(String::from("print"))]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
        trace!("read print statement");
        self.consume(TokenType::LeftParen, "Expected ( before print statement")?;
        let expression = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ) after print statement")?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::PrintStatement { expression, information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()) })
    }

    fn expression_statement(&mut self) -> Result<Statement<ParseInfo>, ParseError> {
        trace!("read expression statement");
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::ExpressionStatement { expression, information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()) })
    }

    fn expression(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("read expression");

        if let Some(_if_tok) = self.match_token([TokenType::If]) {
            self.if_expression()
        } else if let Some(_while_tok) = self.match_token([TokenType::While]) {
            self.while_expression()
        } else if let Some(_block_ty) = self.match_token([TokenType::LeftBrace]) {
            // block
            self.begin_scope();
            let bs = self.block_expression();
            self.end_scope()?;
            bs
        } else {
            self.assignment() 
        }
    }

    fn block_expression(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("read block");
        let mut decls = Vec::new();
        while self.match_token([TokenType::RightBrace, TokenType::Eof]).is_none() {
            let decl = self.declaration()?;
            decls.push(decl);
        }

        Ok(Expression::BlockExpression { statements: decls, information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()) })
    }

    fn if_expression(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("if expression");

        // consume the if expression
        self.consume(TokenType::LeftParen, "Expected '(' after if keyword")?;
        let check_expr = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after if keyword")?;

        // grab the expression which should exist after the if block
        // TODO: enforce braces around if blocks

        let body_expr = self.expression()?;

        // check for else / else if

        let else_clause = if let Some(_else_tok) = self.match_token([TokenType::Else]) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        Ok(Expression::If { check_expression: Box::new(check_expr), body: Box::new(body_expr), else_body: else_clause, information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()) })
    }

    fn while_expression(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("while expression");
        // consume the while expression
        let predicate = if let Some(_left_paren) = self.match_token([TokenType::LeftParen]) {
            let predicate_expression = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after while keyword with predicate")?;
            Some(Box::new(predicate_expression))
        } else {
            None
        };

        // grab the expression which should exist after the while block
        // TODO: enforce braces around while blocks

        let body_expr = self.expression()?;

        Ok(Expression::LoopExpression { predicate, body: Box::new(body_expr), information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()) })

    }

    fn assignment(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("read assignment");
        let lhs = self.equality()?;

        if let Some(eq) = self.match_token([TokenType::Equal]) {
            // we have assignment
            let rhs = self.assignment()?;

            return match lhs {                
                Expression::Literal { literal, information: _ } => {
                    if let TokenType::Identifier(s) = literal.ty() {
                        Ok(Expression::Assignment { name: Symbol::from(s), value: Box::new(rhs), information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()) })
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
        trace!("read equality");
        let mut lhs = self.comparison()?;

        while let Some(oper) = self.match_token([TokenType::BangEqual, TokenType::EqualEqual]) {
            let rhs = self.comparison()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()),
            };
        };
        
        Ok(lhs)
    }

    fn comparison(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("read comparison");
        let mut lhs = self.term()?;

        while let Some(oper) = self.match_token([TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let rhs = self.term()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()),
            };
        }

        Ok(lhs)
    }

    fn term(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("read term");
        let mut lhs = self.factor()?;

        while let Some(oper) = self.match_token([TokenType::Minus, TokenType::Plus]) {
            let rhs = self.factor()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()),
            };
        }

        Ok(lhs)
    }

    fn factor(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("read factor");
        let mut lhs = self.unary()?;

        while let Some(oper) = self.match_token([TokenType::Star, TokenType::Slash]) {
            let rhs = self.unary()?;
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone())
            };
        }

        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("read unary");
        if let Some(oper) = self.match_token([TokenType::Bang, TokenType::Minus]) {
            self.unary().map(|rhs| Expression::Unary {
                operator: oper,
                right: Box::new(rhs),
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone())
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expression<ParseInfo>, ParseError> {
        trace!("read primary");
        if self.match_token([TokenType::LeftParen]).is_some() {
            trace!("read seq");
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
                information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()),
            })
        } else {
            // needs to match literals only
            self
                .advance()
                .filter(|tk| tk.ty().is_literal())
                .map(|tk| Expression::Literal { literal: tk, information: ParseInfo::new(self.symbol_tbl.borrow().scope_depth(), self.symbol_tbl.clone(), self.frame_data.clone()) })
                .ok_or_else(|| ParseError::from_other("Unexpected EOS"))
        }
    }
}

impl<'a> Parser for BasicParser<'a> {
    type Out = Item<ParseInfo>;

    fn parse(mut self) -> Result<Vec<Item<ParseInfo>>, ParseError> {
        let mut trees = Vec::new();

        while !self.is_at_end() {
            let item = self.next()?;
            trees.push(item);
        }

        Ok(trees)
    }

    fn next(&mut self) -> Result<Self::Out, ParseError> {
        // clean frame data and symbol tables. The root should likely not be changed when we add cross-modules'
        // TODO: remove root_symbol_tbl changes here
        self.frame_data = Rc::new(RefCell::new(FrameData::new()));
        self.root_symbol_tbl = Rc::new(RefCell::new(SymbolTable::default()));
        self.symbol_tbl = self.root_symbol_tbl.clone();

        self.item()
    }
}