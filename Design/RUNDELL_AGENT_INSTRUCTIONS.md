# RUNDELL INTERPRETER — AGENT BUILD INSTRUCTIONS
# =================================================
# Target agent   : Claude Opus (claude-opus-4-5)
# Implementation : Rust (stable toolchain)
# Platform       : Windows x86-64 (cross-platform compatible)
# Architecture   : Tree-walk interpreter (no compilation step required)
# Output         : Single binary  rundell.exe  (or  rundell  on Unix)
#
# HOW TO USE THIS FILE
# --------------------
# Read this entire document before writing a single line of code.
# Work through the PHASES in order. Do not skip ahead.
# After completing each phase, run the tests defined for that phase
# before moving to the next. Ask no clarifying questions — all
# decisions are made for you below. Where genuine ambiguity exists
# it is noted and a resolution is prescribed.
# =================================================


# ═══════════════════════════════════════════════════════════════════
# PART 1 — THE RUNDELL LANGUAGE SPECIFICATION
# This is the authoritative definition of the language you are
# implementing. Implement EXACTLY what is written here.
# ═══════════════════════════════════════════════════════════════════

## 1.1  Overview

Rundell is a strongly-typed, structured, imperative, interpreted
programming language. It is designed to be English-readable. All
variables must be declared before use. All variables are strictly
typed. Scope is function-local or global. The language has no
implicit type coercion except where explicitly stated below.


## 1.2  Source Files

- File extension  : .run
- Encoding        : UTF-8
- Line endings    : LF or CRLF (both accepted)


## 1.3  Statement Terminator

Every statement ends with a full stop  ( . )
A statement may span multiple lines. The parser continues reading
tokens until it encounters the terminating full stop.

DISAMBIGUATION RULE:
  A full stop that is immediately preceded AND followed by a decimal
  digit is a decimal point within a numeric literal.
  Any other full stop is a statement terminator.

Examples:
  define x as float = 3.14.    # "3.14" → decimal point; trailing "." → terminator
  define y as integer = 10.    # trailing "." → terminator


## 1.4  Comments

  # comment to end of line

Comments may appear on a line by themselves or at the end of a
statement, after the terminating full stop or anywhere in the token
stream that is not inside a string literal.


## 1.5  Identifiers and Naming

Rules:
  - Alphanumeric plus underscore ( _ ) only
  - Must begin with a letter (a-z, A-Z) — never a digit or underscore
  - camelCase recommended; lowercase, UPPERCASE, PascalCase, snake_case all permitted
  - Leading underscore is FORBIDDEN (reserved for implementation use)
  - Identifiers are case-sensitive:  myVar  ≠  MyVar

Keywords (reserved — may not be used as identifiers):
  define  as  constant  global  set  return  import
  if  else  switch  for  while  each  in  loops
  true  false  yes  no  TRUE  FALSE  YES  NO
  null  and  or  not  is
  print  receive  with  prompt
  try  catch  finally
  integer  float  string  currency  boolean  json
  cast  length  newline  abs  floor  ceil  round
  substr  upper  lower  trim  string  append  remove
  TypeError  NullError  IndexError  DivisionError  IOError  RuntimeError
  returns  null


## 1.6  Data Types

  integer    64-bit signed integer
  float      64-bit IEEE 754 double
  string     UTF-8 string
  currency   Fixed-precision decimal, always 2 decimal places (use Rust's
             own fixed-point representation: store as i64 of cents)
  boolean    Logical value; literals  true / TRUE / yes / YES  are  1
                                      false / FALSE / no / NO   are  0
  json       Collection type — see §1.12


## 1.7  String Literals

Delimited by " or '.
The FIRST delimiter used is the boundary character.
The other delimiter may appear freely inside without escaping.

  "This is my 'thing' and I love it."
  'This is my "thing" and I love it.'

Escape sequences (resolve BEFORE the string is stored):
  \n    newline (LF, 0x0A)
  \r    carriage return (0x0D)
  \t    horizontal tab (0x09)
  \'    literal single quote
  \"    literal double quote
  \\    literal backslash

Multi-line string literals are permitted: the newline characters
within the source are included literally in the string value.


## 1.8  Variable Declaration

Syntax:
  define <identifier> as [constant] <type> [= <expression>].
  define <identifier> as [constant] global <type> [= <expression>].

Rules:
  - The  constant  modifier makes the variable immutable after initial
    assignment. Attempting to  set  a constant is a compile-time error.
  - The  global  modifier declares the variable at program scope.
    Global declarations may only appear at the top level (outside any
    function body). They are visible everywhere in the program.
  - A variable declared without  = <expression>  has initial value  null.
  - All variables must be declared before their first use.
  - Re-declaring an identifier in the same scope is an error.

Examples:
  define myName     as constant string  = "Simon".
  define myAge      as integer           = 21.
  define accountBal as currency          = 1000.00.
  define isReady    as boolean.
  define global maxUsers as constant integer = 100.
  define global sessionCount as integer = 0.


## 1.9  Assignment

  set <identifier> = <expression>.
  set <identifier>++.              # increment integer by 1
  set <identifier>--.              # decrement integer by 1

Rules:
  - Assigning to a different type is a TypeError unless explicitly cast.
  - ++ and -- are only valid on integer variables.
  - Assignment to a constant is a compile-time error.


## 1.10  Operators

ARITHMETIC (left-associative, standard precedence):
  +    addition
  -    subtraction
  *    multiplication
  /    division:
         both operands integer → integer result (truncate toward zero)
         either operand float  → float result
  %    modulo (remainder), integers only
  **   exponentiation (right-associative)

COMPARISON (return boolean):
  ==   equal
  !=   not equal
  <    less than
  <=   less than or equal
  >    greater than
  >=   greater than or equal

