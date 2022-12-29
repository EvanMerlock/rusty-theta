use std::io::Cursor;

use log::{LevelFilter, debug};

use crate::ast::transformers::typeck::TypeCk;
use crate::ast::transformers::{to_bytecode::ToByteCode, ASTTransformer};
use crate::bytecode::{BasicAssembler, Assembler, Disassembler, OpCode, Chunk};
use crate::{vm::{VM}, build_chunk, lexer::{BasicLexer, Lexer}, parser::{BasicParser, Parser}};

pub struct Repl {
    machine: VM,
}

pub enum ReplStatus {
    ReplOk,
    ReplTerminate,
}

impl Repl {
    pub fn init() -> Repl {
        Repl {
            machine: VM::new(),
        }
    }

    pub fn line(&mut self, valid_line: String) -> Result<ReplStatus, Box<dyn std::error::Error>> {
                // CONVERT LINE TO CHUNKS
                if valid_line.starts_with("--") && log::max_level() == LevelFilter::Debug {
                    match valid_line.as_str().trim_end() {
                        "--stack" => {
                            debug!("Stack: {:?}", self.machine.stack());
                        },
                        "--ret" => {
                            let chunks = vec![build_chunk!(OpCode::RETURN)];
                            {
                                let mut code = Box::new(Cursor::new(Vec::new()));
                                {
                                    let mut assembler = BasicAssembler::new(&mut code);
                        
                                    assembler.assemble(chunks)?;
                                }
                                self.machine.disassemble(code.get_ref())?;
                                self.machine.clear_const_pool();
                            }
                        },
                        "--constants" => {
                            debug!("Constants: {:?}", self.machine.constants());
                        },
                        "--heap" => {
                            debug!("Heap: {:?}", self.machine.heap());
                        }
                        "--quit" => {
                            return Ok(ReplStatus::ReplTerminate);
                        }
                        _ => {},
                    }
                    return Ok(ReplStatus::ReplOk);
                }
        
                let mut chars = valid_line.chars();
                let lexer = BasicLexer::new(&mut chars);
                let tokens = lexer.lex()?;
                let mut token_stream = tokens.into_iter();
                let parser = BasicParser::new(&mut token_stream);
                let ast = parser.parse()?;
                let type_check = TypeCk::transform(&ast)?;
                let chunk = ToByteCode::transform(&type_check)?;
                let chunks: Vec<Chunk> = vec![chunk];
                {
                    let mut code = Box::new(Cursor::new(Vec::new()));
                    {
                        let mut assembler = BasicAssembler::new(&mut code);
            
                        assembler.assemble(chunks)?;
                    }
                    self.machine.disassemble(code.get_ref())?;
                    //TODO: should not need to do this. linking/relative offsetting?
                    self.machine.clear_const_pool();
                }
                Ok(ReplStatus::ReplOk)
    }
}