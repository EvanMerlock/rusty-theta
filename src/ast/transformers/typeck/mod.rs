use std::{error::Error, fmt::Display};

use log::{debug, error, trace};

use super::{ASTTransformer, ASTVisitor, TransformError};
use crate::{ast::{symbol::{ExtSymbolTable, SymbolData}, AbstractTree, InnerAbstractTree, Expression, Statement, tree::Function, Item}, lexer::token::{TokenType, Token}, parser::ParseInfo, bytecode::Symbol};

pub struct TypeCk {
    symbol_table: ExtSymbolTable
}

impl TypeCk {
    pub fn new(tbl: ExtSymbolTable) -> TypeCk {
        TypeCk { symbol_table: tbl }
    }
}

#[derive(Debug)]
pub enum TypeCkError {
    ExpressionBinaryTypeCkFail(TypeInformation, TypeInformation, Token),
    ExpressionUnaryTypeCkFail(TypeInformation, Token),
    TypeNotFound(Symbol),
    InvalidTypeInPosition(Symbol),
    InvalidLiteralInPosition(Token),
    IncorrectInitializer(Symbol),
    InvalidAssignment(TypeInformation, TypeInformation),
    InvalidIfExpressionCheck(TypeInformation),
    InvalidIfBranches(TypeInformation, TypeInformation),
    InvalidPredicate(TypeInformation),
    InvalidFunctionReturn(TypeInformation, TypeInformation)
}

impl Error for TypeCkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for TypeCkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeCkError::ExpressionBinaryTypeCkFail(l_ty, r_ty, oper) => write!(f, "Type Mismatch! Left Ty: {}, Right Ty: {}, Oper: {}", l_ty, r_ty, oper),
            TypeCkError::ExpressionUnaryTypeCkFail(r_ty, oper) => write!(f, "Type Mismatch! Right Ty: {}, Oper: {}", r_ty, oper),
            TypeCkError::TypeNotFound(ident) => write!(f, "Type not found for variable: {}", ident),
            TypeCkError::InvalidTypeInPosition(ident) => write!(f, "!! A type was sent where a variable name was expected: {} !!", ident),
            TypeCkError::IncorrectInitializer(ident) => write!(f, "Incorrect initializer for identifier: {}", ident),
            TypeCkError::InvalidAssignment(lhs, rhs) => write!(f, "Invalid assignment; LHS = {}, RHS = {} and there is no type-unity", lhs, rhs),
            TypeCkError::InvalidLiteralInPosition(tk) => write!(f, "!! A non-literal token was found where a literal was expected: {} !!", tk),
            TypeCkError::InvalidIfExpressionCheck(ty) => write!(f, "Type Mismatch! Expected boolean, instead an if expression produced: {}", ty),
            TypeCkError::InvalidIfBranches(ty_l, ty_r) => write!(f, "Type Mismatch! Primary If Body: {}, Else Body: {}", ty_l, ty_r),
            TypeCkError::InvalidPredicate(ty) => write!(f, "Type Mismatch! Expected boolean, got: {ty}"),
            TypeCkError::InvalidFunctionReturn(expected, actual) => write!(f, "Type Mismatch! Expected a function returning {}, got: {}", expected, actual),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeCkOutput {
    pub ty: TypeInformation,
    pub pi: ParseInfo,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypeInformation {
    Int,
    String,
    Float,
    Boolean,
    NonLiteral(Symbol),
    Function(Box<TypeInformation>, Vec<TypeInformation>),
    None,
}

impl Display for TypeInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeInformation::Int => write!(f, "Int"),
            TypeInformation::String => write!(f, "String"),
            TypeInformation::Float => write!(f, "Float"),
            TypeInformation::Boolean => write!(f, "Boolean"),
            TypeInformation::NonLiteral(s) => write!(f, "{}", s),
            TypeInformation::None => write!(f, "!"),
            TypeInformation::Function(return_ty, args) => write!(f, "Fn({args:?}) -> {return_ty}"),
        }
    }
}

impl ASTTransformer<ParseInfo> for TypeCk {

    type ItemOut = Item<TypeCkOutput>;
    type TreeOut = AbstractTree<TypeCkOutput>;

    fn transform_tree(&self, tree: &AbstractTree<ParseInfo>) -> Result<Self::TreeOut, super::TransformError> {
        trace!("Symbol Table: {:#?}", self.symbol_table);
        match tree.inner() {
            InnerAbstractTree::Expression(expr) => { 
                let ty_aug = match self.visit_expression(&expr.0) {
                    Ok(ty) => ty,
                    Err(e) => {
                        error!("{}", e);
                        return Err(e);
                    },
                };
                let info = ty_aug.information().clone();
                debug!("Completed TypeCk on Expr w/ Type: {:?}", info);
                Ok(AbstractTree::expression(ty_aug, info))
            },
            InnerAbstractTree::Statement(stmt) => {
                let ty_aug = match self.visit_statement(&stmt.0) {
                    Ok(ty) => ty,
                    Err(e) => {
                        error!("{}", e);
                        return Err(e);
                    },
                };                
                let info = ty_aug.information().clone();
                debug!("Completed TypeCk on Stmt w/ Type: {:?}", info);
                Ok(AbstractTree::statement(ty_aug, info))
            },
        }
    }

