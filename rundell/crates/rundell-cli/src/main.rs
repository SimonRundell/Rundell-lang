//! Rundell command-line interface.
//!
//! Usage:
//!   `rundell <file.run>`   — execute a source file
//!   `rundell`              — start the interactive REPL

use std::path::PathBuf;
use std::process;

use clap::Parser as ClapParser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use rundell_interpreter::Interpreter;
use rundell_parser::parse;

/// The Rundell language interpreter.
#[derive(ClapParser, Debug)]
#[command(
    name = "rundell",
    version = "0.1.0",
    about = "Rundell language interpreter"
)]
struct Cli {
    /// Source file to execute (.run extension).  Omit to start the REPL.
    file: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    match cli.file {
        Some(path) => run_file(path),
        None => run_repl(),
    }
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
            eprintln!("Parse error: {e}");
            process::exit(1);
        }
    };

    let mut interpreter = Interpreter::new();
    // Set the source directory so imports are resolved relative to the file.
    if let Some(dir) = path.parent() {
        interpreter.set_source_dir(dir.to_path_buf());
    }

    if let Err(e) = interpreter.run(stmts) {
        eprintln!("Runtime error: {e}");
        process::exit(1);
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
                            eprintln!("Parse error: {e}");
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
