//! Textbox (single-line text input) control renderer.

use egui::{Context, TextEdit, Ui, pos2, vec2};
use rundell_interpreter::form_registry::Position;
use crate::form_runtime::{hex_to_color32, EventTuple};

/// Render a textbox. Fires `("change", ...)` when text is edited.
pub fn render(
    _ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    value: &str,
    text_color: &str,
    _text_background: &str,
    readonly: bool,
    placeholder: &str,
) -> Vec<EventTuple> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut text = value.to_string();
    let mut events = Vec::new();

    egui::Area::new(id)
        .fixed_pos(pos2(position.left as f32, position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            let mut edit = TextEdit::singleline(&mut text)
                .desired_width(position.width as f32)
                .hint_text(placeholder);
            if readonly {
                edit = edit.interactive(false);
            }
            let response = ui.add(edit);
            if response.changed() {
                events.push((form_name.to_string(), ctrl_name.to_string(), "change".to_string()));
            }
        });

    // Suppress unused import warning - text_color is accepted for API consistency
    let _ = hex_to_color32(text_color);

    events
}
