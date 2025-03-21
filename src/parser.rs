//! # Recursive Descent Parser
//! 
//! A recursive-descent parser generates an Abstract Syntax Tree by recursively calling 
//! helper functions that are arranged to encode the formal syntax of a language
//! 
//! The public API of this module consists of `Parser::new()` and `parse(&mut self) -> Result<Vec<Stmt>, String>`
//! The parser instance does not itself have ownership of the parsed syntax tree, so it can be freely
//! owned by the caller.
//! 
//! To find the exact syntax order and precendence rules, please see `docs/expressions.txt` and `docs/statements.txt`
//! 
//! ### Usage
//! ```
//! use parser::Parser;
//! 
//! fn main() {
//!     let tokens = vec![] // this would be the tokens as obtained from the Scanner
//!     let mut parser = Parser::new(tokens)
//!     let statements = parser.parse()?;
//! }
//! ``` 

use crate::{
    expr::*,
    stmt::Stmt,
    token::Token, 
    token::TokenType,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            index: 0,
        }
    }

    /// Parses the scanned tokens according to the grammar rules
    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(error) => {
                    errors.push(error);
                    self.synchronise();
                }
            }
        }

        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors.join("\n"))
        }
    }
    
    fn index(&mut self) -> usize {
        let index = self.index;
        self.index += 1;
        index
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        let result = if self.match_tokens(&[TokenType::Var]) {
            self.var_declaration()
        } else if self.match_tokens(&[TokenType::Fun]) {
            self.function("function")
        } else {
            self.statement()
        };
        result
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, &format!("Expected {kind} name"))?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expected '(' after {kind} name"),
        )?;

        let mut params = vec![];
        if !self.check(&TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(format!(
                        "Line {}: Cant have more than 255 arguments",
                        self.peek().line
                    ));
                }
                params.push(self.consume(TokenType::Identifier, "Expected parameter name")?);
                if !self.match_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expected ')' after parameters.")?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expected '{{' before {kind} body."),
        )?;

        let body = match self.block()? {
            Stmt::Block { statements } => statements,
            _ => panic!("Block statement parsed something that was not a block"),
        };

        Ok(Stmt::Function { name, params, body })
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name: Token = self.consume(TokenType::Identifier, "Expected variable name")?;
        
        let initializer = if self.match_tokens(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        if self.peek().token_type == TokenType::SemiColon {
            self.consume(
                TokenType::SemiColon,
                "Expected ';' after variable declaration",
            )?;
        }
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.match_tokens(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_tokens(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_tokens(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.match_tokens(&[TokenType::LeftBrace]) {
            return self.block();
        }
        return self.expr_statement();
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'")?;
        let initializer: Option<Stmt>;
        if self.match_tokens(&[TokenType::SemiColon]) {
            initializer = None;
        } else if self.match_tokens(&[TokenType::Var]) {
            initializer = Some(self.var_declaration()?)
        } else {
            initializer = Some(self.expr_statement()?);
        }

        let condition: Option<Expr>;
        if !self.check(&TokenType::SemiColon) {
            condition = Some(self.expression()?);
        } else {
            condition = None;
        }

        self.consume(TokenType::SemiColon, "Expected ';' after condition")?;

        let change: Option<Expr>;
        if !self.check(&TokenType::RightParen) {
            change = Some(self.expression()?);
        } else {
            change = None;
        }

        self.consume(TokenType::RightParen, "Expected ')' after clauses")?;

        let mut body = self.statement()?;

        if let Some(change) = change {
            let statements = vec![body, Stmt::Expression { expr: change }];
            body = Stmt::Block { statements };
        }

        let condition = match condition {
            None => Expr::Literal {
                literal: Literal::Boolean(true),
            },
            Some(c) => c,
        };

        body = Stmt::While {
            condition,
            body: Box::from(body),
        };

        if let Some(initializer) = initializer {
            let statements = vec![initializer, body];
            body = Stmt::Block { statements };
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition: Expr = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;
        let then_branch: Box<Stmt> = Box::new(self.statement()?);
        let mut else_branch: Option<Box<Stmt>> = None;
        if self.match_tokens(&[TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        }
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition: Expr = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;
        let body: Stmt = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::from(body),
        })
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let value = match self.check(&TokenType::SemiColon) {
            true => None,
            false => Some(self.expression()?),
        };
        if self.peek().token_type == TokenType::SemiColon {
            self.consume(TokenType::SemiColon, "Expected ';' after return value")?;
        }
        Ok(Stmt::Return { value })
    }

    fn block(&mut self) -> Result<Stmt, String> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block")?;
        Ok(Stmt::Block { statements })
    }

    fn expr_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        if self.peek().token_type == TokenType::SemiColon {
            self.consume(TokenType::SemiColon, "Expected ';' after expression")?;
        }
        Ok(Stmt::Expression { expr })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assigment()
    }

    fn assigment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;
        if self.match_tokens(&[TokenType::Equal]) {
            
            let value = self.assigment()?;
            let (name, position, index) = match expr {
                Expr::Variable { index, name } => (name, None, index),
                Expr::Access { name, position, index } => (name, Some(position), index),
                other => return Err(format!("Invalid assignment target {}", other.to_string())),
            };
            return Ok(Expr::create_assigment(name, value, position, index));
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        if self.match_tokens(&[TokenType::Or]) {
            let operator: Token = self.previous();
            let right: Expr = self.and()?;
            expr = Expr::new_logical(expr, operator, right)
        }
        return Ok(expr);
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.equality()?;

        if self.match_tokens(&[TokenType::And]) {
            let operator: Token = self.previous();
            let right: Expr = self.equality()?;
            expr = Expr::new_logical(expr, operator, right);
        }
        return Ok(expr);
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;
        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[TokenType::Slash, TokenType::Star, TokenType::Percent]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::create_unary(operator, right));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_tokens(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }


        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(format!(
                        "Line {}: Can't have more than 255 arguments",
                        self.peek().line
                    ));
                }
                arguments.push(self.expression()?);
                if !self.match_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren: Token = self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
        return Ok(Expr::create_call(callee, paren, arguments));
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek().clone();
        match token.token_type {
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expected `)`")?;
                return Ok(Expr::new_grouping(expr));
            }
            TokenType::LeftBox => {
                self.advance();
                let mut elements: Vec<Expr> = vec![];
                if !self.check(&TokenType::RightBox) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_tokens(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RightBox, "Expected ']' after array elements")?;
                Ok(Expr::array(elements))
            }
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number
            | TokenType::String => {
                self.advance();
                return Ok(Expr::new_literal(Literal::from_token(token)));
            }
            TokenType::Identifier => {
                    self.advance();
                    if self.match_tokens(&[TokenType::LeftBox]) {
                        let position = self.expression()?;
                        self.consume(TokenType::RightBox, "Expected ']' after array access")?;
                        Ok(Expr::access(token, position, self.index()))
                    } else {
                        Ok(Expr::create_variable(self.previous(), self.index()))
                    }    
            },
            _ => return Err(format!("Line {}: Expected Expression", token.line)),
        }
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for typ in types {
            if !self.check(typ) {
                continue;
            }
            self.advance();
            return true;
        }

        false
    }

    fn synchronise(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SemiColon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Return => return,
                _ => (),
            }
            self.advance();
        }
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token, String> {
        let token = self.peek();
        if token.token_type == token_type {
            self.advance();
            let token = self.previous();
            Ok(token)
        } else {
            return Err(format!("Line {}: {}", token.line, msg));
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn check(&mut self, typ: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == *typ
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&mut self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
