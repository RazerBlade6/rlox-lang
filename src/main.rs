mod callable;
mod environment;
mod expr;
mod interpreter;
mod native;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;

use crate::{
    interpreter::Interpreter,
    parser::Parser,
    scanner::Scanner,
    stmt::Stmt,
    resolver::Resolver,
    token::Token
};
use std::{
    collections::HashMap, env, fs, io::{self, Write}
};

fn run_prompt() -> Result<(), String> {
    let esc_key = match env::consts::OS {
        "windows" => "CTRL + Z",
        _ => "CTRL + D",
    };
    println!("Welcome to the Lox Interpreter!\nPress {esc_key} to exit");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut interpreter: Interpreter = Interpreter::new();

    loop {
        let mut src = String::new();
        print!(">>> ");
        stdout.flush().map_err(|err| err.to_string())?;

        if stdin.read_line(&mut src).map_err(|err| err.to_string())? == 0 {
            println!("Interpreter Quit");
            return Ok(())
        }

        run(src, &mut interpreter).unwrap_or_else(|msg| println!("\nERROR:\n{msg}\n"))
    }
}

fn run_file(path: &str) -> Result<(), String> {
    let mut interpreter = Interpreter::new();

    match fs::read_to_string(path) {
        Ok(src) => run(src, &mut interpreter),
        Err(msg) => Err(msg.to_string()),
    }
}

fn run(src: String, interpreter: &mut Interpreter) -> Result<(), String> {
    let mut scanner: Scanner = Scanner::new(src.as_str());
    let tokens: Vec<Token> = scanner.scan_tokens()?;

    dbg!(&tokens);
    
    let mut parser: Parser = Parser::new(tokens);
    let statements: Vec<Stmt> = parser.parse()?;

    let mut resolver: Resolver = Resolver::new();
    let locals: HashMap<usize, usize> = resolver.resolve(&statements)?;

    interpreter.resolve(locals);
    interpreter.interpret(statements.iter().collect())?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    env::set_var("RUST_BACKTRACE", "1");

    match args.len() {
        1 => run_prompt().unwrap_or_else(|msg| println!("{msg}")),
        2 => run_file(&args[1]).unwrap_or_else(|msg| println!("{msg}")),
        _ => println!("Please use as rlox ''filepath")
    }    
}
