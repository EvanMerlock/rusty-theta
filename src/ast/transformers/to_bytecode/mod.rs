
use std::error::Error;
use std::fmt::Display;

use crate::ast::symbol::SymbolData;
use crate::ast::{Expression, Statement, AbstractTree, InnerAbstractTree, Item, Function};
use crate::bytecode::{Chunk, OpCode, ThetaConstant, Symbol, ThetaFunction, ThetaFuncArg, ThetaString};
use crate::{build_chunk, lexer::token};

use super::typeck::TypeCkOutput;
use super::{ASTTerminator, ASTTransformer, TransformError};

pub struct ToByteCode;

impl ASTTransformer<TypeCkOutput> for ToByteCode {

    type ItemOut = ThetaFunction;
    type TreeOut = Chunk;

    fn transform_tree(
        &self,
        tree: &AbstractTree<TypeCkOutput>,
    ) -> Result<Chunk, super::TransformError> {
        // TODO: insert chunk prologue here when necessary
        // should only occur on stack boundaries
        // in interpreter mode, this can happen on expression / statement bounds.

        // note: this doesn't include function bounds because the CALL instruction will allocate space on the stack for params
        let local_size = tree.information().pi.frame_data.borrow().total_locals();
        let mut block_chunk = if local_size > 0 { 
            let mut block_chunk = Chunk::new();
            block_chunk.write_to_chunk(OpCode::Push { size: local_size });
            block_chunk
        } else {
            Chunk::new()
        };

        block_chunk = block_chunk.merge_chunk(match tree.inner() {
            InnerAbstractTree::Expression(exp) => ToByteCode.visit_expression(&exp.0)?,
            InnerAbstractTree::Statement(stmt) => ToByteCode.visit_statement(&stmt.0)?,
        });

        let mut pop_block = Chunk::new();
        for _i in 0..local_size {
            // TODO: PopN instruction
            pop_block.write_to_chunk(OpCode::Pop);
        }
        Ok(block_chunk.merge_chunk(pop_block))
    }

    fn transform_item(&self, item: &Item<TypeCkOutput>) -> Result<Self::ItemOut, TransformError> {
        match item {
            Item::Function(func) => {
                let ck = self.transform_tree(&func.chunk)?;
                let func_name = ThetaString::from(func.name.clone());

                let mut theta_func_args = Vec::new();
                for arg in &func.args {
                    theta_func_args.push(ThetaFuncArg {
                        ty: arg.ty.clone(),
                    })
                }


                Ok(ThetaFunction {
                    args: theta_func_args,
                    chunk: ck,
                    name: func_name,
                    return_ty: func.return_ty.clone(),
                })
            },
        }
    }
}

impl ASTTerminator<TypeCkOutput> for ToByteCode {
    type ChunkOut = Chunk;