LOGICAL (boolean operands):
  and  logical AND
  or   logical OR
  not  logical NOT (unary prefix)

STRING:
  +    concatenation (both operands must be string; otherwise TypeError)

NULL CHECK (special expression form, not an operator):
  <expr> is null
  <expr> is not null

OPERATOR PRECEDENCE (highest to lowest):
  1. Unary  not  -  (negation)
  2. **
  3. *  /  %
  4. +  -
  5. <  <=  >  >=  ==  !=
  6. and
  7. or
  8. is null / is not null

Parentheses override precedence in the usual way.


## 1.11  Type Casting

  cast(<expression>, <targetType>)

Converts expression to targetType. Raises TypeError if the
conversion is impossible (e.g. cast("abc", integer)).

Permitted casts:
  integer ↔ float      (float→integer truncates toward zero)
  integer ↔ string     (string must be a valid integer literal)
  integer ↔ boolean    (0 → false; non-zero → true; true→1 false→0)
  integer ↔ currency   (integer becomes .00)
  float   ↔ string     (string must be a valid float literal)
  float   ↔ currency   (truncated/rounded to 2 dp)
  boolean ↔ string     ("true"/"false")
  any     →  string    (always permitted; produces human-readable form)

Example from spec:
  define value1 as integer = 5.
  define value2 as integer = 2.
  print "Integer division: " + cast(value1 / value2, string) + newline().
  # output: Integer division: 2
  print "Float division: " + cast(cast(value1, float) / cast(value2, float), string) + newline().
  # output: Float division: 2.5


## 1.12  Collections  ( json )

The json type is a free-form hierarchical key-value store whose
structure mirrors JSON exactly.

Declaration:
  define myData as json = {
    "retrieved": [
      { "recordId": 1, "firstName": "Simon", "age": 58 },
      { "recordId": 2, "firstName": "James", "age": 25 },
      { "recordId": 3, "firstName": "John",  "age": 45 }
    ]
  }.

Access — BOTH forms are valid:
  myData["retrieved"][1]["firstName"]   # named key → "James"
  myData[0][1][0]                       # positional index (0-based, key order as defined)

Collection operations (these are STATEMENTS, not expressions, except
where noted):
  set col["key"] = value.                  # set / add a key-value pair
  remove col["key"].                       # remove a key-value pair
  append(col["arrayKey"], value).          # append element to array
  length(col)                              # EXPRESSION → integer element count

length() counts:
  - top-level keys if the collection is a map
  - elements if the collection is an array


## 1.13  Built-in Functions

All built-ins are called as expressions and may appear wherever an
expression of the appropriate return type is valid.

  newline()                → string   : returns "\n" (LF character)
  length(expr)             → integer  : string length OR collection size
  cast(expr, type)         → <type>   : type conversion (see §1.11)
  abs(expr)                → number   : absolute value
  floor(expr)              → integer  : round down
  ceil(expr)               → integer  : round up
  round(expr, dp)          → float    : round to dp decimal places
  substr(str, start, len)  → string   : 0-based substring
  upper(str)               → string   : to uppercase
  lower(str)               → string   : to lowercase
  trim(str)                → string   : strip leading/trailing whitespace
  append(col, val)         → null     : mutates collection in place
  string(expr)             → string   : shorthand for cast(expr, string)


## 1.14  Input and Output

OUTPUT:
  print <expression>.

  - Writes the string representation of expression to stdout.
  - Does NOT append a newline automatically.
  - Use  newline()  for line breaks:
      print "Hello" + newline().
  - A function call's return value may be used directly:
      print multiply(3, 4).
      print "Result: " + string(multiply(3, 4)) + newline().

INPUT:
  receive <identifier> [with prompt <stringExpr>].

  - Reads one line from stdin into the variable.
  - The optional  with prompt  clause prints the prompt string
    before waiting, without a trailing newline.
  - Input is received as a string; the runtime coerces it to the
    declared type of the variable. Coercion failure → TypeError.

Examples:
  receive myName with prompt "Enter your name: ".
  receive myAge  with prompt "Enter your age: ".
  receive rawData.


## 1.15  Functions

Declaration:
  define <name>(<param> as <type>, ...) returns <type> -->
      <body>
  <--

Rules:
  - Parameters are local, immutable within the function body.
  - A void function declares  returns null  and executes
    return null.  as its last statement.
  - The function body is delimited by  -->  and  <--.
  - Variables declared inside a function are local to it.
  - Globals may be read inside a function but not re-declared.
  - Functions must be declared before they are called (top-down order).
  - Recursion is permitted.

Examples:
  define multiply(value1 as integer, value2 as integer) returns integer -->
      define result as integer.
      set result = value1 * value2.
      return result.
  <--

  define greet(name as string) returns null -->
      print "Hello, " + name + newline().
      return null.
  <--

Calling:
  set area = multiply(4, 5).
  print multiply(3, 4).
  print "Area: " + string(multiply(width, height)) + newline().
  greet("Simon").


## 1.16  Conditional Statements

IF / ELSE IF / ELSE:
  if (<condition>) -->
      <body>
  else if (<condition>) -->
      <body>
  else -->
      <body>
  <--

  - Parentheses around the condition are optional but recommended.
  - A single  <--  closes the entire if/else chain.
  - Logical operators  and  or  not  may combine sub-conditions.

SWITCH:
  switch <expression> -->
      <case> : <statement>.
      <case> : <statement>.
      else   : <statement>.
  <--

  - Each case is a comparison value or a comparison expression
    (< 18, == 42, etc.).
  - Cases are tested top-to-bottom; first match wins.
  - There is an IMPLICIT BREAK after each matched case — no fall-through.
  - Cases may be GROUPED by stacking them on consecutive lines
    before a shared action:
      18 :
      19 : print "Newly adult".
  - The  else  case is the default (required).

