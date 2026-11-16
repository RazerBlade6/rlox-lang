//! # Lox Environment
//!
//! This module contains the implementation of the Runtime Environment of the Lox Language.
//! All variables, global and local, along with function pointers, are stored in this environment
//!
//! In most cases, this module will not be directly invoked, but rather contained within an `Interpreter`.
//! Environments may have an `enclosing`, an `Option<Box<Environment>>` that represents a pointer to other,
//! indirected environments. This is to allow scoping and shadowing of variables, along with preventing
//! contamination of values between scopes.
//!
//! New environments are created on the creation of a block, and the calling of a function. It is required
//! to generate a new environment for a function call to allow for recursive function calls.
//!
//! All methods of this struct are public, though they are not to be called directly, but rather within Expression and
//! Statement Blocks
//!

use crate::{expr::Literal, native::globals};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: Rc<RefCell<HashMap<String, Literal>>>,
    pub locals: Rc<RefCell<HashMap<usize, usize>>>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(locals: HashMap<usize, usize>) -> Self {
        Self {
            values: Rc::from(RefCell::from(globals())),
            locals: Rc::from(RefCell::from(locals)),
            enclosing: None,
        }
    }

    pub fn resolve(&mut self, index: usize, distance: usize) {
        self.locals.borrow_mut().insert(index, distance);
    }

    pub fn enclose(&self) -> Environment {
        Self {
            values: Rc::new(RefCell::new(HashMap::new())),
            locals: self.locals.clone(),
            enclosing: Some(Box::new(self.clone())),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str, index: usize) -> Result<Literal, String> {
        let distance = self.locals.borrow().get(&index).cloned();
        match self.get_at_distance(name, distance) {
            Some(literal) => Ok(literal),
            None => Err(format!("Undefined variable {}", name)),
        }
    }

    fn get_at_distance(&self, name: &str, distance: Option<usize>) -> Option<Literal> {
        if let Some(distance) = distance {
            match distance {
                0 => self.values.borrow().get(name).cloned(),
                _ => self
                    .enclosing
                    .as_ref()
                    .expect("Should always be within max depth")
                    .get_at_distance(name, Some(distance - 1)),
            }
        } else {
            match &self.enclosing {
                None => self.values.borrow().get(name).cloned(),
                Some(env) => env.get_at_distance(name, distance),
            }
        }
    }

    pub fn assign(&self, name: &str, value: Literal, index: usize) -> Result<(), String> {
        let distance = self.locals.borrow().get(&index).cloned();
        self.assign_at_distance(name, value, distance)
    }

    fn assign_at_distance(
        &self,
        name: &str,
        value: Literal,
        distance: Option<usize>,
    ) -> Result<(), String> {
        if let Some(distance) = distance {
            match distance {
                0 => {
                    self.values.borrow_mut().insert(name.to_string(), value);
                    return Ok(());
                }
                _ => self
                    .enclosing
                    .as_ref()
                    .expect("Should always be within max depth")
                    .assign_at_distance(name, value, Some(distance - 1))?,
            };
        } else {
            match &self.enclosing {
                None => match self
                    .values
                    .borrow()
                    .contains_key(name)
                    .then(|| self.values.borrow_mut().insert(name.to_string(), value))
                {
                    Some(_) => return Ok(()),
                    None => return Err(format!("Undefined Variable {name}")),
                },
                Some(env) => env.assign_at_distance(name, value, distance)?,
            }
        }

        Ok(())
    }
}
