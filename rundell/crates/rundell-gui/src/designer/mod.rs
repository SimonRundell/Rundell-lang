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
use rundell_parser::ast::{ControlType, Expr, Literal, SetOp, SetTarget, Stmt};
use rundell_parser::parse;

/// A control placed on the design canvas.
#[derive(Debug, Clone)]
pub struct DesignControl {
    /// Identifier used in generated code.
    pub name: String,
    /// Control type.
    pub ctrl_type: ControlType,
    /// Current property state.
    pub state: ControlState,
    /// Event handler bindings.
    pub events: ControlEvents,
}

#[derive(Debug, Clone, Default)]
pub struct ControlEvents {
    pub click: String,
    pub change: String,
    pub select: String,
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
    /// Editor buffer for manual edits.
    editor_code: String,
    /// Whether the editor has unsynced edits.
    editor_dirty: bool,
    /// Whether the editor tab is active.
    editor_active: bool,
    /// Whether the editor auto-syncs to generated code.
    editor_autosync: bool,
    /// Active view tab.
    active_tab: DesignerTab,
    /// File path for save operations.
    file_path: Option<PathBuf>,
    /// Auto-increment counter for control names.
    name_counter: HashMap<String, usize>,
    /// Undo stack for delete operations.
    undo_stack: Vec<DesignerSnapshot>,
    /// Whether a delete confirmation is pending.
    pending_delete: bool,
    /// Last parse error when loading code into the designer.
    parse_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesignerTab {
    Form,
    Code,
    Split,
}

#[derive(Debug, Clone)]
struct DesignerSnapshot {
    controls: Vec<DesignControl>,
    selected: Option<usize>,
    form_name: String,
    name_counter: HashMap<String, usize>,
}

impl DesignerApp {
    /// Create a new designer, optionally loading an existing file.
    pub fn new(file: Option<PathBuf>) -> Self {
        let mut editor_code = String::new();
        let mut editor_active = false;
        let mut editor_autosync = true;

        if let Some(path) = file.as_ref() {
            if let Ok(contents) = std::fs::read_to_string(path) {
                editor_code = contents;
                editor_active = true;
                editor_autosync = false;
            }
        }

        let active_tab = if editor_active { DesignerTab::Code } else { DesignerTab::Form };

        let mut app = DesignerApp {
            controls: Vec::new(),
            selected: None,
            form_name: "myForm".to_string(),
            canvas: DesignCanvas::new(),
            inspector: Inspector::new(),
            generated_code: String::new(),
            editor_code,
            editor_dirty: false,
            editor_active,
            editor_autosync,
            active_tab,
            file_path: file,
            name_counter: HashMap::new(),
            undo_stack: Vec::new(),
            pending_delete: false,
            parse_error: None,
        };

        if !app.editor_code.is_empty() {
            let source = app.editor_code.clone();
            app.apply_code_to_designer(&source);
        }

        app
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
        let events = ControlEvents::default();
        self.controls.push(DesignControl { name, ctrl_type, state, events });
        self.selected = Some(self.controls.len() - 1);
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(DesignerSnapshot {
            controls: self.controls.clone(),
            selected: self.selected,
            form_name: self.form_name.clone(),
            name_counter: self.name_counter.clone(),
        });
    }

    fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.controls = snapshot.controls;
            self.selected = snapshot.selected;
            self.form_name = snapshot.form_name;
            self.name_counter = snapshot.name_counter;
        }
    }

    fn request_delete(&mut self) {
        if self.selected.is_some() {
            self.pending_delete = true;
        }
    }

    fn delete_selected_confirmed(&mut self) {
        let Some(idx) = self.selected else { return; };
        if idx >= self.controls.len() {
            self.selected = None;
            return;
        }
        self.push_undo();
        self.controls.remove(idx);
        if self.controls.is_empty() {
            self.selected = None;
        } else {
            let next_idx = idx.min(self.controls.len() - 1);
            self.selected = Some(next_idx);
        }
    }

    fn active_code(&self) -> &str {
        if self.editor_active { &self.editor_code } else { &self.generated_code }
    }

    fn open_file(&mut self, path: PathBuf) {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            self.editor_code = contents;
            self.editor_active = true;
            self.editor_autosync = false;
            self.editor_dirty = false;
            self.file_path = Some(path);
            self.active_tab = DesignerTab::Code;
            let source = self.editor_code.clone();
            self.apply_code_to_designer(&source);
        }
    }

    fn close_file(&mut self) {
        self.editor_code.clear();
        self.editor_active = false;
        self.editor_autosync = true;
        self.editor_dirty = false;
        self.file_path = None;
        self.active_tab = DesignerTab::Form;
    }

    fn save_to_path(&mut self, path: PathBuf) {
        let _ = std::fs::write(&path, self.active_code());
        self.file_path = Some(path);
    }

    fn apply_code_to_designer(&mut self, source: &str) {
        match parse(source) {
            Ok(stmts) => {
                if let Some(form_def) = stmts.into_iter().find_map(|s| match s {
                    Stmt::FormDef(fd) => Some(fd),
                    _ => None,
                }) {
                    self.load_form_definition(form_def);
                    self.parse_error = None;
                }
            }
            Err(err) => {
                self.parse_error = Some(format!("Parse error: {err}"));
            }
        }
    }

    fn load_form_definition(&mut self, form_def: rundell_parser::ast::FormDefinition) {
        self.form_name = form_def.name;
        self.controls.clear();
        self.selected = None;
        self.name_counter.clear();

        let mut index_by_name: HashMap<String, usize> = HashMap::new();

        for stmt in form_def.body {
            match stmt {
                Stmt::DefineControl(name, ctrl_type) => {
                    let state = rundell_interpreter::form_registry::default_control_state(&ctrl_type);
                    let events = ControlEvents::default();
                    let idx = self.controls.len();
                    self.controls.push(DesignControl { name: name.clone(), ctrl_type, state, events });
                    index_by_name.insert(name, idx);
                }
                Stmt::Set(set_stmt) => {
                    if let SetTarget::ObjectPath(path) = set_stmt.target {
                        if let SetOp::Assign(expr) = set_stmt.op {
                            self.apply_object_path(&index_by_name, &path, &expr);
                        }
                    }
                }
                _ => {}
            }
        }

        self.rebuild_name_counter();
    }

    fn apply_object_path(&mut self, index_by_name: &HashMap<String, usize>, path: &[String], expr: &Expr) {
        if path.is_empty() {
            return;
        }

        let (first, rest) = path.split_first().unwrap();
        if first == "form" {
            return;
        }

        if rest.len() != 1 {
            return;
        }

        let prop = &rest[0];
        let Some(&idx) = index_by_name.get(first) else { return; };
        let ctrl = &mut self.controls[idx];

        if prop == "position" {
            if let Expr::PositionLiteral(top, left, width, height) = expr {
                ctrl.state.set_position(*top, *left, *width, *height);
            }
            return;
        }

        if prop == "click" || prop == "change" || prop == "select" {
            if let Expr::Call(name, args) = expr {
                if args.is_empty() {
                    match prop.as_str() {
                        "click" => ctrl.events.click = name.clone(),
                        "change" => ctrl.events.change = name.clone(),
                        "select" => ctrl.events.select = name.clone(),
                        _ => {}
                    }
                }
            }
        }

        if let Some(val) = expr_to_string(expr) {
            let _ = ctrl.state.set_property(prop, &val);
        }
    }

    fn rebuild_name_counter(&mut self) {
        for ctrl in &self.controls {
            let (prefix, num) = split_name(&ctrl.name);
            if let Some(num) = num {
                let counter = self.name_counter.entry(prefix).or_insert(0);
                if num > *counter {
                    *counter = num;
                }
            }
        }
    }

    fn render_form_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(format!("Form: {}", self.form_name));
        ui.separator();
        self.canvas.show(ui, &mut self.controls, &mut self.selected);
    }

    fn render_code_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Generate Code").clicked() {
                self.generated_code = codegen::generate(&self.form_name, &self.controls);
                if self.editor_autosync || !self.editor_dirty {
                    self.editor_code = self.generated_code.clone();
                }
                let source = self.generated_code.clone();
                self.apply_code_to_designer(&source);
            }
            if ui.button("Copy to Clipboard").clicked() {
                ui.output_mut(|o| o.copied_text = self.active_code().to_string());
            }
            if ui.button("Save to File\u{2026}").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Save Form")
                    .add_filter("Rundell", &["run"])
                    .save_file()
                {
                    let _ = std::fs::write(&path, self.active_code());
                    self.file_path = Some(path);
                }
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.selectable_label(!self.editor_active, "Generated").clicked() {
                self.editor_active = false;
            }
            if ui.selectable_label(self.editor_active, "Editor").clicked() {
                self.editor_active = true;
            }
            ui.checkbox(&mut self.editor_autosync, "Auto-sync editor");
            if self.editor_active && ui.button("Use Generated").clicked() {
                self.editor_code = self.generated_code.clone();
                self.editor_dirty = false;
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.editor_active {
                let available = ui.available_size();
                let response = ui.add_sized(
                    available,
                    egui::TextEdit::multiline(&mut self.editor_code)
                        .code_editor()
                );
                if response.changed() {
                    self.editor_dirty = true;
                    let source = self.editor_code.clone();
                    self.apply_code_to_designer(&source);
                }
            } else {
                let mut text = self.generated_code.clone();
                let available = ui.available_size();
                ui.add_sized(
                    available,
                    egui::TextEdit::multiline(&mut text)
                        .code_editor()
                        .interactive(false)
                );
            }
        });
        if let Some(err) = &self.parse_error {
            ui.separator();
            ui.colored_label(egui::Color32::from_rgb(160, 0, 0), err);
        }
    }
}

