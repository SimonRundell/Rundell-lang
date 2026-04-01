//! Visual form designer application.
//!
//! Launched with `rundell-gui --design`.  Provides a palette, canvas,
//! property inspector, and code generator.

use std::collections::HashMap;
use std::path::PathBuf;

use egui::Context;

mod canvas;
mod inspector;
pub mod codegen;

pub use canvas::DesignCanvas;
pub use inspector::Inspector;

use rundell_interpreter::form_registry::ControlState;
use rundell_parser::ast::ControlType;

/// A control placed on the design canvas.
#[derive(Debug, Clone)]
pub struct DesignControl {
    /// Identifier used in generated code.
    pub name: String,
    /// Control type.
    pub ctrl_type: ControlType,
    /// Current property state.
    pub state: ControlState,
}

/// The designer application — implements `eframe::App`.
pub struct DesignerApp {
    /// Controls placed on the canvas.
    pub controls: Vec<DesignControl>,
    /// Index of the currently selected control (if any).
    pub selected: Option<usize>,
    /// Form name for code generation.
    pub form_name: String,
    /// The design canvas.
    canvas: DesignCanvas,
    /// The property inspector.
    inspector: Inspector,
    /// Last generated code string.
    generated_code: String,
    /// File path for save operations.
    file_path: Option<PathBuf>,
    /// Auto-increment counter for control names.
    name_counter: HashMap<String, usize>,
}

impl DesignerApp {
    /// Create a new designer, optionally loading an existing file.
    pub fn new(file: Option<PathBuf>) -> Self {
        DesignerApp {
            controls: Vec::new(),
            selected: None,
            form_name: "myForm".to_string(),
            canvas: DesignCanvas::new(),
            inspector: Inspector::new(),
            generated_code: String::new(),
            file_path: file,
            name_counter: HashMap::new(),
        }
    }

    /// Generate a unique control name for a given type.
    fn next_name(&mut self, ctrl_type: &ControlType) -> String {
        let prefix = match ctrl_type {
            ControlType::Label       => "label",
            ControlType::Textbox     => "textbox",
            ControlType::Button      => "button",
            ControlType::Radiobutton => "radio",
            ControlType::Checkbox    => "checkbox",
            ControlType::Switch      => "switch",
            ControlType::Select      => "select",
            ControlType::Listbox     => "listbox",
        };
        let counter = self.name_counter.entry(prefix.to_string()).or_insert(0);
        *counter += 1;
        format!("{prefix}{counter}")
    }

    /// Add a control to the canvas at the default position.
    pub fn add_control(&mut self, ctrl_type: ControlType) {
        let name = self.next_name(&ctrl_type);
        let state = rundell_interpreter::form_registry::default_control_state(&ctrl_type);
        self.controls.push(DesignControl { name, ctrl_type, state });
        self.selected = Some(self.controls.len() - 1);
    }
}

impl eframe::App for DesignerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // --- Left panel: tool palette ---
        egui::SidePanel::left("palette").min_width(120.0).show(ctx, |ui| {
            ui.heading("Controls");
            ui.separator();
            let types = [
                ("Label",       ControlType::Label),
                ("Textbox",     ControlType::Textbox),
                ("Button",      ControlType::Button),
                ("Radiobutton", ControlType::Radiobutton),
                ("Checkbox",    ControlType::Checkbox),
                ("Switch",      ControlType::Switch),
                ("Select",      ControlType::Select),
                ("Listbox",     ControlType::Listbox),
            ];
            for (label, ct) in types {
                if ui.button(label).clicked() {
                    self.add_control(ct);
                }
            }
        });

        // --- Right panel: property inspector ---
        egui::SidePanel::right("inspector").min_width(220.0).show(ctx, |ui| {
            ui.heading("Properties");
            ui.separator();
            if let Some(idx) = self.selected {
                if let Some(ctrl) = self.controls.get_mut(idx) {
                    self.inspector.show(ui, ctrl);
                }
            } else {
                ui.label("Select a control to edit its properties.");
            }
        });

        // --- Bottom panel: code output ---
        egui::TopBottomPanel::bottom("code_panel").min_height(120.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Generate Code").clicked() {
                    self.generated_code = codegen::generate(&self.form_name, &self.controls);
                }
                if ui.button("Copy to Clipboard").clicked() {
                    ui.output_mut(|o| o.copied_text = self.generated_code.clone());
                }
                if ui.button("Save to File\u{2026}").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Save Form")
                        .add_filter("Rundell", &["run"])
                        .save_file()
                    {
                        let _ = std::fs::write(&path, &self.generated_code);
                        self.file_path = Some(path);
                    }
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                ui.code(&self.generated_code);
            });
        });

        // --- Central panel: design canvas ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Form: {}", self.form_name));
            ui.separator();
            self.canvas.show(ui, &mut self.controls, &mut self.selected);
        });
    }
}
