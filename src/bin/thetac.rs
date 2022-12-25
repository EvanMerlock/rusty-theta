use clap::{Parser as ClapParser, clap_derive::ArgEnum};
use theta_lang::vm::{chunk::{Chunk, self}, bytecode::{BasicAssembler, Assembler, PlainTextAssembler, AssembleError}, instruction::OpCode, value::ThetaValue};
use std::{fs::File, io::BufReader};
use theta_lang::build_chunk;
#[derive(ClapParser)]
#[clap(version = "0.0.1", author = "Evan Merlock")]
struct ThetaCOptions {
    #[clap(short, long)]
    in_file: Option<String>,
    #[clap(short, long)]
    out_file: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(arg_enum)]
    assembler: AssemblerImpl
}

#[derive(Clone, ArgEnum)]
enum AssemblerImpl {
    Basic,
    String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let options = ThetaCOptions::parse();

    let mut in_file: Box<dyn std::io::BufRead> = 
    {
        if options.in_file.is_some() {
            Box::new(BufReader::new(File::open(options.in_file.unwrap())?))
        } else {
            Box::new(BufReader::new(std::io::stdin()))
        }
    };

    let mut out_file: Box<dyn std::io::Write> =
    {
        if options.out_file.is_some() {
            Box::new(File::create(options.out_file.unwrap())?)
        } else {
            Box::new(std::io::stdout())
        }
    };

    let mut chunks: Vec<Chunk> = Vec::new();
    chunks.push(build_chunk!(OpCode::RETURN, OpCode::CONSTANT { offset: 0 }; ThetaValue::Double(16.0)));
    
    {
        let mut assembler: Box<dyn Assembler<Out = Result<(), AssembleError>>> = match options.assembler {
            AssemblerImpl::Basic => Box::new(BasicAssembler::new(&mut out_file)),
            AssemblerImpl::String => Box::new(PlainTextAssembler::new(&mut out_file)),
        };
        assembler.assemble(chunks)?;
    }

    out_file.flush()?;

    Ok(())
}