//! Property inspector panel for the form designer.

use egui::{Color32, Ui};
use super::DesignControl;
use rundell_interpreter::form_registry::{ControlState, TextAlign};

/// Shows editable properties for the selected control.
pub struct Inspector;

impl Inspector {
    pub fn new() -> Self { Inspector }

    /// Render the inspector panel for a given control.
    pub fn show(&mut self, ui: &mut Ui, ctrl: &mut DesignControl) {
        ui.label(format!("Type: {:?}", ctrl.ctrl_type));
        ui.separator();

        // Name field
        ui.label("Name:");
        ui.text_edit_singleline(&mut ctrl.name);
        ui.separator();

        // Position
        let (mut t, mut l, mut w, mut h) = self.get_pos_mut(&ctrl.state);
        ui.label("Position (top, left, width, height):");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut t).prefix("T:").speed(1.0));
            ui.add(egui::DragValue::new(&mut l).prefix("L:").speed(1.0));
            ui.add(egui::DragValue::new(&mut w).prefix("W:").speed(1.0));
            ui.add(egui::DragValue::new(&mut h).prefix("H:").speed(1.0));
        });
        // Apply changed position
        ctrl.state.set_position(t, l, w, h);

        ui.separator();

        // Control-specific properties
        match &mut ctrl.state {
            ControlState::Label { value, text_color, text_align, .. } => {
                ui.label("Value:"); ui.text_edit_singleline(value);
                ui.label("Text color:"); ui.text_edit_singleline(text_color);
                Self::text_align_picker(ui, text_align);
            }
            ControlState::Textbox { value, placeholder, readonly, text_align, .. } => {
                ui.label("Value:"); ui.text_edit_singleline(value);
                ui.label("Placeholder:"); ui.text_edit_singleline(placeholder);
                ui.checkbox(readonly, "Read-only");
                Self::text_align_picker(ui, text_align);
            }
            ControlState::Button { caption, text_color, background_color, text_align, .. } => {
                ui.label("Caption:"); ui.text_edit_singleline(caption);
                ui.label("Text color:"); ui.text_edit_singleline(text_color);
                ui.label("Background:"); ui.text_edit_singleline(background_color);
                Self::text_align_picker(ui, text_align);
            }
            ControlState::Radiobutton { caption, group, text_align, .. } => {
                ui.label("Caption:"); ui.text_edit_singleline(caption);
                ui.label("Group:"); ui.text_edit_singleline(group);
                Self::text_align_picker(ui, text_align);
            }
            ControlState::Checkbox { caption, checked, text_align, .. } => {
                ui.label("Caption:"); ui.text_edit_singleline(caption);
                ui.checkbox(checked, "Checked");
                Self::text_align_picker(ui, text_align);
            }
            ControlState::Switch { caption, checked, text_align, .. } => {
                ui.label("Caption:"); ui.text_edit_singleline(caption);
                ui.checkbox(checked, "Checked");
                Self::text_align_picker(ui, text_align);
            }
            ControlState::Select { text_align, .. } => {
                ui.label("(Items set via code)");
                Self::text_align_picker(ui, text_align);
            }
            ControlState::Listbox { columns, .. } => {
                ui.label("Columns (comma-separated):");
                let mut cols_str = columns.join(", ");
                if ui.text_edit_singleline(&mut cols_str).changed() {
                    *columns = cols_str.split(',').map(|s| s.trim().to_string()).collect();
                }
            }
        }

        ui.separator();
        ui.label("Events:");
        match ctrl.ctrl_type {
            rundell_parser::ast::ControlType::Button => {
                Self::handler_input(ui, "click:", &mut ctrl.events.click);
            }
            rundell_parser::ast::ControlType::Textbox => {
                Self::handler_input(ui, "change:", &mut ctrl.events.change);
            }
            rundell_parser::ast::ControlType::Radiobutton
            | rundell_parser::ast::ControlType::Checkbox
            | rundell_parser::ast::ControlType::Switch
            | rundell_parser::ast::ControlType::Select => {
                Self::handler_input(ui, "change:", &mut ctrl.events.change);
            }
            rundell_parser::ast::ControlType::Listbox => {
                Self::handler_input(ui, "change:", &mut ctrl.events.change);
                Self::handler_input(ui, "select:", &mut ctrl.events.select);
            }
            rundell_parser::ast::ControlType::Label => {
                ui.label("(No events)");
            }
        }
    }

    fn handler_input(ui: &mut Ui, label: &str, value: &mut String) {
        ui.label(label);
        ui.text_edit_singleline(value);
        let trimmed = value.trim();
        if !trimmed.is_empty() && !is_valid_identifier(trimmed) {
            ui.colored_label(
                Color32::from_rgb(160, 80, 0),
                "Use letters, digits, underscore; must start with a letter",
            );
        }
    }

    fn text_align_picker(ui: &mut Ui, text_align: &mut TextAlign) {
        ui.label("Text align:");
        egui::ComboBox::from_id_source(ui.id().with("text_align"))
            .selected_text(text_align.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(text_align, TextAlign::Left, "left");
                ui.selectable_value(text_align, TextAlign::Center, "center");
                ui.selectable_value(text_align, TextAlign::Right, "right");
            });
    }

    /// Returns (top, left, width, height) as u32 values.
    fn get_pos_mut(&self, state: &ControlState) -> (u32, u32, u32, u32) {
        use rundell_interpreter::form_registry::ControlState;
        let pos = match state {
            ControlState::Label       { position, .. } => position,
            ControlState::Textbox     { position, .. } => position,
            ControlState::Button      { position, .. } => position,
            ControlState::Radiobutton { position, .. } => position,
            ControlState::Checkbox    { position, .. } => position,
            ControlState::Switch      { position, .. } => position,
            ControlState::Select      { position, .. } => position,
            ControlState::Listbox     { position, .. } => position,
        };
        (pos.top, pos.left, pos.width, pos.height)
    }
}

fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return false; };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || ch == '_') {
            return false;
        }
    }
    true
}
