//! Entry point for the Rundell GUI application.
//!
//! Usage:
//!   rundell-gui <file.run>          Run a Rundell program with GUI support.
//!   rundell-gui --design [file.run] Open the visual form designer.

use std::path::PathBuf;
use std::sync::mpsc;

use clap::Parser;
use eframe::NativeOptions;

mod app;
mod controls;
mod dialogs;
mod form_runtime;
pub mod designer;

use app::RundellApp;

/// Timeout for modal form blocking (milliseconds).
pub const MODAL_TIMEOUT_MS: u64 = 30_000;

/// Command-line interface for rundell-gui.
#[derive(Parser, Debug)]
#[command(name = "rundell-gui", about = "Rundell GUI runner and form designer")]
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
        run_designer(cli.file);
    } else if let Some(file) = cli.file {
        run_program(file);
    } else {
        eprintln!("Usage: rundell-gui <file.run>  or  rundell-gui --design [file.run]");
        std::process::exit(1);
    }
}

/// Run a Rundell program with GUI support.
fn run_program(file: PathBuf) {
    let source = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {:?}: {e}", file);
            std::process::exit(1);
        }
    };

    let stmts = match rundell_parser::parse(&source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Parse error: {e}");
            std::process::exit(1);
        }
    };

    // Channel: interpreter → GUI
    let (cmd_tx, cmd_rx) = mpsc::sync_channel::<rundell_interpreter::gui_channel::GuiCommand>(64);
    // Channel: GUI → interpreter
    let (resp_tx, resp_rx) = mpsc::sync_channel::<rundell_interpreter::gui_channel::GuiResponse>(64);

    // Spawn the interpreter on a background thread.
    let source_dir = file.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
    let resp_tx_clone = resp_tx.clone();
    let interpreter_thread = std::thread::spawn(move || {
        let mut interp = rundell_interpreter::Interpreter::new();
        interp.set_source_dir(source_dir);
        interp.gui_tx = Some(cmd_tx);
        interp.gui_rx = Some(resp_rx);
        if let Err(e) = interp.run(stmts) {
            eprintln!("Runtime error: {e}");
        }
        // Signal GUI to quit when interpreter finishes
        let _ = resp_tx_clone.send(rundell_interpreter::gui_channel::GuiResponse::Ready);
    });

    // Run the egui app on the main thread.
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rundell")
            .with_inner_size([900.0, 700.0]),
        ..Default::default()
    };

    let app = RundellApp::new(cmd_rx, resp_tx);
    eframe::run_native(
        "Rundell",
        options,
        Box::new(|_cc| Box::new(app)),
    )
    .unwrap_or_else(|e| eprintln!("GUI error: {e}"));

    let _ = interpreter_thread.join();
}

/// Open the visual form designer.
fn run_designer(file: Option<PathBuf>) {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rundell Form Designer")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    let initial_file = file;
    eframe::run_native(
        "Rundell Form Designer",
        options,
        Box::new(move |_cc| Box::new(designer::DesignerApp::new(initial_file)) as Box<dyn eframe::App>),
    )
    .unwrap_or_else(|e| eprintln!("Designer error: {e}"));
}
