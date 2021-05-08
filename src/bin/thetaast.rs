use clap::{AppSettings, Clap};
use utf8_chars::BufReadCharsExt;
use theta_lang::lexer::{Lexer, BasicLexer};
use std::fs::File;
use std::io::BufReader;

#[derive(Clap)]
#[clap(version = "0.0.1", author = "Evan Merlock")]
#[clap(setting = AppSettings::ColoredHelp)]
struct ThetaASTOptions {
    #[clap(short, long)]
    in_file: String,
    #[clap(short, long)]
    out_file: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() -> Result<(), std::io::Error> {
    let options = ThetaASTOptions::parse();

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

    write!(&mut out_file, "{:?}", tokens)?;

    Ok(())
}