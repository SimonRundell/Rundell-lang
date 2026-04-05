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

    /// Insufficient permissions to execute a program or script.
    #[error("PermissionError: insufficient permissions to execute '{path}'")]
    PermissionError { path: String },

    /// A catch-all for other runtime errors.
    #[error("RuntimeError: {0}")]
    RuntimeError(String),

    /// Used internally to unwind the call stack on `return`.
    ///
    /// This is not a true error; it is caught by the function-call handler.
    #[error("Return: {0:?}")]
    ReturnValue(Option<Value>),

    /// Query timed out waiting for a response.
    #[error("QueryTimeout: endpoint '{endpoint}' timed out after {timeout_ms}ms")]
    QueryTimeout { endpoint: String, timeout_ms: u64 },

    /// A network-level error occurred before receiving any response.
    #[error("QueryNetworkError: {message} (endpoint: {endpoint})")]
    QueryNetworkError { message: String, endpoint: String },

    /// The server returned an HTTP error status code.
    #[error("QueryHttpError: HTTP {status_code} from '{endpoint}'")]
    QueryHttpError { status_code: u16, endpoint: String },

    /// The response body was not valid JSON.
    #[error("QueryInvalidJson: response from '{endpoint}' is not valid JSON")]
    QueryInvalidJson { endpoint: String },

    /// A query identifier was called but not found in the registry.
    #[error("UndefinedQuery: no query named '{name}' is defined")]
    UndefinedQuery { name: String },

    /// A credentials identifier was referenced but not found.
    #[error("UndefinedCredentials: no credentials named '{name}' are defined")]
    UndefinedCredentials { name: String },

    /// env() was called but the key is absent from .rundell.env.
    #[error("EnvKeyNotFound: key '{key}' not found in .rundell.env")]
    EnvKeyNotFound { key: String },

    /// env() was called but decryption failed.
    #[error("EnvDecryptionFailed: failed to decrypt key '{key}' from .rundell.env")]
    EnvDecryptionFailed { key: String },

    /// env() was called but no program path is set (e.g. in REPL mode).
    #[error("NoProgramPath: env() cannot be used without a source file path")]
    NoProgramPath,

    /// An HTTP method other than GET or POST was attempted.
    #[error("UnsupportedHttpMethod: only GET and POST are supported")]
    UnsupportedHttpMethod,
}
