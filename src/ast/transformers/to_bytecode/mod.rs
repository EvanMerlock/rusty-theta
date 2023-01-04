
use std::rc::Rc;

use crate::ast::symbol::SymbolData;
use crate::ast::{Expression, Statement, AbstractTree, InnerAbstractTree};
use crate::bytecode::{Chunk, OpCode, ThetaValue, ThetaHeapValue};
use crate::parser::Identifier;
use crate::{build_chunk, lexer::token};

use super::typeck::TypeCkOutput;
use super::{ASTTerminator, ASTTransformer};

pub struct ToByteCode;

impl ASTTransformer<TypeCkOutput> for ToByteCode {
    type Out = Chunk;

    fn transform(
        &self,
        tree: &AbstractTree<TypeCkOutput>,
    ) -> Result<Chunk, super::TransformError> {
        Ok(match tree.inner() {
            InnerAbstractTree::Expression(exp) => ToByteCode.visit_expression(&exp.0)?,
            InnerAbstractTree::Statement(stmt) => ToByteCode.visit_statement(&stmt.0)?,
        })
    }
}

impl ASTTerminator<TypeCkOutput> for ToByteCode {
    type Out = Chunk;

    fn visit_expression(
        &self,
        expr: &Expression<TypeCkOutput>,
    ) -> Result<Self::Out, super::TransformError> {
        Ok(match expr {
            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let left_val = ToByteCode.visit_expression(left)?;
                let right_val = ToByteCode.visit_expression(right)?;
                let res_chunk = left_val.merge_chunk(right_val);
                let op_chunk = match operator.ty() {
                    token::TokenType::Plus => build_chunk!(OpCode::Add),
                    token::TokenType::Minus => build_chunk!(OpCode::Subtract),
                    token::TokenType::Slash => build_chunk!(OpCode::Divide),
                    token::TokenType::Star => build_chunk!(OpCode::Multiply),
                    token::TokenType::Less => build_chunk!(OpCode::LessThan),
                    token::TokenType::LessEqual => build_chunk!(OpCode::Add),
                    token::TokenType::Greater => build_chunk!(OpCode::GreaterThan),
                    token::TokenType::GreaterEqual => build_chunk!(OpCode::Add),
                    token::TokenType::EqualEqual => build_chunk!(OpCode::Equal),
                    token::TokenType::BangEqual => build_chunk!(OpCode::Equal, OpCode::Negate),
                    _ => panic!(
                        "invalid token in binary precedence when visiting for bytecode transform"
                    ),
                };
                res_chunk.merge_chunk(op_chunk)
            }
            Expression::Unary {
                operator, right, ..
            } => {
                let right_val = ToByteCode.visit_expression(right)?;
                let op_chunk = match operator.ty() {
                    token::TokenType::Minus => build_chunk!(OpCode::Negate),
                    _ => panic!(
                        "invalid token in binary precedence when visiting for bytecode transform"
                    ),
                };
                right_val.merge_chunk(op_chunk)
            }
            Expression::Literal { literal, information: info } => match literal.ty() {
                token::TokenType::Integer(i) => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaValue::Int(i as i64))
                }
                token::TokenType::Float(f) => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaValue::Double(f as f64))
                }
                token::TokenType::True => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaValue::Bool(true))
                }
                token::TokenType::False => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaValue::Bool(false))
                }
                token::TokenType::Str(s) => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaValue::HeapValue(Rc::new(ThetaHeapValue::Str(s))))
                },
                token::TokenType::Identifier(id) => {
                    match info.pi.scope_depth {
                        0 => {
                            build_chunk!(OpCode::GetGlobal { offset: 0 }; ThetaValue::HeapValue(Rc::new(ThetaHeapValue::Str(id))))
                        },
                        sd => {
                            let local = info.pi.current_symbol_table.borrow().get_symbol_data(&Identifier::from(id), sd).expect("local not found");
                            match local {
                                SymbolData::Type { ty: _ } => panic!("non local in local pos"),
                                SymbolData::GlobalVariable { ty: _ } => panic!("non local in local pos"),
                                SymbolData::LocalVariable { ty: _, scope_level: _, slot } => {
                                    build_chunk!(OpCode::GetLocal { offset: slot })
                                },
                            }
                        }
                    }
                },
                _ => {
                    panic!("invalid token in literal location when visiting for bytecode transform")
                }
            },
            Expression::Sequence { seq, .. } => {
                seq.iter()
                    // TODO: this needs to fail safely
                    .map(|expr| {
                        ToByteCode.visit_expression(expr)
                            .expect("could not map ToByteCode visit expression.")
                    })
                    .reduce(|acc, new| acc.merge_chunk(new))
                    .expect("expression vec was empty in seq")
            }
            Expression::Assignment { name, value, information: _ } => {
                let st = Rc::new(ThetaHeapValue::Str(name.id().clone()));
                let set_chunk = self.visit_expression(value)?;
                let glob_chunk = build_chunk!(OpCode::DefineGlobal { offset: 0 }; ThetaValue::HeapValue(st));
                set_chunk.merge_chunk(glob_chunk)
            },
        })
    }

    fn visit_statement(
        &self,
        stmt: &Statement<TypeCkOutput>,
    ) -> Result<Self::Out, super::TransformError> {
        match stmt {
            Statement::ExpressionStatement { expression, information: _ } => {
                let pop_chunk = build_chunk!(OpCode::Pop);
                let expr_chunk = self.visit_expression(expression)?;
                Ok(expr_chunk.merge_chunk(pop_chunk))
            },
            Statement::PrintStatement { expression, information: _ } => {
                let print_chunk = build_chunk!(OpCode::DebugPrint);
                let expr_chunk = self.visit_expression(expression)?;
                Ok(expr_chunk.merge_chunk(print_chunk))
            },
            Statement::VarStatement { ident, init, information: info } => {
                // we will emit the initializer and then define the global here. note that `information` may eventually carry scoping information
                // for now all variables are globals. this should change when lexical scoping is added
                match info.pi.scope_depth {
                    0 => {
                        // emit global when sd == 0
                        let hv = Rc::new(ThetaHeapValue::Str(ident.id().to_owned()));
                        let init_chunk = self.visit_expression(init)?;
                        let glob_chunk = build_chunk!(OpCode::DefineGlobal { offset: 0 }; ThetaValue::HeapValue(hv));
                        Ok(init_chunk.merge_chunk(glob_chunk))
                    },
                    sd => {
                        // emit local when sd > 0
                        let init_chunk = self.visit_expression(init)?;
                        let local = info.pi.current_symbol_table.borrow().get_symbol_data(ident, sd).expect("local not found");
                        match local {
                            SymbolData::Type { ty: _ } => panic!("non local in local pos"),
                            SymbolData::GlobalVariable { ty: _ } => panic!("non local in local pos"),
                            SymbolData::LocalVariable { ty: _, scope_level: _, slot } => {
                                let glob_chunk = build_chunk!(OpCode::DefineLocal { offset: slot });
                                Ok(init_chunk.merge_chunk(glob_chunk))
                            },
                        }
                    },
                }
            },
            Statement::BlockStatement { statements, information } => {
                // we are at scope_depth +1 here.
                // we need to care about scope depth because when expressions are searched, they need to search their localized symbol table for the identifier that matches their scope depth and ID.
                let mut block_chunk = Chunk::new();
                for stmt in statements {
                    block_chunk = block_chunk.merge_chunk(self.visit_statement(stmt)?);
                }
                
                let mut pop_block = Chunk::new();
                for _i in 0..information.pi.current_symbol_table.borrow().total_locals() {
                    // TODO: PopN instruction
                    pop_block.write_to_chunk(OpCode::Pop);
                }
                Ok(block_chunk.merge_chunk(pop_block))
            },
        }
    }
}
