//! # Scanner
//!
//! A rudimentary source string Scanner + Lexer. The Scanner impl contains various
//! utility methods to tokenize the input strings according to Lox's syntax rules.
//! Any new tokens must be implemented within the method call stack, according to
//! its level.
//!
//! The only components of the public API of this module is a constructor `Scanner::new() -> Self` and
//! `Scanner::scan_tokens(&mut self)`.
//!
//! ### Limitations
//! Unfortunately, to maintain overall code integrity (A.K.A my poor design decisions) the Scanner must have tokens
//! as a field, so it is not possible to move it out of Scanner until runtime termination, or by cloning the whole Vec
//!
//! ### Usage
//! ```
//! use scanner::Scanner;
//!
//! fn main() {
//!     let src: &str = "some_example_str";
//!     let mut scanner: Scanner = Scanner::new(src);
//!     let tokens: Vec<Token> = scanner.scan_tokens();
//! }
//! ```

use crate::token::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

use TokenType as TType;

#[derive(Debug)]
pub struct Scanner {
    src: String,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

#[derive(Debug)]
pub struct ScanError((usize, usize), String);

impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[SCANNER]:{}:{}\n{}", self.0 .0, self.0 .1, self.1)
    }
}

impl Error for ScanError {}

impl ScanError {
    fn raise(location: (usize, usize), msg: String) -> Self {
        Self(location, msg)
    }
}

lazy_static! {
    static ref KEYWORD_MAP: HashMap<&'static str, TType> = {
        HashMap::<&'static str, TType>::from([
            ("and", TType::And),
            ("break", TType::Break),
            ("class", TType::Class),
            ("else", TType::Else),
            ("false", TType::False),
            ("for", TType::For),
            ("fun", TType::Fun),
            ("if", TType::If),
            ("nil", TType::Nil),
            ("or", TType::Or),
            ("return", TType::Return),
            ("super", TType::Super),
            ("this", TType::This),
            ("true", TType::True),
            ("var", TType::Var),
            ("while", TType::While),
        ])
    };
}

use ScanError as SE;

impl Scanner {
    pub fn new(src: &str) -> Self {
        Self {
            src: src.to_string(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(token) = self.generate_tokens().map_err(|e| e.to_string())? {
            if token.token_type != TType::None {
                tokens.push(token);
            }
            self.start = self.current;
        }
        tokens.push(Token::new(TType::Eof, "", self.line + 1));
        Ok(tokens)
    }

    fn generate_tokens(&mut self) -> Result<Option<Token>, SE> {
        let token = match self.advance() {
            None => return Ok(None),
            Some('(') => self.generate_token(TType::LeftParen),
            Some(')') => self.generate_token(TType::RightParen),
            Some('[') => self.generate_token(TType::LeftBox),
            Some(']') => self.generate_token(TType::RightBox),
            Some('{') => self.generate_token(TType::LeftBrace),
            Some('}') => self.generate_token(TType::RightBrace),
            Some(',') => self.generate_token(TType::Comma),
            Some('.') => self.generate_token(TType::Dot),
            Some('-') => self.generate_token(TType::Minus),
            Some('+') => self.generate_token(TType::Plus),
            Some('%') => self.generate_token(TType::Percent),
            Some(';') => self.generate_token(TType::SemiColon),
            Some('*') => self.generate_token(TType::Star),
            Some('!') => {
                if self.expect('=') {
                    self.generate_token(TokenType::BangEqual)
                } else {
                    self.generate_token(TokenType::Bang)
                }
            }
            Some('=') => {
                if self.expect('=') {
                    self.generate_token(TokenType::EqualEqual)
                } else {
                    self.generate_token(TokenType::Equal)
                }
            }
            Some('<') => {
                if self.expect('=') {
                    self.generate_token(TokenType::LessEqual)
                } else {
                    self.generate_token(TokenType::Less)
                }
            }
            Some('>') => {
                if self.expect('=') {
                    self.generate_token(TokenType::GreaterEqual)
                } else {
                    self.generate_token(TokenType::Greater)
                }
            }
            Some('/') => {
                if self.expect('/') {
                    self.single_line_comment()
                } else {
                    self.generate_token(TokenType::Slash)
                }
            }
            Some('"') => self.string()?,
            Some(c) => {
                if c.is_whitespace() {
                    self.whitespace();
                    Token::new(TType::None, "", self.line)
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier()
                } else if c.is_numeric() {
                    self.number()?
                } else {
                    return Err(SE::raise((self.line, self.column), format!("Unexpected Character {c}")));
                }
            }
        };

        return Ok(Some(token));
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.src.chars().nth(self.current - 1)
    }

    fn generate_token(&self, token_type: TType) -> Token {
        let text = self.generate_lexeme();
        let token = Token::new(token_type, text, self.line);

        token
    }

    fn expect(&mut self, expected: char) -> bool {
        match self.advance() {
            None => false,
            Some(c) => c == expected,
        }
    }

    fn string(&mut self) -> Result<Token, SE> {
        let lexeme: &str;
        loop {
            match self.advance() {
                None => return Err(SE::raise((self.line, self.column), "Unterminated  String".to_string())),
                Some(c) => {
                    if c == '\n' {
                        self.line += 1
                    } else if c == '"' {
                        break;
                    }
                }
            }
        }
        lexeme = &self.src[self.start + 1..self.current - 1];
        return Ok(Token::new(TType::String, lexeme, self.line));
    }

    fn whitespace(&mut self) -> Token {
        loop {
            match self.advance() {
                None => break,
                Some(' ') => self.column += 1,
                Some('\r') => self.column = 0,
                Some('\n') => self.line += 1,
                Some('\t') => self.column += 4,
                Some(_) => {
                    self.current -= 1;
                    break;
                }
            }
        }

        Token::new(TType::None, "", self.line)
    }

    fn identifier(&mut self) -> Token {
        loop {
            match self.advance() {
                Some(c) => {
                    if !c.is_ascii_alphabetic() && c != '_' {
                        self.current -= 1;
                        break;
                    }
                },
                None => break
            }
        }
        let lexeme = &self.src[self.start..self.current];

        Token::new(match KEYWORD_MAP.get(lexeme) {
                Some(t) => *t,
                None => TType::Identifier
            }, 
            lexeme, 
            self.line
        )
    }

    fn number(&mut self) -> Result<Token, SE> {
        loop {
            match self.advance() {
                Some('.') => (),
                Some(c) => if !c.is_ascii_digit() {
                    self.current -= 1;
                    break;
                }
                None => break
            }
        }

        let lexeme = self.generate_lexeme();

        Ok(Token::new(TType::Number, lexeme, self.line))
    }

    fn single_line_comment(&mut self) -> Token {
        while let Some(c) = self.advance() {
            if c == '\n' {
                break;
            }
        }
        self.line += 1;
        Token::new(TType::None, "", self.line - 1)
    }

    fn generate_lexeme(&self) -> &str {
        &self.src[self.start..self.current]
    }
}
