use theta_types::{bytecode::{Token, TokenType}, token, statement, binary, if_expression, literal};

use crate::ast::{Expression, Statement};

macro_rules! define_parse_test {
    ($test_name:ident, $input:expr, $output:expr) => {
        #[test]
        fn $test_name() {
            use crate::ast::AbstractTree;
            use super::BasicParser;
            let input = $input;
            {
                let mut parser = BasicParser::new(&input);

                let actual_out = parser.declaration();
    
                assert_eq!(actual_out.is_ok(), true);
                let trees = actual_out.expect("failed to unwrap in test");
                let ast: AbstractTree<()> = AbstractTree::statement(trees.strip_information(), ());
                assert_eq!(ast, $output);
            }
        }
    };
}

macro_rules! define_parse_fail_test {
    ($test_name:ident, $input:expr) => {
        #[test]
        fn $test_name() {
            use super::{Parser, BasicParser};
            let input = $input;
            {
                let parser = BasicParser::new(&input);

                let actual_out = parser.parse();
    
                assert_eq!(actual_out.is_err(), true);
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

const LITERAL_LEFT_3: Token = Token::new(0, 1, TokenType::True);
const LITERAL_RIGHT_3: Token = Token::new(2, 3, TokenType::True);
const LITERAL_BINARY_3: Token = Token::new(1, 2, TokenType::EqualEqual);
const BINARY_TEST_2: [Token; 4] = [LITERAL_LEFT_3, LITERAL_BINARY_3, LITERAL_RIGHT_3, SEMICOLON_TOKEN];
define_parse_test!(basic_parser_recog_bool, BINARY_TEST_2, AbstractTree::statement(statement!(Expr: binary!(literal!(LITERAL_LEFT_3), LITERAL_BINARY_3, literal!(LITERAL_RIGHT_3))), ()));

const IF_TEST_1: [Token; 6] = [
    token!(TokenType::If),
    token!(TokenType::LeftParen),
    token!(TokenType::True),
    token!(TokenType::RightParen),
    token!(TokenType::Integer(1)),
    token!(TokenType::Semicolon),
];

const IF_COND_1: Expression<()> = Expression::Literal { literal: token!(TokenType::True), information: () };
const IF_BODY_1: Expression<()> = Expression::Literal { literal: token!(TokenType::Integer(1)), information: () };

define_parse_test!(basic_parser_if_no_else, IF_TEST_1, AbstractTree::statement(statement!(
    Expr: if_expression!(IF_COND_1, IF_BODY_1)
), ()));

const IF_TEST_2: [Token; 8] = [
    token!(TokenType::If),
    token!(TokenType::LeftParen),
    token!(TokenType::True),
    token!(TokenType::RightParen),
    token!(TokenType::Integer(1)),
    token!(TokenType::Else),
    token!(TokenType::Integer(2)),
    token!(TokenType::Semicolon),
];

const IF_COND_2: Expression<()> = Expression::Literal { literal: token!(TokenType::True), information: () };
const IF_BODY_2: Expression<()> = Expression::Literal { literal: token!(TokenType::Integer(1)), information: () };
const IF_ELSE_2: Expression<()> = Expression::Literal { literal: token!(TokenType::Integer(2)), information: () };


define_parse_test!(basic_parser_if_else, IF_TEST_2, AbstractTree::statement(statement!(
    Expr: if_expression!(IF_COND_2, IF_BODY_2, IF_ELSE_2)
), ()));


const FAIL_TEST_1: [Token; 3] = [token!(TokenType::LeftBrace), token!(TokenType::RightBrace), token!(TokenType::RightBrace)];
define_parse_fail_test!(parser_should_fail_extra_closing_braces, FAIL_TEST_1);

const FAIL_TEST_2: [Token; 3] = [token!(TokenType::LeftBrace), token!(TokenType::RightBrace), token!(TokenType::LeftBrace)];
define_parse_fail_test!(parser_should_fail_extra_opening_braces, FAIL_TEST_2);

const LITERAL_PRINT_STATEMENT_AS_FUNCTION_CALL: Token = token!(TokenType::Integer(1));
const PRINT_STATEMENT_AS_FUNCTION_CALL_OUTPUT: Statement<()> = statement!(
    Print: Expression::Literal { literal: LITERAL_PRINT_STATEMENT_AS_FUNCTION_CALL, information: () }
);
define_parse_test!(print_statement_as_function_call_not_as_sequence, [token!(TokenType::Identifier(String::from("print"))), token!(TokenType::LeftParen), token!(TokenType::Integer(1)), token!(TokenType::RightParen), token!(TokenType::Semicolon)], AbstractTree::statement(PRINT_STATEMENT_AS_FUNCTION_CALL_OUTPUT, ()));