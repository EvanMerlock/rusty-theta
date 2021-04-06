pub struct Token(LocationData, TokenType);

impl Token {
    pub fn new(line_num: usize, begin: usize, end: usize, typ: TokenType) -> Token {
        Token (
            LocationData {
                line_num: line_num,
                tok_begin: begin,
                tok_end: end
            },
            typ
        )
    }
}

struct LocationData {
    line_num: usize,
    tok_begin: usize,
    tok_end: usize
}

pub enum TokenType {
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    Colon,

    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Identifier(String),
    Str(String),
    Integer(i32),
    Float(f32),

    And, Class, Else, False, Fun, For, If, Nil, Or,
    Return, Super, This, True, Let, While,

    EOF
}