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

pub struct Scanner {
    src: String,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

lazy_static! {
    static ref KEYWORD_MAP: HashMap<&'static str, TokenType> = {
        HashMap::<&'static str, TokenType>::from([
            ("and", TokenType::And),
            ("break", TokenType::Break),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("for", TokenType::For),
            ("fun", TokenType::Fun),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ])
    };
}

impl Scanner {
    pub fn new(src: String) -> Self {
        Self {
            src,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    /// Tokenizes the provided source string.
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        let eof_token = Token::new(TokenType::Eof, "", self.line);
        self.tokens.push(eof_token);
        Ok(self.tokens.clone())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.src.len()
    }

    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();
        match c {
            '(' => self.add_token_t(TokenType::LeftParen),
            ')' => self.add_token_t(TokenType::RightParen),
            '[' => self.add_token_t(TokenType::LeftBox),
            ']' => self.add_token_t(TokenType::RightBox),
            '{' => self.add_token_t(TokenType::LeftBrace),
            '}' => self.add_token_t(TokenType::RightBrace),
            ',' => self.add_token_t(TokenType::Comma),
            '.' => self.add_token_t(TokenType::Dot),
            '-' => self.add_token_t(TokenType::Minus),
            '+' => self.add_token_t(TokenType::Plus),
            '%' => self.add_token_t(TokenType::Percent),
            ';' => self.add_token_t(TokenType::SemiColon),
            '*' => self.add_token_t(TokenType::Star),
            '!' => {
                if self.expect('=') {
                    self.add_token_t(TokenType::BangEqual);
                } else {
                    self.add_token_t(TokenType::Bang)
                }
            }
            '=' => {
                if self.expect('=') {
                    self.add_token_t(TokenType::EqualEqual);
                } else {
                    self.add_token_t(TokenType::Equal);
                }
            }
            '<' => {
                if self.expect('=') {
                    self.add_token_t(TokenType::LessEqual);
                } else {
                    self.add_token_t(TokenType::Less);
                }
            }
            '>' => {
                if self.expect('=') {
                    self.add_token_t(TokenType::GreaterEqual);
                } else {
                    self.add_token_t(TokenType::Greater);
                }
            }
            '/' => {
                if self.expect('/') {
                    self.single_line_comment();
                } else if self.expect('*') {
                    self.multi_line_comment();
                } else {
                    self.add_token_t(TokenType::Slash);
                }
            }
            '"' => self.string()?,
            ' ' => (),
            '\r' => (),
            '\n' => self.line += 1,
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_ascii_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    return Err(format!("Unexpected character: {c}"));
                }
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> char {
        let c = self.src.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn expect(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.src.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn add_token_t(&mut self, token_type: TokenType) {
        self.add_token(token_type);
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.src[self.start..self.current];
        let token = Token::new(token_type, text, self.line);
        self.tokens.push(token);
    }

    fn add_token_s(&mut self, token_type: TokenType, text: &str) {
        self.tokens.push(Token::new(token_type, text, self.line))
    }

    fn number(&mut self) {
        while self.peek(0).is_ascii_digit() {
            if self.peek(0) == '_' {
                continue;
            }
            self.advance();
        }

        if self.peek(0) == '.' && self.peek(1).is_ascii_digit() {
            self.advance();
            while self.peek(0).is_ascii_digit() {
                if self.peek(0) == '_' {
                    continue;
                }
                self.advance();
            }
        }

        self.add_token(TokenType::Number)
    }

    fn peek(&self, n: usize) -> char {
        match self.src.chars().nth(self.current + n) {
            Some(c) => c,
            None => '\0',
        }
    }

    fn string(&mut self) -> Result<(), String> {
        while self.peek(0) != '"' {
            if self.peek(0) == '\n' {
                self.line += 1
            }
            if self.is_at_end() {
                return Err(format!("Line {}: Unterminated String", self.line));
            }
            self.advance();
        }

        self.advance();

        let text = self.src[self.start + 1..self.current - 1].to_string();
        self.add_token_s(TokenType::String, &text);
        Ok(())
    }

    fn identifier(&mut self) {
        let mut c = self.peek(0);
        while c.is_ascii_alphanumeric() || c == '_' {
            self.advance();
            c = self.peek(0);
        }
        let s = &self.src[self.start..self.current];
        let token_type = match KEYWORD_MAP.get(&s) {
            Some(t) => *t,
            None => TokenType::Identifier,
        };
        self.add_token(token_type)
    }

    fn single_line_comment(&mut self) {
        while self.peek(0) != '\n' && !self.is_at_end() {
            self.advance();
        }
    }

    fn multi_line_comment(&mut self) {
        loop {
            let c = self.advance();
            match c {
                '/' => if self.expect('*') {
                    self.multi_line_comment();
                }
                '*' => if self.expect('/') {
                    return;
                }
                _ => ()
            }
        }
    }
}
