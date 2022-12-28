use crate::{build_chunk, lexer::token};
use crate::bytecode::{Chunk, OpCode, ThetaValue};
use crate::ast::{AbstractTree, Expression};

use super::typeck::TypeInformation;
use super::{ASTTransformer, ASTVisitor, InnerAbstractTree, AugmentedAbstractTree, AugmentedExpression};

pub struct ToByteCode;

impl ASTTransformer<TypeInformation> for ToByteCode {

    type Out = Chunk;

    fn transform(tree: &AugmentedAbstractTree<TypeInformation>) -> Result<Chunk, super::TransformError> {
           Ok(match tree.inner() {
                InnerAbstractTree::Expression(exp) => {
                    ToByteCode::visit_expression(&exp.0)?
                }
           })
    }
}

impl ASTVisitor<TypeInformation> for ToByteCode {
    type Out = Chunk;

    fn visit_expression(expr: &AugmentedExpression<TypeInformation>) -> Result<Self::Out, super::TransformError> {
        Ok(match expr {
            AugmentedExpression::Binary { left, operator, right , ..} => {
                let left_val = ToByteCode::visit_expression(left)?;
                let right_val = ToByteCode::visit_expression(right)?;
                let res_chunk = left_val.merge_chunk(right_val);
                let op_chunk = match operator.ty() {
                    token::TokenType::Plus => build_chunk!(OpCode::ADD),
                    token::TokenType::Minus => build_chunk!(OpCode::SUBTRACT),
                    token::TokenType::Slash => build_chunk!(OpCode::DIVIDE),
                    token::TokenType::Star => build_chunk!(OpCode::MULTIPLY),
                    token::TokenType::Less => build_chunk!(OpCode::LT),
                    token::TokenType::LessEqual => build_chunk!(OpCode::ADD),
                    token::TokenType::Greater => build_chunk!(OpCode::GT),
                    token::TokenType::GreaterEqual => build_chunk!(OpCode::ADD),
                    token::TokenType::EqualEqual => build_chunk!(OpCode::EQ),
                    token::TokenType::BangEqual => build_chunk!(OpCode::EQ, OpCode::NEGATE),
                    _ => panic!("invalid token in binary precedence when visiting for bytecode transform")
                };
                res_chunk.merge_chunk(op_chunk)
            },
            AugmentedExpression::Unary { operator, right, .. } => {
                let right_val = ToByteCode::visit_expression(right)?;
                let op_chunk = match operator.ty() {
                    token::TokenType::Minus => build_chunk!(OpCode::NEGATE),
                    _ => panic!("invalid token in binary precedence when visiting for bytecode transform")
                };
                right_val.merge_chunk(op_chunk)
            },
            AugmentedExpression::Literal { literal, .. } => {
                match literal.ty() {
                    token::TokenType::Integer(i) => build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Int(i as i64)),
                    token::TokenType::Float(f) => build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Double(f as f64)),
                    token::TokenType::True => build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Bool(true)),
                    token::TokenType::False => build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Bool(false)),
                    token::TokenType::Str(s) => todo!(),
                    _ => panic!("invalid token in literal location when visiting for bytecode transform"),
                }
            },
            AugmentedExpression::Sequence { seq, .. } => {
                seq
                    .iter()
                    // TODO: this needs to fail safely
                    .map(|expr| ToByteCode::visit_expression(expr).expect("could not map ToByteCode visit expression."))
                    .reduce(|acc, new| acc.merge_chunk(new))
                    .expect("expression vec was empty in seq")
            },
        })
    }

}