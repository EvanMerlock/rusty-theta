#[macro_export]
macro_rules! token {
    ($tok_type:expr) => {
        Token::new(1, 0, 1, $tok_type)
    };
    ($tok_type:expr, $line:expr, $l_start:expr, $l_end:expr) => {
        Token::new($line, $l_start, $l_end, tok_type)
    }
}

#[macro_export]
macro_rules! literal {
    ($token:expr) => {
        Box::new(Expression::Literal { literal: $token, information: () })
    };
    ($token:expr, $info:expr) => {
        Box::new(Expression::Literal { literal: $token, information: $info })
    }
}

#[macro_export]
macro_rules! binary {
    ($left:expr, $oper:expr, $right:expr) => {
        Expression::Binary { left: $left, operator: $oper, right: $right, information: () }
    };
    ($left:expr, $oper:expr, $right:expr, $info:expr) => {
        Expression::Binary { left: $left, operator: $oper, right: $right, information: $info }
    }
}

#[macro_export]
macro_rules! statement {
    (Expr: $stmt:expr) => {
        Statement::ExpressionStatement { expression: $stmt, information: () }
    };
}

#[macro_export]
macro_rules! if_expression {
    ($cond:expr, $body:expr) => {
        Expression::If { check_expression: Box::new($cond), body: Box::new($body), else_body: None, information: () }
    };
    ($cond:expr, $body:expr, $else_body:expr) => {
        Expression::If { check_expression: Box::new($cond), body: Box::new($body), else_body: Some(Box::new($else_body)), information: () }
    }
}