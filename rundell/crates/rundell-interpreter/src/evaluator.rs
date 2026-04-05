//! Tree-walk evaluator for the Rundell language.
//!
//! The [`Interpreter`] struct walks the AST produced by `rundell-parser`
//! and executes each node, maintaining an [`Environment`] for variable
//! storage.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, SecondsFormat, TimeZone, Duration as ChronoDuration, Datelike, Timelike, Utc};

use rundell_parser::ast::{
    BinOp, CmpOp, DefineStmt, DialogCall, EventTimerDefinition, Expr, ForEachStmt, ForLoopStmt,
    FormDefinition, FunctionDefStmt, IfStmt, Literal, ReceiveStmt, RundellType,
    SetOp, SetStmt, SetTarget, Stmt, SwitchPattern, TryCatchStmt, UnaryOp, WhileLoopStmt,
};
use rundell_parser::parse;

use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::form_registry::{default_control_state, FormInstance, RundellWindow};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Timeout in milliseconds for blocking on a modal form.
pub const MODAL_TIMEOUT_MS: u64 = 30_000;

/// Default timeout for all query calls, in milliseconds.
/// Override per-query using: set myQuery\timeout = <ms>.
pub const QUERY_TIMEOUT_MS: u64 = 10_000;

// ---------------------------------------------------------------------------
// Credentials and query registry
// ---------------------------------------------------------------------------

/// A resolved credentials instance — plaintext values in memory for program lifetime.
#[derive(Debug, Clone)]
pub struct CredentialsInstance {
    /// JWT bearer token (from env()).
    pub token: Option<String>,
    /// X-Rundell-Auth header value (from env()).
    pub authentication: Option<String>,
}

/// Registry of all query definitions seen during program execution.
#[derive(Debug, Default)]
pub struct QueryRegistry {
    pub queries: HashMap<String, rundell_parser::ast::QueryDefinition>,
}

/// Registry of all resolved credentials instances.
#[derive(Debug, Default)]
pub struct CredentialsRegistry {
    pub credentials: HashMap<String, CredentialsInstance>,
}

// ---------------------------------------------------------------------------
// Event timers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct EventTimer {
    interval_ms: u64,
    running: bool,
    on_event: Option<String>,
    next_fire: Option<Instant>,
    in_callback: bool,
}

impl Default for EventTimer {
    fn default() -> Self {
        EventTimer {
            interval_ms: 0,
            running: false,
            on_event: None,
            next_fire: None,
            in_callback: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Value type
// ---------------------------------------------------------------------------

/// A runtime value in the Rundell interpreter.
#[derive(Debug, Clone)]
pub enum Value {
    /// 64-bit signed integer.
    Integer(i64),
    /// 64-bit IEEE 754 double.
    Float(f64),
    /// UTF-8 string.
    Str(String),
    /// Fixed-point currency stored as integer cents.
    Currency(i64),
    /// Boolean.
    Boolean(bool),
    /// JSON collection.
    Json(serde_json::Value),
    /// ISO 8601 datetime with timezone offset.
    DateTime(DateTime<FixedOffset>),
    /// The null / uninitialised value.
    Null,
}

impl Value {
    /// Return a static string naming the type of this value.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Str(_) => "string",
            Value::Currency(_) => "currency",
            Value::Boolean(_) => "boolean",
            Value::Json(_) => "json",
            Value::DateTime(_) => "datetime",
            Value::Null => "null",
        }
    }

    /// Convert the value to its human-readable string representation.
    ///
    /// - Currency: always exactly 2 decimal places.
    /// - Boolean: lowercase `"true"` / `"false"`.
    /// - Float: Rust default Display (no trailing zeros removed; no forced dp).
    pub fn to_display_string(&self) -> String {
        match self {
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => {
                // Use Rust's default float display but ensure at least one
                // decimal place so that 42.0 does not display as "42".
                let s = format!("{f}");
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    s
                } else {
                    format!("{s}.0")
                }
            }
            Value::Str(s) => s.clone(),
            Value::Currency(c) => {
                let whole = c / 100;
                let frac = (c % 100).unsigned_abs();
                format!("{whole}.{frac:02}")
            }
            Value::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            Value::Json(v) => v.to_string(),
            Value::DateTime(dt) => dt.to_rfc3339_opts(SecondsFormat::Secs, true),
            Value::Null => "null".to_string(),
        }
    }

    /// Evaluate the truthiness of a value.
    ///
    /// - `Null` → false
    /// - Numbers → non-zero is true
    /// - String → non-empty is true
    /// - Boolean → direct
    /// - Json → always true
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Integer(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Boolean(b) => *b,
            Value::Currency(c) => *c != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Json(_) => true,
            Value::DateTime(_) => true,
            Value::Null => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Interpreter
// ---------------------------------------------------------------------------

/// The Rundell tree-walk interpreter.
///
/// Holds the environment (symbol table), registered functions, and
/// metadata needed for import path resolution.
pub struct Interpreter {
    /// Variable environment (scoped symbol table).
    pub(crate) env: Environment,
    /// Map of declared functions by name.
    pub(crate) functions: HashMap<String, FunctionDefStmt>,
    /// Directory of the current source file (for import resolution).
    pub(crate) source_dir: PathBuf,
    /// Set of source file paths already being imported (cycle detection).
    pub(crate) import_stack: Vec<PathBuf>,
    /// Stdout writer (allows substitution in tests).
    stdout: Box<dyn Write>,
    /// Stdin reader (allows substitution in tests).
    stdin: Box<dyn BufRead>,
    /// The global rootWindow — the root of all form access.
    pub root_window: RundellWindow,
    /// Sender for GUI commands (None when running headless / in tests).
    pub gui_tx: Option<std::sync::mpsc::SyncSender<crate::gui_channel::GuiCommand>>,
    /// Receiver for GUI responses (None when running headless / in tests).
    pub gui_rx: Option<std::sync::mpsc::Receiver<crate::gui_channel::GuiResponse>>,
    /// Name of the form currently being defined (used inside exec_form_stmt).
    pub(crate) current_form_name: Option<String>,
    /// Path to the running .run source file (needed to locate .rundell.env).
    pub program_path: Option<PathBuf>,
    /// Monotonic dialog request counter.
    dialog_seq: u64,
    /// Tokio async runtime for executing HTTP requests.
    pub rt: tokio::runtime::Runtime,
    /// Registry of query definitions.
    pub query_registry: QueryRegistry,
    /// Registry of resolved credentials instances.
    pub credentials_registry: CredentialsRegistry,
    /// Registry of named event timers.
    event_timers: HashMap<String, EventTimer>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Create a new interpreter with fresh state.
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
            functions: HashMap::new(),
            source_dir: PathBuf::new(),
            import_stack: Vec::new(),
            stdout: Box::new(io::stdout()),
            stdin: Box::new(io::BufReader::new(io::stdin())),
            root_window: RundellWindow::default(),
            gui_tx: None,
            gui_rx: None,
            current_form_name: None,
            program_path: None,
            dialog_seq: 0,
            rt: tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"),
            query_registry: QueryRegistry::default(),
            credentials_registry: CredentialsRegistry::default(),
            event_timers: HashMap::new(),
        }
    }

    /// Create an interpreter that writes to a provided `Write` (for testing).
    pub fn new_with_output(out: Box<dyn Write>) -> Self {
        Interpreter {
            env: Environment::new(),
            functions: HashMap::new(),
            source_dir: PathBuf::new(),
            import_stack: Vec::new(),
            stdout: out,
            stdin: Box::new(io::BufReader::new(io::stdin())),
            root_window: RundellWindow::default(),
            gui_tx: None,
            gui_rx: None,
            current_form_name: None,
            program_path: None,
            dialog_seq: 0,
            rt: tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"),
            query_registry: QueryRegistry::default(),
            credentials_registry: CredentialsRegistry::default(),
            event_timers: HashMap::new(),
        }
    }

    /// Set the directory used to resolve imports (should be the directory of
    /// the source file being executed).
    pub fn set_source_dir(&mut self, dir: PathBuf) {
        self.source_dir = dir;
    }

    /// Sets the path to the running source file.
    /// Used to locate the adjacent .rundell.env credential store.
    pub fn set_program_path(&mut self, path: PathBuf) {
        self.program_path = Some(path);
    }

    /// Execute a program (list of statements).
    pub fn run(&mut self, stmts: Vec<Stmt>) -> Result<(), RuntimeError> {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        self.wait_for_open_forms()?;
        self.wait_for_timers()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Statement execution
    // -----------------------------------------------------------------------

    /// Execute a single statement.
    fn exec_stmt(&mut self, stmt: Stmt) -> Result<(), RuntimeError> {
        let result = match stmt {
            Stmt::Import(path) => self.exec_import(&path),
            Stmt::Define(d) => self.exec_define(d),
            Stmt::Set(s) => self.exec_set(s),
            Stmt::Print(expr) => self.exec_print(expr),
            Stmt::Receive(r) => self.exec_receive(r),
            Stmt::If(i) => self.exec_if(i),
            Stmt::Switch(sw) => self.exec_switch(sw),
            Stmt::ForLoop(fl) => self.exec_for(fl),
            Stmt::WhileLoop(wl) => self.exec_while(wl),
            Stmt::ForEach(fe) => self.exec_foreach(fe),
            Stmt::FunctionDef(fd) => {
                // Register the function; don't execute the body yet.
                self.functions.insert(fd.name.clone(), fd);
                Ok(())
            }
            Stmt::Return(expr) => {
                let val = match expr {
                    Some(e) => Some(self.eval_expr(e)?),
                    None => None,
                };
                Err(RuntimeError::ReturnValue(val))
            }
            Stmt::TryCatch(tc) => self.exec_try(tc),
            Stmt::Remove(expr) => self.exec_remove(expr),
            Stmt::Append(col, val) => {
                self.exec_append(col, val)?;
                Ok(())
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr)?;
                Ok(())
            }
            Stmt::FormDef(fd) => self.exec_form_def(fd),
            Stmt::EventTimerDef(def) => self.exec_eventtimer_def(def),
            Stmt::DefineControl(name, _ctrl_type) => {
                // DefineControl outside a form body — log a warning and continue.
                eprintln!("[WARN] DefineControl '{}' outside form body — ignored", name);
                Ok(())
            }
            Stmt::CredentialsDef(def) => {
                let token = if let Some(expr) = def.token {
                    Some(match self.eval_expr(expr)? {
                        Value::Str(s) => s,
                        other => other.to_display_string(),
                    })
                } else {
                    None
                };
                let authentication = if let Some(expr) = def.authentication {
                    Some(match self.eval_expr(expr)? {
                        Value::Str(s) => s,
                        other => other.to_display_string(),
                    })
                } else {
                    None
                };
                self.credentials_registry.credentials.insert(
                    def.name.clone(),
                    CredentialsInstance { token, authentication },
                );
                Ok(())
            }
            Stmt::QueryDef(def) => {
                self.query_registry.queries.insert(def.name.clone(), def);
                Ok(())
            }
            Stmt::Attempt(block) => self.exec_attempt(block),
        };

        if result.is_ok() {
            self.pump_gui_events()?;
        }
        result
    }

    // Import a module.
    fn exec_import(&mut self, path: &str) -> Result<(), RuntimeError> {
        let mut full_path = self.source_dir.clone();
        full_path.push(format!("{path}.run"));
        let canonical = full_path
            .canonicalize()
            .map_err(|e| RuntimeError::IOError(format!("cannot resolve import '{path}': {e}")))?;
        // Cycle detection
        if self.import_stack.contains(&canonical) {
            return Err(RuntimeError::RuntimeError(format!(
                "Circular import detected: {path}"
            )));
        }
        let source = std::fs::read_to_string(&canonical)
            .map_err(|e| RuntimeError::IOError(format!("cannot read import '{path}': {e}")))?;
        let stmts = parse(&source).map_err(|e| {
            RuntimeError::RuntimeError(format!("parse error in import '{path}': {e}"))
        })?;

        self.import_stack.push(canonical.clone());
        let old_dir = self.source_dir.clone();
        self.source_dir = canonical.parent().unwrap_or(Path::new(".")).to_path_buf();

        // Only execute globals and function definitions from the import.
        for stmt in stmts {
            match stmt {
                Stmt::Define(ref d) if d.global => {
                    self.exec_define_global(d.clone())?;
                }
                Stmt::FunctionDef(fd) => {
                    self.functions.insert(fd.name.clone(), fd);
                }
                _ => {}
            }
        }

        self.source_dir = old_dir;
        self.import_stack.pop();
        Ok(())
    }

    /// Execute a global variable declaration (for imports).
    fn exec_define_global(&mut self, d: DefineStmt) -> Result<(), RuntimeError> {
        let val = match d.init {
            Some(e) => {
                let v = self.eval_expr(e)?;
                coerce_to_declared_type(v, &d.typ)
            }
            None => Value::Null,
        };
        self.env.define_global(&d.name, val, d.typ, d.constant)
    }

    fn exec_define(&mut self, d: DefineStmt) -> Result<(), RuntimeError> {
        if d.global {
            return self.exec_define_global(d);
        }
        let val = match d.init {
            Some(e) => {
                let v = self.eval_expr(e)?;
                coerce_to_declared_type(v, &d.typ)
            }
            None => Value::Null,
        };
        self.env.define(&d.name, val, d.typ, d.constant)
    }

