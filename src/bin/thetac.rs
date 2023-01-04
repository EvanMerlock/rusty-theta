use clap::{Parser as ClapParser, clap_derive::ArgEnum};
use theta_lang::bytecode::{BasicAssembler, Assembler, PlainTextAssembler, AssembleError};
use std::{fs::File, io::BufReader};

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
    
    let chunks = todo!();

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