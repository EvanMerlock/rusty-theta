use crate::lexer::token::{Token, TokenType};

macro_rules! define_parse_test {
    ($test_name:ident, $input:expr, $output:expr) => {
        #[test]
        fn $test_name() {
            use super::{Parser, BasicParser, tree::AbstractTree, tree::Expression};
            let input = $input;
            let mut iter = input.into_iter();
            {
                let parser = BasicParser::new(&mut iter);

                let actual_out = parser.parse();
    
                assert_eq!(actual_out.is_ok(), true);
                assert_eq!(actual_out.expect("failed to unwrap in test"), $output);
            }
        }
    };
}

const LITERAL_1: Token = Token::new(1, 0, 1, TokenType::Integer(1));
define_parse_test!(basic_parser_recog_literal, [LITERAL_1], AbstractTree::new(Expression::Literal(LITERAL_1)));

const LITERAL_LEFT_2: Token = Token::new(1, 0, 1, TokenType::Integer(1));
const LITERAL_RIGHT_2: Token = Token::new(1, 2, 3, TokenType::Integer(3));
const LITERAL_BINARY_2: Token = Token::new(1, 1, 2, TokenType::Plus);
const BINARY_TEST_1: [Token; 3] = [LITERAL_LEFT_2, LITERAL_BINARY_2, LITERAL_RIGHT_2];

define_parse_test!(basic_parser_recog_binary, BINARY_TEST_1, AbstractTree::new(Expression::Binary { left: Box::new(Expression::Literal(LITERAL_LEFT_2)), operator: LITERAL_BINARY_2, right: Box::new(Expression::Literal(LITERAL_RIGHT_2)) }));