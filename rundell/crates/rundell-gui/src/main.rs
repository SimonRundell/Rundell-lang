//! Entry point for the Rundell GUI application.
//!
//! Usage:
//!   rundell-gui <file.run>          Run a Rundell program with GUI support.
//!   rundell-gui --design [file.run] Open the visual form designer.

use std::path::PathBuf;

use clap::Parser;

/// Timeout for modal form blocking (milliseconds).
pub const MODAL_TIMEOUT_MS: u64 = 30_000;

/// Command-line interface for rundell-gui.
#[derive(Parser, Debug)]
#[command(
    name = "rundell-gui",
    version = env!("CARGO_PKG_VERSION"),
    about = "Rundell GUI runner and form designer"
)]
struct Cli {
    /// Rundell source file to run (omit for designer without a file).
    file: Option<PathBuf>,

    /// Open the visual form designer instead of running a program.
    #[arg(long)]
    design: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.design {
        rundell_gui::run_designer(cli.file);
    } else if let Some(file) = cli.file {
        rundell_gui::run_program(file);
    } else {
        eprintln!("Usage: rundell-gui <file.run>  or  rundell-gui --design [file.run]");
        std::process::exit(1);
    }
}