fn expr_to_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(lit) => match lit {
            Literal::Str(s) => Some(s.clone()),
            Literal::Integer(n) => Some(n.to_string()),
            Literal::Float(f) => Some(f.to_string()),
            Literal::Boolean(b) => Some(b.to_string()),
            Literal::Currency(c) => Some(format!("{:.2}", *c as f64 / 100.0)),
            Literal::Null => None,
        },
        Expr::Identifier(name) => Some(name.clone()),
        _ => None,
    }
}

fn split_name(name: &str) -> (String, Option<usize>) {
    let mut chars = name.chars().peekable();
    let mut prefix = String::new();
    let mut digits = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            break;
        }
        prefix.push(ch);
        chars.next();
    }
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else {
            digits.clear();
            break;
        }
    }
    let number = digits.parse::<usize>().ok();
    (prefix, number)
}

impl eframe::App for DesignerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let ui_enabled = !self.pending_delete;
        if self.editor_autosync && !self.editor_active {
            let next_code = codegen::generate(&self.form_name, &self.controls);
            if next_code != self.generated_code {
                self.generated_code = next_code.clone();
            }
            if self.editor_code != next_code {
                self.editor_code = next_code;
            }
            self.editor_dirty = false;
        }
        let delete_pressed = ctx.input(|i| i.key_pressed(egui::Key::Delete));
        let undo_pressed = ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl);

        if delete_pressed {
            self.request_delete();
        }
        if undo_pressed {
            self.undo();
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.set_enabled(ui_enabled);
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Open Form")
                            .add_filter("Rundell", &["run"])
                            .pick_file()
                        {
                            self.open_file(path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        if let Some(path) = self.file_path.clone() {
                            self.save_to_path(path);
                        } else if let Some(path) = rfd::FileDialog::new()
                            .set_title("Save Form")
                            .add_filter("Rundell", &["run"])
                            .save_file()
                        {
                            self.save_to_path(path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Save Form As")
                            .add_filter("Rundell", &["run"])
                            .save_file()
                        {
                            self.save_to_path(path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Close").clicked() {
                        self.close_file();
                        ui.close_menu();
                    }
                });
            });
        });
        if self.active_tab != DesignerTab::Code {
            // --- Left panel: tool palette ---
            egui::SidePanel::left("palette").min_width(120.0).show(ctx, |ui| {
                ui.set_enabled(ui_enabled);
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
                ui.set_enabled(ui_enabled);
                ui.heading("Properties");
                ui.separator();
                if ui.button("Undo").clicked() {
                    self.undo();
                }
                if self.selected.is_some() {
                    if ui.button("Delete Selected").clicked() {
                        self.request_delete();
                    }
                }
                ui.separator();
                if let Some(idx) = self.selected {
                    if let Some(ctrl) = self.controls.get_mut(idx) {
                        self.inspector.show(ui, ctrl);
                    }
                } else {
                    ui.label("Select a control to edit its properties.");
                }
            });
        }

        // --- Central panel: design canvas ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(ui_enabled);
            ui.horizontal(|ui| {
                if ui.selectable_label(self.active_tab == DesignerTab::Form, "Form").clicked() {
                    self.active_tab = DesignerTab::Form;
                }
                if ui.selectable_label(self.active_tab == DesignerTab::Code, "Code").clicked() {
                    self.active_tab = DesignerTab::Code;
                }
                if ui.selectable_label(self.active_tab == DesignerTab::Split, "Split").clicked() {
                    self.active_tab = DesignerTab::Split;
                }
            });
            ui.separator();

            match self.active_tab {
                DesignerTab::Form => self.render_form_panel(ui),
                DesignerTab::Code => self.render_code_panel(ui),
                DesignerTab::Split => {
                    ui.columns(2, |cols| {
                        self.render_form_panel(&mut cols[0]);
                        self.render_code_panel(&mut cols[1]);
                    });
                }
            }
        });

        if self.pending_delete {
            egui::Window::new("Delete control?")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui: &mut egui::Ui| {
                    ui.label("Delete the selected control? This can be undone.");
                    ui.horizontal(|ui: &mut egui::Ui| {
                        if ui.button("Delete").clicked() {
                            self.delete_selected_confirmed();
                            self.pending_delete = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.pending_delete = false;
                        }
                    });
                });
        }
    }
}