Example:
  switch age -->
      < 13  : print "Child" + newline().
      < 18  : print "Teenager" + newline().
      18    :
      19    : print "Newly adult (18 or 19)" + newline().
      else  : print "Adult" + newline().
  <--


## 1.17  Loops

FOR (counted):
  for <identifier> loops (<start>, <end>, <increment>) -->
      <body>
  <--

  - The loop variable must be declared (as integer) before the loop.
  - The loop is INCLUSIVE of both start and end.
  - The loop variable remains in scope after the loop exits.
  - start, end, increment may be literals OR integer expressions.

  Example:
    define i as integer.
    for i loops (0, 5, 1) -->
        print string(i) + newline().
    <--
    # output: 0 1 2 3 4 5

WHILE:
  while <condition> -->
      <body>
  <--

  Example:
    define count as integer = 0.
    define limit as integer = 5.
    while count <= limit -->
        print string(count) + newline().
        set count++.
    <--
    # output: 0 1 2 3 4 5

FOR EACH (collection iterator):
  for each <identifier> in <collectionExpr> -->
      <body>
  <--

  - The iteration variable is implicitly declared as json for each
    iteration. It must NOT be pre-declared by the user.
  - collectionExpr must evaluate to a json array.

  Example:
    for each record in myData["retrieved"] -->
        print record["firstName"] + " is " + string(record["age"]) + newline().
    <--
    # output:
    # Simon is 58
    # James is 25
    # John is 45


## 1.18  Error Handling

  try -->
      <body>
  catch (<ErrorType>) -->
      <body>
  [catch (<ErrorType>) --> ...]
  [finally -->
      <body>]
  <--

  - Multiple  catch  clauses are permitted.
  - The  finally  clause is optional; it always executes.
  - A single  <--  closes the entire try/catch/finally structure.

Standard error types (these are identifiers in the language):
  TypeError       type mismatch or invalid cast
  NullError       accessing a null variable
  IndexError      collection index out of bounds
  DivisionError   division by zero
  IOError         input/output failure
  RuntimeError    catch-all for any other runtime error

Example:
  try -->
      receive userAge with prompt "Enter your age: ".
      set userAge = cast(userAge, integer).
      print "Your age is " + string(userAge) + newline().
  catch (TypeError) -->
      print "That was not a valid integer." + newline().
  catch (RuntimeError) -->
      print "An unexpected error occurred." + newline().
  finally -->
      print "Done." + newline().
  <--


## 1.19  Modules and Imports

  import "<path>".

  - Must appear at the TOP of the file, before any declarations.
  - Path is relative to the importing file; no leading slash.
  - The  .run  extension is OMITTED in the import statement.
  - All top-level non-local definitions in the imported file are
    made available in the importing file.
  - Circular imports are detected at parse time and are an error.

Examples:
  import "mathUtils".
  import "lib/strings".


# ═══════════════════════════════════════════════════════════════════
# PART 2 — CARGO WORKSPACE STRUCTURE
# ═══════════════════════════════════════════════════════════════════

Build the project as a Cargo workspace with the following structure.
Create ALL files and directories exactly as shown.

  rundell/                          ← workspace root
  ├── Cargo.toml                    ← workspace manifest
  ├── README.md
  ├── .gitignore
  │
  ├── crates/
  │   ├── rundell-lexer/            ← tokeniser crate
  │   │   ├── Cargo.toml
  │   │   └── src/
  │   │       ├── lib.rs
  │   │       └── token.rs
  │   │
  │   ├── rundell-parser/           ← parser + AST crate
  │   │   ├── Cargo.toml
  │   │   └── src/
  │   │       ├── lib.rs
  │   │       ├── ast.rs
  │   │       └── parser.rs
  │   │
  │   ├── rundell-interpreter/      ← tree-walk evaluator crate
  │   │   ├── Cargo.toml
  │   │   └── src/
  │   │       ├── lib.rs
  │   │       ├── environment.rs
  │   │       ├── evaluator.rs
  │   │       └── error.rs
  │   │
  │   └── rundell-cli/              ← binary entry point crate
  │       ├── Cargo.toml
  │       └── src/
  │           └── main.rs
  │
  └── tests/                        ← integration test .run files
      ├── 01_variables.run
      ├── 02_arithmetic.run
      ├── 03_strings.run
      ├── 04_boolean.run
      ├── 05_casting.run
      ├── 06_conditionals.run
      ├── 07_loops.run
      ├── 08_functions.run
      ├── 09_collections.run
      ├── 10_error_handling.run
      └── 11_modules/
          ├── main.run
          └── helper.run


# ═══════════════════════════════════════════════════════════════════
# PART 3 — DEPENDENCIES  ( Cargo.toml contents )
# ═══════════════════════════════════════════════════════════════════

## Workspace Cargo.toml

  [workspace]
  resolver = "2"
  members = [
      "crates/rundell-lexer",
      "crates/rundell-parser",
      "crates/rundell-interpreter",
      "crates/rundell-cli",
  ]

## rundell-lexer/Cargo.toml

  [package]
  name    = "rundell-lexer"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  logos = "0.14"

## rundell-parser/Cargo.toml

  [package]
  name    = "rundell-parser"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  rundell-lexer = { path = "../rundell-lexer" }
  thiserror     = "1"

## rundell-interpreter/Cargo.toml

  [package]
  name    = "rundell-interpreter"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  rundell-parser = { path = "../rundell-parser" }
  rundell-lexer  = { path = "../rundell-lexer"  }
  thiserror      = "1"
  serde_json     = "1"
  serde          = { version = "1", features = ["derive"] }

