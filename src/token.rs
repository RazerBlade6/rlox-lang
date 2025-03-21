//! # Tokens
//!
//! Tokens represent the lowest level of lexemes capable of being parsed by the language.
//! While usually one or two characters, tokens may be as long as is needed to represent the data in question
//! 
//! TokenType is an enum that implements basic `to_string()` functionality, though it actually returns
//! a `&str` valid for the lifetime of `&self` on which it is called. This is sufficient for most use cases 
//! of the function, but the caller may need to clone the str if ownership is required
//!
//! Token impl contains one constructor, `Token::new()`, that returns a Token.
//! 
//! ## Usage
//! ```
//! use token::*;
//!
//! fn main() {
//!     let token  = Token::new(
//!         token_type: LeftParen,
//!         lexeme: "(",
//!         literal: Literal::Null,
//!         line: 0
//!     )
//! }
//! ```
//!

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBox,
    RightBox,
    Comma,
    Dot,
    Minus,
    Plus,
    Percent,
    SemiColon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Break,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    // Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // End of File
    Eof,

    // Generic Ignore
    None
}

impl TokenType {
    pub fn to_string(&self) -> &str {
        match self {
            TokenType::LeftParen => return "Left Parenthesis",
            TokenType::RightParen => return "Right Parenthesis",
            TokenType::LeftBrace => return "Left Brace",
            TokenType::LeftBox => return "Left Box",
            TokenType::RightBox => return "Right Box",
            TokenType::RightBrace => return "Right Brace",
            TokenType::Comma => return "Comma",
            TokenType::Dot => return "Dot",
            TokenType::Minus => return "Minus",
            TokenType::Plus => return "Plus",
            TokenType::Percent => return "Percent",
            TokenType::SemiColon => return "Semicolon",
            TokenType::Slash => return "Slash",
            TokenType::Star => return "Star",
            TokenType::Bang => return "Not",
            TokenType::BangEqual => return "Not Equal",
            TokenType::Equal => return "Assignment",
            TokenType::EqualEqual => return "Equals",
            TokenType::Greater => return "Greater",
            TokenType::GreaterEqual => return "Greater Equal",
            TokenType::Less => return "Less",
            TokenType::LessEqual => return "Less Equal",
            TokenType::Identifier => return "Identifier",
            TokenType::String => return "String",
            TokenType::Number => return "Number",
            TokenType::And => return "And",
            TokenType::Break => return "Break",
            TokenType::Class => return "Class",
            TokenType::Else => return "Else",
            TokenType::False => return "False",
            TokenType::Fun => return "Fun",
            TokenType::For => return "For",
            TokenType::If => return "If",
            TokenType::Nil => return "Nil",
            TokenType::Or => return "Or",
            TokenType::Return => return "Return",
            TokenType::Super => return "Super",
            TokenType::This => return "This",
            TokenType::True => return "True",
            TokenType::Var => return "Var",
            TokenType::While => return "While",
            TokenType::Eof => return "Eof",
            TokenType::None => return "None"
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, line: usize) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_string(),
            line,
        }
    }
}