    fn visit_expression(
        &self,
        expr: &Expression<TypeCkOutput>,
    ) -> Result<Self::ChunkOut, TransformError> {
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
                    _ => return Err(TransformError::from(ToByteCodeError::InvalidToken(format!("in binary precedence: {}", operator)))),
                };
                res_chunk.merge_chunk(op_chunk)
            }
            Expression::Unary {
                operator, right, ..
            } => {
                let right_val = ToByteCode.visit_expression(right)?;
                let op_chunk = match operator.ty() {
                    token::TokenType::Minus => build_chunk!(OpCode::Negate),
                    _ => return Err(TransformError::from(ToByteCodeError::InvalidToken(format!("in unary precedence: {}", operator)))),

                };
                right_val.merge_chunk(op_chunk)
            }
            Expression::Literal { literal, information: info } => match literal.ty() {
                token::TokenType::Integer(i) => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaConstant::Int(i as i64))
                }
                token::TokenType::Float(f) => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaConstant::Double(f as f64))
                }
                token::TokenType::True => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaConstant::Bool(true))
                }
                token::TokenType::False => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaConstant::Bool(false))
                }
                token::TokenType::Str(s) => {
                    build_chunk!(OpCode::Constant { offset: 0 }; ThetaConstant::Str(s))
                },
                token::TokenType::Identifier(id) => {
                    // check if ID is a function. if so, load the ID as a string into the engine.
                    match info.pi.current_symbol_table.borrow().get_symbol_data(&Symbol::from(id.clone()), 0) {
                        Some(SymbolData::Function { return_ty: _, args: _, fn_ty: _ }) => {
                            build_chunk!(OpCode::Constant { offset: 0 }; ThetaConstant::Str(id))
                        },
                        _ => match info.pi.scope_depth {
                            0 => {
                                build_chunk!(OpCode::GetGlobal { offset: 0 }; ThetaConstant::Str(id))
                            },
                            sd => {
                                let local = info.pi.current_symbol_table.borrow().get_symbol_data(&Symbol::from(id.clone()), sd);
                                match local {
                                    Some(SymbolData::Type { ty: _ }) => return Err(TransformError::from(ToByteCodeError::InvalidLocal(id))),
                                    // TODO: not correct. need to track globals across CUs
                                    Some(SymbolData::GlobalVariable { ty: _ }) => build_chunk!(OpCode::GetGlobal { offset: 0 }; ThetaConstant::Str(id)),
                                    Some(SymbolData::LocalVariable { ty: _, scope_level: _, slot }) => {
                                        build_chunk!(OpCode::GetLocal { offset: slot })
                                    },
                                    Some(SymbolData::Function { return_ty: _, args: _, fn_ty: _ }) => {
                                        // embed function / closure object
                                        todo!()
                                    }
                                    None => return Err(TransformError::from(ToByteCodeError::NoIdentFound(id)))
                                }
                            }
                        },
                    }
                },
                _ => return Err(TransformError::from(ToByteCodeError::InvalidToken(format!("when expected literal: {}", literal)))),

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
            Expression::Assignment { name, value, information } => {
                let st = ThetaConstant::Str(name.id().clone());
                let set_chunk = self.visit_expression(value)?;
                // TODO: this needs to properly check if it's assigning to a global
                // or a local and continue from there
                let sd = information.pi.scope_depth;
                // TODO: this needs to fail gracefully
                let sym_data = information.pi.current_symbol_table.borrow().get_symbol_data(name, sd).expect("failed to find symbol in symbol table which was asked for");
                let chunk = match sym_data {
                    SymbolData::Type { ty: _ } => panic!("type where variable expected"),
                    // TODO: this isn't right. we need to track globals when compiling a CU :vomits:
                    SymbolData::GlobalVariable { ty: _ } => build_chunk!(OpCode::DefineGlobal { offset: 0 }; st),
                    SymbolData::LocalVariable { ty: _, scope_level: _, slot } => build_chunk!(OpCode::DefineLocal { offset: slot }; st),
                    // embed closure
                    SymbolData::Function { return_ty: _, args: _, fn_ty: _ } => todo!(),
                };
                set_chunk.merge_chunk(chunk)
            },
            Expression::If { check_expression, body, else_body, information: _ } => {
                let check_block = self.visit_expression(check_expression)?;
                let body_block = self.visit_expression(body)?;
                let else_block = if let Some(else_clause) = else_body {
                    Some(self.visit_expression(else_clause)?)
                } else {
                    None
                };

                // we now need to compute how far the if needs to jump.
                // there are multiple types of jump and we use the one that is most relevant.
                // if the jump size is less than 127, we can use a local jump.
                // if the jump size is greater than 127, we will instead switch to a full-sized jump.
                // note that we can only jump at most by 2^(sizeof(isize) - 1) because we need the sign bit
                // for when a jump is negative.

                // first we need to know how big the main body is.
                // TODO: if this overflows code you can execute code at a different location
                let jump_size = body_block.instruction_size() as isize;
                let jump_size: isize = jump_size + match &else_block {
                    Some(_) => 2,
                    None => 2
                };

                // depending on which jump is emitted we need to add the size of the jump to the offset!!!
                // we also need to consider if we're jumping from the START of the jump instruction (we are) vs. the end of the jump instruction

                let jump_chunk = if jump_size + 2 > 127 {
                    // emit non-"local" jump
                    // TODO: clean this up
                    build_chunk!(OpCode::JumpFarIfFalse { offset: jump_size + 2 + (std::mem::size_of::<isize>() as isize) }, OpCode::Pop)
                } else {
                    build_chunk!(OpCode::JumpLocalIfFalse { offset: (jump_size + 3) as i8 }, OpCode::Pop)
                };

                // first we emit the check block, which should leave a boolean on top of the stack
                // then we emit the jump chunk
                let combined_block = check_block.merge_chunk(jump_chunk); 

                // then we emit the body chunk
                let combined_block = combined_block.merge_chunk(body_block);

                // then we emit a jump chunk that skips to the end of the else block if it exists
                // then we emit the else block if it exists

                // TODO: we are missing a POP somewhere in here.
                // occurs when the if statement falls through and there's no else block.

                if let Some(block) = else_block {
                    let jump_size: isize = block.instruction_size() as isize;

                    // depending on which jump is emitted we need to add the size of the jump to the offset!!!
                    // we also need to consider if we're jumping from the START of the jump instruction (we are) vs. the end of the jump instruction


                    // TODO: do we pop after an if expression? probably not given expression semantics
                    // unless we use an if in a statement position
                    // hence we should probably emit a no-op to jump to after the else block for the main body
                    let jump_chunk = if jump_size + 2 > 127 {
                        // emit non-"local" jump
                        build_chunk!(OpCode::JumpFar { offset: jump_size + 2 + (std::mem::size_of::<isize>() as isize) }, OpCode::Pop)
                    } else {
                        // TODO: Why does this have to be 3? what is emitting the 2 after the POP opcode?
                        // 3 because i8 + 2. why 2?
                        // this offset needs to take into account the size of the else block...
                        build_chunk!(OpCode::JumpLocal { offset: (jump_size + 3) as i8 }, OpCode::Pop)
                    };

                    let combined_block = combined_block.merge_chunk(jump_chunk);
                    combined_block.merge_chunk(block)
                } else {
                    // let second_op = match body.information().ty {
                    //     super::typeck::TypeInformation::None => OpCode::Noop,
                    //     _ => OpCode::Pop
                    // };

                    let jump_chunk = build_chunk!(OpCode::JumpLocal { offset: 3 }, OpCode::Pop);
                    combined_block.merge_chunk(jump_chunk)
                }

            },
            Expression::BlockExpression { statements, information: _ } => {
                // we are at scope_depth +1 here.
                // we need to care about scope depth because when expressions are searched, they need to search their localized symbol table for the identifier that matches their scope depth and ID.
                let mut block_chunk = Chunk::new();
                for stmt in statements {
                    block_chunk = block_chunk.merge_chunk(self.visit_statement(stmt)?);
                }

                block_chunk
            },
            Expression::LoopExpression { predicate, body, information: _ } => {
                let body_chunk = self.visit_expression(body)?;
                
                let enc_pred = if let Some(x) = predicate {
                    self.visit_expression(x)?
                } else {
                    Chunk::new()
                };

                // TODO: this might not bypass the jump_to_beginning chunk...
                // jump_to_beginning might be jump far or jump local. we need to know the size of the pred + body here
                // which we do know.
                let jump_to_end_offset: usize = body_chunk.instruction_size() + enc_pred.instruction_size() + 5;
                let jump_to_end_chunk = match predicate {
                    Some(_) => build_conditional_jump(jump_to_end_offset, false),
                    None => Chunk::new(),
                };

                let loop_head = enc_pred.merge_chunk(jump_to_end_chunk).merge_chunk(body_chunk);

                // removing jump optimization to ensure size is known
                // size = 5
                let jump_to_beginning_chunk: Chunk = match predicate {
                    Some(_) => unconditional_far_jump(loop_head.instruction_size(), true),
                    None => unconditional_far_jump(loop_head.instruction_size(), true),
                };

                loop_head.merge_chunk(jump_to_beginning_chunk)
            },
            Expression::Call { callee: function, args, information: _ } => {
                // first we evaluate all arguments and ensure they're on the stack
                let mut call_chunk = Chunk::new();

                for arg in args {
                    let arg_ck = self.visit_expression(arg)?;
                    call_chunk = call_chunk.merge_chunk(arg_ck);
                }

                // put the function on top of the stack and call
                let callee = self.visit_expression(&function)?;
                call_chunk = call_chunk.merge_chunk(callee);
                let op_ck = build_chunk!(OpCode::CallDirect { name_offset: 0 });

                call_chunk.merge_chunk(op_ck)
            },
        })
    }

    fn visit_statement(
        &self,
        stmt: &Statement<TypeCkOutput>,
    ) -> Result<Self::ChunkOut, super::TransformError> {
        match stmt {
            Statement::ExpressionStatement { expression, information: _ } => {
                let expr_chunk = self.visit_expression(expression)?;

                match expression.information().ty {
                    super::typeck::TypeInformation::None => Ok(expr_chunk),
                    _ => {
                        let pop_chunk = build_chunk!(OpCode::Pop);
                        Ok(expr_chunk.merge_chunk(pop_chunk))        
                    }
                }
            },
            Statement::PrintStatement { expression, information: _ } => {
                let print_chunk = build_chunk!(OpCode::DebugPrint);
                let expr_chunk = self.visit_expression(expression)?;
                Ok(expr_chunk.merge_chunk(print_chunk))
            },
            Statement::VarStatement { ident, init, information: info } => {
                // we will emit the initializer and then define the global here. note that `information` may eventually carry scoping information
                // for now all variables are globals. this should change when lexical scoping is added
                // TODO: should we pop here?
                match info.pi.scope_depth {
                    0 => {
                        // emit global when sd == 0
                        let hv = ThetaConstant::Str(ident.id().to_owned());
                        let init_chunk = self.visit_expression(init)?;
                        let glob_chunk = build_chunk!(OpCode::DefineGlobal { offset: 0 }; hv);
                        Ok(init_chunk.merge_chunk(glob_chunk))
                    },
                    sd => {
                        // emit local when sd > 0
                        let init_chunk = self.visit_expression(init)?;
                        let local = info.pi.current_symbol_table.borrow().get_symbol_data(ident, sd);
                        match local {
                            Some(SymbolData::Type { ty: _ }) => return Err(TransformError::from(ToByteCodeError::InvalidLocal(ident.id().clone()))),
                            Some(SymbolData::GlobalVariable { ty: _ }) => return Err(TransformError::from(ToByteCodeError::InvalidLocal(ident.id().clone()))),
                            Some(SymbolData::LocalVariable { ty: _, scope_level: _, slot }) => {
                                let glob_chunk = build_chunk!(OpCode::DefineLocal { offset: slot });
                                Ok(init_chunk.merge_chunk(glob_chunk))
                            },
                            Some(SymbolData::Function { return_ty: _, args: _, fn_ty: _ }) => {
                                // build closure and embed it
                                todo!()
                            },
                            None => return Err(TransformError::from(ToByteCodeError::NoIdentFound(ident.id().clone())))
                        }
                    },
                }
            },
        }
    }

    fn visit_function(&self, func: &Function<TypeCkOutput>) -> Result<ThetaFunction, TransformError> {

        let internal_ck = self.transform_tree(&func.chunk)?;

        let internal_args = func.args.iter().map(|x| ThetaFuncArg { ty: x.ty.clone() }).collect();


        Ok(ThetaFunction {
            args: internal_args,
            chunk: internal_ck,
            name: ThetaString::from(func.name.clone()),
            return_ty: func.return_ty.clone(),
        })
    }
}

