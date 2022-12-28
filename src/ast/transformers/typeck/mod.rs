use std::{error::Error, fmt::{Display, format}};

use log::{debug, error, info};

use super::{ASTTransformer, ASTVisitor, AugmentedAbstractTree, AugmentedExpression, TransformError};
use crate::{ast::{AbstractTree}, lexer::token::{TokenType, Token}};

pub struct TypeCk;

#[derive(Debug)]
pub enum TypeCkError {
    ExpressionBinaryTypeCkFail(TypeInformation, TypeInformation, Token),
    ExpressionUnaryTypeCkFail(TypeInformation, Token),
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
            TypeCkError::ExpressionBinaryTypeCkFail(l_ty, r_ty, oper) => write!(f, "Type Mismatch! Left Ty: {:?}, Right Ty: {:?}, Oper: {:?}", l_ty, r_ty, oper),
            TypeCkError::ExpressionUnaryTypeCkFail(r_ty, oper) => write!(f, "Type Mismatch! Right Ty: {:?}, Oper: {:?}", r_ty, oper),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TypeInformation {
    Int,
    String,
    Float,
    Boolean,
    None,
}

impl ASTTransformer<()> for TypeCk {

    type Out = AugmentedAbstractTree<TypeInformation>;

    fn transform(tree: &AbstractTree) -> Result<Self::Out, super::TransformError> {
        Ok(match tree.inner() {
            super::InnerAbstractTree::Expression(expr) => { 
                let ty_aug = TypeCk::visit_expression(&expr.0)?;
                let info = *ty_aug.information();
                debug!("Completed TypeCk on Expr w/ Type: {:?}", info);
                AugmentedAbstractTree::expression(ty_aug, info)
            },
            super::InnerAbstractTree::Statement(stmt) => {
                let ty_aug = TypeCk::visit_statement(&stmt.0)?;
                let info = *ty_aug.information();
                debug!("Completed TypeCk on Stmt w/ Type: {:?}", info);
                AugmentedAbstractTree::statement(ty_aug, info)
            },
        })
    }

}

impl ASTVisitor<()> for TypeCk {
    type InfoOut = TypeInformation;

    fn visit_expression(expr: &super::AugmentedExpression<()>) -> Result<AugmentedExpression<Self::InfoOut>, super::TransformError> {
        match expr {
            AugmentedExpression::Binary { left, operator, right, information } => {
                let left_ty_res = TypeCk::visit_expression(left);
                let right_ty_res = TypeCk::visit_expression(right);

                let (left_ty, right_ty) = match (left_ty_res, right_ty_res) {
                    (Ok(lty), Ok(rty)) => (lty, rty),
                    (Ok(_), Err(e)) => {
                        error!("When typechecking on line {}, right type is invalid: {}", operator.line_num(), e);
                        return Err(e);
                    },
                    (Err(e), Ok(_)) => {
                        error!("When typechecking on line {}, left type is invalid: {}", operator.line_num(), e);
                        return Err(e);
                    },
                    (Err(el), Err(er)) => {
                        error!("When typechecking on line {}, left type is invalid: {}; right type is invalid: {}", operator.line_num(), el, er);
                        return Err(el);
                    },
                };

                // TODO: WE NEED TO LOOK FOR FUNCTIONS THAT MATCH THE TYPE INFORMATION FOR BINARY OPERATORS.
                // WE MUST CHECK ALL KNOWN SYMBOLS IN THE COMPILE UNIT FOR DISPATCH POSSIBILITIES
                let ty_info = match (left_ty.information(), right_ty.information(), operator.ty()) {
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
                        error!("Type Mismatch! {:?} and {:?} do not have an implementation of {:?}", *left_ty.information(), *right_ty.information(), operator.clone());
                        return Err(TransformError::from(TypeCkError::ExpressionBinaryTypeCkFail(*left_ty.information(), *right_ty.information(), operator.clone()))) 
                    },
                };

                Ok(AugmentedExpression::Binary { left: Box::new(left_ty), operator: operator.clone(), right: Box::new(right_ty), information: ty_info })
            },
            AugmentedExpression::Unary { operator, right, information } => {
                let r_ty_res = TypeCk::visit_expression(right);

                let r_ty = match r_ty_res {
                    Ok(rty) => rty,
                    Err(e) => {
                        error!("When typechecking on line {}, unary operator type is invalid: {}", operator.line_num(), e);
                        return Err(e);
                    },
                };

                let ty_chk = match (r_ty.information(), operator.ty()) {
                    (TypeInformation::Int, TokenType::Minus) => TypeInformation::Int,
                    (TypeInformation::Int, TokenType::Bang) => TypeInformation::Int,
                    (TypeInformation::Float, TokenType::Minus) => TypeInformation::Float,
                    (TypeInformation::Boolean, TokenType::Bang) => TypeInformation::Boolean,

                    _ => return Err(TransformError::from(TypeCkError::ExpressionUnaryTypeCkFail(*r_ty.information(), operator.clone())))

                };

                Ok(AugmentedExpression::Unary { operator: operator.clone(), right: Box::new(r_ty), information: ty_chk })
            },
            AugmentedExpression::Literal { literal,information } => {
                match literal.ty() {
                    TokenType::Str(_) => Ok(AugmentedExpression::Literal { literal: literal.clone(), information: TypeInformation::String }),
                    TokenType::Integer(_) => Ok(AugmentedExpression::Literal { literal: literal.clone(), information: TypeInformation::Int }),
                    TokenType::Float(_) => Ok(AugmentedExpression::Literal { literal: literal.clone(), information: TypeInformation::Float }),
                    TokenType::True => Ok(AugmentedExpression::Literal { literal: literal.clone(), information: TypeInformation::Boolean }),
                    TokenType::False => Ok(AugmentedExpression::Literal { literal: literal.clone(), information: TypeInformation::Boolean }),
                    _ => panic!("a non-literal token was in a literal position."),
                }
            },
            AugmentedExpression::Sequence { seq, information } => {
                // we need to typecheck each portion of the seq but only the last one matters to pass upwards
                let mut new_seq = Vec::new();
                for seq_item in &seq[..seq.len()] {
                    let ty = TypeCk::visit_expression(seq_item)?;
                    new_seq.push(ty);
                }
                let fin_info = *new_seq[new_seq.len()-1].information();

                Ok(AugmentedExpression::Sequence { seq: new_seq, information: fin_info })
            },
        }
    }

    fn visit_statement(stmt: &super::AugmentedStatement<()>) -> Result<super::AugmentedStatement<Self::InfoOut>, TransformError> {
        todo!()
    }
}