## rundell-cli/Cargo.toml

  [package]
  name    = "rundell-cli"
  version = "0.1.0"
  edition = "2021"

  [[bin]]
  name = "rundell"
  path = "src/main.rs"

  [dependencies]
  rundell-interpreter = { path = "../rundell-interpreter" }
  rundell-parser      = { path = "../rundell-parser"      }
  rundell-lexer       = { path = "../rundell-lexer"       }
  rustyline           = "14"
  miette              = { version = "7", features = ["fancy"] }
  clap                = { version = "4", features = ["derive"] }


# ═══════════════════════════════════════════════════════════════════
# PART 4 — IMPLEMENTATION PHASES
# Build and test each phase before starting the next.
# ═══════════════════════════════════════════════════════════════════

## PHASE 1 — Workspace Scaffold

Actions:
  1. Create the workspace directory structure (Part 2 above).
  2. Write all Cargo.toml files (Part 3 above).
  3. Write stub  lib.rs  and  main.rs  files so that  cargo build
     succeeds with zero errors (stubs may be empty or contain a
     placeholder  todo!() ).
  4. Write  .gitignore  (standard Rust: target/, *.lock excluded
     from tracking is fine).
  5. Write  README.md  with one-paragraph project description and
     build / run instructions.

Phase 1 success criterion:
  cargo build --workspace   exits with code 0.


## PHASE 2 — Lexer  ( rundell-lexer )

Implement a complete lexer using the  logos  crate.

### token.rs

Define an enum  Token  with  #[derive(logos::Logos, Debug, Clone, PartialEq)]
covering every token in the language. Include:

LITERALS:
  Integer(i64)
  Float(f64)
  StringLit(String)       ← store the already-unescaped string value
  CurrencyLit(i64)        ← store as integer cents (e.g. 19.99 → 1999)
  BoolTrue                ← matches  true TRUE yes YES
  BoolFalse               ← matches  false FALSE no NO

KEYWORDS (one variant each):
  Define, As, Constant, Global, Set, Return, Import
  If, Else, Switch, For, While, Each, In, Loops
  And, Or, Not, Is
  Null
  Print, Receive, With, Prompt
  Try, Catch, Finally
  KwInteger, KwFloat, KwString, KwCurrency, KwBoolean, KwJson
  Cast, Length, Newline, Abs, Floor, Ceil, Round
  Substr, Upper, Lower, Trim, Append, Remove
  Returns
  KwTypeError, KwNullError, KwIndexError, KwDivisionError, KwIOError, KwRuntimeError

OPERATORS AND PUNCTUATION:
  Plus, Minus, Star, Slash, Percent, StarStar
  Eq, EqEq, BangEq, Lt, LtEq, Gt, GtEq
  PlusPlus, MinusMinus
  LParen, RParen, LBracket, RBracket, LBrace, RBrace
  Comma, Colon, Dot
  Arrow     ← matches  -->
  ArrowEnd  ← matches  <--

IDENTIFIER:
  Ident(String)

WHITESPACE / NEWLINES (skip silently):
  #[regex(r"[ \t\r\n]+", logos::skip)]

COMMENTS (skip silently):
  #[regex(r"#[^\n]*", logos::skip)]

ERROR:
  #[error]
  Error

### lib.rs  (lexer crate)

Expose a public function:
  pub fn lex(source: &str) -> Vec<(Token, std::ops::Range<usize>)>

  - Run logos over the source.
  - Collect all (token, span) pairs.
  - On encountering Token::Error emit a descriptive panic or return
    a Result — use Result<Vec<(Token, Span)>, LexError> with a
    custom LexError type.

Phase 2 success criteria:
  cargo test -p rundell-lexer

Write unit tests in token.rs or a tests/ module covering at minimum:
  - Integer literal           42
  - Float literal             3.14
  - String literals           both quoting styles and all escape sequences
  - Currency literal          19.99
  - Boolean literals          all 4 true-synonyms and all 4 false-synonyms
  - All keywords
  - Identifier
  - -->  and  <--
  - Comment is skipped
  - Multiline statement       define x \n as integer = 5.
  - Decimal point vs terminator: define f as float = 3.14.


## PHASE 3 — AST  ( rundell-parser / ast.rs )

Define the AST as pure Rust data types (enums + structs).
No parsing logic in this file — only the node types.

### Top level

  pub enum Stmt {
      Import(String),                           // path
      Define(DefineStmt),
      Set(SetStmt),
      Print(Expr),
      Receive(ReceiveStmt),
      If(IfStmt),
      Switch(SwitchStmt),
      ForLoop(ForLoopStmt),
      WhileLoop(WhileLoopStmt),
      ForEach(ForEachStmt),
      FunctionDef(FunctionDefStmt),
      Return(Option<Expr>),
      TryCatch(TryCatchStmt),
      Remove(Expr),                             // collection key removal
      Append(Expr, Expr),                       // collection, value
      ExprStmt(Expr),                           // bare function call
  }

### Types

  pub enum RundellType {
      Integer, Float, Str, Currency, Boolean, Json,
  }

### DefineStmt

  pub struct DefineStmt {
      pub name:     String,
      pub typ:      RundellType,
      pub constant: bool,
      pub global:   bool,
      pub init:     Option<Expr>,
  }

### SetStmt

  pub enum SetTarget {
      Identifier(String),
      Index(Box<Expr>, Box<Expr>),   // collection[key] = value
  }

  pub enum SetOp {
      Assign(Expr),
      Increment,
      Decrement,
  }

  pub struct SetStmt {
      pub target: SetTarget,
      pub op:     SetOp,
  }

### ReceiveStmt

  pub struct ReceiveStmt {
      pub variable: String,
      pub prompt:   Option<Expr>,
  }

