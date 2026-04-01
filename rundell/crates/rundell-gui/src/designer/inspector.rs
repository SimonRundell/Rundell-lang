//! Property inspector panel for the form designer.

use egui::Ui;
use super::DesignControl;
use rundell_interpreter::form_registry::ControlState;

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
            ControlState::Label { value, text_color, .. } => {
                ui.label("Value:"); ui.text_edit_singleline(value);
                ui.label("Text color:"); ui.text_edit_singleline(text_color);
            }
            ControlState::Textbox { value, placeholder, readonly, .. } => {
                ui.label("Value:"); ui.text_edit_singleline(value);
                ui.label("Placeholder:"); ui.text_edit_singleline(placeholder);
                ui.checkbox(readonly, "Read-only");
            }
            ControlState::Button { caption, text_color, background_color, .. } => {
                ui.label("Caption:"); ui.text_edit_singleline(caption);
                ui.label("Text color:"); ui.text_edit_singleline(text_color);
                ui.label("Background:"); ui.text_edit_singleline(background_color);
            }
            ControlState::Radiobutton { caption, group, .. } => {
                ui.label("Caption:"); ui.text_edit_singleline(caption);
                ui.label("Group:"); ui.text_edit_singleline(group);
            }
            ControlState::Checkbox { caption, checked, .. } => {
                ui.label("Caption:"); ui.text_edit_singleline(caption);
                ui.checkbox(checked, "Checked");
            }
            ControlState::Switch { caption, checked, .. } => {
                ui.label("Caption:"); ui.text_edit_singleline(caption);
                ui.checkbox(checked, "Checked");
            }
            ControlState::Select { .. } => {
                ui.label("(Items set via code)");
            }
            ControlState::Listbox { columns, .. } => {
                ui.label("Columns (comma-separated):");
                let mut cols_str = columns.join(", ");
                if ui.text_edit_singleline(&mut cols_str).changed() {
                    *columns = cols_str.split(',').map(|s| s.trim().to_string()).collect();
                }
            }
        }
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
