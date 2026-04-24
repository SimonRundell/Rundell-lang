//! Abstract Syntax Tree node definitions for the Rundell language.
//!
//! This module contains only data types — no parsing logic.

/// All types that a Rundell variable may have.
#[derive(Debug, Clone, PartialEq)]
pub enum RundellType {
    /// 64-bit signed integer.
    Integer,
    /// 64-bit IEEE 754 double.
    Float,
    /// UTF-8 string.
    Str,
    /// Fixed-point currency stored as cents (i64).
    Currency,
    /// Boolean (true/false).
    Boolean,
    /// JSON collection type.
    Json,
    /// ISO 8601 datetime with optional timezone offset.
    DateTime,
    /// Ordered list of a given element type.
    List(Box<RundellType>),
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

/// Binary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    /// `+` (numeric addition or string concatenation)
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,
    /// `**`
    Pow,
    /// `==`
    Eq,
    /// `!=`
    NotEq,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// `and`
    And,
    /// `or`
    Or,
    /// String concatenation (explicit variant used in type-checked context)
    StrConcat,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    /// Arithmetic negation `-expr`
    Neg,
    /// Logical negation `not expr`
    Not,
}

/// A literal value embedded in source code.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer literal.
    Integer(i64),
    /// Float literal.
    Float(f64),
    /// String literal (already unescaped).
    Str(String),
    /// Currency literal stored as integer cents.
    Currency(i64),
    /// Boolean literal.
    Boolean(bool),
    /// Datetime literal in ISO 8601 format.
    DateTime(String),
    /// The keyword `null`.
    Null,
}

/// An expression that can appear wherever a value is expected.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal value.
    Literal(Literal),
    /// A reference to a named variable.
    Identifier(String),
    /// A binary operation.
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    /// A unary operation.
    UnaryOp(UnaryOp, Box<Expr>),
    /// Index access: `collection[key]`.
    Index(Box<Expr>, Box<Expr>),
    /// A function or built-in call: `name(arg1, arg2, ...)`.
    Call(String, Vec<Expr>),
    /// `expr is null`
    IsNull(Box<Expr>),
    /// `expr is not null`
    IsNotNull(Box<Expr>),
    /// A JSON object/array literal.
    JsonLiteral(serde_json::Value),

    // -----------------------------------------------------------------
    // GUI expressions
    // -----------------------------------------------------------------
    /// An object path read: `myForm\myLabel\value` or `rootWindow\myForm\title`.
    ///
    /// Segments are the identifiers between `\` separators in order.
    ObjectPath(Vec<String>),

    /// A pixel dimension value parsed from e.g. `10px`.
    PixelValue(u32),

    /// A duration value parsed from literals like `500ms`, `2s`, `1m`, `1h`.
    /// Stored as milliseconds.
    DurationValue(u64),

    /// A position literal: `top_px, left_px, width_px, height_px`.
    ///
    /// Used exclusively as the right-hand side of
    /// `set <path>\position = top, left, width, height.`
    PositionLiteral(u32, u32, u32, u32),

    /// `rootWindow\myForm\show()` or `rootWindow\myForm\show(modal)`.
    ///
    /// `path` is the object-path segments leading up to (but not including)
    /// the `show` segment.  `modal` is true when `show(modal)` is used.
    ShowForm { path: Vec<String>, modal: bool },

    /// `rootWindow\myForm\close()`.
    ///
    /// `path` is the object-path segments leading up to `close`.
    CloseForm { path: Vec<String> },

    /// A `dialog\*` built-in call.
    Dialog(Box<DialogCall>),

    /// An await expression — evaluates a query call asynchronously.
    /// e.g.  `set results = await myQuery(1).`
    Await(Box<AwaitExpr>),
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

/// A variable declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct DefineStmt {
    /// The variable's name.
    pub name: String,
    /// Declared type.
    pub typ: RundellType,
    /// Whether the variable is immutable after initialisation.
    pub constant: bool,
    /// Whether the variable is declared at global scope.
    pub global: bool,
    /// Optional initialiser expression.
    pub init: Option<Expr>,
}

/// Target of a `set` statement.
#[derive(Debug, Clone, PartialEq)]
pub enum SetTarget {
    /// Simple variable assignment: `set x = ...`
    Identifier(String),
    /// Collection-key assignment: `set col["key"] = ...`
    Index(Box<Expr>, Box<Expr>),
    /// Object-path assignment: `set myForm\myLabel\value = ...`
    ObjectPath(Vec<String>),
}

/// The operation performed by a `set` statement.
#[derive(Debug, Clone, PartialEq)]
pub enum SetOp {
    /// `set x = expr`
    Assign(Expr),
    /// `set x++`
    Increment,
    /// `set x--`
    Decrement,
}

/// A `set` assignment statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SetStmt {
    /// What is being assigned to.
    pub target: SetTarget,
    /// The operation (assign / increment / decrement).
    pub op: SetOp,
}

