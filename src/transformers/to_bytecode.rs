use crate::{vm::{chunk::Chunk, instruction::OpCode, value::ThetaValue}, build_chunk, parser::tree::{Expression, AbstractTree}, lexer::token};

use super::{ASTTransformer, ASTVisitor};

pub struct ToByteCode;

impl ASTTransformer for ToByteCode {

    type Out = Chunk;

    fn transform(tree: crate::parser::tree::AbstractTree) -> Result<Chunk, super::TransformError> {
           Ok(match tree {
                AbstractTree::Expression(exp) => {
                    ToByteCode::visit_expression(exp)?
                }
           })
    }
}

impl ASTVisitor for ToByteCode {
    type Out = Chunk;

    fn visit_expression(expr: Expression) -> Result<Self::Out, super::TransformError> {
        Ok(match expr {
            Expression::Binary { left, operator, right } => {
                let left_val = ToByteCode::visit_expression(*left)?;
                let right_val = ToByteCode::visit_expression(*right)?;
                let res_chunk = left_val.merge_chunk(right_val);
                let op_chunk = match operator.ty() {
                    token::TokenType::Plus => build_chunk!(OpCode::ADD),
                    token::TokenType::Minus => build_chunk!(OpCode::SUBTRACT),
                    token::TokenType::Slash => build_chunk!(OpCode::DIVIDE),
                    token::TokenType::Star => build_chunk!(OpCode::MULTIPLY),
                    token::TokenType::Less => build_chunk!(OpCode::ADD),
                    token::TokenType::LessEqual => build_chunk!(OpCode::ADD),
                    token::TokenType::Greater => build_chunk!(OpCode::ADD),
                    token::TokenType::GreaterEqual => build_chunk!(OpCode::ADD),
                    _ => panic!("invalid token in binary precedence when visiting for bytecode transform")
                };
                res_chunk.merge_chunk(op_chunk)
            },
            Expression::Unary { operator, right } => {
                let right_val = ToByteCode::visit_expression(*right)?;
                let op_chunk = match operator.ty() {
                    token::TokenType::Minus => build_chunk!(OpCode::NEGATE),
                    _ => panic!("invalid token in binary precedence when visiting for bytecode transform")
                };
                right_val.merge_chunk(op_chunk)
            },
            Expression::Literal(lit_token) => {
                match lit_token.ty() {
                    token::TokenType::Integer(i) => build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Int(i as i64)),
                    token::TokenType::Float(f) => build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Double(f as f64)),
                    token::TokenType::True => build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Bool(true)),
                    token::TokenType::False => build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Bool(false)),
                    token::TokenType::Str(s) => todo!(),
                    _ => panic!("invalid token in literal location when visiting for bytecode transform"),
                }
            },
            Expression::Sequence(exprs) => {
                exprs
                    .into_iter()
                    // TODO: this needs to fail safely
                    .map(|expr| ToByteCode::visit_expression(expr).expect("could not map ToByteCode visit expression."))
                    .reduce(|acc, new| acc.merge_chunk(new))
                    .expect("expression vec was empty in seq")
            },
        })
    }

}