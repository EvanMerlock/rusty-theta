use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

use log::{LevelFilter, debug};

use crate::ast::symbol::{SymbolTable, ExtSymbolTable};
use crate::ast::transformers::typeck::TypeCk;
use crate::ast::transformers::{to_bytecode::ToByteCode, ASTTransformer};
use crate::bytecode::{BasicAssembler, Assembler, Disassembler, OpCode, Chunk};
use crate::{vm::{VM}, build_chunk, lexer::{BasicLexer, Lexer}, parser::{BasicParser, Parser}};

pub struct Repl {
    machine: VM,
    tbl: ExtSymbolTable,
}

pub enum ReplStatus {
    ReplOk,
    ReplTerminate,
}

impl Repl {
    pub fn init() -> Repl {
        Repl {
            machine: VM::new(),
            tbl: Rc::new(RefCell::new(SymbolTable::default())),
        }
    }

    pub fn line(&mut self, valid_line: String) -> Result<ReplStatus, Box<dyn std::error::Error>> {
                // CONVERT LINE TO CHUNKS
                if valid_line.starts_with("--") && log::max_level() == LevelFilter::Debug {
                    match valid_line.as_str().trim_end() {
                        "--stack" => {
                            debug!("Stack: {:?}", self.machine.stack());
                        },
                        "--constants" => {
                            debug!("Constants: {:?}", self.machine.constants());
                        },
                        "--heap" => {
                            debug!("Heap: {:?}", self.machine.heap());
                        },
                        "--globals" => {
                            debug!("Globals: {:#?}", self.machine.globals());
                        },
                        "--symbols" => {
                            debug!("Symbol Table: {:#?}", self.tbl);
                        }
                        "--quit" | "--exit" => {
                            return Ok(ReplStatus::ReplTerminate);
                        },
                        _ => {},
                    }
                    return Ok(ReplStatus::ReplOk);
                }
        
                let mut chars = valid_line.chars();
                let lexer = BasicLexer::new(&mut chars);
                let tokens = lexer.lex()?;
                let mut token_stream = tokens.into_iter();
                let parser = BasicParser::new_sym(&mut token_stream, self.tbl.clone());
                let (ast, sym) = parser.parse()?;
                debug!("sym: {:?}", sym.borrow());
                let type_cker = TypeCk::new(sym.clone());
                self.tbl = sym;
                let type_check = type_cker.transform(&ast)?;
                let chunk = ToByteCode.transform(&type_check)?;
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