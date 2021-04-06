use std::io::prelude::*;

mod token;

trait Lexer {
    fn lex<'a>(file: &'a dyn Read) -> Vec<token::Token>;
}