#[derive(Debug)]
pub enum ToByteCodeError {
    InvalidToken(String),
    InvalidLocal(String),
    NoIdentFound(String),
}

impl Display for ToByteCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToByteCodeError::InvalidToken(s) => write!(f, "Invalid Token: {}", s),
            ToByteCodeError::InvalidLocal(s) => write!(f, "Invalid Local with Identifier: {}", s),
            ToByteCodeError::NoIdentFound(s) => write!(f, "No identifier found with name {}", s),
        }
    }
}

impl Error for ToByteCodeError {}

fn build_conditional_jump(jump_size: usize, negate_offset: bool) -> Chunk {
    if jump_size > 127 {
        conditional_far_jump(jump_size, negate_offset)
    } else {
        conditional_local_jump(jump_size, negate_offset)
    }
}

fn conditional_local_jump(jump_size: usize, negate_offset: bool) -> Chunk {
    let offset = i8::try_from(jump_size).expect("failed to convert to i8, offset?");
    let offset = if negate_offset {
        -offset
    } else {
        offset
    };
    build_chunk!(OpCode::JumpLocalIfFalse { offset })
}

fn conditional_far_jump(jump_size: usize, negate_offset: bool) -> Chunk {
    let offset = isize::try_from(jump_size).expect("failed to convert to isize, offset too large");
    let offset = if negate_offset {
        -offset
    } else {
        offset
    };
    build_chunk!(OpCode::JumpFarIfFalse { offset })
}

fn build_unconditional_jump(jump_size: usize, negate_offset: bool) -> Chunk {
    if jump_size > 127 {
        unconditional_far_jump(jump_size, negate_offset)
    } else {
        unconditional_local_jump(jump_size, negate_offset)
    }
}

fn unconditional_far_jump(jump_size: usize, negate_offset: bool) -> Chunk {
    let offset = isize::try_from(jump_size).expect("failed to convert to isize, offset too large");
    let offset = if negate_offset {
        -offset
    } else {
        offset
    };
    build_chunk!(OpCode::JumpFar { offset })
}

fn unconditional_local_jump(jump_size: usize, negate_offset: bool) -> Chunk {
    let offset = i8::try_from(jump_size).expect("failed to convert to i8, offset?");
    let offset = if negate_offset {
        -offset
    } else {
        offset
    };
    build_chunk!(OpCode::JumpLocal { offset })
}