//! # Statements
//! Statements represent a portion of code capable of independent operation.
//! They are distinct from Expressions in that they represent functionality.
//! A statement is always executed and will produce an effect, unlike an expression which is evaluated
//! to produce a value.
//! 
//! For example, an `if` statement will always be executed, but it does itself produce no value
//! 
//! ### Creating New Statements
//! Statements can be created using `Stmt::VariantName{}`
//! Unlike `expr::Expr` , `stmt::Stmt` does not implement helper methods for creation of variants, as
//! they are relatively small signatures and may contradict common syntactic bindings
//! 
//! ### Using Statements
//! Statements are to be used at the lowest level of syntactic parsing, as they are the highest level of a syntax tree.
//! As such, methods to parse statements must be above methods parsing expressions.
//! 
//! Statements and Expressions together form the syntax of the Lox Language
//! 
//! ### Example
//! ```
//! use stmt::Stmt;
//! 
//! fn main() {
//!     let statements = vec![];
//!     let block = Stmt::Block { statements };
//! }
//! ```

use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression {
        expr: Expr,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Return {
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Block {
        statements: Vec<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
}

impl Stmt {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        use Stmt::*;

        match self {
            Expression { expr } => expr.to_string(),
            Var { name, .. } => format!(
                "Variable: {} ({})",
                name.lexeme,
                name.token_type.to_string()
            ),
            While { condition, body } => {
                format!("while ({}) {}", condition.to_string(), (*body).to_string())
            }
            Block { statements } => {
                let mut output: Vec<String> = vec![];
                for statement in &**statements {
                    output.push(statement.to_string())
                }
                output.join("\n")
            }
            If {
                condition,
                then_branch,
                else_branch,
            } => {
                let else_branch = match &(*else_branch) {
                    Some(s) => (*s).to_string(),
                    None => "".to_string(),
                };
                format!(
                    "if {} then {}\n else {}",
                    condition.to_string(),
                    (*then_branch).to_string(),
                    else_branch
                )
            }
            Function {
                name,
                params,
                body: _,
            } => {
                format!("<function> {}({})", name.lexeme, params.len())
            }
            Return { value } => format!(
                "returning {}",
                match value {
                    Some(e) => e.to_string(),
                    None => "".to_string(),
                }
            ),
        }
    }
}
