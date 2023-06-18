use clap::{clap_derive::ArgEnum, Parser as ClapParser};
use log::debug;
use std::{fs::File, io::BufReader, cell::RefCell, rc::Rc};
use theta_lang::{bytecode::{AssembleError, Assembler, BasicAssembler, PlainTextAssembler, Chunk}, lexer::{BasicLexer, Lexer}, parser::{BasicParser, Parser}, ast::{symbol::SymbolTable, transformers::{typeck::TypeCk, to_bytecode::ToByteCode, ASTTransformer}}};

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
    assembler: AssemblerImpl,
}

#[derive(Clone, ArgEnum)]
enum AssemblerImpl {
    Basic,
    String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let options = ThetaCOptions::parse();

    let mut in_file: Box<dyn std::io::BufRead> = {
        if options.in_file.is_some() {
            Box::new(BufReader::new(File::open(options.in_file.unwrap())?))
        } else {
            Box::new(BufReader::new(std::io::stdin()))
        }
    };

    let mut out_file: Box<dyn std::io::Write> = {
        if options.out_file.is_some() {
            Box::new(File::create(options.out_file.unwrap())?)
        } else {
            Box::new(std::io::stdout())
        }
    };

    let mut char_buf = Vec::new();
    in_file.read_to_end(&mut char_buf)?;

    let byte_stream = String::from_utf8(char_buf)?;
    let mut characters = byte_stream.chars();

    let lexer = BasicLexer::new(&mut characters);
    let tokens = lexer.lex()?;
    let mut token_stream = tokens.into_iter();
    let tbl = Rc::new(RefCell::new(SymbolTable::default()));
    let parser = BasicParser::new_sym(&mut token_stream, tbl.clone());
    let trees = parser.parse()?;
    let (ast, sym) = &trees[0];
    debug!("sym: {:?}", sym.borrow());
    let type_cker = TypeCk::new(sym.clone());
    let type_check = type_cker.transform(&ast)?;
    let chunk = ToByteCode.transform(&type_check)?;
    let chunks: Vec<Chunk> = vec![chunk];

    {
        let mut assembler: Box<dyn Assembler<Out = Result<(), AssembleError>>> =
            match options.assembler {
                AssemblerImpl::Basic => Box::new(BasicAssembler::new(&mut out_file)),
                AssemblerImpl::String => Box::new(PlainTextAssembler::new(&mut out_file)),
            };
        assembler.assemble(chunks)?;
    }

    out_file.flush()?;

    Ok(())
}
