use theta_compiler::{lexer::{BasicLexer, Lexer}, parser::{BasicParser, Parser, ParseInfo}, ast::{Item, Function, AbstractTree, Expression, Statement, FunctionArg}};
use theta_types::{bytecode::{Token, TokenType, Symbol}, types::{TypeInformation, LocationData}};

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

const FIB_TEST: &'static str = "fun fib(n: Int) -> Int { if (n <= 1) { return n; }; fib(n-1)+fib(n-2) }";

const LOOP_TEST: &'static str = "fun loop_test() {
    let y: Int = 0;
    while (y < 10) {
        print(\"hello, world\");
    };
}";

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

    let parser = BasicParser::new(&tokens.output());

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
                            literal: Token::new(0, 1, TokenType::Integer(1)),
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

    let parser = BasicParser::new(&tokens.output());

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
                            literal: Token::new(0, 1, TokenType::Integer(1)),
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

    let parser = BasicParser::new(&tokens.output());

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
                            literal: Token::new(0, 1, TokenType::Integer(1)),
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

#[test]
fn information_generated_correctly() {
    init();
    let test_case = "fun fib(n: Int) -> Int {
    if (n <= 1) {
        return n;
    };
    fib(n-1) + fib(n-2)
}";

    let mut characters = test_case.chars();
    let lexer = BasicLexer::new(&mut characters);

    let tokens = lexer.lex().expect("failed to get tokens");

    let parser = BasicParser::new(tokens.output());

    let item = parser.parse().expect("failed to parse");

    let expected = match &item[0] {
        Item::Function(func) => func.clone(),
        _ => panic!("not matched function"),
    };

    assert_eq!(Function {
        args: vec![FunctionArg { name: Symbol::from("n"), ty: TypeInformation::Int }],
        chunk: AbstractTree::expression(
            Expression::BlockExpression {
                statements: vec![
                    Statement::ExpressionStatement { expression: 
                        Expression::If { check_expression: Box::new(Expression::Binary { 
                            left: Box::new(Expression::Literal { literal: Token::new(0, 1, TokenType::Identifier(String::from("n"))), information: LocationData::new(0, 1) }), 
                            operator: Token::new(0, 1, TokenType::LessEqual), 
                            right: Box::new(Expression::Literal { literal: Token::new(0, 1, TokenType::Integer(2)), information: LocationData::new(0, 1) }),
                            information: LocationData::new(0, 1)
                        }), 
                        body: Box::new(
                            Expression::BlockExpression { statements: vec![

                            ], final_expression: None, information: LocationData::new(0, 1) }
                        ), 
                        else_body: None, 
                        information: LocationData::new(0, 1) 
                    }, 
                        information: LocationData::new(0, 1)
                    }
                ],
                information: LocationData::new(0, 1),
                final_expression: Some(Box::new(
                    Expression::Binary { 
                        left: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Literal { literal: Token::new(0, 1, TokenType::Identifier(String::from("fib"))), information: LocationData::new(0, 1) }), 
                            args: vec![
                                Expression::Binary { 
                                    left: Box::new(Expression::Literal { literal: Token::new(0, 1, TokenType::Identifier(String::from("n"))), information: LocationData::new(0, 1) }), 
                                    operator: Token::new(0, 1, TokenType::Minus), 
                                    right: Box::new(Expression::Literal { literal: Token::new(0, 1, TokenType::Integer(1)), information: LocationData::new(0, 1) }), 
                                    information: LocationData::new(0, 1) 
                                },
                                
                            ], 
                            information: LocationData::new(0, 1) 
                        }),
                        operator: Token::new(0, 1, TokenType::Plus),
                        right: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Literal { literal: Token::new(0, 1, TokenType::Identifier(String::from("fib"))), information: LocationData::new(0, 1) }), 
                            args: vec![
                                Expression::Binary { 
                                    left: Box::new(Expression::Literal { literal: Token::new(0, 1, TokenType::Identifier(String::from("n"))), information: LocationData::new(0, 1) }), 
                                    operator: Token::new(0, 1, TokenType::Minus), 
                                    right: Box::new(Expression::Literal { literal: Token::new(0, 1, TokenType::Integer(2)), information: LocationData::new(0, 1) }), 
                                    information: LocationData::new(0, 1) 
                                },
                                
                            ], 
                            information: LocationData::new(0, 1) 
                        }),
                        information: LocationData::new(0, 1),
                    }
                )),
            },
            LocationData::new(0, 1)
        ),
        name: Symbol::from("fib"),
        return_ty: TypeInformation::Int,
        information: LocationData::new(0, 1)
    }, expected.map_information(&|x| x.location_data))
}