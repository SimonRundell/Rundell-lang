//! Code generator: produces Rundell form-definition source from designer state.

use super::DesignControl;
use rundell_interpreter::form_registry::ControlState;
use rundell_parser::ast::ControlType;

/// Generate a Rundell `define ... as form --> ... <--` block from the current
/// canvas state.
///
/// Only properties that differ from their defaults are emitted.
pub fn generate(form_name: &str, controls: &[DesignControl]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("define {form_name} as form -->"));

    for ctrl in controls {
        let type_name = ctrl_type_name(&ctrl.ctrl_type);
        lines.push(format!("    define {} as form\\{type_name}.", ctrl.name));
        emit_properties(&mut lines, &ctrl.name, &ctrl.state);
    }

    lines.push("<--".to_string());
    lines.join("\n")
}

fn ctrl_type_name(ct: &ControlType) -> &'static str {
    match ct {
        ControlType::Label       => "label",
        ControlType::Textbox     => "textbox",
        ControlType::Button      => "button",
        ControlType::Radiobutton => "radiobutton",
        ControlType::Checkbox    => "checkbox",
        ControlType::Switch      => "switch",
        ControlType::Select      => "select",
        ControlType::Listbox     => "listbox",
    }
}

/// Emit non-default property assignments for a control.
fn emit_properties(lines: &mut Vec<String>, name: &str, state: &ControlState) {
    // Position is always emitted.
    let pos = get_position(state);
    lines.push(format!(
        "    set {}\\position = {}px, {}px, {}px, {}px.",
        name, pos.top, pos.left, pos.width, pos.height
    ));

    match state {
        ControlState::Label { value, text_color, font_size, .. } => {
            if !value.is_empty() {
                lines.push(format!("    set {}\\value = \"{value}\".", name));
            }
            if text_color != "#000000" {
                lines.push(format!("    set {}\\textcolor = \"{text_color}\".", name));
            }
            if *font_size != 12 {
                lines.push(format!("    set {}\\fontsize = {font_size}.", name));
            }
        }
        ControlState::Textbox { value, placeholder, readonly, text_color, .. } => {
            if !value.is_empty() {
                lines.push(format!("    set {}\\value = \"{value}\".", name));
            }
            if !placeholder.is_empty() {
                lines.push(format!("    set {}\\placeholder = \"{placeholder}\".", name));
            }
            if *readonly {
                lines.push(format!("    set {}\\readonly = true.", name));
            }
            if text_color != "#000000" {
                lines.push(format!("    set {}\\textcolor = \"{text_color}\".", name));
            }
        }
        ControlState::Button { caption, text_color, background_color, .. } => {
            if !caption.is_empty() {
                lines.push(format!("    set {}\\caption = \"{caption}\".", name));
            }
            if text_color != "#000000" {
                lines.push(format!("    set {}\\textcolor = \"{text_color}\".", name));
            }
            if background_color != "#E0E0E0" {
                lines.push(format!("    set {}\\backgroundcolor = \"{background_color}\".", name));
            }
        }
        ControlState::Radiobutton { caption, group, checked, .. } => {
            if !caption.is_empty() {
                lines.push(format!("    set {}\\caption = \"{caption}\".", name));
            }
            if !group.is_empty() {
                lines.push(format!("    set {}\\group = \"{group}\".", name));
            }
            if *checked {
                lines.push(format!("    set {}\\checked = true.", name));
            }
        }
        ControlState::Checkbox { caption, checked, .. } => {
            if !caption.is_empty() {
                lines.push(format!("    set {}\\caption = \"{caption}\".", name));
            }
            if *checked {
                lines.push(format!("    set {}\\checked = true.", name));
            }
        }
        ControlState::Switch { caption, checked, .. } => {
            if !caption.is_empty() {
                lines.push(format!("    set {}\\caption = \"{caption}\".", name));
            }
            if *checked {
                lines.push(format!("    set {}\\checked = true.", name));
            }
        }
        ControlState::Select { .. } => {
            // Items are typically set via code, not in the designer.
        }
        ControlState::Listbox { columns, multi_select, row_height, header_visible, .. } => {
            if !columns.is_empty() {
                let arr: serde_json::Value = serde_json::Value::Array(
                    columns.iter().map(|c| serde_json::Value::String(c.clone())).collect()
                );
                lines.push(format!("    set {}\\columns = {}.", name, arr));
            }
            if *multi_select {
                lines.push(format!("    set {}\\multiselect = true.", name));
            }
            if *row_height != 24 {
                lines.push(format!("    set {}\\rowheight = {row_height}.", name));
            }
            if !header_visible {
                lines.push(format!("    set {}\\headervisible = false.", name));
            }
        }
    }
}

fn get_position(state: &ControlState) -> rundell_interpreter::form_registry::Position {
    use rundell_interpreter::form_registry::ControlState;
    match state {
        ControlState::Label       { position, .. } => position.clone(),
        ControlState::Textbox     { position, .. } => position.clone(),
        ControlState::Button      { position, .. } => position.clone(),
        ControlState::Radiobutton { position, .. } => position.clone(),
        ControlState::Checkbox    { position, .. } => position.clone(),
        ControlState::Switch      { position, .. } => position.clone(),
        ControlState::Select      { position, .. } => position.clone(),
        ControlState::Listbox     { position, .. } => position.clone(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rundell_interpreter::form_registry::{default_control_state, ControlState};
    use rundell_parser::ast::ControlType;

    /// Round-trip test: generate code from a known canvas state and verify.
    #[test]
    fn codegen_round_trip() {
        let mut button_state = default_control_state(&ControlType::Button);
        button_state.set_position(50, 10, 120, 30);
        if let ControlState::Button { ref mut caption, .. } = button_state {
            *caption = "Submit".to_string();
        }

        let controls = vec![DesignControl {
            name: "submitBtn".to_string(),
            ctrl_type: ControlType::Button,
            state: button_state,
        }];

        let code = generate("myForm", &controls);
        assert!(code.contains("define myForm as form -->"));
        assert!(code.contains("define submitBtn as form\\button."));
        assert!(code.contains("set submitBtn\\position = 50px, 10px, 120px, 30px."));
        assert!(code.contains("set submitBtn\\caption = \"Submit\"."));
        assert!(code.contains("<--"));
    }
}
