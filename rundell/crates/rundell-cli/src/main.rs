//! Rundell command-line interface.
//!
//! Usage:
//!   `rundell <file.run>`          — execute a source file
//!   `rundell`                     — start the interactive REPL
//!   `rundell --env-set KEY VALUE` — store a credential in `.rundell.env`
//!   `rundell --env-list`          — list credential keys in `.rundell.env`
//!   `rundell --env-delete KEY`    — delete a credential from `.rundell.env`

use std::path::PathBuf;
use std::process;

use clap::Parser as ClapParser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use rundell_interpreter::Interpreter;
use rundell_parser::{format_parse_error, parse};
use rundell_parser::ast::{Expr, SetOp, Stmt};

/// The Rundell language interpreter.
#[derive(ClapParser, Debug)]
#[command(
    name = "rundell",
    version = env!("CARGO_PKG_VERSION"),
    about = "Rundell language interpreter"
)]
struct Cli {
    /// Source file to execute (.run extension).  Omit to start the REPL.
    file: Option<PathBuf>,

    /// Store a credential: --env-set KEY VALUE
    #[arg(long = "env-set", num_args = 2, value_names = ["KEY", "VALUE"])]
    env_set: Option<Vec<String>>,

    /// List all credential keys stored in .rundell.env
    #[arg(long = "env-list")]
    env_list: bool,

    /// Delete a credential by key: --env-delete KEY
    #[arg(long = "env-delete")]
    env_delete: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Handle env subcommands — these take priority over file/REPL.
    if let Some(kv) = cli.env_set {
        let key = &kv[0];
        let value = &kv[1];
        let env_path = env_file_path();
        match rundell_env::env_set(&env_path, key, value) {
            Ok(()) => println!("Credential '{key}' stored successfully."),
            Err(e) => {
                eprintln!("Error storing credential: {e}");
                process::exit(1);
            }
        }
        return;
    }

    if cli.env_list {
        let env_path = env_file_path();
        match rundell_env::env_list(&env_path) {
            Ok(keys) => {
                if keys.is_empty() {
                    println!("No credentials stored.");
                } else {
                    for key in &keys {
                        println!("{key}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error listing credentials: {e}");
                process::exit(1);
            }
        }
        return;
    }

    if let Some(key) = cli.env_delete {
        let env_path = env_file_path();
        match rundell_env::env_delete(&env_path, &key) {
            Ok(()) => println!("Credential '{key}' deleted."),
            Err(e) => {
                eprintln!("Error deleting credential: {e}");
                process::exit(1);
            }
        }
        return;
    }

    // Normal file / REPL mode.
    match cli.file {
        Some(path) => run_file(path),
        None => run_repl(),
    }
}

/// Returns the path to `.rundell.env` in the current working directory.
fn env_file_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".rundell.env")
}

/// Read, parse, and execute a `.run` source file.
fn run_file(path: PathBuf) {
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {e}", path.display());
            process::exit(1);
        }
    };

    let stmts = match parse(&source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", format_parse_error(&source, &e));
            process::exit(1);
        }
    };

    if uses_gui(&stmts) {
        rundell_gui::run_program(path);
        return;
    }

    let mut interpreter = Interpreter::new();
    // Set the source directory so imports are resolved relative to the file.
    if let Some(dir) = path.parent() {
        interpreter.set_source_dir(dir.to_path_buf());
    }
    // Set the full program path so env() can locate the adjacent .rundell.env file.
    interpreter.set_program_path(path.clone());

    if let Err(e) = interpreter.run(stmts) {
        eprintln!("Runtime error: {e}");
        process::exit(1);
    }
}

fn uses_gui(stmts: &[Stmt]) -> bool {
    stmts.iter().any(stmt_uses_gui)
}