    fn exec_set(&mut self, s: SetStmt) -> Result<(), RuntimeError> {
        match s.target {
            SetTarget::Identifier(name) => match s.op {
                SetOp::Assign(expr) => {
                    let val = self.eval_expr(expr)?;
                    self.env.set(&name, val)
                }
                SetOp::Increment => {
                    let cur = self.env.get(&name)?.clone();
                    match cur {
                        Value::Integer(n) => self.env.set(&name, Value::Integer(n + 1)),
                        _ => Err(RuntimeError::TypeError(
                            "++ only valid on integer variables".to_string(),
                        )),
                    }
                }
                SetOp::Decrement => {
                    let cur = self.env.get(&name)?.clone();
                    match cur {
                        Value::Integer(n) => self.env.set(&name, Value::Integer(n - 1)),
                        _ => Err(RuntimeError::TypeError(
                            "-- only valid on integer variables".to_string(),
                        )),
                    }
                }
            },
            SetTarget::Index(col_expr, key_expr) => {
                // Evaluate new value first
                let new_val = match s.op {
                    SetOp::Assign(expr) => self.eval_expr(expr)?,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "++ / -- not valid on collection elements".to_string(),
                        ))
                    }
                };
                // col_expr should resolve to an Identifier ultimately
                self.set_index(*col_expr, *key_expr, new_val)
            }
            SetTarget::ObjectPath(path) => self.exec_set_object_path(path, s.op),
        }
    }

    /// Recursively set a value into a nested JSON collection.
    fn set_index(
        &mut self,
        col_expr: Expr,
        key_expr: Expr,
        new_val: Value,
    ) -> Result<(), RuntimeError> {
        let key = self.eval_expr(key_expr)?;

        // Find the root identifier for mutation
        let root_name = Self::find_root_ident(&col_expr)?;

        // Clone the current JSON value
        let mut root_json = match self.env.get(&root_name)?.clone() {
            Value::Json(j) => j,
            _ => {
                return Err(RuntimeError::TypeError(
                    "index assignment target is not a json collection".to_string(),
                ))
            }
        };

        // Build a path of keys from the expression
        let mut path = Vec::new();
        Self::collect_index_path(&col_expr, &mut path);

        // Navigate to the parent and insert
        let json_val = value_to_json(new_val)?;
        Self::json_set_nested(&mut root_json, &path, key, json_val)?;

        self.env.set(&root_name, Value::Json(root_json))
    }

    fn find_root_ident(expr: &Expr) -> Result<String, RuntimeError> {
        match expr {
            Expr::Identifier(name) => Ok(name.clone()),
            Expr::Index(inner, _) => Self::find_root_ident(inner),
            _ => Err(RuntimeError::TypeError(
                "set target must be an identifier or index expression".to_string(),
            )),
        }
    }

    fn collect_index_path(expr: &Expr, path: &mut Vec<Expr>) {
        if let Expr::Index(inner, key) = expr {
            Self::collect_index_path(inner, path);
            path.push(*key.clone());
        }
    }

    fn json_set_nested(
        json: &mut serde_json::Value,
        path: &[Expr],
        final_key: Value,
        new_val: serde_json::Value,
    ) -> Result<(), RuntimeError> {
        // We need to evaluate the path keys — but this fn doesn't have &mut self.
        // We pre-evaluate them in exec_set; here the path elements are already
        // literals, or we fall back to treating Expr::Literal.
        // For simplicity, navigate with already-evaluated keys.
        let full_path: Vec<Value> = {
            let mut p = Vec::new();
            for e in path {
                p.push(Self::eval_expr_simple(e)?);
            }
            p.push(final_key);
            p
        };
        Self::json_navigate_and_set(json, &full_path, new_val)
    }

    /// Evaluate simple (non-identifier, non-call) expressions for path navigation.
    fn eval_expr_simple(expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => literal_to_value(lit.clone()),
            _ => Err(RuntimeError::RuntimeError(
                "complex index paths not supported in set target".to_string(),
            )),
        }
    }

    fn json_navigate_and_set(
        json: &mut serde_json::Value,
        keys: &[Value],
        new_val: serde_json::Value,
    ) -> Result<(), RuntimeError> {
        if keys.is_empty() {
            return Err(RuntimeError::RuntimeError("empty index path".to_string()));
        }
        if keys.len() == 1 {
            return Self::json_set_one(json, &keys[0], new_val);
        }
        let next = Self::json_get_mut(json, &keys[0])?;
        Self::json_navigate_and_set(next, &keys[1..], new_val)
    }

    fn json_set_one(
        json: &mut serde_json::Value,
        key: &Value,
        new_val: serde_json::Value,
    ) -> Result<(), RuntimeError> {
        match (json, key) {
            (serde_json::Value::Object(map), Value::Str(k)) => {
                map.insert(k.clone(), new_val);
                Ok(())
            }
            (serde_json::Value::Array(arr), Value::Integer(i)) => {
                let idx = *i as usize;
                if idx < arr.len() {
                    arr[idx] = new_val;
                    Ok(())
                } else {
                    Err(RuntimeError::IndexError(format!("index {i} out of bounds")))
                }
            }
            _ => Err(RuntimeError::TypeError(
                "incompatible collection/key types".to_string(),
            )),
        }
    }

    fn json_get_mut<'a>(
        json: &'a mut serde_json::Value,
        key: &Value,
    ) -> Result<&'a mut serde_json::Value, RuntimeError> {
        match (json, key) {
            (serde_json::Value::Object(map), Value::Str(k)) => map
                .get_mut(k.as_str())
                .ok_or_else(|| RuntimeError::IndexError(format!("key '{k}' not found"))),
            (serde_json::Value::Array(arr), Value::Integer(i)) => {
                let idx = *i as usize;
                arr.get_mut(idx)
                    .ok_or_else(|| RuntimeError::IndexError(format!("index {i} out of bounds")))
            }
            _ => Err(RuntimeError::TypeError(
                "incompatible collection/key types".to_string(),
            )),
        }
    }

    fn exec_print(&mut self, expr: Expr) -> Result<(), RuntimeError> {
        let val = self.eval_expr(expr)?;
        let s = match &val {
            Value::Json(json) => serde_json::to_string_pretty(json)
                .unwrap_or_else(|_| json.to_string()),
            _ => val.to_display_string(),
        };
        self.stdout
            .write_all(s.as_bytes())
            .map_err(|e| RuntimeError::IOError(e.to_string()))?;
        self.stdout
            .flush()
            .map_err(|e| RuntimeError::IOError(e.to_string()))?;
        Ok(())
    }

    fn exec_receive(&mut self, r: ReceiveStmt) -> Result<(), RuntimeError> {
        // Print prompt if present
        if let Some(prompt_expr) = r.prompt {
            let prompt_val = self.eval_expr(prompt_expr)?;
            let prompt_str = prompt_val.to_display_string();
            self.stdout
                .write_all(prompt_str.as_bytes())
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;
            self.stdout
                .flush()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;
        }
        // Read a line from stdin
        let mut line = String::new();
        self.stdin
            .read_line(&mut line)
            .map_err(|e| RuntimeError::IOError(e.to_string()))?;
        let line = line.trim_end_matches(['\n', '\r']).to_string();

        // Coerce to the variable's declared type
        let binding = self.env.get_binding(&r.variable)?.clone();
        let val = coerce_string_to_type(&line, &binding.declared_type)?;
        self.env.set(&r.variable, val)
    }

    fn exec_if(&mut self, i: IfStmt) -> Result<(), RuntimeError> {
        let cond = self.eval_expr(i.condition)?;
        if cond.is_truthy() {
            self.env.push_scope();
            let res = self.run_body(i.then_body);
            self.env.pop_scope();
            return res;
        }
        for (else_cond, else_body) in i.else_ifs {
            let c = self.eval_expr(else_cond)?;
            if c.is_truthy() {
                self.env.push_scope();
                let res = self.run_body(else_body);
                self.env.pop_scope();
                return res;
            }
        }
        if let Some(else_body) = i.else_body {
            self.env.push_scope();
            let res = self.run_body(else_body);
            self.env.pop_scope();
            return res;
        }
        Ok(())
    }

    fn exec_switch(&mut self, sw: rundell_parser::ast::SwitchStmt) -> Result<(), RuntimeError> {
        let subject = self.eval_expr(sw.subject)?;

        // Collect the indices of cases that have non-empty bodies.
        // For grouped cases (empty body) we fall through to the next non-empty body.
        let n = sw.cases.len();
        let mut i = 0;
        while i < n {
            let case = &sw.cases[i];
            let matches = self.switch_case_matches(&subject, &case.pattern)?;
            if matches {
                // Find the first case in the group with a non-empty body.
                let mut j = i;
                while j < n && sw.cases[j].body.is_empty() {
                    j += 1;
                }
                if j < n {
                    let body = sw.cases[j].body.clone();
                    self.env.push_scope();
                    let res = self.run_body(body);
                    self.env.pop_scope();
                    return res;
                }
                return Ok(());
            }
            i += 1;
        }
        Ok(())
    }

    /// Test whether `subject` matches a switch case pattern.
    fn switch_case_matches(
        &mut self,
        subject: &Value,
        pattern: &SwitchPattern,
    ) -> Result<bool, RuntimeError> {
        match pattern {
            SwitchPattern::Default => Ok(true),
            SwitchPattern::Exact(expr) => {
                let val = self.eval_expr(expr.clone())?;
                Ok(values_equal(subject, &val))
            }
            SwitchPattern::Comparison(op, expr) => {
                let val = self.eval_expr(expr.clone())?;
                compare_values(subject, op, &val)
            }
        }
    }

    fn exec_for(&mut self, fl: ForLoopStmt) -> Result<(), RuntimeError> {
        let start = match self.eval_expr(fl.start)? {
            Value::Integer(n) => n,
            v => {
                return Err(RuntimeError::TypeError(format!(
                    "for loop start must be integer, got {}",
                    v.type_name()
                )))
            }
        };
        let end = match self.eval_expr(fl.end)? {
            Value::Integer(n) => n,
            v => {
                return Err(RuntimeError::TypeError(format!(
                    "for loop end must be integer, got {}",
                    v.type_name()
                )))
            }
        };
        let step = match self.eval_expr(fl.increment)? {
            Value::Integer(n) => n,
            v => {
                return Err(RuntimeError::TypeError(format!(
                    "for loop increment must be integer, got {}",
                    v.type_name()
                )))
            }
        };

        // The loop is INCLUSIVE of end.
        let mut cur = start;
        loop {
            if step > 0 && cur > end {
                break;
            }
            if step < 0 && cur < end {
                break;
            }
            if step == 0 {
                break;
            }
            self.env.set(&fl.var, Value::Integer(cur))?;
            self.env.push_scope();
            let res = self.run_body(fl.body.clone());
            self.env.pop_scope();
            res?;
            cur += step;
        }
        Ok(())
    }

    fn exec_while(&mut self, wl: WhileLoopStmt) -> Result<(), RuntimeError> {
        loop {
            let cond = self.eval_expr(wl.condition.clone())?;
            if !cond.is_truthy() {
                break;
            }
            self.env.push_scope();
            let res = self.run_body(wl.body.clone());
            self.env.pop_scope();
            res?;
        }
        Ok(())
    }

    fn exec_foreach(&mut self, fe: ForEachStmt) -> Result<(), RuntimeError> {
        let col = self.eval_expr(fe.collection)?;
        let arr = match col {
            Value::Json(serde_json::Value::Array(arr)) => arr,
            _ => {
                return Err(RuntimeError::TypeError(
                    "for each requires a json array".to_string(),
                ))
            }
        };
        for item in arr {
            self.env.push_scope();
            // The iteration variable is implicitly declared as json.
            self.env
                .define(&fe.var, Value::Json(item), RundellType::Json, false)?;
            let res = self.run_body(fe.body.clone());
            self.env.pop_scope();
            res?;
        }
        Ok(())
    }

    fn exec_try(&mut self, tc: TryCatchStmt) -> Result<(), RuntimeError> {
        let try_result = {
            self.env.push_scope();
            let r = self.run_body(tc.try_body);
            self.env.pop_scope();
            r
        };

        match try_result {
            Ok(()) => {
                // Run finally if present, then succeed.
                if let Some(fb) = tc.finally_body {
                    self.env.push_scope();
                    let fr = self.run_body(fb);
                    self.env.pop_scope();
                    return fr;
                }
                Ok(())
            }
            Err(err) => {
                // Try to match the error to a catch clause.
                let err_name = runtime_error_name(&err);
                let matched = tc.catches.iter().find(|c| c.error_type == err_name);

                let catch_result = if let Some(catch_clause) = matched {
                    let body = catch_clause.body.clone();
                    self.env.push_scope();
                    let r = self.run_body(body);
                    self.env.pop_scope();
                    Some(r)
                } else {
                    None
                };

                // Always run finally.
                if let Some(fb) = tc.finally_body {
                    self.env.push_scope();
                    let fr = self.run_body(fb);
                    self.env.pop_scope();
                    // If finally itself errors, propagate that.
                    fr?;
                }

                match catch_result {
                    Some(r) => r,
                    None => Err(err), // Re-raise unmatched error.
                }
            }
        }
    }

    fn exec_remove(&mut self, expr: Expr) -> Result<(), RuntimeError> {
        // expr must be an Index expression
        match expr {
            Expr::Index(col_expr, key_expr) => {
                let key = self.eval_expr(*key_expr)?;
                let root_name = Self::find_root_ident(&col_expr)?;

                // Collect the path
                let mut path = Vec::new();
                Self::collect_index_path(&col_expr, &mut path);

                let mut root_json = match self.env.get(&root_name)?.clone() {
                    Value::Json(j) => j,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "remove target is not a json collection".to_string(),
                        ))
                    }
                };

                if path.is_empty() {
                    // Direct remove from root
                    Self::json_remove_key(&mut root_json, &key)?;
                } else {
                    // Navigate to parent
                    let parent = Self::json_navigate_mut_simple(&mut root_json, &path)?;
                    Self::json_remove_key(parent, &key)?;
                }

                self.env.set(&root_name, Value::Json(root_json))
            }
            _ => Err(RuntimeError::TypeError(
                "remove requires an index expression".to_string(),
            )),
        }
    }

    fn json_navigate_mut_simple<'a>(
        json: &'a mut serde_json::Value,
        path: &[Expr],
    ) -> Result<&'a mut serde_json::Value, RuntimeError> {
        if path.is_empty() {
            return Ok(json);
        }
        let key = Self::eval_expr_simple(&path[0])?;
        let next = Self::json_get_mut(json, &key)?;
        Self::json_navigate_mut_simple(next, &path[1..])
    }

    fn json_remove_key(json: &mut serde_json::Value, key: &Value) -> Result<(), RuntimeError> {
        match (json, key) {
            (serde_json::Value::Object(map), Value::Str(k)) => {
                map.remove(k.as_str());
                Ok(())
            }
            (serde_json::Value::Array(arr), Value::Integer(i)) => {
                let idx = *i as usize;
                if idx < arr.len() {
                    arr.remove(idx);
                    Ok(())
                } else {
                    Err(RuntimeError::IndexError(format!(
                        "index {i} out of bounds for remove"
                    )))
                }
            }
            _ => Err(RuntimeError::TypeError(
                "incompatible collection/key for remove".to_string(),
            )),
        }
    }

    fn exec_append(&mut self, col_expr: Expr, val_expr: Expr) -> Result<Value, RuntimeError> {
        let val = self.eval_expr(val_expr)?;
        let json_val = value_to_json(val)?;

        let root_name = Self::find_root_ident(&col_expr)?;
        let mut path = Vec::new();
        Self::collect_index_path(&col_expr, &mut path);

        let mut root_json = match self.env.get(&root_name)?.clone() {
            Value::Json(j) => j,
            _ => {
                return Err(RuntimeError::TypeError(
                    "append target is not a json collection".to_string(),
                ))
            }
        };

        if path.is_empty() {
            // Append directly to root
            match &mut root_json {
                serde_json::Value::Array(arr) => arr.push(json_val),
                _ => {
                    return Err(RuntimeError::TypeError(
                        "append target is not a json array".to_string(),
                    ))
                }
            }
        } else {
            let parent = Self::json_navigate_mut_simple(&mut root_json, &path)?;
            match parent {
                serde_json::Value::Array(arr) => arr.push(json_val),
                _ => {
                    return Err(RuntimeError::TypeError(
                        "append target is not a json array".to_string(),
                    ))
                }
            }
        }

        self.env.set(&root_name, Value::Json(root_json))?;
        Ok(Value::Null)
    }

    /// Run a sequence of statements (used for block bodies).
    fn run_body(&mut self, stmts: Vec<Stmt>) -> Result<(), RuntimeError> {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Expression evaluation
    // -----------------------------------------------------------------------

    /// Evaluate an expression to a value.
    pub fn eval_expr(&mut self, expr: Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => literal_to_value(lit),
            Expr::Identifier(name) => {
                let val = self.env.get(&name)?.clone();
                // Accessing a null variable raises NullError
                if matches!(val, Value::Null) {
                    Err(RuntimeError::NullError(name))
                } else {
                    Ok(val)
                }
            }
            Expr::BinaryOp(left, op, right) => self.eval_binop(*left, op, *right),
            Expr::UnaryOp(op, inner) => self.eval_unary(op, *inner),
            Expr::Index(col, key) => self.eval_index(*col, *key),
            Expr::Call(name, args) => self.eval_call(name, args),
            Expr::IsNull(inner) => {
                // Do NOT raise NullError here — that's the whole point.
                let val = self.eval_expr_nullable(*inner)?;
                Ok(Value::Boolean(matches!(val, Value::Null)))
            }
            Expr::IsNotNull(inner) => {
                let val = self.eval_expr_nullable(*inner)?;
                Ok(Value::Boolean(!matches!(val, Value::Null)))
            }
            Expr::JsonLiteral(v) => Ok(Value::Json(v)),
            Expr::ObjectPath(segs) => self.eval_object_path(segs),
            Expr::PixelValue(n) => Ok(Value::Integer(n as i64)),
            Expr::DurationValue(ms) => Ok(Value::Integer(ms as i64)),
            Expr::PositionLiteral(top, left, width, height) => {
                Ok(Value::Json(serde_json::json!([top, left, width, height])))
            }
            Expr::ShowForm { path, modal } => {
                self.exec_show_form(path, modal)?;
                Ok(Value::Null)
            }
            Expr::CloseForm { path } => {
                self.exec_close_form(path)?;
                Ok(Value::Null)
            }
            Expr::Dialog(call) => self.eval_dialog_call(*call),
            Expr::Await(await_expr) => self.eval_await(*await_expr),
        }
    }

    /// Evaluate an expression without raising NullError for null identifiers.
    fn eval_expr_nullable(&mut self, expr: Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Identifier(name) => {
                // Don't raise NullError — return Null directly
                match self.env.get(&name) {
                    Ok(v) => Ok(v.clone()),
                    Err(_) => Ok(Value::Null),
                }
            }
            other => self.eval_expr(other),
        }
    }

    fn eval_binop(&mut self, left: Expr, op: BinOp, right: Expr) -> Result<Value, RuntimeError> {
        // Short-circuit AND/OR before evaluating right side
        match op {
            BinOp::And => {
                let l = self.eval_expr(left)?;
                if !l.is_truthy() {
                    return Ok(Value::Boolean(false));
                }
                let r = self.eval_expr(right)?;
                return Ok(Value::Boolean(r.is_truthy()));
            }
            BinOp::Or => {
                let l = self.eval_expr(left)?;
                if l.is_truthy() {
                    return Ok(Value::Boolean(true));
                }
                let r = self.eval_expr(right)?;
                return Ok(Value::Boolean(r.is_truthy()));
            }
            _ => {}
        }

        let l = self.eval_expr(left)?;
        let r = self.eval_expr(right)?;

        match op {
            BinOp::Add => eval_add(l, r),
            BinOp::Sub => eval_arith(l, r, ArithOp::Sub),
            BinOp::Mul => eval_arith(l, r, ArithOp::Mul),
            BinOp::Div => eval_div(l, r),
            BinOp::Mod => eval_mod(l, r),
            BinOp::Pow => eval_pow(l, r),
            BinOp::Eq => Ok(Value::Boolean(values_equal(&l, &r))),
            BinOp::NotEq => Ok(Value::Boolean(!values_equal(&l, &r))),
            BinOp::Lt => compare_values(&l, &CmpOp::Lt, &r).map(Value::Boolean),
            BinOp::LtEq => compare_values(&l, &CmpOp::LtEq, &r).map(Value::Boolean),
            BinOp::Gt => compare_values(&l, &CmpOp::Gt, &r).map(Value::Boolean),
            BinOp::GtEq => compare_values(&l, &CmpOp::GtEq, &r).map(Value::Boolean),
            BinOp::StrConcat => match (l, r) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(a + &b)),
                _ => Err(RuntimeError::TypeError(
                    "StrConcat requires string operands".to_string(),
                )),
            },
            BinOp::And | BinOp::Or => unreachable!("handled above"),
        }
    }

    fn eval_unary(&mut self, op: UnaryOp, inner: Expr) -> Result<Value, RuntimeError> {
        let val = self.eval_expr(inner)?;
        match op {
            UnaryOp::Neg => match val {
                Value::Integer(n) => Ok(Value::Integer(-n)),
                Value::Float(f) => Ok(Value::Float(-f)),
                Value::Currency(c) => Ok(Value::Currency(-c)),
                _ => Err(RuntimeError::TypeError(format!(
                    "unary negation not valid on {}",
                    val.type_name()
                ))),
            },
            UnaryOp::Not => Ok(Value::Boolean(!val.is_truthy())),
        }
    }

    fn eval_index(&mut self, col_expr: Expr, key_expr: Expr) -> Result<Value, RuntimeError> {
        let col = self.eval_expr(col_expr)?;
        let key = self.eval_expr(key_expr)?;
        match col {
            Value::Json(j) => json_index(j, key),
            _ => Err(RuntimeError::TypeError(format!(
                "index access requires json, got {}",
                col.type_name()
            ))),
        }
    }

    // -----------------------------------------------------------------------
    // Function / built-in calls
    // -----------------------------------------------------------------------

    fn eval_call(&mut self, name: String, args: Vec<Expr>) -> Result<Value, RuntimeError> {
        // Built-ins take priority over user functions.
        if is_builtin(&name) {
            return self.eval_builtin(&name, args);
        }
        self.eval_user_function(&name, args)
    }

    fn eval_builtin(&mut self, name: &str, args: Vec<Expr>) -> Result<Value, RuntimeError> {
        match name {
            "newline" => Ok(Value::Str("\n".to_string())),

            "length" => {
                let v = self.eval_args(args, 1)?;
                match &v[0] {
                    Value::Str(s) => Ok(Value::Integer(s.chars().count() as i64)),
                    Value::Json(serde_json::Value::Array(arr)) => {
                        Ok(Value::Integer(arr.len() as i64))
                    }
                    Value::Json(serde_json::Value::Object(map)) => {
                        Ok(Value::Integer(map.len() as i64))
                    }
                    _ => Err(RuntimeError::TypeError(
                        "length() requires string or json".to_string(),
                    )),
                }
            }

            "cast" => {
                let mut evaled = Vec::with_capacity(2);
                for a in args {
                    evaled.push(self.eval_expr(a)?);
                }
                if evaled.len() != 2 {
                    return Err(RuntimeError::RuntimeError(
                        "cast() requires 2 arguments".to_string(),
                    ));
                }
                let target_type = match &evaled[1] {
                    Value::Str(s) => s.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "cast() second argument must be a type name".to_string(),
                        ))
                    }
                };
                cast_value(evaled.remove(0), &target_type)
            }

            "abs" => {
                let v = self.eval_args(args, 1)?;
                match v[0].clone() {
                    Value::Integer(n) => Ok(Value::Integer(n.abs())),
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    Value::Currency(c) => Ok(Value::Currency(c.abs())),
                    _ => Err(RuntimeError::TypeError(
                        "abs() requires numeric argument".to_string(),
                    )),
                }
            }

            "floor" => {
                let v = self.eval_args(args, 1)?;
                match v[0].clone() {
                    Value::Float(f) => Ok(Value::Integer(f.floor() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(n)),
                    _ => Err(RuntimeError::TypeError(
                        "floor() requires numeric argument".to_string(),
                    )),
                }
            }

            "ceil" => {
                let v = self.eval_args(args, 1)?;
                match v[0].clone() {
                    Value::Float(f) => Ok(Value::Integer(f.ceil() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(n)),
                    _ => Err(RuntimeError::TypeError(
                        "ceil() requires numeric argument".to_string(),
                    )),
                }
            }

            "round" => {
                let v = self.eval_args(args, 2)?;
                let f = match v[0].clone() {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "round() first argument must be numeric".to_string(),
                        ))
                    }
                };
                let dp = match v[1].clone() {
                    Value::Integer(n) => n,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "round() second argument must be integer".to_string(),
                        ))
                    }
                };
                let factor = 10_f64.powi(dp as i32);
                Ok(Value::Float((f * factor).round() / factor))
            }

            "substr" => {
                let v = self.eval_args(args, 3)?;
                let s = match v[0].clone() {
                    Value::Str(s) => s,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "substr() first argument must be string".to_string(),
                        ))
                    }
                };
                let start = match v[1].clone() {
                    Value::Integer(n) => n as usize,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "substr() start must be integer".to_string(),
                        ))
                    }
                };
                let len = match v[2].clone() {
                    Value::Integer(n) => n as usize,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "substr() length must be integer".to_string(),
                        ))
                    }
                };
                let chars: Vec<char> = s.chars().collect();
                let end = (start + len).min(chars.len());
                let result: String = chars[start.min(chars.len())..end].iter().collect();
                Ok(Value::Str(result))
            }

            "upper" => {
                let v = self.eval_args(args, 1)?;
                match v[0].clone() {
                    Value::Str(s) => Ok(Value::Str(s.to_uppercase())),
                    _ => Err(RuntimeError::TypeError(
                        "upper() requires string".to_string(),
                    )),
                }
            }

            "lower" => {
                let v = self.eval_args(args, 1)?;
                match v[0].clone() {
                    Value::Str(s) => Ok(Value::Str(s.to_lowercase())),
                    _ => Err(RuntimeError::TypeError(
                        "lower() requires string".to_string(),
                    )),
                }
            }

            "trim" => {
                let v = self.eval_args(args, 1)?;
                match v[0].clone() {
                    Value::Str(s) => Ok(Value::Str(s.trim().to_string())),
                    _ => Err(RuntimeError::TypeError(
                        "trim() requires string".to_string(),
                    )),
                }
            }

            "execute" => {
                let v = self.eval_args(args, 1)?;
                let raw = self.expect_string_arg(&v[0], "execute", "path")?;
                let target = self.resolve_execute_path(&raw)?;
                let output = Command::new(&target).output().map_err(|e| {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        RuntimeError::PermissionError { path: raw.clone() }
                    } else {
                        RuntimeError::IOError(format!(
                            "execute() failed for '{}': {e}",
                            target.display()
                        ))
                    }
                })?;

                if !output.stdout.is_empty() {
                    self.stdout
                        .write_all(&output.stdout)
                        .map_err(|e| RuntimeError::IOError(e.to_string()))?;
                }
                if !output.stderr.is_empty() {
                    self.stdout
                        .write_all(&output.stderr)
                        .map_err(|e| RuntimeError::IOError(e.to_string()))?;
                }
                self.stdout
                    .flush()
                    .map_err(|e| RuntimeError::IOError(e.to_string()))?;

                Ok(Value::Null)
            }

            "os" => {
                if !args.is_empty() {
                    return Err(RuntimeError::RuntimeError(
                        "os() takes no arguments".to_string(),
                    ));
                }
                let name = if cfg!(windows) {
                    "windows"
                } else if cfg!(target_os = "macos") {
                    "macos"
                } else if cfg!(target_os = "linux") {
                    "linux"
                } else {
                    "unknown"
                };
                Ok(Value::Str(name.to_string()))
            }

            "now" => {
                if !args.is_empty() {
                    return Err(RuntimeError::RuntimeError(
                        "now() takes no arguments".to_string(),
                    ));
                }
                let now = Local::now();
                let fixed = *now.offset();
                Ok(Value::DateTime(now.with_timezone(&fixed)))
            }

            "day" => {
                let v = self.eval_args(args, 1)?;
                let dt = self.expect_datetime_arg(&v[0], "day", "datetime")?;
                Ok(Value::Integer(dt.day() as i64))
            }

            "month" => {
                let v = self.eval_args(args, 1)?;
                let dt = self.expect_datetime_arg(&v[0], "month", "datetime")?;
                Ok(Value::Integer(dt.month() as i64))
            }

            "year" => {
                let v = self.eval_args(args, 1)?;
                let dt = self.expect_datetime_arg(&v[0], "year", "datetime")?;
                Ok(Value::Integer(dt.year() as i64))
            }

            "hour" => {
                let v = self.eval_args(args, 1)?;
                let dt = self.expect_datetime_arg(&v[0], "hour", "datetime")?;
                Ok(Value::Integer(dt.hour() as i64))
            }

            "minute" => {
                let v = self.eval_args(args, 1)?;
                let dt = self.expect_datetime_arg(&v[0], "minute", "datetime")?;
                Ok(Value::Integer(dt.minute() as i64))
            }

            "second" => {
                let v = self.eval_args(args, 1)?;
                let dt = self.expect_datetime_arg(&v[0], "second", "datetime")?;
                Ok(Value::Integer(dt.second() as i64))
            }

            "dateformat" => {
                let v = self.eval_args(args, 2)?;
                let fmt = self.expect_string_arg(&v[0], "dateformat", "format")?;
                let dt = self.expect_datetime_arg(&v[1], "dateformat", "datetime")?;
                Ok(Value::Str(format_datetime(&dt, &fmt)))
            }

            "timestamp" => {
                let v = self.eval_args(args, 1)?;
                let dt = self.expect_datetime_arg(&v[0], "timestamp", "datetime")?;
                Ok(Value::Integer(dt.timestamp_millis()))
            }

            "fromtimestamp" => {
                let v = self.eval_args(args, 1)?;
                let ms = match v[0] {
                    Value::Integer(n) => n,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "fromtimestamp() requires integer milliseconds".to_string(),
                        ))
                    }
                };
                Ok(Value::DateTime(datetime_from_timestamp_ms(ms)?))
            }

            "string" => {
                let v = self.eval_args(args, 1)?;
                Ok(Value::Str(v[0].to_display_string()))
            }

            "append" => {
                if args.len() != 2 {
                    return Err(RuntimeError::RuntimeError(
                        "append() requires 2 arguments".to_string(),
                    ));
                }
                let mut args = args;
                let col_expr = args.remove(0);
                let val_expr = args.remove(0);
                self.exec_append(col_expr, val_expr)
            }

            "env" => {
                if args.len() != 1 {
                    return Err(RuntimeError::RuntimeError(
                        "env() requires exactly one argument".to_string(),
                    ));
                }
                let key = match self.eval_expr(args.into_iter().next().unwrap())? {
                    Value::Str(s) => s,
                    other => {
                        return Err(RuntimeError::TypeError(format!(
                            "env() argument must be a string, got {}",
                            other.type_name()
                        )))
                    }
                };
                let env_path = match &self.program_path {
                    Some(p) => p
                        .parent()
                        .unwrap_or(std::path::Path::new("."))
                        .join(".rundell.env"),
                    None => return Err(RuntimeError::NoProgramPath),
                };
                rundell_env::env_get(&env_path, &key)
                    .map(Value::Str)
                    .map_err(|e| match e {
                        rundell_env::EnvError::KeyNotFound(_) => {
                            RuntimeError::EnvKeyNotFound { key: key.clone() }
                        }
                        rundell_env::EnvError::DecryptionFailed(_) => {
                            RuntimeError::EnvDecryptionFailed { key: key.clone() }
                        }
                        rundell_env::EnvError::Io(msg) => RuntimeError::IOError(msg),
                    })
            }

            "read_text" => {
                let v = self.eval_args(args, 1)?;
                let path = self.expect_string_arg(&v[0], "read_text", "path")?;
                let path = self.resolve_io_path(&path)?;
                let contents = std::fs::read_to_string(&path)
                    .map_err(|e| RuntimeError::IOError(format!(
                        "read_text() failed for '{}': {e}",
                        path.display()
                    )))?;
                Ok(Value::Str(contents))
            }

            "write_text" => {
                let v = self.eval_args(args, 2)?;
                let path = self.expect_string_arg(&v[0], "write_text", "path")?;
                let contents = self.expect_string_arg(&v[1], "write_text", "content")?;
                let path = self.resolve_io_path(&path)?;
                std::fs::write(&path, contents.as_bytes())
                    .map_err(|e| RuntimeError::IOError(format!(
                        "write_text() failed for '{}': {e}",
                        path.display()
                    )))?;
                Ok(Value::Null)
            }

            "read_json" => {
                let v = self.eval_args(args, 1)?;
                let path = self.expect_string_arg(&v[0], "read_json", "path")?;
                let path = self.resolve_io_path(&path)?;
                let contents = std::fs::read_to_string(&path)
                    .map_err(|e| RuntimeError::IOError(format!(
                        "read_json() failed for '{}': {e}",
                        path.display()
                    )))?;
                let json_val = serde_json::from_str(&contents).map_err(|e| {
                    RuntimeError::RuntimeError(format!(
                        "read_json() failed to parse JSON in '{}': {e}",
                        path.display()
                    ))
                })?;
                Ok(Value::Json(json_val))
            }

            "write_json" => {
                let v = self.eval_args(args, 2)?;
                let path = self.expect_string_arg(&v[0], "write_json", "path")?;
                let json_val = match &v[1] {
                    Value::Json(value) => value.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "write_json() value must be json".to_string(),
                        ))
                    }
                };
                let path = self.resolve_io_path(&path)?;
                let contents = serde_json::to_string_pretty(&json_val).map_err(|e| {
                    RuntimeError::RuntimeError(format!("write_json() failed: {e}"))
                })?;
                std::fs::write(&path, contents.as_bytes())
                    .map_err(|e| RuntimeError::IOError(format!(
                        "write_json() failed for '{}': {e}",
                        path.display()
                    )))?;
                Ok(Value::Null)
            }

            "read_csv" => {
                let v = self.eval_args(args, 2)?;
                let path = self.expect_string_arg(&v[0], "read_csv", "path")?;
                let has_headers = self.expect_boolean_arg(&v[1], "read_csv", "has_headers")?;
                let path = self.resolve_io_path(&path)?;
                let mut reader = csv::ReaderBuilder::new()
                    .has_headers(has_headers)
                    .from_path(&path)
                    .map_err(|e| RuntimeError::IOError(format!(
                        "read_csv() failed for '{}': {e}",
                        path.display()
                    )))?;

                let mut rows = Vec::new();
                if has_headers {
                    let headers = reader
                        .headers()
                        .map_err(|e| RuntimeError::IOError(format!(
                            "read_csv() failed for '{}': {e}",
                            path.display()
                        )))?
                        .clone();
                    for record in reader.records() {
                        let record = record.map_err(|e| RuntimeError::IOError(format!(
                            "read_csv() failed for '{}': {e}",
                            path.display()
                        )))?;
                        let mut obj = serde_json::Map::new();
                        for (idx, header) in headers.iter().enumerate() {
                            let value = record.get(idx).unwrap_or("");
                            obj.insert(
                                header.to_string(),
                                serde_json::Value::String(value.to_string()),
                            );
                        }
                        rows.push(serde_json::Value::Object(obj));
                    }
                } else {
                    for record in reader.records() {
                        let record = record.map_err(|e| RuntimeError::IOError(format!(
                            "read_csv() failed for '{}': {e}",
                            path.display()
                        )))?;
                        let arr = record
                            .iter()
                            .map(|value| serde_json::Value::String(value.to_string()))
                            .collect();
                        rows.push(serde_json::Value::Array(arr));
                    }
                }
                Ok(Value::Json(serde_json::Value::Array(rows)))
            }

            "write_csv" => {
                let v = self.eval_args(args, 3)?;
                let path = self.expect_string_arg(&v[0], "write_csv", "path")?;
                let include_headers =
                    self.expect_boolean_arg(&v[2], "write_csv", "include_headers")?;
                let rows = match &v[1] {
                    Value::Json(serde_json::Value::Array(arr)) => arr.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "write_csv() rows must be a json array".to_string(),
                        ))
                    }
                };
                let path = self.resolve_io_path(&path)?;
                let mut writer = csv::WriterBuilder::new()
                    .has_headers(include_headers)
                    .from_path(&path)
                    .map_err(|e| RuntimeError::IOError(format!(
                        "write_csv() failed for '{}': {e}",
                        path.display()
                    )))?;

                if include_headers {
                    let first = rows.first().ok_or_else(|| RuntimeError::TypeError(
                        "write_csv() rows must contain at least one object when include_headers is true"
                            .to_string(),
                    ))?;
                    let first_obj = match first {
                        serde_json::Value::Object(obj) => obj,
                        _ => {
                            return Err(RuntimeError::TypeError(
                                "write_csv() rows must be objects when include_headers is true"
                                    .to_string(),
                            ))
                        }
                    };
                    let headers: Vec<String> = first_obj.keys().cloned().collect();
                    writer
                        .write_record(headers.iter())
                        .map_err(|e| RuntimeError::IOError(format!(
                            "write_csv() failed for '{}': {e}",
                            path.display()
                        )))?;

                    for row in rows {
                        let obj = match row {
                            serde_json::Value::Object(obj) => obj,
                            _ => {
                                return Err(RuntimeError::TypeError(
                                    "write_csv() rows must be objects when include_headers is true"
                                        .to_string(),
                                ))
                            }
                        };
                        let record: Vec<String> = headers
                            .iter()
                            .map(|key| {
                                obj.get(key)
                                    .map(json_value_to_csv)
                                    .unwrap_or_else(String::new)
                            })
                            .collect();
                        writer
                            .write_record(record.iter())
                            .map_err(|e| RuntimeError::IOError(format!(
                                "write_csv() failed for '{}': {e}",
                                path.display()
                            )))?;
                    }
                } else {
                    for row in rows {
                        let arr = match row {
                            serde_json::Value::Array(arr) => arr,
                            _ => {
                                return Err(RuntimeError::TypeError(
                                    "write_csv() rows must be arrays when include_headers is false"
                                        .to_string(),
                                ))
                            }
                        };
                        let record: Vec<String> =
                            arr.iter().map(json_value_to_csv).collect();
                        writer
                            .write_record(record.iter())
                            .map_err(|e| RuntimeError::IOError(format!(
                                "write_csv() failed for '{}': {e}",
                                path.display()
                            )))?;
                    }
                }
                writer.flush().map_err(|e| RuntimeError::IOError(format!(
                    "write_csv() failed for '{}': {e}",
                    path.display()
                )))?;
                Ok(Value::Null)
            }

            other => Err(RuntimeError::RuntimeError(format!(
                "unknown built-in: {other}"
            ))),
        }
    }

    /// Evaluate exactly `expected` arguments.
    fn eval_args(&mut self, args: Vec<Expr>, expected: usize) -> Result<Vec<Value>, RuntimeError> {
        if args.len() != expected {
            return Err(RuntimeError::RuntimeError(format!(
                "expected {expected} arguments, got {}",
                args.len()
            )));
        }
        let mut vals = Vec::with_capacity(expected);
        for a in args {
            vals.push(self.eval_expr(a)?);
        }
        Ok(vals)
    }

    fn eval_user_function(&mut self, name: &str, args: Vec<Expr>) -> Result<Value, RuntimeError> {
        let func =
            self.functions.get(name).cloned().ok_or_else(|| {
                RuntimeError::RuntimeError(format!("undefined function '{name}'"))
            })?;

        if args.len() != func.params.len() {
            return Err(RuntimeError::RuntimeError(format!(
                "function '{name}' expects {} arguments, got {}",
                func.params.len(),
                args.len()
            )));
        }

        // Evaluate arguments in the current scope.
        let mut arg_vals = Vec::with_capacity(args.len());
        for a in args {
            arg_vals.push(self.eval_expr(a)?);
        }

        // Push a new scope for the function body.
        self.env.push_scope();
        for (param, val) in func.params.iter().zip(arg_vals) {
            let coerced = coerce_to_declared_type(val, &param.typ);
            self.env.define(&param.name, coerced, param.typ.clone(), true)?;
        }

        let result = self.run_body(func.body.clone());

        self.env.pop_scope();

        match result {
            Ok(()) => Ok(Value::Null), // void function
            Err(RuntimeError::ReturnValue(val)) => Ok(val.unwrap_or(Value::Null)),
            Err(e) => Err(e),
        }
    }

    /// Call a named function with pre-evaluated `Value` arguments.
    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let func =
            self.functions.get(name).cloned().ok_or_else(|| {
                RuntimeError::RuntimeError(format!("undefined function '{name}'"))
            })?;

        if args.len() != func.params.len() {
            return Err(RuntimeError::RuntimeError(format!(
                "function '{name}' expects {} arguments, got {}",
                func.params.len(),
                args.len()
            )));
        }

        self.env.push_scope();
        for (param, val) in func.params.iter().zip(args) {
            let coerced = coerce_to_declared_type(val, &param.typ);
            self.env.define(&param.name, coerced, param.typ.clone(), true)?;
        }

        let result = self.run_body(func.body.clone());
        self.env.pop_scope();

        match result {
            Ok(()) => Ok(Value::Null),
            Err(RuntimeError::ReturnValue(val)) => Ok(val.unwrap_or(Value::Null)),
            Err(e) => Err(e),
        }
    }

    // -----------------------------------------------------------------------
    // GUI form methods
    // -----------------------------------------------------------------------

    /// Execute a form definition: register the form in rootWindow.
    fn exec_form_def(&mut self, fd: FormDefinition) -> Result<(), RuntimeError> {
        let form_name = fd.name.clone();
        // Create a fresh form instance and register it.
        let form = FormInstance::new();
        self.root_window.forms.insert(form_name.clone(), form);

        // Track the current form being built so exec_form_stmt can use it.
        let prev_form = self.current_form_name.take();
        self.current_form_name = Some(form_name.clone());

        for stmt in fd.body {
            self.exec_form_stmt(&form_name.clone(), stmt)?;
        }

        self.current_form_name = prev_form;
        Ok(())
    }

    /// Execute an event timer definition: register and configure the timer.
    fn exec_eventtimer_def(&mut self, def: EventTimerDefinition) -> Result<(), RuntimeError> {
        let name = def.name.clone();
        self.event_timers.insert(name.clone(), EventTimer::default());
        for stmt in def.body {
            self.exec_eventtimer_stmt(&name, stmt)?;
        }
        Ok(())
    }

    /// Execute a statement inside an event timer definition body.
    fn exec_eventtimer_stmt(&mut self, _timer_name: &str, stmt: Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Set(set_stmt) => match set_stmt.target {
                SetTarget::ObjectPath(path) => self.exec_set_object_path(path, set_stmt.op),
                _ => self.exec_set(set_stmt),
            },
            other => self.exec_stmt(other),
        }
    }

    /// Execute a statement inside a form definition body.
    fn exec_form_stmt(&mut self, form_name: &str, stmt: Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::DefineControl(ctrl_name, ctrl_type) => {
                let state = default_control_state(&ctrl_type);
                if let Some(form) = self.root_window.forms.get_mut(form_name) {
                    form.controls.insert(ctrl_name, state);
                }
                Ok(())
            }
            Stmt::Set(set_stmt) => {
                match set_stmt.target {
                    SetTarget::ObjectPath(path) => {
                        self.exec_set_object_path(path, set_stmt.op)
                    }
                    _ => self.exec_set(set_stmt),
                }
            }
            // Allow other statements in the form body for flexibility.
            other => self.exec_stmt(other),
        }
    }

    /// Execute `set <object-path> <op>`.
    fn exec_set_object_path(&mut self, path: Vec<String>, op: SetOp) -> Result<(), RuntimeError> {
        // Evaluate the value first (before mutably borrowing root_window).
        let value = match op {
            SetOp::Assign(expr) => {
                if let Some((timer_name, prop)) = self.timer_path_parts(&path) {
                    if prop == "event" {
                        if let Expr::Call(name, args) = &expr {
                            if args.is_empty() {
                                return self.set_timer_property(&timer_name, &prop, Value::Str(name.clone()));
                            }
                        }
                    }
                }
                if let Expr::Call(name, args) = &expr {
                    if args.is_empty() && is_event_path(&path) {
                        Value::Str(name.clone())
                    } else {
                        self.eval_expr(expr)?
                    }
                } else {
                    self.eval_expr(expr)?
                }
            }
            SetOp::Increment | SetOp::Decrement => {
                let current = self.eval_object_path(path.clone())?;
                match (current, &op) {
                    (Value::Integer(n), SetOp::Increment) => Value::Integer(n + 1),
                    (Value::Integer(n), SetOp::Decrement) => Value::Integer(n - 1),
                    _ => return Err(RuntimeError::TypeError(
                        "++ / -- on object path requires integer value".to_string()
                    )),
                }
            }
        };
        self.set_object_path_value(&path, value)
    }

    /// Write a value to an object path in rootWindow.
    fn set_object_path_value(&mut self, path: &[String], value: Value) -> Result<(), RuntimeError> {
        // Resolve relative paths inside form bodies.
        //   "form\prop"         → current_form_name\prop
        //   "controlName\prop"  → current_form_name\controlName\prop
        //                         (when first segment is not a known form and not rootWindow)
        let expanded: Option<Vec<String>> = {
            let first = path.first().map(|s| s.as_str()).unwrap_or("");
            if first == "form" {
                self.current_form_name.as_ref().map(|fname| {
                    std::iter::once(fname.clone())
                        .chain(path[1..].iter().cloned())
                        .collect()
                })
            } else if first != "rootWindow" {
                if let Some(ref fname) = self.current_form_name {
                    if !self.root_window.forms.contains_key(first)
                        && !self.event_timers.contains_key(first) {
                        Some(std::iter::once(fname.clone())
                            .chain(path.iter().cloned())
                            .collect())
                    } else { None }
                } else { None }
            } else { None }
        };
        let effective_path: &[String] = expanded.as_deref().unwrap_or(path);

        if let Some((timer_name, prop)) = self.timer_path_parts(effective_path) {
            return self.set_timer_property(&timer_name, &prop, value);
        }

        let (form_name, rest) = self.resolve_path_root(effective_path)?;

        let mut update_form = false;
        let result = match rest {
            [control_name, prop] if prop == "position" => {
                // Value should be JSON array [top, left, width, height]
                let mut handled = false;
                if let Value::Json(serde_json::Value::Array(ref arr)) = value {
                    if arr.len() == 4 {
                        let nums: Vec<u32> = arr.iter()
                            .map(|v| v.as_u64().unwrap_or(0) as u32)
                            .collect();
                        let form = self.root_window.forms.get_mut(&form_name)
                            .ok_or_else(|| RuntimeError::RuntimeError(
                                format!("no form named '{}' in rootWindow", form_name)
                            ))?;
                        if let Some(ctrl) = form.controls.get_mut(control_name) {
                            ctrl.set_position(nums[0], nums[1], nums[2], nums[3]);
                        } else {
                            eprintln!("[WARN] no control named '{}' in form '{}'", control_name, form_name);
                        }
                        update_form = true;
                        handled = true;
                    }
                }
                if !handled {
                    eprintln!("[WARN] position value is not a 4-element JSON array: {:?}", value);
                }
                Ok(())
            }
            [control_name, prop] => {
                let val_str = value_to_string(&value);
                let form = self.root_window.forms.get_mut(&form_name)
                    .ok_or_else(|| RuntimeError::RuntimeError(
                        format!("no form named '{}' in rootWindow", form_name)
                    ))?;
                if let Some(ctrl) = form.controls.get_mut(control_name) {
                    if let Err(warn) = ctrl.set_property(prop, &val_str) {
                        eprintln!("{warn}");
                    }
                } else {
                    eprintln!("[WARN] no control named '{}' in form '{}'", control_name, form_name);
                }
                update_form = true;
                Ok(())
            }
            [prop] => {
                // set formname\property = value  (form-level property)
                let val_str = value_to_string(&value);
                let form = self.root_window.forms.get_mut(&form_name)
                    .ok_or_else(|| RuntimeError::RuntimeError(
                        format!("no form named '{}' in rootWindow", form_name)
                    ))?;
                if let Err(warn) = form.properties.set_property(prop, &val_str) {
                    eprintln!("{warn}");
                }
                update_form = true;
                Ok(())
            }
            _ => Err(RuntimeError::RuntimeError(
                format!("invalid object path: {:?}", path)
            )),
        };

        if update_form {
            if let Some(ref tx) = self.gui_tx {
                if let Some(updated) = self.root_window.forms.get(&form_name).cloned() {
                    let _ = tx.send(crate::gui_channel::GuiCommand::UpdateForm {
                        name: form_name.clone(),
                        instance: updated,
                    });
                }
            }
        }

        result
    }

    /// Read a value from an object path.
    fn eval_object_path(&self, path: Vec<String>) -> Result<Value, RuntimeError> {
        // Resolve relative paths inside form bodies.
        //   "form\prop"         → current_form_name\prop
        //   "controlName\prop"  → current_form_name\controlName\prop
        //                         (when first segment is not a known form and not rootWindow)
        let expanded: Option<Vec<String>> = {
            let first = path.first().map(|s| s.as_str()).unwrap_or("");
            if first == "form" {
                self.current_form_name.as_ref().map(|fname| {
                    std::iter::once(fname.clone())
                        .chain(path[1..].iter().cloned())
                        .collect()
                })
            } else if first != "rootWindow" {
                if let Some(ref fname) = self.current_form_name {
                    if !self.root_window.forms.contains_key(first)
                        && !self.event_timers.contains_key(first) {
                        Some(std::iter::once(fname.clone())
                            .chain(path.iter().cloned())
                            .collect())
                    } else { None }
                } else { None }
            } else { None }
        };
        let effective_path: &[String] = expanded.as_deref().unwrap_or(&path);

        if let Some((timer_name, prop)) = self.timer_path_parts(effective_path) {
            return self.eval_timer_property(&timer_name, &prop);
        }

        let (form_name, rest) = self.resolve_path_root(effective_path)?;

        match rest {
            [prop] => {
                let form = self.root_window.forms.get(&form_name)
                    .ok_or_else(|| RuntimeError::RuntimeError(
                        format!("no form named '{}' in rootWindow", form_name)
                    ))?;
                let val = form.properties.get_property(prop)
                    .ok_or_else(|| RuntimeError::RuntimeError(
                        format!("no property '{}' on form '{}'", prop, form_name)
                    ))?;
                Ok(Value::Str(val))
            }
            [control_name, prop] => {
                let form = self.root_window.forms.get(&form_name)
                    .ok_or_else(|| RuntimeError::RuntimeError(
                        format!("no form named '{}' in rootWindow", form_name)
                    ))?;
                let ctrl = form.controls.get(control_name)
                    .ok_or_else(|| RuntimeError::RuntimeError(
                        format!("no control named '{}' in form '{}'", control_name, form_name)
                    ))?;
                let val = ctrl.get_property(prop)
                    .ok_or_else(|| RuntimeError::RuntimeError(
                        format!("no property '{}' on control '{}' in form '{}'", prop, control_name, form_name)
                    ))?;
                Ok(Value::Str(val))
            }
            _ => Err(RuntimeError::RuntimeError(
                format!("invalid object path for read: {:?}", path)
            )),
        }
    }

    /// Resolve the "root" of an object path, skipping optional "rootWindow" prefix.
    /// Returns (form_name, remaining_segments).
    fn resolve_path_root<'a>(&self, path: &'a [String]) -> Result<(String, &'a [String]), RuntimeError> {
        match path {
            [first, rest @ ..] if first == "rootWindow" => {
                match rest {
                    [form_name, rest2 @ ..] => Ok((form_name.clone(), rest2)),
                    _ => Err(RuntimeError::RuntimeError(
                        "object path after 'rootWindow' must include form name".to_string()
                    )),
                }
            }
            [form_name, rest @ ..] => Ok((form_name.clone(), rest)),
            [] => Err(RuntimeError::RuntimeError("empty object path".to_string())),
        }
    }

    /// Execute `show()` — open a form.
    fn exec_show_form(&mut self, path: Vec<String>, modal: bool) -> Result<(), RuntimeError> {
        let (form_name, _) = self.resolve_path_root(&path)?;

        if form_name == "rootWindow" {
            return Err(RuntimeError::RuntimeError(
                "ReservedIdentifier(\"rootWindow\")".to_string()
            ));
        }

        let instance = {
            let form = self.root_window.forms.get_mut(&form_name)
                .ok_or_else(|| RuntimeError::RuntimeError(
                    format!("no form named '{}' in rootWindow", form_name)
                ))?;
            form.is_open = true;
            form.is_modal = modal;
            form.clone()
        };

        // Phase 8: No actual GUI window yet. Send command if channel is present.
        if let Some(ref tx) = self.gui_tx {
            let _ = tx.send(crate::gui_channel::GuiCommand::ShowForm {
                name: form_name.clone(),
                modal,
                instance,
            });
        }
        if modal && self.gui_rx.is_some() {
            let deadline = std::time::Instant::now()
                + std::time::Duration::from_millis(MODAL_TIMEOUT_MS);
            loop {
                if std::time::Instant::now() > deadline {
                    eprintln!("[WARN] modal form '{}' timed out after {}ms", form_name, MODAL_TIMEOUT_MS);
                    break;
                }
                // Receive without holding a borrow across the dispatch call.
                let msg = self.gui_rx.as_ref().and_then(|rx| rx.try_recv().ok());
                match msg {
                    Some(crate::gui_channel::GuiResponse::FormClosed { name }) if name == form_name => break,
                    Some(crate::gui_channel::GuiResponse::EventFired { form, control, event, value }) => {
                        let _ = self.dispatch_event(&form, &control, &event, value);
                    }
                    None => {
                        self.pump_timers()?;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn pump_gui_events(&mut self) -> Result<(), RuntimeError> {
        if self.gui_rx.is_none() {
            return Ok(());
        }
        loop {
            let msg = match self.gui_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
                Some(msg) => msg,
                None => break,
            };
            match msg {
                crate::gui_channel::GuiResponse::EventFired { form, control, event, value } => {
                    self.dispatch_event(&form, &control, &event, value)?;
                }
                crate::gui_channel::GuiResponse::FormClosed { name } => {
                    if let Some(form) = self.root_window.forms.get_mut(&name) {
                        form.is_open = false;
                    }
                }
                crate::gui_channel::GuiResponse::DialogResult { .. } => {}
                crate::gui_channel::GuiResponse::Ready => {}
            }
        }
        self.pump_timers()?;
        Ok(())
    }

    fn wait_for_open_forms(&mut self) -> Result<(), RuntimeError> {
        if self.gui_rx.is_none() {
            return Ok(());
        }
        while self.root_window.forms.values().any(|f| f.is_open) {
            self.pump_gui_events()?;
            self.pump_timers()?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }

    fn wait_for_timers(&mut self) -> Result<(), RuntimeError> {
        if self.gui_rx.is_some() {
            return Ok(());
        }
        while self.has_active_timers() {
            self.pump_timers()?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }

    fn has_active_timers(&self) -> bool {
        self.event_timers
            .values()
            .any(|timer| timer.running && timer.interval_ms > 0)
    }

    fn pump_timers(&mut self) -> Result<(), RuntimeError> {
        if self.event_timers.is_empty() {
            return Ok(());
        }

        let now = Instant::now();
        let due: Vec<(String, Option<String>)> = self
            .event_timers
            .iter()
            .filter(|(_, timer)| timer.running && timer.interval_ms > 0 && !timer.in_callback)
            .filter(|(_, timer)| timer.next_fire.map_or(true, |t| now >= t))
            .map(|(name, timer)| (name.clone(), timer.on_event.clone()))
            .collect();

        for (name, callback) in due {
            if let Some(timer) = self.event_timers.get_mut(&name) {
                if timer.in_callback {
                    continue;
                }
                timer.in_callback = true;
                if timer.running && timer.interval_ms > 0 {
                    timer.next_fire = Some(Instant::now() + Duration::from_millis(timer.interval_ms));
                }
            }
            if let Some(handler) = callback {
                let _ = self.call_function(&handler, vec![])?;
            }
            if let Some(timer) = self.event_timers.get_mut(&name) {
                timer.in_callback = false;
                if !timer.running || timer.interval_ms == 0 {
                    timer.next_fire = None;
                }
            }
        }
        Ok(())
    }

    fn timer_path_parts(&self, path: &[String]) -> Option<(String, String)> {
        if path.len() == 2 {
            let name = path[0].clone();
            let prop = path[1].clone();
            if self.event_timers.contains_key(&name) {
                return Some((name, prop));
            }
        }
        None
    }

    fn eval_timer_property(&self, timer_name: &str, prop: &str) -> Result<Value, RuntimeError> {
        let timer = self.event_timers.get(timer_name).ok_or_else(|| {
            RuntimeError::RuntimeError(format!("no eventtimer named '{timer_name}'"))
        })?;
        let val = match prop {
            "interval" => timer.interval_ms.to_string(),
            "running" => timer.running.to_string(),
            "event" => timer.on_event.clone().unwrap_or_default(),
            _ => {
                return Err(RuntimeError::RuntimeError(
                    format!("no property '{}' on eventtimer '{}'", prop, timer_name)
                ));
            }
        };
        Ok(Value::Str(val))
    }

    fn set_timer_property(&mut self, timer_name: &str, prop: &str, value: Value) -> Result<(), RuntimeError> {
        let timer = self.event_timers.get_mut(timer_name).ok_or_else(|| {
            RuntimeError::RuntimeError(format!("no eventtimer named '{timer_name}'"))
        })?;

        match prop {
            "interval" => {
                let interval_ms = parse_duration_ms(&value)?;
                timer.interval_ms = interval_ms;
                if timer.running && interval_ms > 0 {
                    timer.next_fire = Some(Instant::now() + Duration::from_millis(interval_ms));
                } else {
                    timer.next_fire = None;
                }
            }
            "running" => {
                let running = parse_bool_value(&value)?;
                timer.running = running;
                if running && timer.interval_ms > 0 {
                    timer.next_fire = Some(Instant::now() + Duration::from_millis(timer.interval_ms));
                } else {
                    timer.next_fire = None;
                }
            }
            "event" => {
                let name = match value {
                    Value::Str(s) => s,
                    other => other.to_display_string(),
                };
                timer.on_event = if name.trim().is_empty() { None } else { Some(name) };
            }
            _ => {
                return Err(RuntimeError::RuntimeError(
                    format!("no property '{}' on eventtimer '{}'", prop, timer_name)
                ));
            }
        }
        Ok(())
    }

    /// Execute `close()` — close a form.
    fn exec_close_form(&mut self, path: Vec<String>) -> Result<(), RuntimeError> {
        let (form_name, _) = self.resolve_path_root(&path)?;
        if let Some(form) = self.root_window.forms.get_mut(&form_name) {
            form.is_open = false;
        }
        if let Some(ref tx) = self.gui_tx {
            let _ = tx.send(crate::gui_channel::GuiCommand::CloseForm { name: form_name });
        }
        Ok(())
    }

    /// Evaluate a dialog call.
    fn eval_dialog_call(&mut self, call: DialogCall) -> Result<Value, RuntimeError> {
        if self.gui_tx.is_none() || self.gui_rx.is_none() {
            return match call {
                DialogCall::OpenFile { .. } | DialogCall::SaveFile { .. } => Ok(Value::Str(String::new())),
                DialogCall::Message { .. } => Ok(Value::Str("ok".to_string())),
                DialogCall::ColorPicker { initial } => {
                    let val = self.eval_expr(*initial)?;
                    Ok(val)
                }
            };
        }

        let request = match call {
            DialogCall::OpenFile { title, filter } => {
                let title = self.eval_expr(*title)?.to_display_string();
                let filter = self.eval_expr(*filter)?.to_display_string();
                crate::gui_channel::DialogRequest::OpenFile { title, filter }
            }
            DialogCall::SaveFile { title, filter } => {
                let title = self.eval_expr(*title)?.to_display_string();
                let filter = self.eval_expr(*filter)?.to_display_string();
                crate::gui_channel::DialogRequest::SaveFile { title, filter }
            }
            DialogCall::Message { title, message, kind } => {
                let title = self.eval_expr(*title)?.to_display_string();
                let message = self.eval_expr(*message)?.to_display_string();
                crate::gui_channel::DialogRequest::Message { title, message, kind }
            }
            DialogCall::ColorPicker { initial } => {
                let initial = self.eval_expr(*initial)?.to_display_string();
                crate::gui_channel::DialogRequest::ColorPicker { initial }
            }
        };

        self.dialog_seq += 1;
        let id = self.dialog_seq;
        if let Some(ref tx) = self.gui_tx {
            let _ = tx.send(crate::gui_channel::GuiCommand::DialogCall { id, request });
        }
        self.wait_for_dialog_result(id)
    }

    fn wait_for_dialog_result(&mut self, id: u64) -> Result<Value, RuntimeError> {
        loop {
            let msg = self.gui_rx.as_ref().and_then(|rx| rx.try_recv().ok());
            match msg {
                Some(crate::gui_channel::GuiResponse::DialogResult { id: got_id, value })
                    if got_id == id => {
                    return Ok(Value::Str(value));
                }
                Some(crate::gui_channel::GuiResponse::EventFired { form, control, event, value }) => {
                    self.dispatch_event(&form, &control, &event, value)?;
                }
                Some(crate::gui_channel::GuiResponse::FormClosed { name }) => {
                    if let Some(form) = self.root_window.forms.get_mut(&name) {
                        form.is_open = false;
                    }
                }
                Some(crate::gui_channel::GuiResponse::Ready) => {}
                Some(crate::gui_channel::GuiResponse::DialogResult { .. }) => {}
                None => std::thread::sleep(std::time::Duration::from_millis(10)),
            }
        }
    }

    // -----------------------------------------------------------------------
    // REST / query methods
    // -----------------------------------------------------------------------

    /// Execute an `attempt / catch` error-handling block.
    fn exec_attempt(&mut self, block: rundell_parser::ast::AttemptBlock) -> Result<(), RuntimeError> {
        let body_result = {
            self.env.push_scope();
            let r = self.run_body(block.body.clone());
            self.env.pop_scope();
            r
        };

        match body_result {
            Ok(()) => Ok(()),
            Err(e) => {
                let is_catchable = matches!(
                    &e,
                    RuntimeError::QueryTimeout { .. }
                        | RuntimeError::QueryNetworkError { .. }
                        | RuntimeError::QueryHttpError { .. }
                        | RuntimeError::QueryInvalidJson { .. }
                        | RuntimeError::UndefinedQuery { .. }
                        | RuntimeError::UndefinedCredentials { .. }
                        | RuntimeError::EnvKeyNotFound { .. }
                        | RuntimeError::EnvDecryptionFailed { .. }
                        | RuntimeError::NoProgramPath
                );

                if !is_catchable {
                    return Err(e);
                }

                let (message, status_code, endpoint) = match &e {
                    RuntimeError::QueryTimeout { endpoint, timeout_ms } => (
                        format!("Query timed out after {}ms", timeout_ms),
                        0u16,
                        endpoint.clone(),
                    ),
                    RuntimeError::QueryNetworkError { message, endpoint } => (
                        message.clone(),
                        0u16,
                        endpoint.clone(),
                    ),
                    RuntimeError::QueryHttpError { status_code, endpoint } => (
                        format!("HTTP error {} from {}", status_code, endpoint),
                        *status_code,
                        endpoint.clone(),
                    ),
                    RuntimeError::QueryInvalidJson { endpoint } => (
                        format!("Response from {} is not valid JSON", endpoint),
                        0u16,
                        endpoint.clone(),
                    ),
                    _ => (e.to_string(), 0u16, String::new()),
                };

                let error_obj = serde_json::json!({
                    "message": message,
                    "statusCode": status_code,
                    "endpoint": endpoint,
                });

                // Bind the error variable in the current scope so the handler can read it.
                // We use define (not set) since it may not exist yet.
                let scope_pushed = if self.env.get(&block.error_name).is_err() {
                    self.env.push_scope();
                    self.env.define(
                        &block.error_name,
                        Value::Json(error_obj),
                        RundellType::Json,
                        false,
                    )?;
                    true
                } else {
                    self.env.set(&block.error_name, Value::Json(error_obj))?;
                    false
                };

                let handler_result = self.run_body(block.handler.clone());

                if scope_pushed {
                    self.env.pop_scope();
                }

                handler_result
            }
        }
    }

    /// Helper: evaluate an expression with extra local variable bindings in scope.
    fn eval_expr_with_locals(
        &mut self,
        expr: &Expr,
        locals: &HashMap<String, Value>,
    ) -> Result<Value, RuntimeError> {
        self.env.push_scope();
        for (name, val) in locals {
            // Ignore errors from shadowing (e.g. already defined in outer scope).
            let _ = self.env.define(name, val.clone(), RundellType::Json, false);
        }
        let result = self.eval_expr(expr.clone());
        self.env.pop_scope();
        result
    }

    /// Evaluate an `await queryCall(...)` expression — performs the HTTP request.
    fn eval_await(
        &mut self,
        await_expr: rundell_parser::ast::AwaitExpr,
    ) -> Result<Value, RuntimeError> {
        // The inner expression must be a function call.
        let (query_name, call_args) = match *await_expr.call {
            Expr::Call(name, args) => (name, args),
            _ => {
                return Err(RuntimeError::RuntimeError(
                    "await must be followed by a query call".to_string(),
                ))
            }
        };

        // Look up the query definition.
        let query_def = self
            .query_registry
            .queries
            .get(&query_name)
            .ok_or_else(|| RuntimeError::UndefinedQuery { name: query_name.clone() })?
            .clone();

        // Evaluate call arguments and bind to parameters.
        let mut local_env: HashMap<String, Value> = HashMap::new();
        for (param, arg_expr) in query_def.params.iter().zip(call_args.iter()) {
            let val = self.eval_expr(arg_expr.clone())?;
            local_env.insert(param.name.clone(), val);
        }

        // Evaluate endpoint with parameter substitution.
        let endpoint_val = self.eval_expr_with_locals(&query_def.endpoint, &local_env)?;
        let endpoint = match endpoint_val {
            Value::Str(s) => s,
            _ => {
                return Err(RuntimeError::TypeError(
                    "query endpoint must evaluate to a string".to_string(),
                ))
            }
        };

        // Resolve credentials.
        let creds: Option<CredentialsInstance> = if let Some(creds_name) = &query_def.credentials {
            let inst = self
                .credentials_registry
                .credentials
                .get(creds_name)
                .ok_or_else(|| RuntimeError::UndefinedCredentials { name: creds_name.clone() })?
                .clone();
            Some(inst)
        } else {
            None
        };

        // Determine timeout.
        let timeout_ms = if let Some(ref t_expr) = query_def.timeout_ms {
            match self.eval_expr_with_locals(t_expr, &local_env)? {
                Value::Integer(ms) => ms as u64,
                _ => QUERY_TIMEOUT_MS,
            }
        } else {
            QUERY_TIMEOUT_MS
        };

        // Evaluate POST body before moving into async block.
        let post_body: Option<String> = if matches!(query_def.method, rundell_parser::ast::HttpMethod::Post) {
            if let Some(ref qp_expr) = query_def.query_params {
                let qp_val = self.eval_expr_with_locals(qp_expr, &local_env)?;
                match qp_val {
                    Value::Json(json_val) => Some(json_val.to_string()),
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "queryParams must be a json value".to_string(),
                        ))
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // Clone everything needed for the async block.
        let endpoint_clone = endpoint.clone();
        let method = query_def.method.clone();
        let token = creds.as_ref().and_then(|c| c.token.clone());
        let authentication = creds.as_ref().and_then(|c| c.authentication.clone());

        // Build and execute the request inside the Tokio runtime.
        let result = self.rt.block_on(async move {
            let client = reqwest::Client::new();
            let mut req_builder = match method {
                rundell_parser::ast::HttpMethod::Get => client.get(&endpoint_clone),
                rundell_parser::ast::HttpMethod::Post => client.post(&endpoint_clone),
            };

            if let Some(ref tok) = token {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", tok));
            }
            if let Some(ref auth) = authentication {
                req_builder = req_builder.header("X-Rundell-Auth", auth.as_str());
            }

            if let Some(body) = post_body {
                req_builder = req_builder
                    .header("Content-Type", "application/json")
                    .body(body);
            }

            let request = req_builder.build().map_err(|e| RuntimeError::QueryNetworkError {
                message: e.to_string(),
                endpoint: endpoint_clone.clone(),
            })?;

            let response = tokio::time::timeout(
                std::time::Duration::from_millis(timeout_ms),
                client.execute(request),
            )
            .await
            .map_err(|_| RuntimeError::QueryTimeout {
                endpoint: endpoint_clone.clone(),
                timeout_ms,
            })?
            .map_err(|e| RuntimeError::QueryNetworkError {
                message: e.to_string(),
                endpoint: endpoint_clone.clone(),
            })?;

            if response.status().is_client_error() || response.status().is_server_error() {
                return Err(RuntimeError::QueryHttpError {
                    status_code: response.status().as_u16(),
                    endpoint: endpoint_clone.clone(),
                });
            }

            let body_text = response.text().await.map_err(|e| RuntimeError::QueryNetworkError {
                message: e.to_string(),
                endpoint: endpoint_clone.clone(),
            })?;

            let json_val: serde_json::Value = serde_json::from_str(&body_text)
                .map_err(|_| RuntimeError::QueryInvalidJson {
                    endpoint: endpoint_clone.clone(),
                })?;

            Ok(json_val)
        });

        result.map(Value::Json)
    }

    /// Dispatch a GUI event to the registered callback function.
    pub fn dispatch_event(
        &mut self,
        form: &str,
        control: &str,
        event: &str,
        value: Option<String>,
    ) -> Result<(), RuntimeError> {
        use crate::form_registry::ControlState;

        if let Some(value) = value {
            if let Some(form_state) = self.root_window.forms.get_mut(form) {
                if let Some(ctrl_state) = form_state.controls.get_mut(control) {
                    match ctrl_state {
                        ControlState::Textbox { value: v, .. } if event == "change" => {
                            *v = value;
                        }
                        ControlState::Checkbox { checked, .. } if event == "change" => {
                            *checked = value == "true";
                        }
                        ControlState::Switch { checked, .. } if event == "change" => {
                            *checked = value == "true";
                        }
                        ControlState::Radiobutton { checked, .. } if event == "change" => {
                            *checked = value == "true";
                        }
                        ControlState::Select { items, selected_index, .. } if event == "change" => {
                            *selected_index = items.iter().position(|item| item == &value);
                        }
                        _ => {}
                    }
                }
            }
        }

        let callback = self.root_window.forms.get(form)
            .and_then(|f| f.controls.get(control))
            .and_then(|c| match (c, event) {
                (ControlState::Button { on_click, .. }, "click") => on_click.clone(),
                (ControlState::Textbox { on_change, .. }, "change") => on_change.clone(),
                (ControlState::Radiobutton { on_change, .. }, "change") => on_change.clone(),
                (ControlState::Checkbox { on_change, .. }, "change") => on_change.clone(),
                (ControlState::Switch { on_change, .. }, "change") => on_change.clone(),
                (ControlState::Select { on_change, .. }, "change") => on_change.clone(),
                (ControlState::Listbox { on_change, .. }, "change") => on_change.clone(),
                (ControlState::Listbox { on_select, .. }, "select") => on_select.clone(),
                _ => None,
            });

        if let Some(fn_name) = callback {
            self.call_function(&fn_name, vec![])?;
        }
        Ok(())
    }

    fn resolve_io_path(&self, raw: &str) -> Result<PathBuf, RuntimeError> {
        let path = Path::new(raw);
        if path.is_absolute() {
            return Ok(path.to_path_buf());
        }

        if let Some(program_path) = &self.program_path {
            let base = program_path
                .parent()
                .unwrap_or(std::path::Path::new("."));
            return Ok(base.join(path));
        }

        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|e| RuntimeError::IOError(e.to_string()))
    }

    fn resolve_execute_path(&self, raw: &str) -> Result<PathBuf, RuntimeError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(RuntimeError::RuntimeError(
                "execute() path cannot be empty".to_string(),
            ));
        }
        if has_mixed_separators(trimmed) {
            return Err(RuntimeError::RuntimeError(
                "execute() path cannot mix '/' and '\\'".to_string(),
            ));
        }

        if trimmed.contains('/') || trimmed.contains('\\') {
            let candidate = self.resolve_io_path(trimmed)?;
            if candidate.exists() {
                return Ok(candidate);
            }
            return Err(RuntimeError::RuntimeError(format!(
                "execute() could not find '{}'",
                trimmed
            )));
        }

        if let Some(found) = find_executable_in_path(trimmed) {
            return Ok(found);
        }

        Err(RuntimeError::RuntimeError(format!(
            "execute() could not find '{}' in PATH",
            trimmed
        )))
    }

    fn expect_string_arg(
        &self,
        value: &Value,
        func: &str,
        arg: &str,
    ) -> Result<String, RuntimeError> {
        match value {
            Value::Str(s) => Ok(s.clone()),
            _ => Err(RuntimeError::TypeError(format!(
                "{func}() {arg} must be string"
            ))),
        }
    }

    fn expect_boolean_arg(
        &self,
        value: &Value,
        func: &str,
        arg: &str,
    ) -> Result<bool, RuntimeError> {
        match value {
            Value::Boolean(b) => Ok(*b),
            _ => Err(RuntimeError::TypeError(format!(
                "{func}() {arg} must be boolean"
            ))),
        }
    }

    fn expect_datetime_arg(
        &self,
        value: &Value,
        func: &str,
        arg: &str,
    ) -> Result<DateTime<FixedOffset>, RuntimeError> {
        match value {
            Value::DateTime(dt) => Ok(dt.clone()),
            _ => Err(RuntimeError::TypeError(format!(
                "{func}() {arg} must be datetime"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions (free)
// ---------------------------------------------------------------------------

/// Convert an AST `Literal` to a runtime `Value`.
fn literal_to_value(lit: Literal) -> Result<Value, RuntimeError> {
    match lit {
        Literal::Integer(n) => Ok(Value::Integer(n)),
        Literal::Float(f) => Ok(Value::Float(f)),
        Literal::Str(s) => Ok(Value::Str(s)),
        Literal::Currency(c) => Ok(Value::Currency(c)),
        Literal::Boolean(b) => Ok(Value::Boolean(b)),
        Literal::DateTime(s) => parse_datetime_literal(&s).map(Value::DateTime),
        Literal::Null => Ok(Value::Null),
    }
}

/// Convert a runtime `Value` into a `serde_json::Value` for JSON storage.
fn value_to_json(val: Value) -> Result<serde_json::Value, RuntimeError> {
    match val {
        Value::Integer(n) => Ok(serde_json::Value::Number(n.into())),
        Value::Float(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| RuntimeError::TypeError("float cannot be stored in JSON".to_string())),
        Value::Str(s) => Ok(serde_json::Value::String(s)),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        Value::Json(j) => Ok(j),
        Value::Null => Ok(serde_json::Value::Null),
        Value::DateTime(dt) => Ok(serde_json::Value::String(
            dt.to_rfc3339_opts(SecondsFormat::Secs, true),
        )),
        Value::Currency(c) => {
            let f = c as f64 / 100.0;
            serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .ok_or_else(|| {
                    RuntimeError::TypeError("currency cannot be stored in JSON".to_string())
                })
        }
    }
}

/// Convert a `serde_json::Value` into a runtime `Value`.
fn json_to_value(j: serde_json::Value) -> Value {
    match j {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::Str(s),
        other => Value::Json(other),
    }
}

/// Index into a JSON value.
fn json_index(j: serde_json::Value, key: Value) -> Result<Value, RuntimeError> {
    match (j, key) {
        (serde_json::Value::Object(map), Value::Str(k)) => map
            .get(&k)
            .cloned()
            .map(json_to_value)
            .ok_or_else(|| RuntimeError::IndexError(format!("key '{k}' not found"))),
        (serde_json::Value::Object(map), Value::Integer(i)) => {
            // Positional access by key insertion order.
            map.values()
                .nth(i as usize)
                .cloned()
                .map(json_to_value)
                .ok_or_else(|| {
                    RuntimeError::IndexError(format!("positional index {i} out of bounds"))
                })
        }
        (serde_json::Value::Array(arr), Value::Integer(i)) => arr
            .into_iter()
            .nth(i as usize)
            .map(json_to_value)
            .ok_or_else(|| RuntimeError::IndexError(format!("index {i} out of bounds"))),
        (col, key) => Err(RuntimeError::TypeError(format!(
            "cannot index {} with {}",
            col,
            key.type_name()
        ))),
    }
}

/// True if two values are equal (across compatible types).
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Integer(x), Value::Float(y)) => (*x as f64) == *y,
        (Value::Float(x), Value::Integer(y)) => *x == (*y as f64),
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Currency(x), Value::Currency(y)) => x == y,
        (Value::DateTime(x), Value::DateTime(y)) => x == y,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

/// Compare two values with the given comparison operator.
fn compare_values(a: &Value, op: &CmpOp, b: &Value) -> Result<bool, RuntimeError> {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Ok(apply_cmp(*x, op, *y)),
        (Value::Float(x), Value::Float(y)) => Ok(apply_cmp_f(*x, op, *y)),
        (Value::Integer(x), Value::Float(y)) => Ok(apply_cmp_f(*x as f64, op, *y)),
        (Value::Float(x), Value::Integer(y)) => Ok(apply_cmp_f(*x, op, *y as f64)),
        (Value::Str(x), Value::Str(y)) => Ok(apply_cmp_ord(x.as_str(), op, y.as_str())),
        (Value::Currency(x), Value::Currency(y)) => Ok(apply_cmp(*x, op, *y)),
        (Value::DateTime(x), Value::DateTime(y)) => Ok(apply_cmp(x, op, y)),
        _ => Err(RuntimeError::TypeError(format!(
            "cannot compare {} with {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

fn is_event_path(path: &[String]) -> bool {
    matches!(path.last().map(String::as_str), Some("click") | Some("change") | Some("select"))
}

fn parse_bool_value(value: &Value) -> Result<bool, RuntimeError> {
    match value {
        Value::Boolean(b) => Ok(*b),
        Value::Integer(n) => Ok(*n != 0),
        Value::Float(f) => Ok(*f != 0.0),
        Value::Str(s) => match s.to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(RuntimeError::TypeError(format!(
                "cannot cast string '{s}' to boolean"
            ))),
        },
        Value::Null => Ok(false),
        other => Err(RuntimeError::TypeError(format!(
            "cannot cast {} to boolean",
            other.type_name()
        ))),
    }
}

fn parse_duration_ms(value: &Value) -> Result<u64, RuntimeError> {
    match value {
        Value::Integer(n) if *n >= 0 => Ok(*n as u64),
        Value::Float(f) if *f >= 0.0 => Ok(*f as u64),
        Value::Str(s) => parse_duration_ms_str(s),
        Value::Null => Ok(0),
        other => Err(RuntimeError::TypeError(format!(
            "cannot cast {} to duration",
            other.type_name()
        ))),
    }
}

fn parse_duration_ms_str(s: &str) -> Result<u64, RuntimeError> {
    let trimmed = s.trim();
    if let Some(num) = trimmed.strip_suffix("ms") {
        return num.parse::<u64>().map_err(|_| RuntimeError::TypeError(
            format!("invalid duration '{trimmed}'")
        ));
    }
    if let Some(num) = trimmed.strip_suffix('s') {
        return num.parse::<u64>().map(|n| n * 1_000).map_err(|_| RuntimeError::TypeError(
            format!("invalid duration '{trimmed}'")
        ));
    }
    if let Some(num) = trimmed.strip_suffix('m') {
        return num.parse::<u64>().map(|n| n * 60_000).map_err(|_| RuntimeError::TypeError(
            format!("invalid duration '{trimmed}'")
        ));
    }
    if let Some(num) = trimmed.strip_suffix('h') {
        return num.parse::<u64>().map(|n| n * 3_600_000).map_err(|_| RuntimeError::TypeError(
            format!("invalid duration '{trimmed}'")
        ));
    }
    trimmed.parse::<u64>().map_err(|_| RuntimeError::TypeError(
        format!("invalid duration '{trimmed}'")
    ))
}

fn parse_datetime_literal(s: &str) -> Result<DateTime<FixedOffset>, RuntimeError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(RuntimeError::TypeError("empty datetime literal".to_string()));
    }

    let mut candidate = trimmed.to_string();
    if !candidate.contains('T') {
        if let Some(space_pos) = candidate.find(' ') {
            candidate.replace_range(space_pos..=space_pos, "T");
        }
    }

    if candidate.len() < 19 {
        return Err(RuntimeError::TypeError(format!("invalid datetime '{trimmed}'")));
    }

    let time_start = 10;
    let has_offset = candidate.ends_with('Z')
        || candidate[time_start..].contains('+')
        || candidate[time_start..].contains('-');

    if has_offset {
        return DateTime::parse_from_rfc3339(&candidate).map_err(|_| {
            RuntimeError::TypeError(format!("invalid datetime '{trimmed}'"))
        });
    }

    let naive = NaiveDateTime::parse_from_str(&candidate, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| RuntimeError::TypeError(format!("invalid datetime '{trimmed}'")))?;
    let offset = *Local::now().offset();
    offset
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| RuntimeError::TypeError(format!("invalid datetime '{trimmed}'")))
}

fn datetime_from_timestamp_ms(ms: i64) -> Result<DateTime<FixedOffset>, RuntimeError> {
    let secs = ms.div_euclid(1_000);
    let nsec = (ms.rem_euclid(1_000) as u32) * 1_000_000;
    let utc = DateTime::<Utc>::from_timestamp(secs, nsec).ok_or_else(|| {
        RuntimeError::TypeError(format!("invalid timestamp '{ms}'"))
    })?;
    let offset = FixedOffset::east_opt(0).ok_or_else(|| {
        RuntimeError::TypeError("invalid UTC offset".to_string())
    })?;
    Ok(utc.with_timezone(&offset))
}

fn format_datetime(dt: &DateTime<FixedOffset>, fmt: &str) -> String {
    let mut pattern = fmt.to_string();
    pattern = pattern.replace("YYYY", "%Y");
    pattern = pattern.replace("MM", "%m");
    pattern = pattern.replace("DD", "%d");
    pattern = pattern.replace("HH", "%H");
    pattern = pattern.replace("mm", "%M");
    pattern = pattern.replace("SS", "%S");
    pattern = pattern.replace("ZZ", "%:z");
    pattern = pattern.replace("Z", "%z");
    dt.format(&pattern).to_string()
}

fn apply_cmp<T: PartialOrd>(a: T, op: &CmpOp, b: T) -> bool {
    match op {
        CmpOp::Lt => a < b,
        CmpOp::LtEq => a <= b,
        CmpOp::Gt => a > b,
        CmpOp::GtEq => a >= b,
        CmpOp::Eq => a == b,
        CmpOp::NotEq => a != b,
    }
}

fn apply_cmp_f(a: f64, op: &CmpOp, b: f64) -> bool {
    apply_cmp(a, op, b)
}

fn apply_cmp_ord(a: &str, op: &CmpOp, b: &str) -> bool {
    apply_cmp(a, op, b)
}

enum ArithOp {
    Sub,
    Mul,
}

/// Evaluate an arithmetic (non-add, non-div) binary operation.
fn eval_arith(l: Value, r: Value, op: ArithOp) -> Result<Value, RuntimeError> {
    // Null check
    if matches!(l, Value::Null) || matches!(r, Value::Null) {
        return Err(RuntimeError::NullError(
            "arithmetic on null value".to_string(),
        ));
    }
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => match op {
            ArithOp::Sub => Ok(Value::Integer(a - b)),
            ArithOp::Mul => Ok(Value::Integer(a * b)),
        },
        (Value::Float(a), Value::Float(b)) => match op {
            ArithOp::Sub => Ok(Value::Float(a - b)),
            ArithOp::Mul => Ok(Value::Float(a * b)),
        },
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a))
            if matches!(op, ArithOp::Sub) =>
        {
            Ok(Value::Float(a as f64 - b))
        }
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * b as f64)),
        (Value::Currency(a), Value::Currency(b)) => match op {
            ArithOp::Sub => Ok(Value::Currency(a - b)),
            ArithOp::Mul => Ok(Value::Currency(a * b / 100)),
        },
        // Currency mixed with Float or Integer: promote to float.
        (Value::Currency(c), Value::Float(f)) | (Value::Float(f), Value::Currency(c)) => {
            let cf = currency_as_float(c);
            match op {
                ArithOp::Sub => Ok(Value::Float(cf - f)),
                ArithOp::Mul => Ok(Value::Float(cf * f)),
            }
        }
        (Value::Currency(c), Value::Integer(n)) | (Value::Integer(n), Value::Currency(c)) => {
            let cf = currency_as_float(c);
            match op {
                ArithOp::Sub => Ok(Value::Float(cf - n as f64)),
                ArithOp::Mul => Ok(Value::Float(cf * n as f64)),
            }
        }
        (Value::DateTime(a), Value::DateTime(b)) if matches!(op, ArithOp::Sub) => {
            Ok(Value::Integer(a.timestamp_millis() - b.timestamp_millis()))
        }
        (Value::DateTime(a), Value::Integer(ms)) if matches!(op, ArithOp::Sub) => {
            Ok(Value::DateTime(a - ChronoDuration::milliseconds(ms)))
        }
        (a, b) => Err(RuntimeError::TypeError(format!(
            "arithmetic type mismatch: {} and {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

/// Evaluate addition (may be string concatenation).
fn eval_add(l: Value, r: Value) -> Result<Value, RuntimeError> {
    if matches!(l, Value::Null) || matches!(r, Value::Null) {
        return Err(RuntimeError::NullError(
            "arithmetic on null value".to_string(),
        ));
    }
    match (l, r) {
        (Value::Str(a), Value::Str(b)) => Ok(Value::Str(a + &b)),
        (Value::Str(_), other) => Err(RuntimeError::TypeError(format!(
            "string concatenation requires string operand, got {}",
            other.type_name()
        ))),
        (other, Value::Str(_)) => Err(RuntimeError::TypeError(format!(
            "string concatenation requires string operand, got {}",
            other.type_name()
        ))),
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + b as f64)),
        (Value::Currency(a), Value::Currency(b)) => Ok(Value::Currency(a + b)),
        // Currency mixed with Float or Integer: promote to float.
        (Value::Currency(c), Value::Float(f)) | (Value::Float(f), Value::Currency(c)) => {
            Ok(Value::Float(currency_as_float(c) + f))
        }
        (Value::Currency(c), Value::Integer(n)) | (Value::Integer(n), Value::Currency(c)) => {
            Ok(Value::Float(currency_as_float(c) + n as f64))
        }
        (Value::DateTime(dt), Value::Integer(ms)) => {
            Ok(Value::DateTime(dt + ChronoDuration::milliseconds(ms)))
        }
        (Value::Integer(ms), Value::DateTime(dt)) => {
            Ok(Value::DateTime(dt + ChronoDuration::milliseconds(ms)))
        }
        (a, b) => Err(RuntimeError::TypeError(format!(
            "add type mismatch: {} and {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

fn eval_div(l: Value, r: Value) -> Result<Value, RuntimeError> {
    if matches!(l, Value::Null) || matches!(r, Value::Null) {
        return Err(RuntimeError::NullError(
            "arithmetic on null value".to_string(),
        ));
    }
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => {
            if b == 0 {
                Err(RuntimeError::DivisionError)
            } else {
                // Truncate toward zero
                Ok(Value::Integer(a / b))
            }
        }
        (Value::Float(a), Value::Float(b)) => {
            if b == 0.0 {
                Err(RuntimeError::DivisionError)
            } else {
                Ok(Value::Float(a / b))
            }
        }
        (Value::Integer(a), Value::Float(b)) => {
            if b == 0.0 {
                Err(RuntimeError::DivisionError)
            } else {
                Ok(Value::Float(a as f64 / b))
            }
        }
        (Value::Float(a), Value::Integer(b)) => {
            if b == 0 {
                Err(RuntimeError::DivisionError)
            } else {
                Ok(Value::Float(a / b as f64))
            }
        }
        // Currency mixed with Float or Integer: promote to float.
        (Value::Currency(c), Value::Float(f)) => {
            if f == 0.0 {
                Err(RuntimeError::DivisionError)
            } else {
                Ok(Value::Float(currency_as_float(c) / f))
            }
        }
        (Value::Float(f), Value::Currency(c)) => {
            if c == 0 {
                Err(RuntimeError::DivisionError)
            } else {
                Ok(Value::Float(f / currency_as_float(c)))
            }
        }
        (Value::Currency(c), Value::Integer(n)) => {
            if n == 0 {
                Err(RuntimeError::DivisionError)
            } else {
                Ok(Value::Float(currency_as_float(c) / n as f64))
            }
        }
        (Value::Integer(n), Value::Currency(c)) => {
            if c == 0 {
                Err(RuntimeError::DivisionError)
            } else {
                Ok(Value::Float(n as f64 / currency_as_float(c)))
            }
        }
        (a, b) => Err(RuntimeError::TypeError(format!(
            "division type mismatch: {} and {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

fn eval_mod(l: Value, r: Value) -> Result<Value, RuntimeError> {
    if matches!(l, Value::Null) || matches!(r, Value::Null) {
        return Err(RuntimeError::NullError(
            "arithmetic on null value".to_string(),
        ));
    }
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => {
            if b == 0 {
                Err(RuntimeError::DivisionError)
            } else {
                Ok(Value::Integer(a % b))
            }
        }
        (a, b) => Err(RuntimeError::TypeError(format!(
            "modulo requires integer operands, got {} and {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

fn eval_pow(l: Value, r: Value) -> Result<Value, RuntimeError> {
    if matches!(l, Value::Null) || matches!(r, Value::Null) {
        return Err(RuntimeError::NullError(
            "arithmetic on null value".to_string(),
        ));
    }
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => {
            if b < 0 {
                Ok(Value::Float((a as f64).powi(b as i32)))
            } else {
                Ok(Value::Integer(a.pow(b as u32)))
            }
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((a as f64).powf(b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powi(b as i32))),
        // Currency mixed with Float or Integer: promote to float.
        (Value::Currency(c), Value::Float(f)) => Ok(Value::Float(currency_as_float(c).powf(f))),
        (Value::Float(f), Value::Currency(c)) => Ok(Value::Float(f.powf(currency_as_float(c)))),
        (Value::Currency(c), Value::Integer(n)) => {
            Ok(Value::Float(currency_as_float(c).powi(n as i32)))
        }
        (Value::Integer(n), Value::Currency(c)) => {
            Ok(Value::Float((n as f64).powf(currency_as_float(c))))
        }
        (a, b) => Err(RuntimeError::TypeError(format!(
            "exponentiation type mismatch: {} and {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

/// Convert currency cents to its float equivalent.
///
/// Used for Currency→Float promotion in mixed-type arithmetic.
#[inline]
fn currency_as_float(cents: i64) -> f64 {
    cents as f64 / 100.0
}

/// Coerce an initial value to match the variable's declared type.
///
/// This resolves the lexer ambiguity where a two-decimal-place literal
/// (e.g. `3.14`) is tokenised as `CurrencyLit` regardless of the declared
/// type.  Only numeric types are silently promoted; all other mismatches
/// are left untouched (the runtime will raise a TypeError if they are
/// used incorrectly).
fn coerce_to_declared_type(val: Value, typ: &RundellType) -> Value {
    match (val, typ) {
        // Currency literal used where float is declared → convert to float.
        (Value::Currency(c), RundellType::Float) => Value::Float(currency_as_float(c)),
        // Integer used where float is declared → widen.
        (Value::Integer(n), RundellType::Float) => Value::Float(n as f64),
        // Currency literal used where integer is declared → truncate.
        (Value::Currency(c), RundellType::Integer) => Value::Integer(c / 100),
        // Integer/float/currency used where currency is declared → convert.
        (Value::Integer(n), RundellType::Currency) => Value::Currency(n * 100),
        (Value::Float(f), RundellType::Currency) => Value::Currency((f * 100.0).round() as i64),
        // All other combinations are returned unchanged.
        (v, _) => v,
    }
}

/// Cast a value to a target type identified by name.
fn cast_value(val: Value, target: &str) -> Result<Value, RuntimeError> {
    match target {
        "string" => Ok(Value::Str(val.to_display_string())),
        "integer" => match val {
            Value::Integer(n) => Ok(Value::Integer(n)),
            Value::Float(f) => Ok(Value::Integer(f.trunc() as i64)),
            Value::Boolean(b) => Ok(Value::Integer(if b { 1 } else { 0 })),
            Value::Currency(c) => Ok(Value::Integer(c / 100)),
            Value::DateTime(_) => Err(RuntimeError::TypeError(
                "cannot cast datetime to integer".to_string(),
            )),
            Value::Str(s) => s.parse::<i64>().map(Value::Integer).map_err(|_| {
                RuntimeError::TypeError(format!("cannot cast string '{s}' to integer"))
            }),
            Value::Null => Err(RuntimeError::NullError("cast null to integer".to_string())),
            Value::Json(_) => Err(RuntimeError::TypeError(
                "cannot cast json to integer".to_string(),
            )),
        },
        "float" => match val {
            Value::Float(f) => Ok(Value::Float(f)),
            Value::Integer(n) => Ok(Value::Float(n as f64)),
            Value::Currency(c) => Ok(Value::Float(c as f64 / 100.0)),
            Value::Str(s) => s
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| RuntimeError::TypeError(format!("cannot cast string '{s}' to float"))),
            Value::Null => Err(RuntimeError::NullError("cast null to float".to_string())),
            v => Err(RuntimeError::TypeError(format!(
                "cannot cast {} to float",
                v.type_name()
            ))),
        },
        "boolean" => match val {
            Value::Boolean(b) => Ok(Value::Boolean(b)),
            Value::Integer(n) => Ok(Value::Boolean(n != 0)),
            Value::Str(s) => match s.to_lowercase().as_str() {
                "true" | "yes" => Ok(Value::Boolean(true)),
                "false" | "no" => Ok(Value::Boolean(false)),
                _ => Err(RuntimeError::TypeError(format!(
                    "cannot cast string '{s}' to boolean"
                ))),
            },
            Value::Null => Err(RuntimeError::NullError("cast null to boolean".to_string())),
            v => Err(RuntimeError::TypeError(format!(
                "cannot cast {} to boolean",
                v.type_name()
            ))),
        },
        "currency" => match val {
            Value::Currency(c) => Ok(Value::Currency(c)),
            Value::Integer(n) => Ok(Value::Currency(n * 100)),
            Value::Float(f) => Ok(Value::Currency((f * 100.0).round() as i64)),
            Value::Str(s) => {
                // Parse as float first
                s.parse::<f64>()
                    .map(|f| Value::Currency((f * 100.0).round() as i64))
                    .map_err(|_| {
                        RuntimeError::TypeError(format!("cannot cast string '{s}' to currency"))
                    })
            }
            Value::Null => Err(RuntimeError::NullError("cast null to currency".to_string())),
            v => Err(RuntimeError::TypeError(format!(
                "cannot cast {} to currency",
                v.type_name()
            ))),
        },
        "json" => match val {
            Value::Json(j) => Ok(Value::Json(j)),
            v => Err(RuntimeError::TypeError(format!(
                "cannot cast {} to json",
                v.type_name()
            ))),
        },
        "datetime" => match val {
            Value::DateTime(dt) => Ok(Value::DateTime(dt)),
            Value::Str(s) => parse_datetime_literal(&s).map(Value::DateTime),
            Value::Null => Err(RuntimeError::NullError("cast null to datetime".to_string())),
            v => Err(RuntimeError::TypeError(format!(
                "cannot cast {} to datetime",
                v.type_name()
            ))),
        },
        t => Err(RuntimeError::TypeError(format!("unknown cast target: {t}"))),
    }
}

/// Coerce a string (from stdin) to the declared variable type.
fn coerce_string_to_type(s: &str, typ: &RundellType) -> Result<Value, RuntimeError> {
    match typ {
        RundellType::Str => Ok(Value::Str(s.to_string())),
        RundellType::Integer => s
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| RuntimeError::TypeError(format!("cannot coerce '{s}' to integer"))),
        RundellType::Float => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| RuntimeError::TypeError(format!("cannot coerce '{s}' to float"))),
        RundellType::Boolean => match s.to_lowercase().as_str() {
            "true" | "yes" => Ok(Value::Boolean(true)),
            "false" | "no" => Ok(Value::Boolean(false)),
            _ => Err(RuntimeError::TypeError(format!(
                "cannot coerce '{s}' to boolean"
            ))),
        },
        RundellType::Currency => s
            .parse::<f64>()
            .map(|f| Value::Currency((f * 100.0).round() as i64))
            .map_err(|_| RuntimeError::TypeError(format!("cannot coerce '{s}' to currency"))),
        RundellType::Json => serde_json::from_str(s)
            .map(Value::Json)
            .map_err(|_| RuntimeError::TypeError(format!("cannot coerce '{s}' to json"))),
        RundellType::DateTime => parse_datetime_literal(s).map(Value::DateTime),
    }
}

/// Return the string name of the error variant (for try/catch matching).
fn runtime_error_name(err: &RuntimeError) -> String {
    match err {
        RuntimeError::TypeError(_) => "TypeError".to_string(),
        RuntimeError::NullError(_) => "NullError".to_string(),
        RuntimeError::IndexError(_) => "IndexError".to_string(),
        RuntimeError::DivisionError => "DivisionError".to_string(),
        RuntimeError::IOError(_) => "IOError".to_string(),
        RuntimeError::PermissionError { .. } => "PermissionError".to_string(),
        RuntimeError::RuntimeError(_) => "RuntimeError".to_string(),
        RuntimeError::ReturnValue(_) => "ReturnValue".to_string(),
        RuntimeError::QueryTimeout { .. } => "QueryTimeout".to_string(),
        RuntimeError::QueryNetworkError { .. } => "QueryNetworkError".to_string(),
        RuntimeError::QueryHttpError { .. } => "QueryHttpError".to_string(),
        RuntimeError::QueryInvalidJson { .. } => "QueryInvalidJson".to_string(),
        RuntimeError::UndefinedQuery { .. } => "UndefinedQuery".to_string(),
        RuntimeError::UndefinedCredentials { .. } => "UndefinedCredentials".to_string(),
        RuntimeError::EnvKeyNotFound { .. } => "EnvKeyNotFound".to_string(),
        RuntimeError::EnvDecryptionFailed { .. } => "EnvDecryptionFailed".to_string(),
        RuntimeError::NoProgramPath => "NoProgramPath".to_string(),
        RuntimeError::UnsupportedHttpMethod => "UnsupportedHttpMethod".to_string(),
    }
}

/// Convert a Value to its string representation for property storage.
fn value_to_string(value: &Value) -> String {
    value.to_display_string()
}

/// Returns true if `name` is a built-in function.
fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "newline"
            | "length"
            | "cast"
            | "abs"
            | "floor"
            | "ceil"
            | "round"
            | "substr"
            | "upper"
            | "lower"
            | "trim"
            | "execute"
            | "os"
            | "now"
            | "day"
            | "month"
            | "year"
            | "hour"
            | "minute"
            | "second"
            | "dateformat"
            | "timestamp"
            | "fromtimestamp"
            | "string"
            | "append"
            | "env"
            | "read_text"
            | "write_text"
            | "read_json"
            | "write_json"
            | "read_csv"
            | "write_csv"
    )
}

fn has_mixed_separators(path: &str) -> bool {
    path.contains('/') && path.contains('\\')
}

fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let paths = std::env::split_paths(&path_var);

    #[cfg(windows)]
    let candidates: Vec<String> = {
        let has_ext = Path::new(name).extension().is_some();
        if has_ext {
            vec![name.to_string()]
        } else {
            let pathext = std::env::var_os("PATHEXT")
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let mut list = Vec::new();
            if pathext.is_empty() {
                list.push(format!("{name}.exe"));
            } else {
                for ext in pathext.split(';') {
                    let ext = ext.trim();
                    if ext.is_empty() {
                        continue;
                    }
                    list.push(format!("{name}{ext}"));
                }
            }
            list
        }
    };

    #[cfg(not(windows))]
    let candidates: Vec<String> = vec![name.to_string()];

    for dir in paths {
        for candidate in &candidates {
            let full = dir.join(candidate);
            if full.is_file() {
                return Some(full);
            }
        }
    }
    None
}

fn json_value_to_csv(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Array(arr) => serde_json::Value::Array(arr.clone()).to_string(),
        serde_json::Value::Object(map) => serde_json::Value::Object(map.clone()).to_string(),
    }
}
