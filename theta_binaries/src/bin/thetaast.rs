use clap::Parser as ClapParser;
use theta_compiler::lexer::{BasicLexer, Lexer};
use theta_compiler::parser::{BasicParser, Parser};
use utf8_chars::BufReadCharsExt;
use std::fs::File;
use std::io::BufReader;

#[derive(ClapParser)]
#[clap(version = "0.0.1", author = "Evan Merlock")]
struct ThetaASTOptions {
    #[clap(short, long)]
    in_file: Option<String>,
    #[clap(short, long)]
    out_file: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let options = ThetaASTOptions::parse();

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

    let mut iter = in_file.chars().map(|x| x.unwrap());
    let lexer = BasicLexer::new(&mut iter);

    let tokens = lexer.lex()?;

    let parser = BasicParser::new(tokens.output());
    let pi = parser.parse()?;

    for item in pi {
        write!(&mut out_file, "{:?}", item)?;
    }


    Ok(())
}