use std::fmt::Debug;

use theta_types::bytecode::Symbol;

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
    /// Only necessary for block expression
    Partial {
        expression: Expression<T>,
        information: T
    },
}

impl<T: Debug + PartialEq> Statement<T> {
    pub fn information(&self) -> &T {
        match self {
            Statement::ExpressionStatement { expression: _, information } => information,
            Statement::PrintStatement { information, expression: _ } => information,
            Statement::VarStatement { ident: _, init: _, information } => information,
            Statement::Partial { expression: _, information } => information,
        }
    }

    pub fn strip_information(self) -> Statement<()> {
        match self {
            Statement::ExpressionStatement { expression, information: _ } => Statement::ExpressionStatement { expression: expression.strip_information(), information: () },
            Statement::PrintStatement { expression, information: _ } => Statement::PrintStatement { expression: expression.strip_information(), information: () },
            Statement::VarStatement { ident, init, information: _ } => Statement::VarStatement { ident, init: init.strip_information(), information: () },
            Statement::Partial { expression, information: _ } => Statement::Partial { expression: expression.strip_information(), information: () },
        }
    }

    pub fn strip_token_information(self) -> Statement<T> {
        match self {
            Statement::ExpressionStatement { expression, information } => Statement::ExpressionStatement { expression: expression.strip_token_information(), information },
            Statement::PrintStatement { expression, information } => Statement::PrintStatement { expression: expression.strip_token_information(), information },
            Statement::VarStatement { ident, init, information } => Statement::VarStatement { ident, init: init.strip_token_information(), information },
            Statement::Partial { expression, information } => Statement::Partial { expression: expression.strip_token_information(), information },
        }
    }

    pub fn map_information<V: Debug + PartialEq>(self, map_fn: &dyn Fn(T) -> V) -> Statement<V> {
        match self {
            Statement::ExpressionStatement { expression, information } => Statement::ExpressionStatement { expression: expression.map_information(map_fn), information: map_fn(information) },
            Statement::PrintStatement { expression, information } => Statement::PrintStatement { expression: expression.map_information(map_fn), information: map_fn(information) },
            Statement::VarStatement { ident, init, information } => Statement::VarStatement { ident, init: init.map_information(map_fn), information: map_fn(information) },
            Statement::Partial { expression, information } => Statement::Partial { expression: expression.map_information(map_fn), information: map_fn(information) },
        }
    }
}