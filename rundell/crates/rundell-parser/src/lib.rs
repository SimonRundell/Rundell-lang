//! Rundell parser crate.
//!
//! Parses a Rundell source string into an AST ([`Vec<Stmt>`]).

pub mod ast;
pub mod error_format;
pub mod parser;

pub use ast::Stmt;
pub use error_format::format_parse_error;
pub use parser::{parse, ParseError};