### IfStmt

  pub struct IfStmt {
      pub condition:   Expr,
      pub then_body:   Vec<Stmt>,
      pub else_ifs:    Vec<(Expr, Vec<Stmt>)>,
      pub else_body:   Option<Vec<Stmt>>,
  }

### SwitchStmt

  pub struct SwitchCase {
      pub pattern:  SwitchPattern,
      pub body:     Vec<Stmt>,
  }

  pub enum SwitchPattern {
      Comparison(CmpOp, Expr),   // < 18, >= 65, etc.
      Exact(Expr),               // 18, "hello", etc.
      Default,
  }

  pub enum CmpOp { Lt, LtEq, Gt, GtEq, Eq, NotEq }

  pub struct SwitchStmt {
      pub subject: Expr,
      pub cases:   Vec<SwitchCase>,
  }

### Loop statements

  pub struct ForLoopStmt {
      pub var:       String,
      pub start:     Expr,
      pub end:       Expr,
      pub increment: Expr,
      pub body:      Vec<Stmt>,
  }

  pub struct WhileLoopStmt {
      pub condition: Expr,
      pub body:      Vec<Stmt>,
  }

  pub struct ForEachStmt {
      pub var:        String,
      pub collection: Expr,
      pub body:       Vec<Stmt>,
  }

### FunctionDefStmt

  pub struct Param {
      pub name: String,
      pub typ:  RundellType,
  }

  pub struct FunctionDefStmt {
      pub name:        String,
      pub params:      Vec<Param>,
      pub return_type: Option<RundellType>,   // None == returns null
      pub body:        Vec<Stmt>,
  }

### TryCatchStmt

  pub struct CatchClause {
      pub error_type: String,
      pub body:       Vec<Stmt>,
  }

  pub struct TryCatchStmt {
      pub try_body:     Vec<Stmt>,
      pub catches:      Vec<CatchClause>,
      pub finally_body: Option<Vec<Stmt>>,
  }

### Expressions

  pub enum Expr {
      Literal(Literal),
      Identifier(String),
      BinaryOp(Box<Expr>, BinOp, Box<Expr>),
      UnaryOp(UnaryOp, Box<Expr>),
      Index(Box<Expr>, Box<Expr>),              // collection[key]
      Call(String, Vec<Expr>),                  // function or built-in call
      IsNull(Box<Expr>),
      IsNotNull(Box<Expr>),
      JsonLiteral(serde_json::Value),
  }

  pub enum Literal {
      Integer(i64),
      Float(f64),
      Str(String),
      Currency(i64),                            // cents
      Boolean(bool),
      Null,
  }

  pub enum BinOp {
      Add, Sub, Mul, Div, Mod, Pow,
      Eq, NotEq, Lt, LtEq, Gt, GtEq,
      And, Or,
      StrConcat,
  }

  pub enum UnaryOp { Neg, Not }

Phase 3 success criterion:
  cargo build --workspace   exits with code 0.
  (No tests required for this phase — the AST is pure data.)


## PHASE 4 — Parser  ( rundell-parser / parser.rs )

Implement a hand-written recursive-descent parser.
Do NOT use a parser generator or parser combinator library for the
grammar rules themselves (logos is fine for lexing only).

### Public API  ( lib.rs )

  pub fn parse(source: &str) -> Result<Vec<Stmt>, ParseError>

  - Calls lex(), then the parser.
  - Returns a Vec<Stmt> (the program) or a ParseError.

### ParseError

  #[derive(Debug, thiserror::Error)]
  pub enum ParseError {
      #[error("Unexpected token {found:?} at position {pos}, expected {expected}")]
      UnexpectedToken { found: String, pos: usize, expected: String },
      #[error("Unexpected end of input")]
      UnexpectedEof,
      #[error("Circular import detected: {path}")]
      CircularImport { path: String },
      #[error("{0}")]
      Other(String),
  }

### Parser structure

  struct Parser {
      tokens:  Vec<(Token, Range<usize>)>,
      pos:     usize,
  }

  impl Parser {
      fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError>
      fn parse_stmt(&mut self)    -> Result<Stmt, ParseError>
      fn parse_expr(&mut self)    -> Result<Expr, ParseError>
      // ... one method per grammar production
  }

### Key parsing rules

IMPORT:
  "import" StringLit "."

DEFINE:
  "define" Ident "as" ["constant"] ["global"] Type ["=" Expr] "."

SET:
  "set" target ("++" | "--" | "=" Expr) "."
  where target is  Ident  or  Ident [ Expr ] [ "[" Expr "]" ]*

PRINT:
  "print" Expr "."

RECEIVE:
  "receive" Ident ["with" "prompt" Expr] "."

IF chain:
  "if" ["("] Expr [")"] "-->"  body
  { "else" "if" ["("] Expr [")"] "-->"  body }
  [ "else" "-->"  body ]
  "<--"

SWITCH:
  "switch" Expr "-->"
    { case_line }
  "<--"

  case_line:
    [ cmp_op ] Expr ":"  [ body_stmt "." ]   # empty body = grouped case
    | "else" ":"  body_stmt "."

FOR:
  "for" Ident "loops" "(" Expr "," Expr "," Expr ")" "-->"  body  "<--"

WHILE:
  "while" Expr "-->"  body  "<--"

FOR EACH:
  "for" "each" Ident "in" Expr "-->"  body  "<--"

FUNCTION DEF:
  "define" Ident "(" params ")" "returns" (Type | "null") "-->"  body  "<--"

  params: empty | Ident "as" Type { "," Ident "as" Type }

RETURN:
  "return" [Expr] "."

TRY/CATCH:
  "try" "-->"  body
  { "catch" "(" ErrorIdent ")" "-->"  body }
  [ "finally" "-->"  body ]
  "<--"

REMOVE:
  "remove" Expr "."

FUNCTION CALL STATEMENT:
  Ident "(" args ")" "."