fn stmt_uses_gui(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::FormDef(_) => true,
        Stmt::ExprStmt(expr) => expr_uses_gui(expr),
        Stmt::Define(def) => def.init.as_ref().map(expr_uses_gui).unwrap_or(false),
        Stmt::Set(set_stmt) => match &set_stmt.op {
            SetOp::Assign(expr) => expr_uses_gui(expr),
            _ => false,
        },
        Stmt::Print(expr) => expr_uses_gui(expr),
        Stmt::Receive(_) => false,
        Stmt::If(i) => {
            expr_uses_gui(&i.condition)
                || i.then_body.iter().any(stmt_uses_gui)
                || i.else_ifs.iter().any(|(cond, body)| {
                    expr_uses_gui(cond) || body.iter().any(stmt_uses_gui)
                })
                || i.else_body.as_ref().map(|b| b.iter().any(stmt_uses_gui)).unwrap_or(false)
        }
        Stmt::Switch(sw) => {
            expr_uses_gui(&sw.subject)
                || sw.cases.iter().any(|case| case.body.iter().any(stmt_uses_gui))
        }
        Stmt::ForLoop(fl) => {
            expr_uses_gui(&fl.start)
                || expr_uses_gui(&fl.end)
                || expr_uses_gui(&fl.increment)
                || fl.body.iter().any(stmt_uses_gui)
        }
        Stmt::WhileLoop(wl) => {
            expr_uses_gui(&wl.condition) || wl.body.iter().any(stmt_uses_gui)
        }
        Stmt::ForEach(fe) => {
            expr_uses_gui(&fe.collection) || fe.body.iter().any(stmt_uses_gui)
        }
        Stmt::FunctionDef(fd) => fd.body.iter().any(stmt_uses_gui),
        Stmt::Return(expr) => expr.as_ref().map(expr_uses_gui).unwrap_or(false),
        Stmt::TryCatch(tc) => {
            tc.try_body.iter().any(stmt_uses_gui)
                || tc.catches.iter().any(|c| c.body.iter().any(stmt_uses_gui))
                || tc.finally_body.as_ref().map(|b| b.iter().any(stmt_uses_gui)).unwrap_or(false)
        }
        Stmt::Remove(expr) => expr_uses_gui(expr),
        Stmt::Append(expr, val) => expr_uses_gui(expr) || expr_uses_gui(val),
        Stmt::DefineControl(_, _) => true,
        Stmt::CredentialsDef(def) => {
            def.token.as_ref().map(expr_uses_gui).unwrap_or(false)
                || def.authentication.as_ref().map(expr_uses_gui).unwrap_or(false)
        }
        Stmt::QueryDef(def) => {
            expr_uses_gui(&def.endpoint)
                || def.timeout_ms.as_ref().map(expr_uses_gui).unwrap_or(false)
                || def.query_params.as_ref().map(expr_uses_gui).unwrap_or(false)
        }
        Stmt::Attempt(block) => {
            block.body.iter().any(stmt_uses_gui) || block.handler.iter().any(stmt_uses_gui)
        }
        Stmt::Import(_) => false,
    }
}

fn expr_uses_gui(expr: &Expr) -> bool {
    match expr {
        Expr::Dialog(_) => true,
        Expr::ShowForm { .. } | Expr::CloseForm { .. } => true,
        Expr::BinaryOp(lhs, _, rhs) => expr_uses_gui(lhs) || expr_uses_gui(rhs),
        Expr::UnaryOp(_, inner) => expr_uses_gui(inner),
        Expr::Index(target, idx) => expr_uses_gui(target) || expr_uses_gui(idx),
        Expr::Call(_, args) => args.iter().any(expr_uses_gui),
        Expr::IsNull(inner) | Expr::IsNotNull(inner) => expr_uses_gui(inner),
        Expr::Await(await_expr) => expr_uses_gui(await_expr.call.as_ref()),
        _ => false,
    }
}

/// Start the interactive REPL.
fn run_repl() {
    println!("Rundell 0.1.0  \u{2014}  type 'exit' or Ctrl+D to quit");

    let mut rl = DefaultEditor::new().unwrap_or_else(|e| {
        eprintln!("Failed to initialise readline: {e}");
        process::exit(1);
    });

    let mut interpreter = Interpreter::new();
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { ">> " } else { ".. " };
        match rl.readline(prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed == "exit" {
                    break;
                }
                buffer.push_str(&line);
                buffer.push('\n');

                // Try to parse the buffer.  If it parses successfully, execute it.
                // If parsing fails with UnexpectedEof, wait for more input.
                // On any other error, print it and reset.
                if is_complete(&buffer) {
                    let _ = rl.add_history_entry(buffer.trim());
                    match parse(&buffer) {
                        Ok(stmts) => {
                            if let Err(e) = interpreter.run(stmts) {
                                eprintln!("Runtime error: {e}");
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", format_parse_error(&buffer, &e));
                        }
                    }
                    buffer.clear();
                }
            }
            Err(ReadlineError::Interrupted) => {
                buffer.clear();
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                eprintln!("Readline error: {e}");
                break;
            }
        }
    }
}

/// Heuristic: the buffer is complete if the last non-whitespace token
/// is a `.` (statement terminator) or `<--` (end of block).
fn is_complete(buf: &str) -> bool {
    let trimmed = buf.trim_end();
    trimmed.ends_with('.') || trimmed.ends_with("<--")
}

