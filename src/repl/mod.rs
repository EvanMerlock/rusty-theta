use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::rc::Rc;

use log::{LevelFilter, debug};

use crate::ast::Item;
use crate::ast::symbol::{SymbolTable, ExtSymbolTable};
use crate::ast::transformers::typeck::TypeCk;
use crate::ast::transformers::{to_bytecode::ToByteCode, ASTTransformer};
use crate::bytecode::{BasicAssembler, Assembler, Disassembler, Chunk, ThetaBitstream, ThetaString, ThetaFunction, ThetaCompiledFunction, ThetaCompiledBitstream, ThetaValue, BasicDisassembler};
use crate::parser::{ReplParser, ReplItem};
use crate::{vm::VM, lexer::{BasicLexer, Lexer}, parser::{BasicParser, Parser}};

// TODO: move REPL to using a direct to instruction assembler. Then we don't need the disassembly step.
// chunk building should only occur when a stream of bytecode is known.
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
            tbl: ExtSymbolTable::default(),
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
                        "--strings" => {
                            debug!("Interned Strings: {:#?}", self.machine.strings());
                        },
                        "--functions" => {
                            debug!("Function Pool: {:#?}", self.machine.functions());
                        },
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
                let parser = ReplParser::new(parser);
                let trees = parser.parse()?;

                let mut bitstream = ThetaBitstream::new();
                let mut chunk = Chunk::new();

                for item in trees {
                    self.repl_item(item, &mut bitstream, &mut chunk)?;
                }

                // compile bitstream 
                
                let mut compiled_bitstream = Vec::new();
                let mut basic_assembler = BasicAssembler::new(&mut compiled_bitstream);
                basic_assembler.assemble_bitstream(bitstream)?;

                // load into machine

                let mut intern_fn = |x| self.machine.intern_string(x);
                let mut basic_diassembler = BasicDisassembler::new(&mut intern_fn);
                let comp_bs = basic_diassembler.disassemble(&compiled_bitstream)?;
                self.machine.load_bitstream(comp_bs);
                

                if !chunk.instructions().is_empty() {
                    // compile chunk
                    let mut compiled_chunk = Vec::new();
                    let mut basic_assembler = BasicAssembler::new(&mut compiled_chunk);
                    basic_assembler.assemble_chunk(chunk)?;
    
                    // execute chunk
                    self.machine.execute_code(&compiled_chunk)?;
                }

                Ok(ReplStatus::ReplOk)
    }

    fn repl_item(&mut self, item: ReplItem, bitstream: &mut ThetaBitstream, chunk: &mut Chunk) -> Result<(), Box<dyn std::error::Error>> {
        match item {
            ReplItem::ParserItem(pi) => {
                    let sym = &pi.information().current_symbol_table;
                    debug!("sym: {:?}", sym.borrow());
                    let type_cker = TypeCk::new(sym.clone());
                    let type_check = type_cker.transform_item(&pi)?;
                    let mut theta_func = ToByteCode.transform_item(&type_check)?;

                    // need to pull out constants and reloc
                    let consts = theta_func.chunk.constants();

                    let reloc = bitstream.constants.len();
                    bitstream.constants.extend_from_slice(consts);
                    
                    let new_ck = theta_func.chunk.relocate(reloc);
                    theta_func.chunk = new_ck;
                    bitstream.functions.push(theta_func);
            },
            ReplItem::Declaration(decl) => {
                let sym = decl.information().current_symbol_table.clone();
                debug!("sym: {:?}", sym.borrow());
                let type_cker = TypeCk::new(sym);
                let type_check = type_cker.transform_tree(&decl)?;
                let ty_chunk = ToByteCode.transform_tree(&type_check)?;

                let reloc = bitstream.constants.len();
                let new_chunk = ty_chunk.relocate(reloc);

                bitstream.constants.extend_from_slice(new_chunk.constants());

                let new_ck = chunk.clone().merge_chunk(new_chunk);
                *chunk = new_ck;
            },
        };

        Ok(())
    }
}