//! Label control renderer.

use egui::{Context, FontId, RichText, Ui, pos2, vec2};
use rundell_interpreter::form_registry::Position;
use crate::form_runtime::{hex_to_color32, EventTuple};

/// Render a label control. Labels produce no events.
pub fn render(
    _ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    value: &str,
    text_color: &str,
    font_size: u32,
) -> Vec<EventTuple> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    egui::Area::new(id)
        .fixed_pos(pos2(position.left as f32, position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            let color = hex_to_color32(text_color);
            ui.colored_label(color, RichText::new(value).font(FontId::proportional(font_size as f32)));
        });
    vec![]
}
