//! Rundell parser crate.
//!
//! Parses a Rundell source string into an AST ([`Vec<Stmt>`]).

pub mod ast;
pub mod parser;

pub use ast::Stmt;
pub use parser::{parse, ParseError};
