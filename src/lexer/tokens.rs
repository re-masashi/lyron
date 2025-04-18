/// A token that is parsed by the [`Lexer`].
///
/// [`Lexer`]: ../struct.Lexer.html
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    /// An identifier of a variable or function with its name.
    Identifier(String),
    /// Keywords
    If, // if
    Else,   // else
    Let,    // let
    Def,    // def
    Class,  // class
    Extern, // extern
    Use,    // use
    Return, // return
    True,   // true
    False,  // false
    Module, // mod
    While,  // while
    Do,     // do
    End,    // end
    None,   // none
    Then,   // then

    /// Literals
    Integer(i32),
    Str(String),
    Double(f64),

    /// Punctuators
    Semicolon, // ;
    Colon,  // :
    Comma,  // ,
    LParen, // (
    RParen, // )
    LBrack, // [
    RBrack, // ]
    LBrace, // {
    RBrace, // }
    Arrow,  // ->

    /// Operators
    Minus, // -
    Plus,      // +
    Div,       // /
    Mul,       // *
    Dot,       // .
    Assign,    // =
    Less,      // <
    Greater,   // >
    LessEq,    // <=
    GreaterEq, // >=
    Equal,     // ==
    Not,       // !
    NotEq,     // !=

    /// AugAssign operators
    PlusEq, // +=
    MinusEq, // -=
    MulEq,   // *=
    DivEq,   // /=
    Walrus,  // =:

    Async, // async
    Await, // await

    Unknown,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub type_: TokenType,
    pub pos: i32,
    pub line_no: i32,
    pub file: String,
}
