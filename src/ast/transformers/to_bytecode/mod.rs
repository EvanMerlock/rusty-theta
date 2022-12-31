
use std::rc::Rc;

use crate::ast::symbol::SymbolTable;
use crate::bytecode::{Chunk, OpCode, ThetaValue, ThetaHeapValue};
use crate::{build_chunk, lexer::token};

use super::typeck::TypeInformation;
use super::{ASTTerminator, ASTTransformer, AugmentedAbstractTree, AugmentedExpression, InnerAbstractTree};

pub struct ToByteCode;

impl ASTTransformer<TypeInformation> for ToByteCode {
    type Out = Chunk;

    fn transform(
        &self,
        tree: &AugmentedAbstractTree<TypeInformation>,
    ) -> Result<Chunk, super::TransformError> {
        Ok(match tree.inner() {
            InnerAbstractTree::Expression(exp) => ToByteCode.visit_expression(&exp.0)?,
            InnerAbstractTree::Statement(stmt) => ToByteCode.visit_statement(&stmt.0)?,
        })
    }
}

impl ASTTerminator<TypeInformation> for ToByteCode {
    type Out = Chunk;

    fn visit_expression(
        &self,
        expr: &AugmentedExpression<TypeInformation>,
    ) -> Result<Self::Out, super::TransformError> {
        Ok(match expr {
            AugmentedExpression::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let left_val = ToByteCode.visit_expression(left)?;
                let right_val = ToByteCode.visit_expression(right)?;
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
                    _ => panic!(
                        "invalid token in binary precedence when visiting for bytecode transform"
                    ),
                };
                res_chunk.merge_chunk(op_chunk)
            }
            AugmentedExpression::Unary {
                operator, right, ..
            } => {
                let right_val = ToByteCode.visit_expression(right)?;
                let op_chunk = match operator.ty() {
                    token::TokenType::Minus => build_chunk!(OpCode::NEGATE),
                    _ => panic!(
                        "invalid token in binary precedence when visiting for bytecode transform"
                    ),
                };
                right_val.merge_chunk(op_chunk)
            }
            AugmentedExpression::Literal { literal, .. } => match literal.ty() {
                token::TokenType::Integer(i) => {
                    build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Int(i as i64))
                }
                token::TokenType::Float(f) => {
                    build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Double(f as f64))
                }
                token::TokenType::True => {
                    build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Bool(true))
                }
                token::TokenType::False => {
                    build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::Bool(false))
                }
                token::TokenType::Str(s) => {
                    build_chunk!(OpCode::CONSTANT { offset: 0 }; ThetaValue::HeapValue(Rc::new(ThetaHeapValue::Str(s))))
                },
                token::TokenType::Identifier(id) => {
                    build_chunk!(OpCode::GET_GLOBAL { offset: 0 }; ThetaValue::HeapValue(Rc::new(ThetaHeapValue::Str(id))))
                },
                _ => {
                    panic!("invalid token in literal location when visiting for bytecode transform")
                }
            },
            AugmentedExpression::Sequence { seq, .. } => {
                seq.iter()
                    // TODO: this needs to fail safely
                    .map(|expr| {
                        ToByteCode.visit_expression(expr)
                            .expect("could not map ToByteCode visit expression.")
                    })
                    .reduce(|acc, new| acc.merge_chunk(new))
                    .expect("expression vec was empty in seq")
            }
        })
    }

    fn visit_statement(
        &self,
        stmt: &super::AugmentedStatement<TypeInformation>,
    ) -> Result<Self::Out, super::TransformError> {
        match stmt {
            super::AugmentedStatement::ExpressionStatement { expression, information } => {
                let pop_chunk = build_chunk!(OpCode::POP);
                let expr_chunk = self.visit_expression(expression)?;
                Ok(expr_chunk.merge_chunk(pop_chunk))
            },
            super::AugmentedStatement::PrintStatement { expression, information } => {
                let print_chunk = build_chunk!(OpCode::DEBUG_PRINT);
                let expr_chunk = self.visit_expression(expression)?;
                Ok(expr_chunk.merge_chunk(print_chunk))
            },
            super::AugmentedStatement::VarStatement { ident, init, information } => {
                // we will emit the initializer and then define the global here. note that `information` may eventually carry scoping information
                // for now all variables are globals. this should change when lexical scoping is added
                let hv = Rc::new(ThetaHeapValue::Str(ident.id().to_owned()));
                let init_chunk = self.visit_expression(init)?;
                let glob_chunk = build_chunk!(OpCode::DEFINE_GLOBAL { offset: 0 }; ThetaValue::HeapValue(hv));
                Ok(init_chunk.merge_chunk(glob_chunk))
            },
        }
    }
}
