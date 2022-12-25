use clap::{Parser as ClapParser, clap_derive::ArgEnum};
use log::{LevelFilter, debug};
use theta_lang::{vm::{chunk::{Chunk, self}, bytecode::{BasicAssembler, Assembler, PlainTextAssembler, AssembleError, Disassembler}, instruction::OpCode, value::ThetaValue, VM}, lexer::{BasicLexer, Lexer}, parser::{BasicParser, Parser}, transformers::{to_bytecode::ToByteCode, ASTTransformer}};
use std::{fs::File, io::{BufReader, Cursor, Write}};
use theta_lang::build_chunk;
#[derive(ClapParser)]
#[clap(version = "0.0.1", author = "Evan Merlock")]
struct ThetaCOptions {
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let options = ThetaCOptions::parse();

    // REPL
    let mut machine = VM::new();

    // READ IN LINE
    for line in std::io::stdin().lines() {
        // CONVERT LINE TO CHUNKS
        let valid_line = line?;

        if valid_line.starts_with("--") && log::max_level() == LevelFilter::Debug {
            match valid_line.as_str() {
                "--stack" => {
                    debug!("Stack: {:?}", machine.stack());
                },
                "--ret" => {
                    let chunks = vec![build_chunk!(OpCode::RETURN)];
                    {
                        let mut code = Box::new(Cursor::new(Vec::new()));
                        {
                            let mut assembler = BasicAssembler::new(&mut code);
                
                            assembler.assemble(chunks)?;
                        }
                        machine.disassemble(code.get_ref())?;
                        machine.clear_const_pool();
                    }
                },
                "--constants" => {
                    debug!("Constants: {:?}", machine.constants());
                }
                _ => {},
            }
            continue;
        }

        let mut chars = valid_line.chars();
        let lexer = BasicLexer::new(&mut chars);
        let tokens = lexer.lex();
        let mut token_stream = tokens.into_iter();
        let parser = BasicParser::new(&mut token_stream);
        let ast = parser.parse()?;
        let chunk = ToByteCode::transform(ast)?;
        let chunks: Vec<Chunk> = vec![chunk];
        {
            let mut code = Box::new(Cursor::new(Vec::new()));
            {
                let mut assembler = BasicAssembler::new(&mut code);
    
                assembler.assemble(chunks)?;
            }
            machine.disassemble(code.get_ref())?;
            //TODO: should not need to do this. linking/relative offsetting?
            machine.clear_const_pool();
        }
    
    }


    Ok(())
}