/// A `receive` input statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceiveStmt {
    /// The variable to store the input in.
    pub variable: String,
    /// Optional prompt string expression.
    pub prompt: Option<Expr>,
}

/// An `if / else if / else` conditional.
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    /// The condition for the `if` branch.
    pub condition: Expr,
    /// Statements in the `if` branch.
    pub then_body: Vec<Stmt>,
    /// Zero or more `else if` branches (condition, body).
    pub else_ifs: Vec<(Expr, Vec<Stmt>)>,
    /// Optional `else` branch body.
    pub else_body: Option<Vec<Stmt>>,
}

/// Comparison operators used in switch case patterns.
#[derive(Debug, Clone, PartialEq)]
pub enum CmpOp {
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// `==`
    Eq,
    /// `!=`
    NotEq,
}

/// A pattern in a `switch` case.
#[derive(Debug, Clone, PartialEq)]
pub enum SwitchPattern {
    /// `< expr`, `>= expr`, etc.
    Comparison(CmpOp, Expr),
    /// An exact match value.
    Exact(Expr),
    /// The `else` default case.
    Default,
}

/// A single `switch` case arm.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    /// The pattern to match against the subject.
    pub pattern: SwitchPattern,
    /// The body to execute when this case matches.
    pub body: Vec<Stmt>,
}

/// A `switch` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStmt {
    /// The expression being switched on.
    pub subject: Expr,
    /// Ordered list of cases.
    pub cases: Vec<SwitchCase>,
}

/// A counted `for` loop.
#[derive(Debug, Clone, PartialEq)]
pub struct ForLoopStmt {
    /// Loop variable name (must be pre-declared as integer).
    pub var: String,
    /// Loop start value (inclusive).
    pub start: Expr,
    /// Loop end value (inclusive).
    pub end: Expr,
    /// Step increment.
    pub increment: Expr,
    /// Loop body.
    pub body: Vec<Stmt>,
}

/// A `while` loop.
#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoopStmt {
    /// Continuation condition.
    pub condition: Expr,
    /// Loop body.
    pub body: Vec<Stmt>,
}

/// A `for each` collection iterator.
#[derive(Debug, Clone, PartialEq)]
pub struct ForEachStmt {
    /// Iteration variable name (implicitly declared per iteration).
    pub var: String,
    /// Expression that must evaluate to a JSON array.
    pub collection: Expr,
    /// Loop body.
    pub body: Vec<Stmt>,
}

/// A function parameter declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub typ: RundellType,
}

/// A function definition.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefStmt {
    /// Function name.
    pub name: String,
    /// Ordered list of parameters.
    pub params: Vec<Param>,
    /// Return type (`None` means `returns null` / void).
    pub return_type: Option<RundellType>,
    /// Function body statements.
    pub body: Vec<Stmt>,
}

/// A single `catch` clause in a try/catch block.
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    /// The error type name this clause catches (e.g. `"TypeError"`).
    pub error_type: String,
    /// Handler body.
    pub body: Vec<Stmt>,
}

/// A `try / catch / finally` error-handling block.
#[derive(Debug, Clone, PartialEq)]
pub struct TryCatchStmt {
    /// The guarded body.
    pub try_body: Vec<Stmt>,
    /// One or more catch handlers.
    pub catches: Vec<CatchClause>,
    /// Optional finally body (always runs).
    pub finally_body: Option<Vec<Stmt>>,
}

/// A top-level statement in a Rundell program.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `import "path"` — load another module.
    Import(String),
    /// A variable (or function) declaration.
    Define(DefineStmt),
    /// An assignment statement.
    Set(SetStmt),
    /// `print expr`
    Print(Expr),
    /// `debug [("path")] expr` — emit timestamped output to stdout or a file.
    Debug(Option<Expr>, Expr),
    /// `receive identifier [with prompt expr]`
    Receive(ReceiveStmt),
    /// `if ... else if ... else ...`
    If(IfStmt),
    /// `switch expr --> cases <--`
    Switch(SwitchStmt),
    /// `for i loops (start, end, step) --> body <--`
    ForLoop(ForLoopStmt),
    /// `while cond --> body <--`
    WhileLoop(WhileLoopStmt),
    /// `for each var in collection --> body <--`
    ForEach(ForEachStmt),
    /// A function definition.
    FunctionDef(FunctionDefStmt),
    /// `return [expr]`
    Return(Option<Expr>),
    /// `try --> body catch(...) --> body [finally --> body] <--`
    TryCatch(TryCatchStmt),
    /// `remove expr` — remove a key from a collection.
    Remove(Expr),
    /// `append(collection, value)` — append an element to a JSON array.
    Append(Expr, Expr),
    /// A bare expression statement (typically a void function call or
    /// `show()`/`close()` call).
    ExprStmt(Expr),

    // -----------------------------------------------------------------
    // GUI statements
    // -----------------------------------------------------------------
    /// `define name as form --> ... <--`
    FormDef(FormDefinition),
    /// `define name as eventtimer --> ... <--`
    EventTimerDef(EventTimerDefinition),
    /// `define name as form\controltype.`  (inside a form body)
    DefineControl(String, ControlType),

    // -----------------------------------------------------------------
    // REST / query statements
    // -----------------------------------------------------------------
    /// `define name as credentials --> ... <--`
    CredentialsDef(CredentialsDefinition),
    /// `define name(params) as query returns json --> ... <--`
    QueryDef(QueryDefinition),
    /// `attempt --> ... <-- catch id --> ... <--`
    Attempt(AttemptBlock),
}

