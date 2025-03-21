//! # Expressions
//! 
//! Expressions are the heart of the Lox Language, they represent a piece of code that 
//! will be resolved into a value, and are used as the mid-level interface between the Token and Statements
//! 
//! Expressions are of many different types, any change to this enum will very likely be reflected in both the syntax 
//! and semantics. Though an expression is a syntactic unit, it is tighly bound to the runtime as well, and as such
//! changes to this module are difficult to implement. 
//! 
//! `Expr` implements various helper methods to create new variants, mainly because it extensively used `Box<Expr>` to represent 
//! nested expressions. To simplify calls, these methods are to be used rather than directly creating `Expr::Variant {}`
//! 
//! Unlike Statements, which are **interpreted** during runtime, Expressions are **evaluated**. As such, the evaluation will 
//! return a Literal, not generate an effect visible to the user. 
//! 
//! ## Literal
//! `Literal` is the implementation of the type system of Lox, a discriminated enum that is what allows dynamic 
//! type systems in the statically-typed Rust. Literal is used extensively throughout the code, so adding more Literal Variants
//! is highly discouraged. If an item can be implemented without adding extra Literals, that implementation is preferred.
//! 
//! Both `Expr` and `Literal` implement a `to_string()` method, used extensively.
//! 
//! ### Usage
//! ```
//! use expr::{Expr, Literal};
//! use token::*;
//! 
//! fn main() {
//!     let left: Expr = Expr::literal(Literal::Number(45.2));
//!     let token: Token = Token::new(Token::new(TT::Plus, "+", 0));
//!     let right: Expr = Expr::literal(Literal::Number(21.1))
//!     let expr: Expr = Expr::binary(left, token, right)
//! 
//! 
//!     Expr retains general info pertaining to overall structure of the 
//! }
//! ```

use crate::callable::Callables;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::stmt::Stmt;
use crate::token::*;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    Str(String),
    Boolean(bool),
    Nil,
    Array(Vec<Literal>),
    Callable(Callables),
}
use {Callables::*, Literal::*};
use Literal as L;
use TokenType as TT;