    fn transform_item(&self, item: &Item<ParseInfo>) -> Result<Self::ItemOut, TransformError> {
        match item {
            Item::Function(func) => {
                let ty_aug = match self.visit_function(func) {
                    Ok(ty) => ty,
                    Err(e) => {
                        error!("{}", e);
                        return Err(e);
                    },
                };                
                let info = ty_aug.information().clone();
                debug!("Completed TypeCk on Func w/ Type: {:?}", info);
                Ok(Item::Function(ty_aug))
            },
        }
    }

}

impl ASTVisitor<ParseInfo> for TypeCk {
    type InfoOut = TypeCkOutput;

    fn visit_expression(&self, expr: &Expression<ParseInfo>) -> Result<Expression<Self::InfoOut>, super::TransformError> {
        match expr {
            Expression::Binary { left, operator, right, information: info } => {
                let left_ty_res = self.visit_expression(left);
                let right_ty_res = self.visit_expression(right);

                let (left_ty, right_ty) = match (left_ty_res, right_ty_res) {
                    (Ok(lty), Ok(rty)) => (lty, rty),
                    (Ok(_), Err(e)) => {
                        return Err(e);
                    },
                    (Err(e), Ok(_)) => {
                        return Err(e);
                    },
                    (Err(el), Err(_er)) => {
                        // TODO: merge errors
                        return Err(el);
                    },
                };

                // TODO: WE NEED TO LOOK FOR FUNCTIONS THAT MATCH THE TYPE INFORMATION FOR BINARY OPERATORS.
                // WE MUST CHECK ALL KNOWN SYMBOLS IN THE COMPILE UNIT FOR DISPATCH POSSIBILITIES
                let ty_info = match (left_ty.information().ty.clone(), right_ty.information().ty.clone(), operator.ty()) {
                    (_, _, TokenType::EqualEqual) => TypeInformation::Boolean,
                    (_, _, TokenType::BangEqual) => TypeInformation::Boolean,

                    (TypeInformation::Int, TypeInformation::Int, TokenType::Plus) => TypeInformation::Int,
                    (TypeInformation::Int, TypeInformation::Int, TokenType::Minus) => TypeInformation::Int,
                    (TypeInformation::Int, TypeInformation::Int, TokenType::Star) => TypeInformation::Int,
                    (TypeInformation::Int, TypeInformation::Int, TokenType::Slash) => TypeInformation::Int,
                    (TypeInformation::Int, TypeInformation::Int, TokenType::Less) => TypeInformation::Boolean,
                    (TypeInformation::Int, TypeInformation::Int, TokenType::LessEqual) => TypeInformation::Boolean,
                    (TypeInformation::Int, TypeInformation::Int, TokenType::Greater) => TypeInformation::Boolean,
                    (TypeInformation::Int, TypeInformation::Int, TokenType::GreaterEqual) => TypeInformation::Boolean,

                    (TypeInformation::String, TypeInformation::String, TokenType::Plus) => TypeInformation::String,
                    (TypeInformation::String, TypeInformation::String, TokenType::Less) => TypeInformation::Boolean,
                    (TypeInformation::String, TypeInformation::String, TokenType::LessEqual) => TypeInformation::Boolean,
                    (TypeInformation::String, TypeInformation::String, TokenType::Greater) => TypeInformation::Boolean,
                    (TypeInformation::String, TypeInformation::String, TokenType::GreaterEqual) => TypeInformation::Boolean,

                    (TypeInformation::Float, TypeInformation::Float, TokenType::Plus) => TypeInformation::Float,
                    (TypeInformation::Float, TypeInformation::Float, TokenType::Minus) => TypeInformation::Float,
                    (TypeInformation::Float, TypeInformation::Float, TokenType::Star) => TypeInformation::Float,
                    (TypeInformation::Float, TypeInformation::Float, TokenType::Slash) => TypeInformation::Float,
                    (TypeInformation::Float, TypeInformation::Float, TokenType::Less) => TypeInformation::Boolean,
                    (TypeInformation::Float, TypeInformation::Float, TokenType::LessEqual) => TypeInformation::Boolean,
                    (TypeInformation::Float, TypeInformation::Float, TokenType::Greater) => TypeInformation::Boolean,
                    (TypeInformation::Float, TypeInformation::Float, TokenType::GreaterEqual) => TypeInformation::Boolean,

                    _ => {
                        return Err(TransformError::from(TypeCkError::ExpressionBinaryTypeCkFail(left_ty.information().ty.clone(), right_ty.information().ty.clone(), operator.clone()))) 
                    },
                };

                Ok(Expression::Binary { left: Box::new(left_ty), operator: operator.clone(), right: Box::new(right_ty), information: TypeCkOutput { ty: ty_info, pi: info.clone() } })
            },
            Expression::Unary { operator, right, information: info } => {
                let r_ty_res = self.visit_expression(right);

                let r_ty = match r_ty_res {
                    Ok(rty) => rty,
                    Err(e) => {
                        return Err(e);
                    },
                };

                let ty_chk = match (r_ty.information().ty.clone(), operator.ty()) {
                    (TypeInformation::Int, TokenType::Minus) => TypeInformation::Int,
                    (TypeInformation::Int, TokenType::Bang) => TypeInformation::Int,
                    (TypeInformation::Float, TokenType::Minus) => TypeInformation::Float,
                    (TypeInformation::Boolean, TokenType::Bang) => TypeInformation::Boolean,

                    _ => return Err(TransformError::from(TypeCkError::ExpressionUnaryTypeCkFail(r_ty.information().ty.clone(), operator.clone())))

                };
                Ok(Expression::Unary { operator: operator.clone(), right: Box::new(r_ty), information: TypeCkOutput { ty: ty_chk, pi: info.clone() } })
            },
            Expression::Literal { literal,information: info } => {
                match literal.ty() {
                    TokenType::Str(_) => Ok(Expression::Literal { literal: literal.clone(), information: TypeCkOutput { ty: TypeInformation::String, pi: info.clone() } }),
                    TokenType::Integer(_) => Ok(Expression::Literal { literal: literal.clone(), information: TypeCkOutput { ty: TypeInformation::Int, pi: info.clone() } }),
                    TokenType::Float(_) => Ok(Expression::Literal { literal: literal.clone(), information: TypeCkOutput { ty: TypeInformation::Float, pi: info.clone() } }),
                    TokenType::Identifier(id) => { 
                        let id = Symbol::from(id);
                        Ok(
                            Expression::Literal { 
                                literal: literal.clone(), 
                                information: TypeCkOutput { 
                                    ty: self.symbol_table.borrow().get_symbol_data(&id, info.scope_depth).ok_or(TypeCkError::TypeNotFound(id)).map(|x| x.ty().clone())?,
                                    pi: info.clone(),
                                }
                            }
                        ) },
                    TokenType::True => Ok(Expression::Literal { literal: literal.clone(), information: TypeCkOutput { ty: TypeInformation::Boolean, pi: info.clone() } }),
                    TokenType::False => Ok(Expression::Literal { literal: literal.clone(), information: TypeCkOutput { ty: TypeInformation::Boolean, pi: info.clone() } }),
                    _ => Err(TransformError::from(TypeCkError::InvalidLiteralInPosition(literal.clone()))),
                }
            },
            Expression::Sequence { seq, information: _ } => {
                // we need to typecheck each portion of the seq but only the last one matters to pass upwards
                let mut new_seq = Vec::new();
                for seq_item in &seq[..seq.len()] {
                    let ty = self.visit_expression(seq_item)?;
                    new_seq.push(ty);
                }
                let fin_info = new_seq[new_seq.len()-1].information().clone();

                // subtle bug _might_ be possible here
                // can fin_info.pi != info?
                Ok(Expression::Sequence { seq: new_seq, information: fin_info })
            },
            Expression::Assignment { name, value, information: info } => {
                let lhs_ty = self.symbol_table.borrow().get_symbol_data(name, info.scope_depth).ok_or_else(|| TypeCkError::TypeNotFound(name.clone())).map(|x| x.ty().clone())?;
                let rhs_ty = self.visit_expression(value)?;

                if lhs_ty != rhs_ty.information().ty {
                    Err(TransformError::from(TypeCkError::InvalidAssignment(lhs_ty, rhs_ty.information().ty.clone())))
                } else {
                    Ok(Expression::Assignment { name: name.clone(), value: Box::new(rhs_ty), information: TypeCkOutput { ty: lhs_ty, pi: info.clone() } })
                }

            },
            Expression::If { check_expression, body, else_body, information } => {
                // TODO: if the else clause does not exist the primary body MUST have the unit type.
                let check_ty = self.visit_expression(check_expression)?;
                if check_ty.information().ty != TypeInformation::Boolean {
                    return Err(TransformError::from(TypeCkError::InvalidIfExpressionCheck(check_ty.information().ty.clone())))
                }

                let else_body_type = if let Some(exists_else_body) = else_body {
                    Some(Box::new(self.visit_expression(exists_else_body)?))
                } else {
                    None
                };

                let primary_body_type = self.visit_expression(body)?;

                if let Some(else_body_info) = else_body_type.clone() {
                    if primary_body_type.information().ty != else_body_info.information().ty {
                        return Err(TransformError::TypeCkError(TypeCkError::InvalidIfBranches(else_body_info.information().ty.clone(), primary_body_type.information().ty.clone())));
                    }
                };

                Ok(Expression::If { check_expression: Box::new(check_ty), body: Box::new(primary_body_type.clone()), else_body: else_body_type, information: TypeCkOutput { ty: primary_body_type.information().ty.clone(), pi: information.clone() } })
            },
            Expression::BlockExpression { statements, information: info } => {
                // is it possible for the last statement's type to carry for the block?
                // we create a new typechecker because we need to look at the symbol table for this block.
                let internal_typeck = TypeCk::new(info.current_symbol_table.clone());
                let mut annotated_statements = Vec::new();
                for statement in statements {
                    let stmt = internal_typeck.visit_statement(statement)?;
                    annotated_statements.push(stmt);
                }
                Ok(Expression::BlockExpression { statements: annotated_statements, information: TypeCkOutput { ty: TypeInformation::None, pi: info.clone() } })
            },
            Expression::LoopExpression { predicate, body, information } => {
                let predicate_checked = if let Some(pred_body) = predicate {
                    let pred = Box::new(self.visit_expression(pred_body)?);

                    if pred.information().ty != TypeInformation::Boolean {
                        return Err(TransformError::TypeCkError(TypeCkError::InvalidPredicate(pred.information().ty.clone())))
                    }

                    Some(pred)
                } else {
                    None
                };

                let body_checked = Box::new(self.visit_expression(body)?);
                let body_ty = body_checked.information().ty.clone();

                Ok(Expression::LoopExpression { predicate: predicate_checked, body: body_checked, information: TypeCkOutput { ty: body_ty, pi: information.clone() } })
            },
        }
    }

