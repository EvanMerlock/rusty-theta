use clap::{Parser as ClapParser};
use utf8_chars::BufReadCharsExt;
use theta_lang::lexer::{Lexer, BasicLexer};
use theta_lang::parser::{Parser, BasicParser};
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

    let tokens = lexer.lex();
    let mut tok_stream = tokens.into_iter();

    let parser = BasicParser::new(&mut tok_stream);
    let expression = parser.parse()?;

    write!(&mut out_file, "{:?}", expression)?;

    Ok(())
}