impl Literal {
    pub fn to_string(&self) -> String {
        match self {
            L::Number(n) => return format!("{}", n),
            L::Str(s) => return s.to_string(),
            L::Boolean(b) => return format!("{b}"),
            L::Nil => return String::from("nil"),
            L::Array(a) => return format!("[{}]", a.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")),
            L::Callable(other) => other.to_string(),
        }
    }

    pub fn to_type(&self) -> &str {
        match self {
            L::Number(_) => return "Number",
            L::Str(_) => return "String",
            L::Boolean(_) => return "Boolean",
            L::Nil => return "nil",
            L::Array(_) => return "array",
            L::Callable(LoxFunction {
                name: _,
                params: _,
                arity: _,
                body: _,
                environment: _,
            }) => return "<function>",
            L::Callable(NativeFunction {
                name: _,
                arity: _,
                fun: _,
            }) => return "<native function>",
        }
    }

    pub fn from_token(token: Token) -> Self {
        match token.token_type {
            TT::Number => Self::Number(match token.lexeme.parse::<f64>() {
                Ok(f) => f,
                Err(_) => panic!("Invalid Syntax: attempted to parse non-numeric as f64"),
            }),
            TT::String => Self::Str(token.lexeme.to_string()),
            TT::Identifier => Self::Str(token.lexeme.to_string()),
            TT::True => Self::Boolean(true),
            TT::False => Self::Boolean(false),
            TT::Nil => Nil,
            other => panic!(
                "Invalid Syntax: Attempted extracting literal alue from non-valued type {}",
                other.to_string()
            ),
        }
    }

    pub fn is_falsy(&self) -> Literal {
        match self {
            Number(x) => Boolean(*x == 0.0),
            Str(s) => Boolean(s.len() == 0),
            Boolean(b) => Boolean(*b),
            Array(a) => Boolean(a.is_empty()),
            Nil => Boolean(true),
            Callable(_) => Boolean(false),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Number(x) => return *x != 0.0,
            Str(s) => return s.len() != 0,
            Boolean(b) => return *b,
            Array(a) => return !a.is_empty(),
            Nil => return false,
            Callable(_) => return true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Array {
        elements: Vec<Expr>
    },
    Access {
        name: Token,
        position: Box<Expr>,
        index: usize
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
    Literal {
        literal: Literal,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        index: usize,
        name: Token,
    },
    Assignment {
        name: Token,
        value: Box<Expr>,
        position: Option<Box<Expr>>,
        index: usize,
    },
}

impl Expr {
    pub fn to_string(&self) -> String {
        match self {
            Expr::Access { name, position, index: _ } => {
                format!("{}[{}]", name.lexeme, position.to_string())
            }
            Expr::Array { elements } => {
                format!("[{}]", elements.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", "))
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {} {}",
                    operator.lexeme,
                    (*left).to_string(),
                    (*right).to_string()
                )
            }
            Expr::Grouping { expr } => {
                format!("({})", (*expr).to_string())
            }
            Expr::Literal { literal } => {
                format!("{}", literal.to_string())
            }
            Expr::Unary { operator, right } => {
                format!("{}{}", operator.lexeme, (*right).to_string())
            }
            Expr::Variable { index: _, name } => name.lexeme.to_string(),
            Expr::Assignment {
                name,
                value,
                position: _,
                index: _,
            } => {
                format!("{} = {}", name.lexeme, (*value).to_string())
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                format!(
                    "`{}` {} `{}`",
                    (*left).to_string(),
                    operator.lexeme,
                    (*right).to_string()
                )
            }
            Expr::Call {
                callee: _,
                paren,
                args: _,
            } => {
                format!("function {}()", paren.lexeme)
            }
        }
    }

    pub fn array(elements: Vec<Expr>) -> Self {
        Self::Array { elements }
    }

    pub fn access(name: Token, position: Expr, index: usize) -> Self {
        Self::Access { name, position: Box::from(position), index }
    }

    pub fn new_binary(left: Expr, operator: Token, right: Expr) -> Self {
        Self::Binary {
            left: Box::from(left),
            operator,
            right: Box::from(right),
        }
    }

    pub fn new_grouping(expr: Expr) -> Self {
        Self::Grouping {
            expr: Box::from(expr),
        }
    }

    pub fn new_literal(literal: Literal) -> Self {
        Self::Literal { literal }
    }

    pub fn new_logical(left: Expr, operator: Token, right: Expr) -> Self {
        Self::Logical {
            left: Box::from(left),
            operator,
            right: Box::from(right),
        }
    }

    pub fn create_unary(operator: Token, right: Expr) -> Self {
        Self::Unary {
            operator,
            right: Box::from(right),
        }
    }

    pub fn create_variable(name: Token, index: usize) -> Self {
        Self::Variable { index, name }
    }

    pub fn create_assigment(name: Token, value: Expr, position: Option<Box<Expr>>, index: usize) -> Self {
        Self::Assignment {
            name,
            value: Box::from(value),
            position,
            index,
        }
    }

    pub fn create_call(callee: Expr, paren: Token, args: Vec<Expr>) -> Self {
        Self::Call {
            callee: Box::from(callee),
            paren,
            args,
        }
    }

    pub fn evaluate(&self, environment: &Environment) -> Result<Literal, String> {
        match &self {
            Expr::Literal { literal } => Ok((*literal).clone()),

            Expr::Grouping { expr } => (*expr).evaluate(environment),

            Expr::Unary { operator, right } => Self::evaluate_unary(environment, operator, right),

            Expr::Binary {
                left,
                operator,
                right,
            } => Self::evaluate_binary(environment, left, operator, right),

            Expr::Variable { name, index } => environment.get(&name.lexeme, *index),

            Expr::Assignment { name, value, position, index } => {
                Self::evaluate_assignment(environment, name, value, position, *index)
            }

            Expr::Array { elements } => Self::evaluate_array(environment, elements),

            Expr::Access { name, position, index } => Self::evaluate_access(environment, name, position, *index),

            Expr::Logical {
                left,
                operator,
                right,
            } => Self::evaluate_logical(environment, left, operator, right),

            Expr::Call {
                callee,
                paren,
                args,
            } => Self::evaluate_call(environment, callee, paren, args),
        }
    }

    fn evaluate_unary(
        environment: &Environment,
        operator: &Token,
        right: &Box<Expr>,
    ) -> Result<Literal, String> {
        let right = (*right).evaluate(environment)?;

        match (operator.token_type, &right) {
            (TT::Minus, Number(x)) => return Ok(Number(-x)),
            (TT::Minus, _) => {
                return Err(format!("negation not implemented for {}", right.to_type()))
            }
            (TT::Bang, any) => Ok(any.is_falsy()),
            _ => panic!("Invalid syntax: Parser specified non-unary expression as unary!"),
        }
    }

    fn evaluate_binary(
        environment: &Environment,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> Result<Literal, String> {
        let left = (*left).evaluate(environment)?;
        let right = (*right).evaluate(environment)?;

        match (&left, operator.token_type, &right) {
            (Number(x), TT::Plus, Number(y)) => Ok(Number(x + y)),

            (Number(x), TT::Minus, Number(y)) => Ok(Number(x - y)),

            (Number(x), TT::Star, Number(y)) => Ok(Number(x * y)),

            (Number(x), TT::Slash, Number(y)) => Ok(Number(x / y)),

            (Number(x), TT::Percent, Number(y)) => Ok(Number(x % y)),

            (Str(s1), TT::Plus, Str(s2)) => Ok(Str(s1.clone() + s2)),

            (Number(x), TT::Greater, Number(y)) => Ok(Boolean(x > y)),

            (Number(x), TT::GreaterEqual, Number(y)) => Ok(Boolean(x >= y)),

            (Number(x), TT::Less, Number(y)) => Ok(Boolean(x < y)),

            (Number(x), TT::LessEqual, Number(y)) => Ok(Boolean(x <= y)),

            (x, TT::EqualEqual, y) => Ok(Boolean(x == y)),

            (x, TT::BangEqual, y) => Ok(Boolean(x != y)),

            (Str(s), TT::Plus, other) => Ok(Str(s.to_owned() + &other.to_string())),

            (some, TT::Plus, Str(s)) => Ok(Str(some.to_string() + s)),

            _ => Err(format!(
                "{} not implemented between {} and {}",
                operator.lexeme,
                left.to_type(),
                right.to_type()
            )),
        }
    }

    fn evaluate_assignment(
        environment: &Environment,
        name: &Token,
        value: &Expr,
        position: &Option<Box<Expr>>,
        index: usize,
    ) -> Result<Literal, String> {
        let value = value.evaluate(environment)?;
        
        if let Some(position) = position {
            let position = if let Number(n) = position.evaluate(environment)? {
                n.round() as usize
            } else {
                return Err(format!("Can only index into a list with number"))
            };

            let mut target = environment.get(&name.lexeme, index)?;
            match target {
                Array(ref mut array) => {
                    if position > array.len() {
                        return Err(format!("Attempted to assign at {} but array is of length {}", position, array.len()))
                    } else if position == array.len() {
                        array.push(value)
                    } else {
                        array[position] = value;
                    }
                    let value = Literal::Array(array.clone());
                    environment.assign(&name.lexeme, value.clone(), index)?;
                    return Ok(value)
                },
                Str(ref mut string) => {
                    if position > string.len() {
                        return Err(format!("Attempted to assign at {} but string is of length {}", position, string.len()))
                    } else if position == string.len() {
                        string.push_str(&value.to_string());
                    } else {
                        if value.to_string().len() > 1 {
                            return Err(format!("Cannot assign length greater than 1 to middle of string"))
                        } else {
                            *string = (&string[0..position]).to_string() + &value.to_string() + &string[position + 1..string.len()];
                        }
                    }
                    let value = Literal::Str(string.clone());
                    environment.assign(&name.lexeme, value.clone(), index)?;
                    return Ok(value);
                },
                other => return Err(format!("Cannot index into {}", other.to_type()))
            }
            
        } else {
            environment.assign(&name.lexeme, value.clone(), index)?;
            Ok(value)
        }
    }

    fn evaluate_logical(
        environment: &Environment,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Literal, String> {
        let left: Literal = left.evaluate(environment)?;
        if operator.token_type == TT::Or {
            if left.is_truthy() {
                return Ok(left);
            }
        } else {
            if !left.is_truthy() {
                return Ok(left);
            }
        }

        return right.evaluate(environment);
    }

    fn evaluate_call(
        environment: &Environment,
        callee: &Expr,
        _paren: &Token,
        args: &[Expr],
    ) -> Result<Literal, String> {
        let callee = (*callee).evaluate(environment)?;

        let mut arguments = vec![];
        for arg in args {
            arguments.push(arg.evaluate(environment)?);
        }

        match callee {
            Callable(callable) => match callable {
                LoxFunction {
                    name,
                    params,
                    arity,
                    body,
                    environment,
                } => return Self::call_function(name, params, arity, body, &environment, arguments),
                NativeFunction { name, arity, fun } => {
                    return Self::call_native(name, arity, fun, arguments)
                }
            },
            other => return Err(format!("Could not call {}", other.to_type())),
        }
    }

    fn call_function(
        name: Token,
        params: Vec<Token>,
        arity: usize,
        body: Vec<Stmt>,
        environment: &Environment,
        arguments: Vec<Literal>,
    ) -> Result<Literal, String> {
        if arguments.len() != arity {
            return Err(format!(
                "Function {} expected {} arguments but got {}",
                name.lexeme,
                arity,
                arguments.len()
            ));
        }

        let mut environment = environment.enclose();

        for (index, value) in arguments.iter().enumerate() {
            environment.define(params[index].lexeme.clone(), value.clone());
        }

        let mut interpreter = Interpreter::new_with_env(environment);

        for statement in &body {
            let result = interpreter.interpret(vec![statement]);
            if let Err(e) = result {
                return Err(e);
            }
            if let Some(ret) = interpreter.ret.clone() {
                return Ok(ret);
            }
        }
        Ok(Literal::Nil)
    }

    fn call_native(
        name: String,
        arity: usize,
        fun: Rc<dyn Fn(Vec<Literal>) -> Result<Literal, String>>,
        arguments: Vec<Literal>,
    ) -> Result<Literal, String> {
        if arguments.len() != arity {
            return Err(format!(
                "Native function {name} expected {arity} arguments but got {}",
                arguments.len()
            ));
        }
        Ok(fun(arguments)?)
    }
    
    fn evaluate_array(environment: &Environment, elements: &[Expr]) -> Result<Literal, String> {
        let mut array = vec![];
        for element in elements {
            array.push(element.evaluate(environment)?)
        }

        Ok(Literal::Array(array))
    }
    
    fn evaluate_access(environment: &Environment, name: &Token, position: &Box<Expr>, index: usize) -> Result<Literal, String> {
        let position = position.evaluate(environment)?;
        let pos: usize;
        if let Literal::Number(n) = position {
            pos = n.round() as usize
        } else {
            return Err(format!("Cannot use type {} to index a list", position.to_type()))
        }
        
        match environment.get(&name.lexeme, index)? {
            Array(array) => match array.get(pos) {
                Some(l) => return Ok(l.clone()),
                None => return Err(format!("Attempted to get element {} from array of length {}", pos, array.len()))
            }
            Str(string) => match string.chars().nth(pos) {
                Some(c) => return Ok(Literal::Str(String::from(c))),
                None => return Err(format!("Attempted to get element {} from string of length {}", pos, string.len()))
            },
            other => return Err(format!("Cannot index into type {}", other.to_type()))
        };
    }
}
