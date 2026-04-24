//! Rundell-to-Rust transpiler.
//!
//! Converts a Rundell `.run` source file into a standalone Rust/Cargo project,
//! then optionally invokes `cargo build` to produce a native binary.

pub mod codegen;
pub mod prelude;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub use codegen::CodeGen;

/// Error returned by the compiler.
#[derive(Debug)]
pub enum CompileError {
    ParseError(String),
    IoError(String),
    BuildError(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::ParseError(s) => write!(f, "Parse error: {s}"),
            CompileError::IoError(s) => write!(f, "IO error: {s}"),
            CompileError::BuildError(s) => write!(f, "Build error: {s}"),
        }
    }
}

/// Compile a Rundell source string to a self-contained Cargo project.
///
/// - `source`: the Rundell source text.
/// - `source_dir`: directory of the source file (used to resolve `import`).
/// - `output_dir`: directory where the Cargo project will be written.
///
/// After this call, `output_dir/src/main.rs` and `output_dir/Cargo.toml` exist.
pub fn compile(
    source: &str,
    source_dir: &Path,
    output_dir: &Path,
) -> Result<(), CompileError> {
    let stmts = rundell_parser::parse(source)
        .map_err(|e| CompileError::ParseError(e.to_string()))?;

    let mut gen = CodeGen::new(source_dir.to_path_buf());
    let rust_src = gen.generate(&stmts);

    let cargo_toml = r#"[package]
name = "rundell_output"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "rundell_output"
path = "src/main.rs"

[dependencies]
serde_json = "1"
chrono = { version = "0.4", features = ["clock"] }
csv = "1"
"#;

    let src_dir = output_dir.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| CompileError::IoError(e.to_string()))?;
    fs::write(output_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| CompileError::IoError(e.to_string()))?;
    fs::write(src_dir.join("main.rs"), &rust_src)
        .map_err(|e| CompileError::IoError(e.to_string()))?;

    Ok(())
}

/// Locate the `cargo` executable.
///
/// Checks (in order):
/// 1. The `CARGO` environment variable (set by Cargo itself when building).
/// 2. `~/.cargo/bin/cargo[.exe]`.
/// 3. Plain `cargo` on PATH.
fn find_cargo() -> PathBuf {
    // When the compiler runs inside `cargo run`, $CARGO is set automatically.
    if let Ok(cargo_env) = std::env::var("CARGO") {
        let p = PathBuf::from(cargo_env);
        if p.exists() { return p; }
    }
    // Try the conventional ~/.cargo/bin location.
    if let Some(home) = dirs_next::home_dir() {
        let ext = if cfg!(windows) { ".exe" } else { "" };
        let p = home.join(".cargo").join("bin").join(format!("cargo{ext}"));
        if p.exists() { return p; }
    }
    PathBuf::from("cargo")
}

/// Build a previously compiled Rundell project using `cargo build`.
///
/// Returns the path to the compiled binary.
pub fn build_project(output_dir: &Path, release: bool) -> Result<PathBuf, CompileError> {
    let cargo = find_cargo();
    let mut cmd = Command::new(cargo);
    cmd.arg("build").current_dir(output_dir);
    if release {
        cmd.arg("--release");
    }

    let output = cmd
        .output()
        .map_err(|e| CompileError::BuildError(format!("failed to invoke cargo: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CompileError::BuildError(stderr.into_owned()));
    }

    let profile = if release { "release" } else { "debug" };
    let ext = if cfg!(windows) { ".exe" } else { "" };
    Ok(output_dir
        .join("target")
        .join(profile)
        .join(format!("rundell_output{ext}")))
}