EXPRESSION parsing (Pratt/precedence climbing recommended):
  Handle all BinOp precedence levels.
  Handle  is null  and  is not null  as postfix unary expressions.
  Handle index access:  Expr "[" Expr "]"  left-associative.

Phase 4 success criteria:
  cargo test -p rundell-parser

Write unit tests covering at minimum one parse for:
  - Variable declaration with and without init
  - Assignment variants (= , ++, --)
  - if / else if / else chain
  - switch with grouped cases
  - for, while, for each loops
  - function declaration and call
  - try / catch / finally
  - import statement
  - JSON collection literal
  - Operator precedence (e.g.  2 + 3 * 4  parses as  2 + (3*4) )
  - is null / is not null


## PHASE 5 — Interpreter  ( rundell-interpreter )

Implement a tree-walk interpreter. The interpreter traverses the
AST Vec<Stmt> produced by the parser and executes each node.

### error.rs

  #[derive(Debug, thiserror::Error)]
  pub enum RuntimeError {
      #[error("TypeError: {0}")]
      TypeError(String),
      #[error("NullError: variable '{0}' is null")]
      NullError(String),
      #[error("IndexError: {0}")]
      IndexError(String),
      #[error("DivisionError: division by zero")]
      DivisionError,
      #[error("IOError: {0}")]
      IOError(String),
      #[error("RuntimeError: {0}")]
      RuntimeError(String),
      #[error("Return: {0:?}")]
      ReturnValue(Option<Value>),       // used internally to unwind the call stack
  }

### environment.rs

Implement a scoped environment (symbol table):

  pub struct Environment {
      scopes: Vec<HashMap<String, (Value, bool)>>,   // (value, is_constant)
  }

  impl Environment {
      pub fn new() -> Self
      pub fn push_scope(&mut self)
      pub fn pop_scope(&mut self)
      pub fn define(&mut self, name: &str, value: Value, constant: bool) -> Result<(), RuntimeError>
      pub fn get(&self, name: &str) -> Result<&Value, RuntimeError>
      pub fn set(&mut self, name: &str, value: Value) -> Result<(), RuntimeError>
  }

  Scoping rules:
    - define() inserts into the CURRENT (innermost) scope.
    - get() searches from innermost scope outward.
    - set() updates the NEAREST scope that contains the name.
    - Attempting to set() a constant binding → TypeError.
    - Attempting to define() a name already in the current scope → RuntimeError.
    - Globals are placed in scope[0] at startup.

### Value type  ( evaluator.rs or a values.rs module )

  #[derive(Debug, Clone)]
  pub enum Value {
      Integer(i64),
      Float(f64),
      Str(String),
      Currency(i64),               // cents
      Boolean(bool),
      Json(serde_json::Value),
      Null,
  }

  impl Value {
      pub fn type_name(&self) -> &'static str
      pub fn to_display_string(&self) -> String
      pub fn is_truthy(&self) -> bool
  }

  is_truthy rules:
    Integer(n)   →  n != 0
    Float(f)     →  f != 0.0
    Boolean(b)   →  b
    Currency(c)  →  c != 0
    Str(s)       →  !s.is_empty()
    Json(_)      →  true
    Null         →  false

### evaluator.rs

  pub struct Interpreter {
      env: Environment,
      functions: HashMap<String, FunctionDefStmt>,
  }

  impl Interpreter {
      pub fn new() -> Self
      pub fn run(&mut self, stmts: Vec<Stmt>) -> Result<(), RuntimeError>
      fn exec_stmt(&mut self, stmt: Stmt) -> Result<(), RuntimeError>
      fn eval_expr(&mut self, expr: Expr) -> Result<Value, RuntimeError>
  }

Implement exec_stmt for EVERY Stmt variant.
Implement eval_expr for EVERY Expr variant.

BUILT-IN FUNCTION DISPATCH:
  In eval_expr, when  Expr::Call(name, args)  is encountered, check
  if name matches a built-in BEFORE checking user-defined functions.

  Built-in implementations:

  newline()              → Value::Str("\n".to_string())
  length(expr)           → integer: string.chars().count() or json array length or object key count
  cast(expr, type)       → see §1.11 for all permitted casts; TypeError on failure
  abs(expr)              → numeric absolute value
  floor(expr)            → integer (f64.floor() as i64)
  ceil(expr)             → integer (f64.ceil() as i64)
  round(expr, dp)        → float rounded to dp places
  substr(str, start, len)→ string slice (0-based, Unicode-safe)
  upper(str)             → str.to_uppercase()
  lower(str)             → str.to_lowercase()
  trim(str)              → str.trim().to_string()
  string(expr)           → shorthand cast to string
  append(col, val)       → mutates the json array in place; returns Null

COLLECTION OPERATIONS IN exec_stmt:
  Remove: evaluate the collection[key] target; remove the key.
  Append: see built-in above.
  Set with Index target: evaluate collection and key; set value.

COLLECTION ACCESS (in eval_expr):
  Expr::Index(col_expr, key_expr):
    - Evaluate col_expr → must be Json.
    - Evaluate key_expr.
    - If key is Str → serde_json object lookup by key.
    - If key is Integer → array index (0-based) OR object nth-key-order.
    - Out of bounds → IndexError.

OPERATOR EXECUTION:
  BinaryOp:
    Add on Str + Str → concatenation.
    Add on Str + non-Str → TypeError.
    Arithmetic on mismatched numeric types → promote to float.
    Division: integer/integer → integer (truncate); otherwise float.
    Pow: both integers → integer if result fits i64; otherwise float.
    Boolean operands for And/Or: evaluate is_truthy().

  UnaryOp:
    Neg on Integer/Float/Currency.
    Not on any → boolean negation of is_truthy().

