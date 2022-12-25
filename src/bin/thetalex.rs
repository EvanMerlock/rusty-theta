use clap::{Parser as ClapParser};
use utf8_chars::BufReadCharsExt;
use theta_lang::lexer::{Lexer, BasicLexer};
use std::fs::File;
use std::io::BufReader;

#[derive(ClapParser)]
#[clap(version = "0.0.1", author = "Evan Merlock")]
struct ThetaLexOptions {
    #[clap(short, long)]
    in_file: String,
    #[clap(short, long)]
    out_file: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let options = ThetaLexOptions::parse();

    let mut in_file = BufReader::new(File::open(options.in_file)?);

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

    let mut line = 1;
    for token in tokens {
        if token.line_num() != line {
            writeln!(&mut out_file)?;
            line = token.line_num();
        }

        write!(&mut out_file, "{:?}, ", token)?;
    }

    Ok(())
}