use theta_types::bytecode::{Token, TokenType};

use super::{Lexer, BasicLexer};

macro_rules! define_single_char_test {
    ($test_name:ident, $expression:expr, $tokenty:expr) => {
        #[test]
        fn $test_name() {
            let input = String::from($expression);

            let mut iter = input.chars();

            let lexer = BasicLexer::new(&mut iter);

            assert_eq!(lexer.lex().expect("should not fail").output().clone(), vec![Token::new(0, 1, $tokenty), Token::new(1, 1, TokenType::Eof)])
        }
    };
}

macro_rules! define_complex_char_test {
    ($test_name:ident, $expression:expr, $tokenty:expr, $toklen:expr) => {
        #[test]
        fn $test_name() {
            let input = String::from($expression);

            let mut iter = input.chars();

            let lexer = BasicLexer::new(&mut iter);

            assert_eq!(lexer.lex().expect("should not fail").output().clone(), vec![Token::new(0, $toklen, $tokenty), Token::new($toklen, $toklen, TokenType::Eof)])
        }
    };
}

define_single_char_test!(basic_lexer_recog_lparen, "(", TokenType::LeftParen);
define_single_char_test!(basic_lexer_recog_rparen, ")", TokenType::RightParen);
define_single_char_test!(basic_lexer_recog_lbrace, "[", TokenType::LeftBrace);
define_single_char_test!(basic_lexer_recog_rbrace, "]", TokenType::RightBrace);
define_single_char_test!(basic_lexer_recog_comma, ",", TokenType::Comma);
define_single_char_test!(basic_lexer_recog_dot, ".", TokenType::Dot);
define_single_char_test!(basic_lexer_recog_minus, "-", TokenType::Minus);
define_single_char_test!(basic_lexer_recog_plus, "+", TokenType::Plus);
define_single_char_test!(basic_lexer_recog_semicolon, ";", TokenType::Semicolon);
define_single_char_test!(basic_lexer_recog_slash, "/", TokenType::Slash);
define_single_char_test!(basic_lexer_recog_star, "*", TokenType::Star);
define_single_char_test!(basic_lexer_recog_colon, ":", TokenType::Colon);
define_single_char_test!(basic_lexer_recog_bang, "!", TokenType::Bang);
define_complex_char_test!(basic_lexer_recog_bangequal, "!=", TokenType::BangEqual, 2);
define_single_char_test!(basic_lexer_recog_equal, "=", TokenType::Equal);
define_complex_char_test!(basic_lexer_recog_equalequal, "==", TokenType::EqualEqual, 2);
define_single_char_test!(basic_lexer_recog_greater, ">", TokenType::Greater);
define_complex_char_test!(basic_lexer_recog_gte, ">=", TokenType::GreaterEqual, 2);
define_single_char_test!(basic_lexer_recog_less, "<", TokenType::Less);
define_complex_char_test!(basic_lexer_recog_lte, "<=", TokenType::LessEqual, 2);
define_complex_char_test!(basic_lexer_recog_arrow, "->", TokenType::Arrow, 2);

define_complex_char_test!(basic_lexer_recog_and, "and", TokenType::And, 3);
define_complex_char_test!(basic_lexer_recog_class, "class", TokenType::Class, 5);
define_complex_char_test!(basic_lexer_recog_else, "else", TokenType::Else, 4);
define_complex_char_test!(basic_lexer_recog_false, "false", TokenType::False, 5);
define_complex_char_test!(basic_lexer_recog_fun, "fun", TokenType::Fun, 3);
define_complex_char_test!(basic_lexer_recog_for, "for", TokenType::For, 3);
define_complex_char_test!(basic_lexer_recog_if, "if", TokenType::If, 2);
define_complex_char_test!(basic_lexer_recog_or, "or", TokenType::Or, 2);
define_complex_char_test!(basic_lexer_recog_return, "return", TokenType::Return, 6);
define_complex_char_test!(basic_lexer_recog_super, "super", TokenType::Super, 5);
define_complex_char_test!(basic_lexer_recog_this, "this", TokenType::This, 4);
define_complex_char_test!(basic_lexer_recog_true, "true", TokenType::True, 4);
define_complex_char_test!(basic_lexer_recog_let, "let", TokenType::Let, 3);
define_complex_char_test!(basic_lexer_recog_while, "while", TokenType::While, 5);

#[test]
fn basic_lexer_recog_line_comment() {
    let input = "// this is a code comment";

    let mut iter = input.chars();

    let lexer = BasicLexer::new(&mut iter);

    assert_eq!(lexer.lex().expect("should not fail").output().clone(), vec![Token::new(25, 25, TokenType::Eof)])
}

#[test]
fn basic_lexer_recog_block_comment() {
    let input = "/*
    this is a block comment
    */";

    let mut iter = input.chars();

    let lexer = BasicLexer::new(&mut iter);

    assert_eq!(lexer.lex().expect("should not fail").output().clone(), vec![Token::new(37, 37, TokenType::Eof)])
}

#[test]
fn basic_lexer_recog_integer() {
    let input = "120";

    let mut iter = input.chars();

    let lexer = BasicLexer::new(&mut iter);

    assert_eq!(lexer.lex().expect("should not fail").output().clone(), vec![Token::new(0, 3, TokenType::Integer(120)), Token::new(3, 3, TokenType::Eof)])
}

#[test]
fn basic_lexer_recog_float() {
    let input = "1.0";

    let mut iter = input.chars();

    let lexer = BasicLexer::new(&mut iter);

    assert_eq!(lexer.lex().expect("should not fail").output().clone(), vec![Token::new(0, 3, TokenType::Float(1.0)), Token::new(3, 3, TokenType::Eof)])
}

#[test]
fn basic_lexer_not_recog_float_leading_period() {
    let input = ".1";

    let mut iter = input.chars();

    let lexer = BasicLexer::new(&mut iter);

    assert_eq!(lexer.lex().expect("should not fail").output().clone(), 
        vec![
            Token::new(0, 1, TokenType::Dot), 
            Token::new(1, 2, TokenType::Integer(1)), 
            Token::new(2, 2, TokenType::Eof)
        ])
}

#[test]
fn basic_lexer_recog_float_trailing_dot() {
    let input = "1.";

    let mut iter = input.chars();

    let lexer = BasicLexer::new(&mut iter);

    assert_eq!(lexer.lex().expect("should not fail").output().clone(), 
        vec![ 
            Token::new(0, 2, TokenType::Float(1.0)), 
            Token::new(2, 2, TokenType::Eof)
        ])
}