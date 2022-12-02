use clap::{AppSettings, Clap};
use theta_lang::vm::bytecode::{StringDisassembler, Disassembler};
use std::fs::File;
use std::io::BufReader;

#[derive(Clap)]
#[clap(version = "0.0.1", author = "Evan Merlock")]
#[clap(setting = AppSettings::ColoredHelp)]
struct ThetaDOptions {
    #[clap(short, long)]
    in_file: Option<String>,
    #[clap(short, long)]
    out_file: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ThetaDOptions::parse();

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

    let mut decompiler = StringDisassembler::new(&mut in_file);
    let chunks = decompiler.disassemble()?;

    write!(&mut out_file, "{}", chunks)?;

    Ok(())
}