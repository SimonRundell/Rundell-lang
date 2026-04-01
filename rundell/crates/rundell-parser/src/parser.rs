//! Recursive-descent parser for the Rundell language.
//!
//! Converts a stream of [`Token`]s (produced by `rundell-lexer`) into
//! an abstract syntax tree represented as [`Vec<Stmt>`].

use std::ops::Range;

use rundell_lexer::{lex, Token};

use crate::ast::{
    AttemptBlock, AwaitExpr, BinOp, CatchClause, CmpOp, ControlType, CredentialsDefinition,
    DefineStmt, DialogCall, Expr, ForEachStmt, ForLoopStmt, FormDefinition, FunctionDefStmt,
    HttpMethod, IfStmt, Literal, MessageKind, Param, QueryDefinition, ReceiveStmt, RundellType,
    SetOp, SetStmt, SetTarget, Stmt, SwitchCase, SwitchPattern, SwitchStmt, TryCatchStmt,
    UnaryOp, WhileLoopStmt,
};

/// Errors that can occur during parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// An unexpected token was encountered.
    #[error("Unexpected token {found:?} at position {pos}, expected {expected}")]
    UnexpectedToken {
        /// String description of the token found.
        found: String,
        /// Byte offset in the source.
        pos: usize,
        /// What was expected instead.
        expected: String,
    },
    /// The token stream ended prematurely.
    #[error("Unexpected end of input")]
    UnexpectedEof,
    /// A circular import was detected.
    #[error("Circular import detected: {path}")]
    CircularImport {
        /// The import path that caused the cycle.
        path: String,
    },
    /// A general parse error with a message.
    #[error("{0}")]
    Other(String),
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Parse a Rundell source string into a program (list of statements).
///
/// This function lexes `source`, then runs the recursive-descent parser.
/// Returns an error if the input is syntactically invalid.
pub fn parse(source: &str) -> Result<Vec<Stmt>, ParseError> {
    let tokens = lex(source).map_err(|e| ParseError::Other(format!("Lex error: {e}")))?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// ---------------------------------------------------------------------------
// Parser struct
// ---------------------------------------------------------------------------

/// Hand-written recursive-descent parser.
struct Parser {
    tokens: Vec<(Token, Range<usize>)>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<(Token, Range<usize>)>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // -----------------------------------------------------------------------
    // Token stream helpers
    // -----------------------------------------------------------------------

    /// Return the current token without consuming it.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    /// Return the token N positions ahead without consuming.
    fn peek_ahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n).map(|(t, _)| t)
    }

    /// Return the byte offset of the current token.
    fn current_pos(&self) -> usize {
        self.tokens.get(self.pos).map(|(_, r)| r.start).unwrap_or(0)
    }

    /// Consume and return the current token.
    fn advance(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            let t = &self.tokens[self.pos].0;
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    /// Consume the current token if it matches `expected`, else return error.
    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        match self.peek() {
            Some(t) if t == expected => {
                self.advance();
                Ok(())
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{t:?}"),
                pos: self.current_pos(),
                expected: format!("{expected:?}"),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Consume the current token if it is a `Dot` (statement terminator).
    fn expect_dot(&mut self) -> Result<(), ParseError> {
        self.expect(&Token::Dot)
    }

    /// Consume an `Ident` token and return the identifier string.
    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.peek().cloned() {
            Some(Token::Ident(name)) => {
                self.advance();
                Ok(name)
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{t:?}"),
                pos: self.current_pos(),
                expected: "identifier".to_string(),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Consume the current token as a name, accepting both `Ident` tokens and
    /// any GUI keyword tokens that may legitimately appear as a name in context
    /// (e.g. `form` as a property path segment, `show` / `close` as method
    /// names in an object path).
    fn parse_name(&mut self) -> Result<String, ParseError> {
        match self.peek().cloned() {
            Some(Token::Ident(name)) => {
                self.advance();
                Ok(name)
            }
            // GUI keywords that can appear as path segments
            Some(Token::KwForm) => { self.advance(); Ok("form".to_string()) }
            Some(Token::KwShow) => { self.advance(); Ok("show".to_string()) }
            Some(Token::KwClose) => { self.advance(); Ok("close".to_string()) }
            Some(Token::KwModal) => { self.advance(); Ok("modal".to_string()) }
            Some(Token::KwAutorefresh) => { self.advance(); Ok("autorefresh".to_string()) }
            Some(Token::KwDatasource) => { self.advance(); Ok("datasource".to_string()) }
            Some(Token::KwColumns) => { self.advance(); Ok("columns".to_string()) }
            Some(Token::KwDialog) => { self.advance(); Ok("dialog".to_string()) }
            Some(Token::KwLabel) => { self.advance(); Ok("label".to_string()) }
            Some(Token::KwTextbox) => { self.advance(); Ok("textbox".to_string()) }
            Some(Token::KwButton) => { self.advance(); Ok("button".to_string()) }
            Some(Token::KwRadiobutton) => { self.advance(); Ok("radiobutton".to_string()) }
            Some(Token::KwCheckbox) => { self.advance(); Ok("checkbox".to_string()) }
            Some(Token::Switch) => { self.advance(); Ok("switch".to_string()) }
            Some(Token::KwSelect) => { self.advance(); Ok("select".to_string()) }
            Some(Token::KwListbox) => { self.advance(); Ok("listbox".to_string()) }
            // REST / query keywords that may appear as property path segments
            Some(Token::KwQuery) => { self.advance(); Ok("query".to_string()) }
            Some(Token::KwCredentials) => { self.advance(); Ok("credentials".to_string()) }
            Some(Token::KwMethod) => { self.advance(); Ok("method".to_string()) }
            Some(Token::KwEndpoint) => { self.advance(); Ok("endpoint".to_string()) }
            Some(Token::KwToken) => { self.advance(); Ok("token".to_string()) }
            Some(Token::KwAuthentication) => { self.advance(); Ok("authentication".to_string()) }
            Some(Token::KwTimeout) => { self.advance(); Ok("timeout".to_string()) }
            Some(Token::KwEnv) => { self.advance(); Ok("env".to_string()) }
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{t:?}"),
                pos: self.current_pos(),
                expected: "name".to_string(),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Return true if the current token can start an object path segment
    /// (Ident or any GUI keyword that is valid as a path segment).
    #[allow(dead_code)]
    fn is_name_token(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Ident(_))
                | Some(Token::KwForm)
                | Some(Token::KwShow)
                | Some(Token::KwClose)
                | Some(Token::KwModal)
                | Some(Token::KwAutorefresh)
                | Some(Token::KwDatasource)
                | Some(Token::KwColumns)
                | Some(Token::KwLabel)
                | Some(Token::KwTextbox)
                | Some(Token::KwButton)
                | Some(Token::KwRadiobutton)
                | Some(Token::KwCheckbox)
                | Some(Token::Switch)
                | Some(Token::KwSelect)
                | Some(Token::KwListbox)
        )
    }

    /// Check (without consuming) whether the current token matches `tok`.
    fn check(&self, tok: &Token) -> bool {
        self.peek() == Some(tok)
    }

    /// Consume the token if it matches `tok` and return true; else false.
    fn eat(&mut self, tok: &Token) -> bool {
        if self.check(tok) {
            self.advance();
            true
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // Top-level parsing
    // -----------------------------------------------------------------------

    /// Parse the entire program.
    fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // -----------------------------------------------------------------------
    // Statement parsing
    // -----------------------------------------------------------------------

    /// Parse a single statement.
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Some(Token::Import) => self.parse_import(),
            Some(Token::Define) => self.parse_define_or_funcdef(),
            Some(Token::Set) => self.parse_set(),
            Some(Token::Print) => self.parse_print(),
            Some(Token::Receive) => self.parse_receive(),
            Some(Token::If) => self.parse_if(),
            Some(Token::Switch) => self.parse_switch(),
            Some(Token::For) => self.parse_for(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Try) => self.parse_try(),
            Some(Token::KwAttempt) => self.parse_attempt(),
            Some(Token::Remove) => self.parse_remove(),
            Some(Token::Append) => self.parse_append_stmt(),
            // Object-path expression statements (show/close calls, or bare
            // object-path calls that the interpreter dispatches at runtime).
            Some(Token::Ident(_)) => self.parse_expr_stmt(),
            // `dialog\openfile(...)` etc. as a bare statement (result discarded)
            Some(Token::KwDialog) => self.parse_expr_stmt(),
            Some(t) => {
                let t = t.clone();
                Err(ParseError::UnexpectedToken {
                    found: format!("{t:?}"),
                    pos: self.current_pos(),
                    expected: "statement".to_string(),
                })
            }
            None => Err(ParseError::UnexpectedEof),
        }
    }

    // import "path".
    fn parse_import(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Import)?;
        let path = match self.peek().cloned() {
            Some(Token::StringLit(s)) => {
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::Other(
                    "import expects a string literal path".to_string(),
                ))
            }
        };
        self.expect_dot()?;
        Ok(Stmt::Import(path))
    }

    // define …  — dispatches to variable declaration, function definition,
    //             form definition, control declaration, or query definition.
    fn parse_define_or_funcdef(&mut self) -> Result<Stmt, ParseError> {
        // Peek ahead to see if this is a function or query definition.
        // Forms:
        //   define <name> ( ...  → function (or query if `as query` follows)
        //   define global <name> ( ...  → function (global is not valid for funcdef per spec,
        //                                 but we check anyway to produce a better error)
        let ident_offset = if self.peek_ahead(1) == Some(&Token::Global) {
            2 // define global <name>
        } else {
            1 // define <name>
        };
        if let Some(Token::Ident(_)) = self.peek_ahead(ident_offset) {
            if self.peek_ahead(ident_offset + 1) == Some(&Token::LParen) {
                // Could be a function OR a query definition.
                // To distinguish, we need to scan past the parameter list to
                // find `as query`.  We do a lightweight scan: if `as` is
                // followed by `query` somewhere after the closing `)`, it's a
                // query def; otherwise it's a function.
                if self.peek_ahead_is_query_def(ident_offset + 1) {
                    return self.parse_query_def_with_params();
                }
                return self.parse_funcdef();
            }
        }
        self.parse_define()
    }

    /// Scan forward from the `(` at `start_offset` to see whether this is
    /// a query definition (`… as query returns json -->`).
    ///
    /// This is a lightweight heuristic: scan for `) as query` after balancing
    /// parentheses.
    fn peek_ahead_is_query_def(&self, start_offset: usize) -> bool {
        // start_offset points to the LParen token (relative to current pos).
        // We need to find the matching RParen, then check for `as query`.
        let mut depth = 0usize;
        let mut i = start_offset;
        loop {
            match self.tokens.get(self.pos + i).map(|(t, _)| t) {
                Some(Token::LParen) => { depth += 1; i += 1; }
                Some(Token::RParen) => {
                    if depth == 0 { break; }
                    depth -= 1;
                    i += 1;
                }
                None => return false,
                _ => { i += 1; }
            }
        }
        // After `(params)` we expect `) as query`
        // i now points to the RParen that closed the params.
        i += 1; // skip RParen
        // Next should be `as`
        if self.tokens.get(self.pos + i).map(|(t, _)| t) != Some(&Token::As) {
            return false;
        }
        i += 1;
        // Then `query`
        matches!(
            self.tokens.get(self.pos + i).map(|(t, _)| t),
            Some(Token::KwQuery)
        )
    }

    /// Parse `define name(params) as query returns json --> body <--`.
    fn parse_query_def_with_params(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Define)?;
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::As)?;
        // consume `query`
        self.expect(&Token::KwQuery)?;
        self.parse_query_body(name, params)
    }

    // define [global] <name> as [constant] [global] <type> [= <expr>].
    //
    // Also handles:
    //   define <name> as form --> ... <--   (form definition)
    //   define <name> as form\<type>.       (control declaration inside a form)
    //
    // The spec examples show `global` can appear EITHER between `define` and
    // the identifier name, OR between `as` and the type keyword.  Both forms
    // are accepted here.
    fn parse_define(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Define)?;
        // Optional early `global` (before the identifier)
        let global_prefix = self.eat(&Token::Global);
        let name = self.expect_ident()?;
        self.expect(&Token::As)?;
        let constant = self.eat(&Token::Constant);
        // Optional `global` after `as` (the spec grammar form)
        let global_suffix = self.eat(&Token::Global);
        let global = global_prefix || global_suffix;

        // --- GUI: form definition or control declaration ---
        if self.check(&Token::KwForm) {
            self.advance(); // consume 'form'
            if self.check(&Token::Arrow) {
                // define name as form --> body <--
                return self.parse_form_body(name);
            } else if self.check(&Token::BackslashSep) {
                // define name as form\controltype.
                self.advance(); // consume '\'
                let ctrl_type = self.parse_control_type()?;
                self.expect_dot()?;
                return Ok(Stmt::DefineControl(name, ctrl_type));
            } else {
                return Err(ParseError::Other(
                    "expected '-->' or '\\' after 'form' in define statement".to_string(),
                ));
            }
        }

        // --- REST: credentials definition ---
        if self.check(&Token::KwCredentials) {
            self.advance(); // consume 'credentials'
            self.expect(&Token::Arrow)?;
            return self.parse_credentials_body(name);
        }

        // --- REST: query definition ---
        // Already consumed: define <name>(<params>) as
        // But wait — parse_define_or_funcdef dispatched here because the name
        // was NOT followed by '(' (that path goes to parse_funcdef).
        // For a query we need params, so we handle it via a separate dispatcher.
        // If we see 'query', dispatch to parse_query_body.
        if self.check(&Token::KwQuery) {
            self.advance(); // consume 'query'
            return self.parse_query_body(name, vec![]);
        }

        let typ = self.parse_type()?;
        let init = if self.eat(&Token::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect_dot()?;
        Ok(Stmt::Define(DefineStmt {
            name,
            typ,
            constant,
            global,
            init,
        }))
    }

    /// Parse the body of a `define name as form --> ... <--` block.
    fn parse_form_body(&mut self, name: String) -> Result<Stmt, ParseError> {
        self.expect(&Token::Arrow)?;
        let mut body = Vec::new();
        while !self.check(&Token::ArrowEnd) {
            if self.peek().is_none() {
                return Err(ParseError::UnexpectedEof);
            }
            body.push(self.parse_stmt()?);
        }
        self.expect(&Token::ArrowEnd)?;
        Ok(Stmt::FormDef(FormDefinition { name, body }))
    }

    /// Parse a control type keyword after `form\`.
    fn parse_control_type(&mut self) -> Result<ControlType, ParseError> {
        match self.peek().cloned() {
            Some(Token::KwLabel) => { self.advance(); Ok(ControlType::Label) }
            Some(Token::KwTextbox) => { self.advance(); Ok(ControlType::Textbox) }
            Some(Token::KwButton) => { self.advance(); Ok(ControlType::Button) }
            Some(Token::KwRadiobutton) => { self.advance(); Ok(ControlType::Radiobutton) }
            Some(Token::KwCheckbox) => { self.advance(); Ok(ControlType::Checkbox) }
            Some(Token::Switch) => { self.advance(); Ok(ControlType::Switch) }
            Some(Token::KwSelect) => { self.advance(); Ok(ControlType::Select) }
            Some(Token::KwListbox) => { self.advance(); Ok(ControlType::Listbox) }
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{t:?}"),
                pos: self.current_pos(),
                expected: "control type (label/textbox/button/radiobutton/checkbox/switch/select/listbox)".to_string(),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    // define <name>(<params>) returns <type> --> body <--
    fn parse_funcdef(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Define)?;
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::Returns)?;
        let return_type = if self.check(&Token::Null) {
            self.advance();
            None
        } else {
            Some(self.parse_type()?)
        };
        self.expect(&Token::Arrow)?;
        let body = self.parse_body()?;
        Ok(Stmt::FunctionDef(FunctionDefStmt {
            name,
            params,
            return_type,
            body,
        }))
    }

    // Parse a comma-separated parameter list (may be empty).
    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) {
            return Ok(params);
        }
        loop {
            let name = self.expect_ident()?;
            self.expect(&Token::As)?;
            let typ = self.parse_type()?;
            params.push(Param { name, typ });
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        Ok(params)
    }

    // Parse a block body: statements until <--
    fn parse_body(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.check(&Token::ArrowEnd) {
            if self.peek().is_none() {
                return Err(ParseError::UnexpectedEof);
            }
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::ArrowEnd)?;
        Ok(stmts)
    }

    // set <target> (++ | -- | = <expr>).
    fn parse_set(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Set)?;
        // Determine target
        let target = self.parse_set_target()?;
        let op = if self.eat(&Token::PlusPlus) {
            SetOp::Increment
        } else if self.eat(&Token::MinusMinus) {
            SetOp::Decrement
        } else {
            self.expect(&Token::Eq)?;
            // Special case: `set path\position = top, left, width, height.`
            let is_position = match &target {
                SetTarget::ObjectPath(segs) => {
                    segs.last().map(|s| s == "position").unwrap_or(false)
                }
                _ => false,
            };
            if is_position && matches!(self.peek(), Some(Token::PixelValue(_))) {
                SetOp::Assign(self.parse_position_literal()?)
            } else {
                SetOp::Assign(self.parse_expr()?)
            }
        };
        self.expect_dot()?;
        Ok(Stmt::Set(SetStmt { target, op }))
    }

    /// Parse a position literal: `top_px, left_px, width_px, height_px`.
    ///
    /// Each component is `<integer>px`.  Called only when the assignment
    /// target's last segment is `"position"`.
    fn parse_position_literal(&mut self) -> Result<Expr, ParseError> {
        let top = self.parse_px_value()?;
        self.expect(&Token::Comma)?;
        let left = self.parse_px_value()?;
        self.expect(&Token::Comma)?;
        let width = self.parse_px_value()?;
        self.expect(&Token::Comma)?;
        let height = self.parse_px_value()?;
        Ok(Expr::PositionLiteral(top, left, width, height))
    }

    /// Consume a `PixelValue(n)` token and return `n`.
    fn parse_px_value(&mut self) -> Result<u32, ParseError> {
        match self.peek().cloned() {
            Some(Token::PixelValue(n)) => {
                self.advance();
                Ok(n)
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{t:?}"),
                pos: self.current_pos(),
                expected: "pixel value (e.g. 10px)".to_string(),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    // Parse set target: Ident, Ident[expr][expr]..., or ObjectPath (Ident\Ident\...).
    fn parse_set_target(&mut self) -> Result<SetTarget, ParseError> {
        // Check for object path: current token is a name AND next token is BackslashSep.
        // Also accept 'form' keyword as first segment (inside form body).
        let is_kw_form_first = self.check(&Token::KwForm)
            && self.peek_ahead(1) == Some(&Token::BackslashSep);
        let is_ident_path = matches!(self.peek(), Some(Token::Ident(_)))
            && self.peek_ahead(1) == Some(&Token::BackslashSep);

        if is_kw_form_first || is_ident_path {
            return self.parse_object_path_target();
        }

        let name = self.expect_ident()?;
        if self.check(&Token::LBracket) {
            // collection index target
            let mut base: Expr = Expr::Identifier(name);
            while self.eat(&Token::LBracket) {
                let idx = self.parse_expr()?;
                self.expect(&Token::RBracket)?;
                base = Expr::Index(Box::new(base), Box::new(idx));
            }
            // Unwrap into SetTarget::Index
            match base {
                Expr::Index(col, key) => Ok(SetTarget::Index(col, key)),
                _ => Err(ParseError::Other("invalid set target".to_string())),
            }
        } else {
            Ok(SetTarget::Identifier(name))
        }
    }

    /// Parse an object-path set target: `seg0\seg1\...\segN`.
    fn parse_object_path_target(&mut self) -> Result<SetTarget, ParseError> {
        let mut segments = Vec::new();
        segments.push(self.parse_name()?);
        while self.eat(&Token::BackslashSep) {
            segments.push(self.parse_name()?);
        }
        Ok(SetTarget::ObjectPath(segments))
    }

    // print <expr>.
    fn parse_print(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Print)?;
        let expr = self.parse_expr()?;
        self.expect_dot()?;
        Ok(Stmt::Print(expr))
    }

    // receive <ident> [with prompt <expr>].
    fn parse_receive(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Receive)?;
        let variable = self.expect_ident()?;
        let prompt = if self.eat(&Token::With) {
            self.expect(&Token::Prompt)?;
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect_dot()?;
        Ok(Stmt::Receive(ReceiveStmt { variable, prompt }))
    }

    // if (cond) --> body [else if (cond) --> body]* [else --> body] <--
    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::If)?;
        let has_paren = self.eat(&Token::LParen);
        let condition = self.parse_expr()?;
        if has_paren {
            self.expect(&Token::RParen)?;
        }
        self.expect(&Token::Arrow)?;
        let then_body = self.parse_if_body()?;

        let mut else_ifs = Vec::new();
        let mut else_body = None;

        // Check for else if or else
        loop {
            if self.check(&Token::ArrowEnd) {
                self.advance();
                break;
            }
            if self.check(&Token::Else) {
                self.advance(); // consume `else`
                if self.check(&Token::If) {
                    self.advance(); // consume `if`
                    let has_paren2 = self.eat(&Token::LParen);
                    let cond = self.parse_expr()?;
                    if has_paren2 {
                        self.expect(&Token::RParen)?;
                    }
                    self.expect(&Token::Arrow)?;
                    let body = self.parse_if_body()?;
                    else_ifs.push((cond, body));
                } else {
                    // plain else
                    self.expect(&Token::Arrow)?;
                    let body = self.parse_if_body()?;
                    else_body = Some(body);
                    // next must be <--
                    self.expect(&Token::ArrowEnd)?;
                    break;
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: format!("{:?}", self.peek()),
                    pos: self.current_pos(),
                    expected: "else or <--".to_string(),
                });
            }
        }

        Ok(Stmt::If(IfStmt {
            condition,
            then_body,
            else_ifs,
            else_body,
        }))
    }

    /// Parse if-body: statements until we see `else` or `<--` (don't consume).
    fn parse_if_body(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        loop {
            match self.peek() {
                Some(Token::ArrowEnd) | Some(Token::Else) => break,
                Some(_) => stmts.push(self.parse_stmt()?),
                None => return Err(ParseError::UnexpectedEof),
            }
        }
        Ok(stmts)
    }

    // switch <expr> --> cases <--
    fn parse_switch(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Switch)?;
        let subject = self.parse_expr()?;
        self.expect(&Token::Arrow)?;

        let mut cases: Vec<SwitchCase> = Vec::new();
        // Parse case lines until <--
        while !self.check(&Token::ArrowEnd) {
            if self.peek().is_none() {
                return Err(ParseError::UnexpectedEof);
            }

            let pattern = self.parse_switch_pattern()?;
            self.expect(&Token::Colon)?;

            // Body: if next is a pattern-like token or else or ArrowEnd → grouped (empty body)
            // Otherwise parse a single statement.
            let body = if self.is_switch_case_start() || self.check(&Token::ArrowEnd) {
                // grouped case — no body, falls to the next case
                Vec::new()
            } else {
                let stmt = self.parse_stmt()?;
                vec![stmt]
            };

            cases.push(SwitchCase { pattern, body });
        }
        self.expect(&Token::ArrowEnd)?;

        Ok(Stmt::Switch(SwitchStmt { subject, cases }))
    }

    /// Return true if the current token is the start of a new switch case line.
    fn is_switch_case_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Else)
                | Some(Token::Lt)
                | Some(Token::LtEq)
                | Some(Token::Gt)
                | Some(Token::GtEq)
                | Some(Token::EqEq)
                | Some(Token::BangEq)
                | Some(Token::Integer(_))
                | Some(Token::Float(_))
                | Some(Token::CurrencyLit(_))
                | Some(Token::StringLit(_))
                | Some(Token::BoolTrue)
                | Some(Token::BoolFalse)
                | Some(Token::Ident(_))
        )
    }

    fn parse_switch_pattern(&mut self) -> Result<SwitchPattern, ParseError> {
        if self.eat(&Token::Else) {
            return Ok(SwitchPattern::Default);
        }
        // Check for comparison operator prefix
        let cmp = match self.peek() {
            Some(Token::Lt) => {
                self.advance();
                Some(CmpOp::Lt)
            }
            Some(Token::LtEq) => {
                self.advance();
                Some(CmpOp::LtEq)
            }
            Some(Token::Gt) => {
                self.advance();
                Some(CmpOp::Gt)
            }
            Some(Token::GtEq) => {
                self.advance();
                Some(CmpOp::GtEq)
            }
            Some(Token::EqEq) => {
                self.advance();
                Some(CmpOp::Eq)
            }
            Some(Token::BangEq) => {
                self.advance();
                Some(CmpOp::NotEq)
            }
            _ => None,
        };
        let expr = self.parse_expr()?;
        if let Some(op) = cmp {
            Ok(SwitchPattern::Comparison(op, expr))
        } else {
            Ok(SwitchPattern::Exact(expr))
        }
    }

    // for … — either for i loops (…) or for each …
    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::For)?;
        if self.check(&Token::Each) {
            self.advance();
            return self.parse_foreach();
        }
        self.parse_forloop()
    }

    // for <var> loops (<start>, <end>, <step>) --> body <--
    fn parse_forloop(&mut self) -> Result<Stmt, ParseError> {
        let var = self.expect_ident()?;
        self.expect(&Token::Loops)?;
        self.expect(&Token::LParen)?;
        let start = self.parse_expr()?;
        self.expect(&Token::Comma)?;
        let end = self.parse_expr()?;
        self.expect(&Token::Comma)?;
        let increment = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::Arrow)?;
        let body = self.parse_body()?;
        Ok(Stmt::ForLoop(ForLoopStmt {
            var,
            start,
            end,
            increment,
            body,
        }))
    }

    // each <var> in <collection> --> body <--  (for was already consumed)
    fn parse_foreach(&mut self) -> Result<Stmt, ParseError> {
        let var = self.expect_ident()?;
        self.expect(&Token::In)?;
        let collection = self.parse_expr()?;
        self.expect(&Token::Arrow)?;
        let body = self.parse_body()?;
        Ok(Stmt::ForEach(ForEachStmt {
            var,
            collection,
            body,
        }))
    }

    // while <cond> --> body <--
    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::While)?;
        let condition = self.parse_expr()?;
        self.expect(&Token::Arrow)?;
        let body = self.parse_body()?;
        Ok(Stmt::WhileLoop(WhileLoopStmt { condition, body }))
    }

    // return [expr].
    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Return)?;
        if self.eat(&Token::Dot) {
            return Ok(Stmt::Return(None));
        }
        let expr = self.parse_expr()?;
        self.expect_dot()?;
        Ok(Stmt::Return(Some(expr)))
    }

    // try --> body catch(ErrType) --> body [finally --> body] <--
    fn parse_try(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Try)?;
        self.expect(&Token::Arrow)?;
        let try_body = self.parse_try_body()?;

        let mut catches = Vec::new();
        let mut finally_body = None;

        loop {
            if self.check(&Token::ArrowEnd) {
                self.advance();
                break;
            }
            if self.check(&Token::Catch) {
                self.advance();
                self.expect(&Token::LParen)?;
                let error_type = self.parse_error_type()?;
                self.expect(&Token::RParen)?;
                self.expect(&Token::Arrow)?;
                let body = self.parse_try_body()?;
                catches.push(CatchClause { error_type, body });
            } else if self.check(&Token::Finally) {
                self.advance();
                self.expect(&Token::Arrow)?;
                let body = self.parse_try_body()?;
                finally_body = Some(body);
                self.expect(&Token::ArrowEnd)?;
                break;
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: format!("{:?}", self.peek()),
                    pos: self.current_pos(),
                    expected: "catch, finally, or <--".to_string(),
                });
            }
        }

        Ok(Stmt::TryCatch(TryCatchStmt {
            try_body,
            catches,
            finally_body,
        }))
    }

    /// Parse body until `catch`, `finally`, or `<--` (don't consume).
    fn parse_try_body(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        loop {
            match self.peek() {
                Some(Token::ArrowEnd) | Some(Token::Catch) | Some(Token::Finally) => break,
                Some(_) => stmts.push(self.parse_stmt()?),
                None => return Err(ParseError::UnexpectedEof),
            }
        }
        Ok(stmts)
    }

    fn parse_error_type(&mut self) -> Result<String, ParseError> {
        match self.peek().cloned() {
            Some(Token::KwTypeError) => {
                self.advance();
                Ok("TypeError".to_string())
            }
            Some(Token::KwNullError) => {
                self.advance();
                Ok("NullError".to_string())
            }
            Some(Token::KwIndexError) => {
                self.advance();
                Ok("IndexError".to_string())
            }
            Some(Token::KwDivisionError) => {
                self.advance();
                Ok("DivisionError".to_string())
            }
            Some(Token::KwIOError) => {
                self.advance();
                Ok("IOError".to_string())
            }
            Some(Token::KwRuntimeError) => {
                self.advance();
                Ok("RuntimeError".to_string())
            }
            Some(Token::Ident(s)) => {
                self.advance();
                Ok(s)
            }
            _ => Err(ParseError::Other("expected error type".to_string())),
        }
    }

    // remove expr.
    fn parse_remove(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Remove)?;
        let expr = self.parse_expr()?;
        self.expect_dot()?;
        Ok(Stmt::Remove(expr))
    }

    // append(col, val).
    fn parse_append_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Append)?;
        self.expect(&Token::LParen)?;
        let col = self.parse_expr()?;
        self.expect(&Token::Comma)?;
        let val = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        self.expect_dot()?;
        Ok(Stmt::Append(col, val))
    }

    // An expression used as a statement (bare function call).
    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;
        self.expect_dot()?;
        Ok(Stmt::ExprStmt(expr))
    }

    // -----------------------------------------------------------------------
    // Type parsing
    // -----------------------------------------------------------------------

    fn parse_type(&mut self) -> Result<RundellType, ParseError> {
        match self.peek().cloned() {
            Some(Token::KwInteger) => {
                self.advance();
                Ok(RundellType::Integer)
            }
            Some(Token::KwFloat) => {
                self.advance();
                Ok(RundellType::Float)
            }
            Some(Token::KwString) => {
                self.advance();
                Ok(RundellType::Str)
            }
            Some(Token::KwCurrency) => {
                self.advance();
                Ok(RundellType::Currency)
            }
            Some(Token::KwBoolean) => {
                self.advance();
                Ok(RundellType::Boolean)
            }
            Some(Token::KwJson) => {
                self.advance();
                Ok(RundellType::Json)
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{t:?}"),
                pos: self.current_pos(),
                expected: "type name".to_string(),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    // -----------------------------------------------------------------------
    // Expression parsing (precedence climbing)
    // -----------------------------------------------------------------------

    /// Parse an expression at the lowest precedence level (or/is-null).
    pub(crate) fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_is_null()?;
        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_is_null()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::And, Box::new(right));
        }
        Ok(left)
    }

    /// Handle postfix `is null` / `is not null`.
    fn parse_is_null(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_comparison()?;
        if self.check(&Token::Is) {
            self.advance(); // consume `is`
            if self.check(&Token::Not) {
                self.advance(); // consume `not`
                self.expect(&Token::Null)?;
                return Ok(Expr::IsNotNull(Box::new(expr)));
            }
            self.expect(&Token::Null)?;
            return Ok(Expr::IsNull(Box::new(expr)));
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Some(Token::EqEq) => BinOp::Eq,
                Some(Token::BangEq) => BinOp::NotEq,
                Some(Token::Lt) => BinOp::Lt,
                Some(Token::LtEq) => BinOp::LtEq,
                Some(Token::Gt) => BinOp::Gt,
                Some(Token::GtEq) => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Percent) => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr, ParseError> {
        let base = self.parse_unary()?;
        if self.check(&Token::StarStar) {
            self.advance();
            // Right-associative
            let exp = self.parse_power()?;
            return Ok(Expr::BinaryOp(Box::new(base), BinOp::Pow, Box::new(exp)));
        }
        Ok(base)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.eat(&Token::Not) {
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)));
        }
        if self.eat(&Token::Minus) {
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr)));
        }
        self.parse_postfix()
    }

    /// Parse postfix index access `expr[key]`.
    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        while self.check(&Token::LBracket) {
            self.advance();
            let idx = self.parse_expr()?;
            self.expect(&Token::RBracket)?;
            expr = Expr::Index(Box::new(expr), Box::new(idx));
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().cloned() {
            Some(Token::Integer(n)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Integer(n)))
            }
            Some(Token::Float(f)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(f)))
            }
            Some(Token::CurrencyLit(c)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Currency(c)))
            }
            Some(Token::StringLit(s)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Str(s)))
            }
            Some(Token::BoolTrue) => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(true)))
            }
            Some(Token::BoolFalse) => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(false)))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Some(Token::LBrace) => self.parse_json_literal(),
            // Built-in functions / keywords used as calls
            Some(Token::Cast) => self.parse_builtin_call("cast"),
            Some(Token::Length) => self.parse_builtin_call("length"),
            Some(Token::Newline) => self.parse_builtin_call("newline"),
            Some(Token::Abs) => self.parse_builtin_call("abs"),
            Some(Token::Floor) => self.parse_builtin_call("floor"),
            Some(Token::Ceil) => self.parse_builtin_call("ceil"),
            Some(Token::Round) => self.parse_builtin_call("round"),
            Some(Token::Substr) => self.parse_builtin_call("substr"),
            Some(Token::Upper) => self.parse_builtin_call("upper"),
            Some(Token::Lower) => self.parse_builtin_call("lower"),
            Some(Token::Trim) => self.parse_builtin_call("trim"),
            Some(Token::KwString) => self.parse_builtin_call("string"),
            Some(Token::Append) => self.parse_builtin_call("append"),
            // REST / query built-ins
            Some(Token::KwEnv) => self.parse_builtin_call("env"),
            // `await <call>` — only valid as RHS of a set statement
            Some(Token::KwAwait) => {
                self.advance(); // consume 'await'
                let call = self.parse_primary()?;
                Ok(Expr::Await(Box::new(AwaitExpr { call: Box::new(call) })))
            }
            Some(Token::Ident(name)) => {
                self.advance();
                // Object path? Ident followed by BackslashSep.
                if self.check(&Token::BackslashSep) {
                    return self.parse_object_path_expr(name);
                }
                // Function call?
                if self.check(&Token::LParen) {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            // `dialog\openfile(...)`, `dialog\savefile(...)`, etc.
            Some(Token::KwDialog) => {
                self.advance(); // consume 'dialog'
                self.expect(&Token::BackslashSep)?;
                self.parse_dialog_call()
            }
            // Pixel value literal: `10px`
            Some(Token::PixelValue(n)) => {
                self.advance();
                Ok(Expr::PixelValue(n))
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{t:?}"),
                pos: self.current_pos(),
                expected: "expression".to_string(),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Parse an object-path expression starting with `first_seg` (already
    /// consumed).  Handles `show()`/`close()` method calls and plain path
    /// reads.
    fn parse_object_path_expr(&mut self, first_seg: String) -> Result<Expr, ParseError> {
        let mut segments = vec![first_seg];
        while self.eat(&Token::BackslashSep) {
            segments.push(self.parse_name()?);
        }
        // Check for method call at the end of the path.
        if self.check(&Token::LParen) {
            let method = segments.pop().unwrap_or_default();
            self.advance(); // consume '('
            match method.as_str() {
                "show" => {
                    let modal = self.eat(&Token::KwModal);
                    self.expect(&Token::RParen)?;
                    return Ok(Expr::ShowForm { path: segments, modal });
                }
                "close" => {
                    self.expect(&Token::RParen)?;
                    return Ok(Expr::CloseForm { path: segments });
                }
                other => {
                    // Generic method call — treat as a function call for
                    // future extensibility; interpreter may dispatch it.
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    // Encode as Call with the full path joined by '\' as name.
                    let mut full = segments.join("\\");
                    full.push('\\');
                    full.push_str(other);
                    return Ok(Expr::Call(full, args));
                }
            }
        }
        Ok(Expr::ObjectPath(segments))
    }

    /// Parse a `dialog\<type>(args)` call expression.
    ///
    /// The `dialog` keyword and first `\` have already been consumed.
    fn parse_dialog_call(&mut self) -> Result<Expr, ParseError> {
        let dialog_type = self.parse_name()?;
        self.expect(&Token::LParen)?;
        let call = match dialog_type.as_str() {
            "openfile" => {
                let title = self.parse_expr()?;
                self.expect(&Token::Comma)?;
                let filter = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                DialogCall::OpenFile {
                    title: Box::new(title),
                    filter: Box::new(filter),
                }
            }
            "savefile" => {
                let title = self.parse_expr()?;
                self.expect(&Token::Comma)?;
                let filter = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                DialogCall::SaveFile {
                    title: Box::new(title),
                    filter: Box::new(filter),
                }
            }
            "message" => {
                let title = self.parse_expr()?;
                self.expect(&Token::Comma)?;
                let message = self.parse_expr()?;
                self.expect(&Token::Comma)?;
                let kind = self.parse_message_kind()?;
                self.expect(&Token::RParen)?;
                DialogCall::Message {
                    title: Box::new(title),
                    message: Box::new(message),
                    kind,
                }
            }
            "colorpicker" => {
                let initial = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                DialogCall::ColorPicker { initial: Box::new(initial) }
            }
            other => {
                return Err(ParseError::Other(format!(
                    "unknown dialog type '{other}': expected openfile/savefile/message/colorpicker"
                )));
            }
        };
        Ok(Expr::Dialog(Box::new(call)))
    }

    /// Parse a message-box kind keyword: `ok`, `okcancel`, or `yesno`.
    fn parse_message_kind(&mut self) -> Result<MessageKind, ParseError> {
        match self.peek().cloned() {
            Some(Token::Ident(s)) => match s.as_str() {
                "ok" => { self.advance(); Ok(MessageKind::Ok) }
                "okcancel" => { self.advance(); Ok(MessageKind::OkCancel) }
                "yesno" => { self.advance(); Ok(MessageKind::YesNo) }
                other => Err(ParseError::Other(format!(
                    "unknown message kind '{other}': expected ok/okcancel/yesno"
                ))),
            },
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{t:?}"),
                pos: self.current_pos(),
                expected: "ok, okcancel, or yesno".to_string(),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Parse a built-in function call after consuming the keyword.
    fn parse_builtin_call(&mut self, name: &str) -> Result<Expr, ParseError> {
        self.advance(); // consume the keyword token
        self.expect(&Token::LParen)?;
        // Special case: cast(expr, type) — the second arg is a type keyword
        let args = if name == "cast" {
            let expr = self.parse_expr()?;
            self.expect(&Token::Comma)?;
            let type_expr = self.parse_type_as_expr()?;
            self.expect(&Token::RParen)?;
            vec![expr, type_expr]
        } else {
            let args = self.parse_args()?;
            self.expect(&Token::RParen)?;
            args
        };
        Ok(Expr::Call(name.to_string(), args))
    }

    /// Parse a type keyword as a string-literal expression (for cast()).
    fn parse_type_as_expr(&mut self) -> Result<Expr, ParseError> {
        let type_name = match self.peek().cloned() {
            Some(Token::KwInteger) => "integer",
            Some(Token::KwFloat) => "float",
            Some(Token::KwString) => "string",
            Some(Token::KwCurrency) => "currency",
            Some(Token::KwBoolean) => "boolean",
            Some(Token::KwJson) => "json",
            Some(t) => {
                return Err(ParseError::UnexpectedToken {
                    found: format!("{t:?}"),
                    pos: self.current_pos(),
                    expected: "type keyword for cast".to_string(),
                })
            }
            None => return Err(ParseError::UnexpectedEof),
        };
        self.advance();
        Ok(Expr::Literal(Literal::Str(type_name.to_string())))
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if self.check(&Token::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr()?);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        Ok(args)
    }

    // Parse a JSON object literal { ... }
    fn parse_json_literal(&mut self) -> Result<Expr, ParseError> {
        let value = self.parse_json_value()?;
        Ok(Expr::JsonLiteral(value))
    }

    fn parse_json_value(&mut self) -> Result<serde_json::Value, ParseError> {
        match self.peek().cloned() {
            Some(Token::LBrace) => self.parse_json_object(),
            Some(Token::LBracket) => self.parse_json_array(),
            Some(Token::StringLit(s)) => {
                self.advance();
                Ok(serde_json::Value::String(s))
            }
            Some(Token::Integer(n)) => {
                self.advance();
                Ok(serde_json::Value::Number(n.into()))
            }
            Some(Token::Float(f)) => {
                self.advance();
                Ok(serde_json::json!(f))
            }
            Some(Token::CurrencyLit(c)) => {
                // Store as float for JSON representation
                self.advance();
                let f = c as f64 / 100.0;
                Ok(serde_json::json!(f))
            }
            Some(Token::BoolTrue) => {
                self.advance();
                Ok(serde_json::Value::Bool(true))
            }
            Some(Token::BoolFalse) => {
                self.advance();
                Ok(serde_json::Value::Bool(false))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(serde_json::Value::Null)
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{t:?}"),
                pos: self.current_pos(),
                expected: "JSON value".to_string(),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn parse_json_object(&mut self) -> Result<serde_json::Value, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut map = serde_json::Map::new();
        if self.eat(&Token::RBrace) {
            return Ok(serde_json::Value::Object(map));
        }
        loop {
            let key = match self.peek().cloned() {
                Some(Token::StringLit(s)) => {
                    self.advance();
                    s
                }
                Some(t) => {
                    return Err(ParseError::UnexpectedToken {
                        found: format!("{t:?}"),
                        pos: self.current_pos(),
                        expected: "string key".to_string(),
                    })
                }
                None => return Err(ParseError::UnexpectedEof),
            };
            self.expect(&Token::Colon)?;
            let val = self.parse_json_value()?;
            map.insert(key, val);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(serde_json::Value::Object(map))
    }

    fn parse_json_array(&mut self) -> Result<serde_json::Value, ParseError> {
        self.expect(&Token::LBracket)?;
        let mut arr = Vec::new();
        if self.eat(&Token::RBracket) {
            return Ok(serde_json::Value::Array(arr));
        }
        loop {
            arr.push(self.parse_json_value()?);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RBracket)?;
        Ok(serde_json::Value::Array(arr))
    }

    // -----------------------------------------------------------------------
    // REST / query parsing
    // -----------------------------------------------------------------------

    /// Parse the body of `define name as credentials --> ... <--`.
    ///
    /// Accepts `set name\token = expr.` and `set name\authentication = expr.`
    /// statements inside the block.  Any other property is a parse error.
    fn parse_credentials_body(&mut self, name: String) -> Result<Stmt, ParseError> {
        let mut token_expr: Option<Expr> = None;
        let mut auth_expr: Option<Expr> = None;

        while !self.check(&Token::ArrowEnd) {
            if self.peek().is_none() {
                return Err(ParseError::UnexpectedEof);
            }
            // Each inner statement must be:  set <name>\<prop> = <expr>.
            self.expect(&Token::Set)?;
            let obj_name = self.expect_ident()?;
            if obj_name != name {
                return Err(ParseError::Other(format!(
                    "expected '{name}\\...' inside credentials block, got '{obj_name}'"
                )));
            }
            self.expect(&Token::BackslashSep)?;
            // The property name is either `token` or `authentication`.
            let prop = match self.peek().cloned() {
                Some(Token::KwToken) => { self.advance(); "token" }
                Some(Token::KwAuthentication) => { self.advance(); "authentication" }
                Some(Token::Ident(ref s)) if s == "token" => { self.advance(); "token" }
                Some(Token::Ident(ref s)) if s == "authentication" => {
                    self.advance(); "authentication"
                }
                Some(t) => {
                    return Err(ParseError::Other(format!(
                        "invalid credentials property '{t:?}': expected 'token' or 'authentication'"
                    )))
                }
                None => return Err(ParseError::UnexpectedEof),
            };
            self.expect(&Token::Eq)?;
            let expr = self.parse_expr()?;
            self.expect_dot()?;
            match prop {
                "token" => token_expr = Some(expr),
                "authentication" => auth_expr = Some(expr),
                _ => unreachable!(),
            }
        }
        self.expect(&Token::ArrowEnd)?;
        Ok(Stmt::CredentialsDef(CredentialsDefinition {
            name,
            token: token_expr,
            authentication: auth_expr,
        }))
    }

    /// Parse the body of `define name(params) as query returns json --> ... <--`.
    ///
    /// Called AFTER consuming `define name(params) as query`.
    /// Expects `returns json -->` next.
    fn parse_query_body(&mut self, name: String, params: Vec<Param>) -> Result<Stmt, ParseError> {
        // `returns json` is mandatory.
        if !self.check(&Token::Returns) {
            return Err(ParseError::Other(
                "query definition requires 'returns json'".to_string(),
            ));
        }
        self.advance(); // consume `returns`
        if !self.check(&Token::KwJson) {
            return Err(ParseError::Other(
                "query definition requires 'returns json' (expected 'json' after 'returns')"
                    .to_string(),
            ));
        }
        self.advance(); // consume `json`
        self.expect(&Token::Arrow)?;

        let mut method: Option<HttpMethod> = None;
        let mut endpoint: Option<Expr> = None;
        let mut credentials: Option<String> = None;
        let mut timeout_ms: Option<Expr> = None;
        let mut query_params: Option<Expr> = None;

        while !self.check(&Token::ArrowEnd) {
            if self.peek().is_none() {
                return Err(ParseError::UnexpectedEof);
            }

            // Each line is one of:
            //   set <name>\method = GET | POST.
            //   set <name>\endpoint = <expr>.
            //   set <name>\credentials = <ident>.
            //   set <name>\timeout = <expr>.
            //   define queryParams as json = { ... }.
            match self.peek().cloned() {
                Some(Token::Set) => {
                    self.advance(); // consume `set`
                    let obj_name = self.expect_ident()?;
                    if obj_name != name {
                        return Err(ParseError::Other(format!(
                            "expected '{name}\\...' inside query block, got '{obj_name}'"
                        )));
                    }
                    self.expect(&Token::BackslashSep)?;
                    let prop = match self.peek().cloned() {
                        Some(Token::KwMethod) => { self.advance(); "method" }
                        Some(Token::KwEndpoint) => { self.advance(); "endpoint" }
                        Some(Token::KwCredentials) => { self.advance(); "credentials" }
                        Some(Token::KwTimeout) => { self.advance(); "timeout" }
                        Some(Token::Ident(ref s)) if s == "method" => { self.advance(); "method" }
                        Some(Token::Ident(ref s)) if s == "endpoint" => { self.advance(); "endpoint" }
                        Some(Token::Ident(ref s)) if s == "credentials" => { self.advance(); "credentials" }
                        Some(Token::Ident(ref s)) if s == "timeout" => { self.advance(); "timeout" }
                        Some(t) => {
                            return Err(ParseError::Other(format!(
                                "invalid query property '{t:?}': expected method/endpoint/credentials/timeout"
                            )))
                        }
                        None => return Err(ParseError::UnexpectedEof),
                    };
                    self.expect(&Token::Eq)?;
                    match prop {
                        "method" => {
                            let m = match self.peek().cloned() {
                                Some(Token::KwGet) => { self.advance(); HttpMethod::Get }
                                Some(Token::KwPost) => { self.advance(); HttpMethod::Post }
                                Some(t) => {
                                    return Err(ParseError::Other(format!(
                                        "expected GET or POST, got {t:?}"
                                    )))
                                }
                                None => return Err(ParseError::UnexpectedEof),
                            };
                            method = Some(m);
                        }
                        "endpoint" => {
                            endpoint = Some(self.parse_expr()?);
                        }
                        "credentials" => {
                            let cred_name = self.expect_ident()?;
                            credentials = Some(cred_name);
                        }
                        "timeout" => {
                            timeout_ms = Some(self.parse_expr()?);
                        }
                        _ => unreachable!(),
                    }
                    self.expect_dot()?;
                }
                Some(Token::Define) => {
                    // Only `define queryParams as json = { ... }.` is valid here.
                    self.advance(); // consume `define`
                    let qp_name = self.expect_ident()?;
                    if qp_name != "queryParams" {
                        return Err(ParseError::Other(format!(
                            "only 'define queryParams as json = ...' is valid inside a query block, got '{qp_name}'"
                        )));
                    }
                    self.expect(&Token::As)?;
                    self.expect(&Token::KwJson)?;
                    self.expect(&Token::Eq)?;
                    let val = self.parse_json_literal()?;
                    self.expect_dot()?;
                    query_params = Some(val);
                }
                Some(t) => {
                    return Err(ParseError::Other(format!(
                        "unexpected token {t:?} inside query block"
                    )))
                }
                None => return Err(ParseError::UnexpectedEof),
            }
        }
        self.expect(&Token::ArrowEnd)?;

        // Validate mandatory fields.
        let method = method.ok_or_else(|| {
            ParseError::Other("query definition missing mandatory 'method' property".to_string())
        })?;
        let endpoint = endpoint.ok_or_else(|| {
            ParseError::Other(
                "query definition missing mandatory 'endpoint' property".to_string(),
            )
        })?;

        // queryParams only valid for POST.
        if query_params.is_some() && method == HttpMethod::Get {
            return Err(ParseError::Other(
                "'queryParams' is only valid for POST queries, not GET".to_string(),
            ));
        }

        Ok(Stmt::QueryDef(QueryDefinition {
            name,
            params,
            method,
            endpoint,
            credentials,
            timeout_ms,
            query_params,
        }))
    }

    /// Parse `attempt --> body <-- catch <ident> --> handler <--`.
    fn parse_attempt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::KwAttempt)?;
        self.expect(&Token::Arrow)?;

        // Parse attempt body until `<--`
        let mut body = Vec::new();
        loop {
            match self.peek() {
                Some(Token::ArrowEnd) => break,
                Some(_) => body.push(self.parse_stmt()?),
                None => return Err(ParseError::UnexpectedEof),
            }
        }
        self.expect(&Token::ArrowEnd)?;

        // `catch` is MANDATORY.
        if !self.check(&Token::Catch) {
            return Err(ParseError::Other(
                "attempt block requires a 'catch' handler".to_string(),
            ));
        }
        self.advance(); // consume `catch`
        let error_name = self.expect_ident()?;
        self.expect(&Token::Arrow)?;

        let mut handler = Vec::new();
        loop {
            match self.peek() {
                Some(Token::ArrowEnd) => break,
                Some(_) => handler.push(self.parse_stmt()?),
                None => return Err(ParseError::UnexpectedEof),
            }
        }
        self.expect(&Token::ArrowEnd)?;

        Ok(Stmt::Attempt(AttemptBlock {
            body,
            error_name,
            handler,
        }))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    fn parse_ok(src: &str) -> Vec<Stmt> {
        parse(src).unwrap_or_else(|e| panic!("parse error: {e}\nsource: {src}"))
    }

    #[test]
    fn var_decl_no_init() {
        let stmts = parse_ok("define x as integer.");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Define(d) => {
                assert_eq!(d.name, "x");
                assert_eq!(d.typ, RundellType::Integer);
                assert!(d.init.is_none());
            }
            _ => panic!("expected Define"),
        }
    }

    #[test]
    fn var_decl_with_init() {
        let stmts = parse_ok("define x as integer = 42.");
        match &stmts[0] {
            Stmt::Define(d) => assert!(d.init.is_some()),
            _ => panic!(),
        }
    }

    #[test]
    fn var_decl_constant_global() {
        let stmts = parse_ok("define x as constant global integer = 5.");
        match &stmts[0] {
            Stmt::Define(d) => {
                assert!(d.constant);
                assert!(d.global);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn assign_eq() {
        let stmts = parse_ok("set x = 10.");
        match &stmts[0] {
            Stmt::Set(s) => {
                assert_eq!(s.target, SetTarget::Identifier("x".to_string()));
                assert!(matches!(s.op, SetOp::Assign(_)));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn assign_increment() {
        let stmts = parse_ok("set x++.");
        match &stmts[0] {
            Stmt::Set(s) => assert_eq!(s.op, SetOp::Increment),
            _ => panic!(),
        }
    }

    #[test]
    fn assign_decrement() {
        let stmts = parse_ok("set x--.");
        match &stmts[0] {
            Stmt::Set(s) => assert_eq!(s.op, SetOp::Decrement),
            _ => panic!(),
        }
    }

    #[test]
    fn if_else_chain() {
        let src = r#"
            if (x > 0) -->
                print "pos".
            else if (x < 0) -->
                print "neg".
            else -->
                print "zero".
            <--
        "#;
        let stmts = parse_ok(src);
        match &stmts[0] {
            Stmt::If(i) => {
                assert_eq!(i.else_ifs.len(), 1);
                assert!(i.else_body.is_some());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn switch_grouped_cases() {
        let src = r#"
            switch age -->
                18 :
                19 : print "young adult".
                else : print "other".
            <--
        "#;
        let stmts = parse_ok(src);
        match &stmts[0] {
            Stmt::Switch(s) => {
                assert_eq!(s.cases.len(), 3);
                // First case has empty body (grouped)
                assert!(s.cases[0].body.is_empty());
                // Second case has a body
                assert!(!s.cases[1].body.is_empty());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn for_loop() {
        let src = "for i loops (0, 10, 1) --> print string(i). <--";
        let stmts = parse_ok(src);
        assert!(matches!(stmts[0], Stmt::ForLoop(_)));
    }

    #[test]
    fn while_loop() {
        let src = "while x < 10 --> set x++. <--";
        let stmts = parse_ok(src);
        assert!(matches!(stmts[0], Stmt::WhileLoop(_)));
    }

    #[test]
    fn for_each() {
        let src = "for each item in col --> print item[\"name\"]. <--";
        let stmts = parse_ok(src);
        assert!(matches!(stmts[0], Stmt::ForEach(_)));
    }

    #[test]
    fn function_decl() {
        let src = r#"
            define add(a as integer, b as integer) returns integer -->
                return a + b.
            <--
        "#;
        let stmts = parse_ok(src);
        match &stmts[0] {
            Stmt::FunctionDef(f) => {
                assert_eq!(f.name, "add");
                assert_eq!(f.params.len(), 2);
                assert_eq!(f.return_type, Some(RundellType::Integer));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn function_call_stmt() {
        let stmts = parse_ok("greet(\"World\").");
        assert!(matches!(stmts[0], Stmt::ExprStmt(Expr::Call(_, _))));
    }

    #[test]
    fn try_catch_finally() {
        let src = r#"
            try -->
                print "ok".
            catch (TypeError) -->
                print "type error".
            finally -->
                print "done".
            <--
        "#;
        let stmts = parse_ok(src);
        match &stmts[0] {
            Stmt::TryCatch(t) => {
                assert_eq!(t.catches.len(), 1);
                assert!(t.finally_body.is_some());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn import_stmt() {
        let stmts = parse_ok("import \"mathUtils\".");
        match &stmts[0] {
            Stmt::Import(p) => assert_eq!(p, "mathUtils"),
            _ => panic!(),
        }
    }

    #[test]
    fn json_literal() {
        let src = r#"define d as json = {"key": [1, 2]}."#;
        let stmts = parse_ok(src);
        match &stmts[0] {
            Stmt::Define(d) => assert!(d.init.is_some()),
            _ => panic!(),
        }
    }

    #[test]
    fn operator_precedence() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        let stmts = parse_ok("print 2 + 3 * 4.");
        match &stmts[0] {
            Stmt::Print(expr) => match expr {
                Expr::BinaryOp(_, BinOp::Add, right) => match right.as_ref() {
                    Expr::BinaryOp(_, BinOp::Mul, _) => {}
                    _ => panic!("expected Mul on right"),
                },
                _ => panic!("expected Add at top"),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn is_null() {
        let stmts = parse_ok("print x is null.");
        match &stmts[0] {
            Stmt::Print(Expr::IsNull(_)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn is_not_null() {
        let stmts = parse_ok("print x is not null.");
        match &stmts[0] {
            Stmt::Print(Expr::IsNotNull(_)) => {}
            _ => panic!(),
        }
    }

    // -----------------------------------------------------------------------
    // GUI parsing tests (Phase 7)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_form_definition() {
        // Use r##"..."## so that "#A2A2A2" inside does not close the raw string.
        let src = r##"
define myForm as form -->
    set form\backgroundColor = "#A2A2A2".
    define myLabel as form\label.
    define myButton as form\button.
    set myLabel\position = 10px, 10px, 200px, 30px.
    set myLabel\value = "Hello World".
    set myButton\caption = "Click Me".
    set myButton\position = 10px, 50px, 100px, 30px.
    set myButton\click = handleClick().
<--
"##;
        let stmts = parse_ok(src);
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::FormDef(f) => {
                assert_eq!(f.name, "myForm");
                assert_eq!(f.body.len(), 8);
            }
            _ => panic!("expected FormDef, got {:?}", stmts[0]),
        }
    }

    #[test]
    fn parse_object_path_set() {
        let stmts = parse_ok(r#"set myForm\myLabel\value = "Updated"."#);
        match &stmts[0] {
            Stmt::Set(s) => {
                assert_eq!(
                    s.target,
                    SetTarget::ObjectPath(vec![
                        "myForm".to_string(),
                        "myLabel".to_string(),
                        "value".to_string(),
                    ])
                );
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_position_literal() {
        let stmts = parse_ok("set myLabel\\position = 10px, 20px, 200px, 30px.");
        match &stmts[0] {
            Stmt::Set(s) => match &s.op {
                SetOp::Assign(Expr::PositionLiteral(10, 20, 200, 30)) => {}
                other => panic!("expected PositionLiteral(10,20,200,30), got {other:?}"),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn parse_show_modeless() {
        let stmts = parse_ok("rootWindow\\myForm\\show().");
        match &stmts[0] {
            Stmt::ExprStmt(Expr::ShowForm { path, modal }) => {
                assert_eq!(path, &["rootWindow".to_string(), "myForm".to_string()]);
                assert!(!modal);
            }
            _ => panic!("expected ExprStmt(ShowForm), got {:?}", stmts[0]),
        }
    }

    #[test]
    fn parse_show_modal() {
        let stmts = parse_ok("rootWindow\\myForm\\show(modal).");
        match &stmts[0] {
            Stmt::ExprStmt(Expr::ShowForm { path: _, modal }) => {
                assert!(*modal);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_close_call() {
        let stmts = parse_ok("rootWindow\\myForm\\close().");
        match &stmts[0] {
            Stmt::ExprStmt(Expr::CloseForm { path }) => {
                assert_eq!(path, &["rootWindow".to_string(), "myForm".to_string()]);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_dialog_openfile() {
        let stmts = parse_ok("set p = dialog\\openfile(\"Open\", \"*.run\").");
        match &stmts[0] {
            Stmt::Set(s) => match &s.op {
                SetOp::Assign(Expr::Dialog(d)) => match d.as_ref() {
                    DialogCall::OpenFile { .. } => {}
                    other => panic!("expected OpenFile, got {other:?}"),
                },
                other => panic!("expected Dialog, got {other:?}"),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn parse_dialog_message() {
        let stmts = parse_ok("set ans = dialog\\message(\"Q\", \"Continue?\", yesno).");
        match &stmts[0] {
            Stmt::Set(s) => match &s.op {
                SetOp::Assign(Expr::Dialog(d)) => match d.as_ref() {
                    DialogCall::Message { kind: MessageKind::YesNo, .. } => {}
                    other => panic!("{other:?}"),
                },
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn backslash_in_string_is_literal() {
        // Regression: backslash inside a string must NOT be tokenised as
        // BackslashSep — the string must survive parsing intact.
        let stmts = parse_ok(r#"define p as string = "C:\Users\Simon\Documents"."#);
        match &stmts[0] {
            Stmt::Define(d) => match d.init.as_ref().unwrap() {
                Expr::Literal(Literal::Str(s)) => {
                    // The backslashes are preserved literally via unknown-escape pass-through.
                    assert_eq!(s, "C:\\Users\\Simon\\Documents");
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn parse_object_path_expr_in_print() {
        let stmts = parse_ok("print myForm\\myLabel\\value.");
        match &stmts[0] {
            Stmt::Print(Expr::ObjectPath(segs)) => {
                assert_eq!(
                    segs,
                    &["myForm".to_string(), "myLabel".to_string(), "value".to_string()]
                );
            }
            _ => panic!("expected Print(ObjectPath), got {:?}", stmts[0]),
        }
    }
}
