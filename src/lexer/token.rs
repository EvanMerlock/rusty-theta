pub struct Token(LocationData, TokenType);

struct LocationData {
    line_num: usize,
    line_pos: usize,
}

enum TokenType {
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