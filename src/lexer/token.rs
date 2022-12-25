use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub(crate) static ref IDENTIFIERS: HashMap<&'static str, TokenType> = {
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
    pub const fn new(line_num: usize, begin: usize, end: usize, typ: TokenType) -> Token {
        Token (
            LocationData {
                line_num,
                tok_begin: begin,
                tok_end: end
            },
            typ
        )
    }

    pub fn line_num(&self) -> usize {
        self.0.line_num
    }

    pub fn char_loc(&self) -> usize {
        self.0.tok_begin
    }

    pub fn ty(&self) -> TokenType {
        self.1.clone()
    }
}

#[derive(PartialEq, Debug, Clone)]
struct LocationData {
    line_num: usize,
    tok_begin: usize,
    tok_end: usize
}

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
        matches!(self, Self::Identifier(_) | Self::Str(_) | Self::Integer(_) | Self::Float(_))
    }
}