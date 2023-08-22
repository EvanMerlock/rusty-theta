use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Display;

use crate::types::LocationData;

lazy_static! {
    pub static ref IDENTIFIERS: HashMap<&'static str, TokenType> = {
        let mut hm = HashMap::new();
        hm.insert("and", TokenType::And);
        hm.insert("class", TokenType::Class);
        hm.insert("else", TokenType::Else);
        hm.insert("false", TokenType::False);
        hm.insert("fun", TokenType::Fun);
        hm.insert("for", TokenType::For);
        hm.insert("if", TokenType::If);
        hm.insert("or", TokenType::Or);
        hm.insert("return", TokenType::Return);
        hm.insert("super", TokenType::Super);
        hm.insert("this", TokenType::This);
        hm.insert("true", TokenType::True);
        hm.insert("let", TokenType::Let);
        hm.insert("while", TokenType::While);
        hm
    };
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token(LocationData, TokenType);

impl Token {
    pub const fn new(begin: usize, end: usize, typ: TokenType) -> Token {
        Token (
            LocationData::new(begin, end),
            typ
        )
    }

    pub fn char_loc(&self) -> usize {
        self.0.begin()
    }

    pub fn ty(&self) -> TokenType {
        self.1.clone()
    }

    pub fn strip_information(self) -> Token {
        Token (
            LocationData::new(0, 1),
            self.1
        )
    }

    pub fn location(&self) -> LocationData {
        self.0.clone()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.1, self.0)
    }
}

// TODO:
// MISSING &, |, ^

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    Colon, Arrow,

    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Identifier(String),
    Str(String),
    Integer(i32),
    Float(f32),

    And, Class, Else, False, Fun, For, If, Or,
    Return, Super, This, True, Let, While,

    Eof
}

impl TokenType {
    pub fn is_literal(&self) -> bool {
        matches!(self, Self::Identifier(_) | Self::Str(_) | Self::Integer(_) | Self::Float(_) | Self::True | Self::False)
    }

    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Identifier(_))
    }

    pub fn as_ident(&self) -> String {
        match self {
            TokenType::Identifier(s) => s.clone(),
            _ => panic!("as ident as non ident")
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Minus => write!(f, "-"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Star => write!(f, "*"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Arrow => write!(f, "->"),
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::Identifier(id) => write!(f, "ID: {}", id),
            TokenType::Str(s) => write!(f, "String: {}", s),
            TokenType::Integer(i) => write!(f, "Int: {}", i),
            TokenType::Float(fl) => write!(f, "Float: {},", fl),
            TokenType::And => write!(f, "&&"),
            TokenType::Class => write!(f, "class"),
            TokenType::Else => write!(f, "else"),
            TokenType::False => write!(f, "false"),
            TokenType::Fun => write!(f, "fun"),
            TokenType::For => write!(f, "for"),
            TokenType::If => write!(f, "if"),
            TokenType::Or => write!(f, "||"),
            TokenType::Return => write!(f, "return"),
            TokenType::Super => write!(f, "super"),
            TokenType::This => write!(f, "this"),
            TokenType::True => write!(f, "true"),
            TokenType::Let => write!(f, "let"),
            TokenType::While => write!(f, "while"),
            TokenType::Eof => write!(f, "EOF"),
        }
    }
}