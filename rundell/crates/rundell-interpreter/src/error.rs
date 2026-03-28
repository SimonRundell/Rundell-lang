//! Runtime error types for the Rundell interpreter.

use crate::evaluator::Value;

/// All errors that can occur at runtime in a Rundell program.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// A type mismatch or invalid cast.
    #[error("TypeError: {0}")]
    TypeError(String),

    /// Accessing a variable whose value is null.
    #[error("NullError: variable '{0}' is null")]
    NullError(String),

    /// A collection index is out of bounds or a key is missing.
    #[error("IndexError: {0}")]
    IndexError(String),

    /// Division or modulo by zero.
    #[error("DivisionError: division by zero")]
    DivisionError,

    /// An input/output failure.
    #[error("IOError: {0}")]
    IOError(String),

    /// A catch-all for other runtime errors.
    #[error("RuntimeError: {0}")]
    RuntimeError(String),

    /// Used internally to unwind the call stack on `return`.
    ///
    /// This is not a true error; it is caught by the function-call handler.
    #[error("Return: {0:?}")]
    ReturnValue(Option<Value>),
}
