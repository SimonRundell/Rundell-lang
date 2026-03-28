//! Rundell lexer crate.
//!
//! This crate tokenizes Rundell source code into a stream of [`Token`]s
//! using the [`logos`] crate.

pub mod token;

pub use token::Token;

use logos::Logos;

/// A span in the source: a byte-range `[start, end)`.
pub type Span = std::ops::Range<usize>;

/// Error produced when an unrecognised character sequence is encountered.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("Lexer error at byte offset {pos}: unexpected character")]
pub struct LexError {
    /// Byte offset of the offending character.
    pub pos: usize,
}

/// Tokenize `source` into a list of `(Token, Span)` pairs.
///
/// Returns [`Err`] if an unrecognised character is encountered.
pub fn lex(source: &str) -> Result<Vec<(Token, Span)>, LexError> {
    let mut result = Vec::new();
    let mut lex = Token::lexer(source);
    while let Some(tok) = lex.next() {
        let span = lex.span();
        match tok {
            Ok(t) => result.push((t, span)),
            Err(()) => return Err(LexError { pos: span.start }),
        }
    }
    Ok(result)
}