NULL HANDLING:
  Accessing a Null value in most contexts → NullError.
  Exception: is null / is not null expressions never raise NullError.

RETURN UNWINDING:
  When  return  is executed, throw RuntimeError::ReturnValue(value).
  In exec_stmt for FunctionDef calls, catch ReturnValue and extract
  the inner value. All other RuntimeError variants propagate normally.

TRY/CATCH EXECUTION:
  Execute try_body.
  If a RuntimeError is raised, match its variant to the catch clauses
  by the string name of the error type:
    TypeError       → catch(TypeError)
    NullError       → catch(NullError)
    IndexError      → catch(IndexError)
    DivisionError   → catch(DivisionError)
    IOError         → catch(IOError)
    any other       → catch(RuntimeError)
  If no catch matches, re-raise.
  Always execute finally_body regardless.

PRINT:
  Evaluate expr → to_display_string() → write to stdout WITHOUT
  a trailing newline (newline() is explicit in the language).

RECEIVE:
  If prompt exists, print it (no newline).
  Read one line from stdin (strip the trailing \n).
  Attempt to coerce the string to the variable's declared type.
  Coercion failure → TypeError.
  Store result in environment.

  NOTE: receive stores into an EXISTING variable (already declared).
  The interpreter must know the variable's declared type. Store the
  declared type alongside the value in the environment:
    (Value, RundellType, bool)  →  (value, declared_type, is_constant)

IMPORT:
  When exec_stmt encounters Import(path):
    1. Resolve the path relative to the current source file's directory.
    2. Append ".run" to form the file path.
    3. Read and parse the file.
    4. Execute all top-level Stmt::Define(global=true) and
       Stmt::FunctionDef statements to register them.
    5. Track visited paths to detect circular imports.

Phase 5 success criteria:
  cargo test -p rundell-interpreter

Write integration tests that execute the .run test files in tests/.
The test harness should capture stdout and compare to expected output.
Provide expected outputs inline as constants in the test file.


## PHASE 6 — CLI  ( rundell-cli )

### main.rs

Use  clap  for argument parsing.

CLI interface:

  rundell <file>           # interpret a .run source file
  rundell                  # start the REPL

REPL behaviour:
  - Use  rustyline  for readline-style input.
  - Print a welcome banner:
      Rundell 0.1.0  —  type 'exit' or Ctrl+D to quit
  - Accept multi-line input: accumulate lines until a full stop or
    <-- is the last non-whitespace token on the line,
    then parse and execute the accumulated buffer.
  - On parse or runtime error, print the error message and
    continue — do not exit.
  - On  exit  or Ctrl+D, exit cleanly.

FILE mode:
  - Read the source file.
  - Call parse() then interpreter.run().
  - On error, use  miette  to format and print a rich diagnostic.
  - Exit with code 1 on error, 0 on success.

Phase 6 success criteria:
  cargo build --release
  The binary  target/release/rundell  (or .exe) exists.
  Running  rundell tests/01_variables.run  produces correct output.
  Running  rundell  (no args) starts the REPL.


# ═══════════════════════════════════════════════════════════════════
# PART 5 — TEST FILE CONTENTS
# Write these .run files into  tests/  exactly as shown.
# The interpreter must produce the stated output for each.
# ═══════════════════════════════════════════════════════════════════

## tests/01_variables.run

  define name as string = "Rundell".
  define age  as integer = 5.
  define pi   as float = 3.14.
  define price as currency = 9.99.
  define flag as boolean = true.
  define nothing as integer.

  print name + newline().
  print string(age) + newline().
  print string(pi) + newline().
  print string(price) + newline().
  print string(flag) + newline().

  if (nothing is null) -->
      print "nothing is null" + newline().
  <--

  # Expected output:
  # Rundell
  # 5
  # 3.14
  # 9.99
  # true
  # nothing is null


## tests/02_arithmetic.run

  define a as integer = 10.
  define b as integer = 3.

  print string(a + b) + newline().
  print string(a - b) + newline().
  print string(a * b) + newline().
  print string(a / b) + newline().
  print string(a % b) + newline().
  print string(2 ** 8) + newline().
  print string(cast(a, float) / cast(b, float)) + newline().

  # Expected output:
  # 13
  # 7
  # 30
  # 3
  # 1
  # 256
  # 3.3333333333333335


## tests/03_strings.run

  define s as string = "Hello, World!".
  print s + newline().
  print string(length(s)) + newline().
  print upper(s) + newline().
  print lower(s) + newline().
  print substr(s, 0, 5) + newline().
  print trim("  spaced  ") + newline().

  # Expected output:
  # Hello, World!
  # 13
  # HELLO, WORLD!
  # hello, world!
  # Hello
  # spaced


## tests/04_boolean.run

  define t as boolean = true.
  define f as boolean = FALSE.
  define y as boolean = yes.
  define n as boolean = no.

  print string(t) + newline().
  print string(f) + newline().
  print string(y) + newline().
  print string(n) + newline().

  if (t and not f) -->
      print "logic works" + newline().
  <--

  # Expected output:
  # true
  # false
  # true
  # false
  # logic works


## tests/05_casting.run

  define i as integer = 42.
  define f as float = cast(i, float).
  define s as string = cast(i, string).
  define b as boolean = cast(0, boolean).

  print string(f) + newline().
  print s + newline().
  print string(b) + newline().

  # Expected output:
  # 42.0
  # 42
  # false


## tests/06_conditionals.run

  define score as integer = 75.

  if (score >= 70) -->
      print "Distinction" + newline().
  else if (score >= 40) -->
      print "Pass" + newline().
  else -->
      print "Fail" + newline().
  <--

  switch score -->
      >= 70 : print "Grade A" + newline().
      >= 40 : print "Grade B" + newline().
      else  : print "Grade F" + newline().
  <--

  # Expected output:
  # Distinction
  # Grade A