// ===========================================================================
// GUI AST nodes
// ===========================================================================

/// The type of a GUI control.
#[derive(Debug, Clone, PartialEq)]
pub enum ControlType {
    /// Static text label.
    Label,
    /// Single-line text input.
    Textbox,
    /// Clickable button.
    Button,
    /// Mutually-exclusive radio button.
    Radiobutton,
    /// Boolean tick-box.
    Checkbox,
    /// Yes/no toggle switch.
    Switch,
    /// Dropdown single-choice selector.
    Select,
    /// Multi-column data-bound list.
    Listbox,
}

/// A complete form definition block.
///
/// The `body` is a flat list of statements that are executed at form
/// registration time to configure the form and its controls:
/// - `Stmt::DefineControl` declarations
/// - `Stmt::Set` with `SetTarget::ObjectPath` for property/event assignments
#[derive(Debug, Clone, PartialEq)]
pub struct FormDefinition {
    /// The form's Rundell identifier (used as the key in `rootWindow.forms`).
    pub name: String,
    /// Statements inside the `define name as form --> ... <--` block.
    pub body: Vec<Stmt>,
}

/// A named event timer definition block.
///
/// The `body` is a flat list of statements executed at registration time to
/// configure the timer, typically `set <timer>\interval = ...` and
/// `set <timer>\event = callback()`.
#[derive(Debug, Clone, PartialEq)]
pub struct EventTimerDefinition {
    /// The timer's identifier.
    pub name: String,
    /// Statements inside the `define name as eventtimer --> ... <--` block.
    pub body: Vec<Stmt>,
}

/// Message box kind for `dialog\message(...)`.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageKind {
    /// Single OK button.
    Ok,
    /// OK and Cancel buttons.
    OkCancel,
    /// Yes and No buttons.
    YesNo,
}

/// A `dialog\*` built-in call expression.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogCall {
    /// `dialog\openfile(title, filter)` — returns selected path or `""`.
    OpenFile { title: Box<Expr>, filter: Box<Expr> },
    /// `dialog\savefile(title, filter)` — returns chosen path or `""`.
    SaveFile { title: Box<Expr>, filter: Box<Expr> },
    /// `dialog\message(title, message, kind)` — returns `"ok"`, `"cancel"`,
    /// `"yes"`, or `"no"`.
    Message {
        title: Box<Expr>,
        message: Box<Expr>,
        kind: MessageKind,
    },
    /// `dialog\colorpicker(initial)` — returns chosen `"#RRGGBB"` or `initial`.
    ColorPicker { initial: Box<Expr> },
}

// ===========================================================================
// REST / query AST nodes
// ===========================================================================

/// The HTTP method for a query.
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
}

/// A credentials definition — a named, reusable authentication block.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialsDefinition {
    /// The Rundell identifier for this credentials object.
    pub name: String,
    /// The JWT bearer token expression (typically an env() call).
    pub token: Option<Expr>,
    /// The authentication value expression (typically an env() call).
    pub authentication: Option<Expr>,
}

/// A query definition — a named, parameterised REST call.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryDefinition {
    /// The Rundell identifier for this query.
    pub name: String,
    /// Optional parameters accepted by the query when called.
    pub params: Vec<Param>,
    /// The HTTP method (GET or POST).
    pub method: HttpMethod,
    /// The endpoint URL expression.
    pub endpoint: Expr,
    /// Reference to a CredentialsDefinition by name, if any.
    pub credentials: Option<String>,
    /// Optional per-query timeout in milliseconds.
    pub timeout_ms: Option<Expr>,
    /// The queryParams JSON expression (POST only). None for GET requests.
    pub query_params: Option<Expr>,
}

/// An await expression — evaluates a query call asynchronously.
/// e.g.  `set results = await myQuery(1).`
#[derive(Debug, Clone, PartialEq)]
pub struct AwaitExpr {
    /// The function call expression (must resolve to a QueryDefinition call).
    pub call: Box<Expr>,
}

/// An attempt/catch block for error handling around query calls.
#[derive(Debug, Clone, PartialEq)]
pub struct AttemptBlock {
    /// Statements inside the attempt block.
    pub body: Vec<Stmt>,
    /// Identifier bound in the catch block.
    pub error_name: String,
    /// Statements inside the catch block.
    pub handler: Vec<Stmt>,
}
