use theta_lang::{
    ast::{
        transformers::typeck::TypeInformation, AbstractTree, Expression, Function, FunctionArg,
        Item, Statement,
    },
    bytecode::Symbol,
    lexer::{
        token::{Token, TokenType},
        BasicLexer, Lexer,
    },
    parser::{BasicParser, Parser},
};

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

const FIB_TEST: &'static str = "fun fib(n: Int) -> Int { if (n <= 1) { return n; }; fib(n-1)+fib(n-2) }";

#[test]
fn function_generated() {
    init();

    let test_case = "
        fun test() {
            print(1);
        }";

    let mut characters = test_case.chars();
    let lexer = BasicLexer::new(&mut characters);

    let tokens = lexer.lex().expect("failed to get tokens");

    let parser = BasicParser::new(&tokens);

    let item = parser.parse().expect("failed to parse");

    let expected = match &item[0] {
        Item::Function(func) => func.clone(),
        _ => panic!("not matched function"),
    };

    assert_eq!(
        Function {
            args: vec![],
            chunk: AbstractTree::expression(
                Expression::BlockExpression {
                    statements: vec![Statement::PrintStatement {
                        expression: Expression::Literal {
                            literal: Token::new(1, 0, 1, TokenType::Integer(1)),
                            information: ()
                        },
                        information: ()
                    }],
                    information: (),
                    final_expression: None
                },
                ()
            ),
            name: Symbol::from("test"),
            return_ty: TypeInformation::None,
            information: ()
        },
        expected.strip_information().strip_token_information()
    )
}

#[test]
fn function_with_args_generated() {
    init();

    let test_case = "
        fun test(t: String) {
            print(1);
        }";

    let mut characters = test_case.chars();
    let lexer = BasicLexer::new(&mut characters);

    let tokens = lexer.lex().expect("failed to get tokens");

    let parser = BasicParser::new(&tokens);

    let item = parser.parse().expect("failed to parse");

    let expected = match &item[0] {
        Item::Function(func) => func.clone(),
        _ => panic!("not matched function"),
    };

    assert_eq!(expected.name.id(), "test");

    assert_eq!(
        Function {
            args: vec![FunctionArg {
                name: Symbol::from("t"),
                ty: TypeInformation::String
            }],
            chunk: AbstractTree::expression(
                Expression::BlockExpression {
                    statements: vec![Statement::PrintStatement {
                        expression: Expression::Literal {
                            literal: Token::new(1, 0, 1, TokenType::Integer(1)),
                            information: ()
                        },
                        information: ()
                    }],
                    information: (),
                    final_expression: None
                },
                ()
            ),
            name: expected.name.clone(),
            return_ty: TypeInformation::None,
            information: ()
        },
        expected.strip_information().strip_token_information()
    )
}

#[test]
fn function_with_return_type_generated() {
    init();

    let test_case = "
        fun test() -> String {
            print(1);
        }";

    let mut characters = test_case.chars();
    let lexer = BasicLexer::new(&mut characters);

    let tokens = lexer.lex().expect("failed to get tokens");

    let parser = BasicParser::new(&tokens);

    let item = parser.parse().expect("failed to parse");

    let expected = match &item[0] {
        Item::Function(func) => func.clone(),
        _ => panic!("not matched function"),
    };

    assert_eq!(
        Function {
            args: vec![],
            chunk: AbstractTree::expression(
                Expression::BlockExpression {
                    statements: vec![Statement::PrintStatement {
                        expression: Expression::Literal {
                            literal: Token::new(1, 0, 1, TokenType::Integer(1)),
                            information: ()
                        },
                        information: ()
                    }],
                    information: (),
                    final_expression: None
                },
                ()
            ),
            name: Symbol::from("test"),
            return_ty: TypeInformation::String,
            information: ()
        },
        expected.strip_information().strip_token_information()
    )
}
