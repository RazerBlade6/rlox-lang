//! # Native Functions
//! 
//! This module contains various functions that contain core functionality that I 
//! can't be bothered to try and figure out myself
//! 
//! For now, this contains `clock()`, which returns the current UNIX epoch time,
//! `clear(),` which clears stdout, `print()`, which prints to stdout, appending `\n`,
//! `input()`, which accepts input from the user, and `parse()` which converts between types
//! 
//! ### Future plans
//! Currently this offers almost all basic functionality, I'd like to reduce my dependence on this
//! by writing an independent standard library. Probably not happening anytime soon though.  

use crate::callable::Callables;
use crate::expr::Literal;
use std::collections::HashMap;
use std::rc::Rc;

pub fn globals() -> HashMap<String, Literal> {
    let mut globals = HashMap::new();
    globals.insert(
        "clock".to_string(),
        Literal::Callable(Callables::NativeFunction {
            name: "clock".to_string(),
            arity: 0,
            fun: Rc::from(clock),
        }),
    );

    globals.insert(
        "clear".to_string(),
        Literal::Callable(Callables::NativeFunction {
            name: "clock".to_string(),
            arity: 0,
            fun: Rc::from(clear),
        }),
    );
    globals.insert(
        "input".to_string(),
        Literal::Callable(Callables::NativeFunction {
            name: "input".to_string(),
            arity: 1,
            fun: Rc::from(input),
        }),
    );
    globals.insert(
        "parse".to_string(),
        Literal::Callable(Callables::NativeFunction {
            name: "parse".to_string(),
            arity: 2,
            fun: Rc::from(parse),
        }),
    );
    globals.insert(
        "print".to_string(),
        Literal::Callable(Callables::NativeFunction {
            name: "print".to_string(),
            arity: 1,
            fun: Rc::from(print),
        }),
    );
    globals
}

fn clock(_args: Vec<Literal>) -> Result<Literal, String> {
    use std::time::SystemTime;

    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => Ok(Literal::Number(d.as_secs_f64())),
        Err(msg) => Err(msg.to_string()),
    }
}

fn clear(_args: Vec<Literal>) -> Result<Literal, String> {
    use std::{env::consts::OS, process::Command};

    match Command::new(match OS {
        "windows" => "powershell",
        "macos" => "terminal",
        "linux" => "sh",
        _ => return Err("Commands only implemented for windows, macos and linux".to_string()),
    })
    .arg("clear")
    .output()
    {
        Ok(_) => Ok(Literal::Nil),
        Err(msg) => Err(msg.to_string()),
    }
}

fn input(args: Vec<Literal>) -> Result<Literal, String> {
    use std::io::{stdin, stdout, Write};
    print!("{}", args[0].to_string());

    stdout().flush().unwrap();
    let mut buf = String::new();

    match stdin().read_line(&mut buf) {
        Ok(_) => Ok(Literal::Str(buf.trim().to_string())),
        Err(msg) => Err(msg.to_string()),
    }
}

fn parse(args: Vec<Literal>) -> Result<Literal, String> {
    match (&args[0], &args[1]) {
        (Literal::Number(n), Literal::Str(typ)) => match typ.to_lowercase().as_str() {
            "number" => return Ok(Literal::Number(*n)),
            "string" => return Ok(Literal::Str(n.to_string())),
            "boolean" => return Ok(Literal::Boolean(*n != 0.0)),
            "nil" => (),
            _ => return Err("Type should be one of number, string, boolean, nil".to_string()),
        },
        (Literal::Str(x), Literal::Str(typ)) => match typ.to_lowercase().as_str() {
            "number" => match x.parse::<f64>() {
                Ok(n) => return Ok(Literal::Number(n)),
                Err(e) => return Err(e.to_string()),
            },
            "string" => return Ok(Literal::Str(x.to_string())),
            "boolean" => match x.parse::<bool>() {
                Ok(b) => return Ok(Literal::Boolean(b)),
                Err(e) => return Err(e.to_string()),
            },
            "nil" => (),
            _ => return Err("Type should be one of number, string, boolean, nil".to_string()),
        },
        (Literal::Boolean(b), Literal::Str(typ)) => match typ.to_lowercase().as_str() {
            "number" => (),
            "string" => return Ok(Literal::Str(b.to_string())),
            "boolean" => return Ok(Literal::Boolean(*b)),
            "nil" => (),
            _ => return Err("Type should be one of number, string, boolean, nil".to_string()),
        },
        (Literal::Nil, Literal::Str(typ)) => match typ.to_lowercase().as_str() {
            "number" => (),
            "string" => return Ok(Literal::Str(String::from("nil"))),
            "boolean" => (),
            "nil" => return Ok(Literal::Nil),
            _ => return Err("Type should be one of number, string, boolean, nil".to_string()),
        },
        _ => return Err("Please use as parse(`variable`, `\"Type\"` (Number, String, Boolean, Nil))".to_string(),)
        
    }

    Err(format!(
        "Cannot parse {} to {}",
        args[0].to_type(),
        args[1].to_type()
    ))
}

fn print(args: Vec<Literal>) -> Result<Literal, String> {
    println!("{}", args[0].to_string());
    return Ok(Literal::Nil);
}