## tests/07_loops.run

  define i as integer.

  for i loops (1, 3, 1) -->
      print string(i) + newline().
  <--

  define count as integer = 0.
  while count < 3 -->
      set count++.
      print string(count) + newline().
  <--

  # Expected output:
  # 1
  # 2
  # 3
  # 1
  # 2
  # 3


## tests/08_functions.run

  define multiply(a as integer, b as integer) returns integer -->
      return a * b.
  <--

  define greet(name as string) returns null -->
      print "Hello, " + name + "!" + newline().
      return null.
  <--

  print string(multiply(6, 7)) + newline().
  greet("World").

  # Expected output:
  # 42
  # Hello, World!


## tests/09_collections.run

  define data as json = {
      "items": [
          { "name": "Apple",  "price": 1.20 },
          { "name": "Banana", "price": 0.50 }
      ]
  }.

  print data["items"][0]["name"] + newline().
  print string(length(data["items"])) + newline().

  for each item in data["items"] -->
      print item["name"] + newline().
  <--

  # Expected output:
  # Apple
  # 2
  # Apple
  # Banana


## tests/10_error_handling.run

  try -->
      define x as integer.
      print string(x + 1) + newline().
  catch (NullError) -->
      print "caught null error" + newline().
  finally -->
      print "finally ran" + newline().
  <--

  # Expected output:
  # caught null error
  # finally ran


## tests/11_modules/helper.run

  define global GREETING as constant string = "Hello from helper".

  define sayHello(name as string) returns null -->
      print GREETING + ", " + name + "!" + newline().
      return null.
  <--


## tests/11_modules/main.run

  import "helper".

  sayHello("Rundell").
  print GREETING + newline().

  # Expected output:
  # Hello from helper, Rundell!
  # Hello from helper


# ═══════════════════════════════════════════════════════════════════
# PART 6 — CODE QUALITY REQUIREMENTS
# ═══════════════════════════════════════════════════════════════════

1. Every public function, struct, enum, and module must have a
   doc comment ( /// ) explaining its purpose.

2. Every non-trivial private function should have an inline comment
   explaining WHAT it does and WHY, not just what the code says.

3. Use  Result<T, E>  throughout — no  unwrap()  in library code.
   unwrap() is only permitted in test code and main.rs where a
   panic is an acceptable outcome.

4. Run  cargo clippy -- -D warnings  and resolve all warnings before
   declaring a phase complete.

5. Run  cargo fmt  before declaring a phase complete.

6. Do not use  unsafe  anywhere in this codebase.

7. All string manipulation must be Unicode-safe. Use  .chars()
   rather than  .bytes()  or indexing when operating on string content.


# ═══════════════════════════════════════════════════════════════════
# PART 7 — KNOWN EDGE CASES TO HANDLE CORRECTLY
# ═══════════════════════════════════════════════════════════════════

1. FULL STOP IN FLOAT LITERAL
   The token  3.14  must not be split into  3  Dot  14.
   The lexer regex for Float must consume the decimal point as part
   of the number when surrounded by digits.

2. MULTI-LINE STRINGS
   A string literal may contain raw newlines in the source.
   The lexer regex for StringLit must allow  \n  inside the match.

3. SWITCH GROUPED CASES
   A case line with no body (just  value :  with no statement)
   is a "grouped" case — it falls through to the next case's body.
   This is the ONLY fall-through permitted in switch.

4. FOR LOOP INCLUSIVITY
   for i loops (0, 5, 1) produces 0 1 2 3 4 5 (six values, not five).
   The end value IS included.

5. RECEIVE TYPE COERCION
   The interpreter must store the declared type of every variable
   so that receive knows what type to coerce to.

6. CURRENCY DISPLAY
   Currency values must always display with exactly 2 decimal places:
   string(currency_value)  →  "9.99"  not  "9.9"  or  "9.990"

7. FLOAT DISPLAY
   Avoid displaying floats with excessive precision where not needed,
   but do not truncate significant digits. Use Rust's default
   Display for f64 unless round() has been applied.

8. BOOLEAN DISPLAY
   string(true) → "true"  (lowercase, always)
   string(false) → "false" (lowercase, always)

9. NULL ARITHMETIC
   Any arithmetic operation involving a Null value must raise NullError,
   NOT produce a default value silently.

10. CONSTANT REASSIGNMENT
    Attempting to  set  a constant variable must be detected and
    raise TypeError at runtime (or flag at parse time if feasible).

11. IMPORT PATH RESOLUTION
    Import paths are relative to the IMPORTING FILE's directory,
    not the working directory of the process. This matters when
    importing from a subdirectory.

12. RETURN FROM VOID FUNCTION
    A  returns null  function must execute  return null.
    If the body reaches the end without a return statement, the
    interpreter should treat it as an implicit  return null.


# ═══════════════════════════════════════════════════════════════════
# PART 8 — WHAT NOT TO BUILD (OUT OF SCOPE FOR THIS VERSION)
# ═══════════════════════════════════════════════════════════════════

The following are explicitly deferred to future versions.
Do not implement them, and do not add placeholder code for them.

  - Bytecode compilation or VM
  - LLVM backend
  - Classes or object-oriented features
  - Closures or first-class functions
  - Generics or templates
  - Concurrency or threading
  - Networking or file I/O (beyond stdin/stdout)
  - A standard library beyond the built-in functions listed above
  - Package management
  - A debugger or stepping mode
  - Source maps


# ═══════════════════════════════════════════════════════════════════
# END OF INSTRUCTIONS
# When all six phases are complete and all test files produce the
# expected output, the Rundell 0.1.0 interpreter is done.
# ═══════════════════════════════════════════════════════════════════
