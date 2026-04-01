//! Button control renderer.

use egui::{Button, Context, Ui, pos2, vec2};
use rundell_interpreter::form_registry::Position;
use crate::form_runtime::{hex_to_color32, EventTuple};

/// Render a button. Fires `"click"` when clicked.
pub fn render(
    _ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    caption: &str,
    text_color: &str,
    background_color: &str,
    enabled: bool,
) -> Vec<EventTuple> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut events = Vec::new();

    egui::Area::new(id)
        .fixed_pos(pos2(position.left as f32, position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            let bg = hex_to_color32(background_color);
            let fg = hex_to_color32(text_color);
            let btn = Button::new(egui::RichText::new(caption).color(fg))
                .fill(bg)
                .min_size(vec2(position.width as f32, position.height as f32));
            let response = ui.add_enabled(enabled, btn);
            if response.clicked() {
                events.push((form_name.to_string(), ctrl_name.to_string(), "click".to_string()));
            }
        });

    events
}
