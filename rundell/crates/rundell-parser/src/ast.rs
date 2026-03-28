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
    /// A bare expression statement (typically a void function call).
    ExprStmt(Expr),
}
