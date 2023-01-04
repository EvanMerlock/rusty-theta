use crate::{lexer::token::{Token, TokenType}, token, literal, binary, statement};

macro_rules! define_parse_test {
    ($test_name:ident, $input:expr, $output:expr) => {
        #[test]
        fn $test_name() {
            use crate::ast::{AbstractTree, Expression, Statement};
            use super::{Parser, BasicParser};
            let input = $input;
            let mut iter = input.into_iter();
            {
                let parser = BasicParser::new(&mut iter);

                let actual_out = parser.parse();
    
                assert_eq!(actual_out.is_ok(), true);
                assert_eq!(actual_out.expect("failed to unwrap in test").0.strip_information(), $output);
            }
        }
    };
}

const SEMICOLON_TOKEN: Token = token!(TokenType::Semicolon);

const LITERAL_LEFT_2: Token = token!(TokenType::Integer(1));
const LITERAL_RIGHT_2: Token = token!(TokenType::Integer(3));
const LITERAL_BINARY_2: Token = token!(TokenType::Plus);
const BINARY_TEST_1: [Token; 4] = [LITERAL_LEFT_2, LITERAL_BINARY_2, LITERAL_RIGHT_2, SEMICOLON_TOKEN];

define_parse_test!(basic_parser_recog_binary, BINARY_TEST_1, AbstractTree::statement(statement!(Expr: binary!(literal!(LITERAL_LEFT_2), LITERAL_BINARY_2, literal!(LITERAL_RIGHT_2))), ()));

const LITERAL_LEFT_3: Token = Token::new(1, 0, 1, TokenType::True);
const LITERAL_RIGHT_3: Token = Token::new(1, 2, 3, TokenType::True);
const LITERAL_BINARY_3: Token = Token::new(1, 1, 2, TokenType::EqualEqual);
const BINARY_TEST_2: [Token; 4] = [LITERAL_LEFT_3, LITERAL_BINARY_3, LITERAL_RIGHT_3, SEMICOLON_TOKEN];

define_parse_test!(basic_parser_recog_bool, BINARY_TEST_2, AbstractTree::statement(statement!(Expr: binary!(literal!(LITERAL_LEFT_3), LITERAL_BINARY_3, literal!(LITERAL_RIGHT_3))), ()));