    fn visit_statement(&self, stmt: &Statement<ParseInfo>) -> Result<Statement<Self::InfoOut>, TransformError> {
        match stmt {
            Statement::ExpressionStatement { expression, information: info } => {
                let aug_expr = self.visit_expression(expression)?;
                Ok(Statement::ExpressionStatement { expression: aug_expr, information: TypeCkOutput { ty: TypeInformation::None, pi: info.clone() } })
            },
            Statement::PrintStatement { expression, information: info } => {
                let aug_expr = self.visit_expression(expression)?;
                Ok(Statement::PrintStatement { expression: aug_expr, information: TypeCkOutput { ty: TypeInformation::None, pi: info.clone() } })
            },
            Statement::VarStatement { ident, init, information: info } => {
                // need symbol table to be built for this.
                // we just check that the init expr creates the same type as requested.
                let aug_expr = self.visit_expression(init)?;

                // at this point we should know about all symbols in the program, even from imports.
                // if we cannot locate a symbol, we cannot use it here.
                // that should also be enforced by the visit_expression call.
                // However, we will still report an error if the symbol isn't found
                let ty_match = match self.symbol_table.borrow().get_symbol_data(ident, info.scope_depth) {
                    Some(info) => {
                        match info {
                            SymbolData::Type { ty: _ } => return Err(TransformError::from(TypeCkError::InvalidTypeInPosition(ident.clone()))),
                            SymbolData::GlobalVariable { ty } => ty == aug_expr.information().ty,
                            SymbolData::LocalVariable { ty, slot: _, scope_level: _ } => ty == aug_expr.information().ty,
                            SymbolData::Function { return_ty: _, args: _, fn_ty } => fn_ty == aug_expr.information().ty,
                        }
                    },
                    None => {
                        return Err(TransformError::from(TypeCkError::TypeNotFound(ident.clone())));
                    }
                };

                if !ty_match {
                    return Err(TransformError::from(TypeCkError::IncorrectInitializer(ident.clone())));
                }

                Ok(Statement::VarStatement { ident: ident.clone(), init: aug_expr, information: TypeCkOutput { ty: TypeInformation::None, pi: info.clone() } })
            },
        }
    }

    fn visit_function(&self, func: &Function<ParseInfo>) -> Result<Function<Self::InfoOut>, TransformError> {
        let body_ty = self.transform_tree(&func.chunk)?;

        if body_ty.information().ty != func.return_ty {
            error!("function body does not match return ty");
            return Err(TransformError::TypeCkError(TypeCkError::InvalidFunctionReturn(body_ty.information().ty.clone(), func.return_ty.clone())));
        };

        Ok(Function { args: func.args.clone(), chunk: body_ty, name: func.name.clone(), return_ty: func.return_ty.clone(), information: TypeCkOutput { ty: func.return_ty.clone(), pi: func.information.clone() } })

    }
}