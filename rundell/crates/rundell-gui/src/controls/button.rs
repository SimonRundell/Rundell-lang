//! Button control renderer.

use egui::{Align, Button, Context, Layout, Ui, pos2, vec2};
use rundell_interpreter::form_registry::{Position, TextAlign};
use crate::form_runtime::{hex_to_color32, GuiEvent};

/// Render a button. Fires `"click"` when clicked.
pub fn render(
    ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    caption: &str,
    text_color: &str,
    background_color: &str,
    enabled: bool,
    text_align: TextAlign,
) -> Vec<GuiEvent> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut events = Vec::new();
    let origin = ui.available_rect_before_wrap().min;

    egui::Area::new(id)
        .fixed_pos(pos2(origin.x + position.left as f32, origin.y + position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            let bg = hex_to_color32(background_color);
            let fg = hex_to_color32(text_color);
            let btn = Button::new(egui::RichText::new(caption).color(fg))
                .fill(bg)
                .min_size(vec2(position.width as f32, position.height as f32));
            let layout = layout_from_text_align(text_align);
            let response = ui.allocate_ui_with_layout(
                vec2(position.width as f32, position.height as f32),
                layout,
                |ui| ui.add_enabled(enabled, btn),
            ).inner;
            if response.clicked() {
                events.push(GuiEvent {
                    form: form_name.to_string(),
                    control: ctrl_name.to_string(),
                    event: "click".to_string(),
                    value: None,
                });
            }
        });

    events
}

fn layout_from_text_align(text_align: TextAlign) -> Layout {
    match text_align {
        TextAlign::Left => Layout::left_to_right(Align::Min),
        TextAlign::Center => Layout::left_to_right(Align::Center),
        TextAlign::Right => Layout::left_to_right(Align::Max),
    }
}
