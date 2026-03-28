//! Scoped symbol table for the Rundell interpreter.
//!
//! Variables are stored as `(Value, RundellType, bool)` triples where the
//! bool indicates immutability (`constant`).

use std::collections::HashMap;

use rundell_parser::ast::RundellType;

use crate::error::RuntimeError;
use crate::evaluator::Value;

/// A single binding stored in the environment.
#[derive(Debug, Clone)]
pub struct Binding {
    /// Current value of the variable.
    pub value: Value,
    /// Declared type (needed for `receive` coercion).
    pub declared_type: RundellType,
    /// Whether this binding is immutable.
    pub is_constant: bool,
}

/// A stack of scopes implementing lexical variable lookup.
///
/// - `scope[0]` is the global scope.
/// - Each function call and block pushes a new scope on top.
/// - Lookup walks from innermost to outermost.
#[derive(Debug)]
pub struct Environment {
    scopes: Vec<HashMap<String, Binding>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    /// Create a new environment with one empty global scope.
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    /// Push a new (empty) inner scope onto the stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the innermost scope, discarding all local variables.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Declare a new variable in the current (innermost) scope.
    ///
    /// Returns an error if the name is already defined in the *same* scope.
    pub fn define(
        &mut self,
        name: &str,
        value: Value,
        declared_type: RundellType,
        is_constant: bool,
    ) -> Result<(), RuntimeError> {
        let scope = self.scopes.last_mut().expect("environment has no scopes");
        if scope.contains_key(name) {
            return Err(RuntimeError::RuntimeError(format!(
                "variable '{name}' is already defined in this scope"
            )));
        }
        scope.insert(
            name.to_string(),
            Binding {
                value,
                declared_type,
                is_constant,
            },
        );
        Ok(())
    }

    /// Define a variable in the global (outermost) scope.
    pub fn define_global(
        &mut self,
        name: &str,
        value: Value,
        declared_type: RundellType,
        is_constant: bool,
    ) -> Result<(), RuntimeError> {
        let scope = self.scopes.first_mut().expect("environment has no scopes");
        if scope.contains_key(name) {
            // Globals may be defined once; importing the same file twice is
            // ignored silently.
            return Ok(());
        }
        scope.insert(
            name.to_string(),
            Binding {
                value,
                declared_type,
                is_constant,
            },
        );
        Ok(())
    }

    /// Look up a variable, searching from innermost scope outward.
    pub fn get(&self, name: &str) -> Result<&Value, RuntimeError> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.get(name) {
                return Ok(&b.value);
            }
        }
        Err(RuntimeError::RuntimeError(format!(
            "undefined variable '{name}'"
        )))
    }

    /// Look up the full binding (value + metadata).
    pub fn get_binding(&self, name: &str) -> Result<&Binding, RuntimeError> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.get(name) {
                return Ok(b);
            }
        }
        Err(RuntimeError::RuntimeError(format!(
            "undefined variable '{name}'"
        )))
    }

    /// Assign a new value to an existing variable in the nearest enclosing scope.
    ///
    /// Returns `TypeError` if the variable is declared `constant`.
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(b) = scope.get_mut(name) {
                if b.is_constant {
                    return Err(RuntimeError::TypeError(format!(
                        "cannot assign to constant variable '{name}'"
                    )));
                }
                b.value = value;
                return Ok(());
            }
        }
        Err(RuntimeError::RuntimeError(format!(
            "undefined variable '{name}'"
        )))
    }
}
