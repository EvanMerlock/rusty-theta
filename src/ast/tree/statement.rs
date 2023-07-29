use crate::bytecode::Symbol;
use std::fmt::Debug;

use super::Expression;


#[derive(Debug, PartialEq, Clone)]
pub enum Statement<T> where T: Debug + PartialEq {
    ExpressionStatement {
        expression: Expression<T>,
        information: T
    },
    PrintStatement {
        expression: Expression<T>,
        information: T
    },
    VarStatement {
        ident: Symbol,
        init: Expression<T>,
        information: T,
    },
}

impl<T: Debug + PartialEq> Statement<T> {
    pub fn information(&self) -> &T {
        match self {
            Statement::ExpressionStatement { expression: _, information } => information,
            Statement::PrintStatement { information, expression: _ } => information,
            Statement::VarStatement { ident: _, init: _, information } => information,
        }
    }

    pub fn strip_information(self) -> Statement<()> {
        match self {
            Statement::ExpressionStatement { expression, information: _ } => Statement::ExpressionStatement { expression: expression.strip_information(), information: () },
            Statement::PrintStatement { expression, information: _ } => Statement::PrintStatement { expression: expression.strip_information(), information: () },
            Statement::VarStatement { ident, init, information: _ } => Statement::VarStatement { ident, init: init.strip_information(), information: () },
        }
    }

    pub fn strip_token_information(self) -> Statement<T> {
        match self {
            Statement::ExpressionStatement { expression, information } => Statement::ExpressionStatement { expression: expression.strip_token_information(), information },
            Statement::PrintStatement { expression, information } => Statement::PrintStatement { expression: expression.strip_token_information(), information },
            Statement::VarStatement { ident, init, information } => Statement::VarStatement { ident, init: init.strip_token_information(), information },
        }
    }
}