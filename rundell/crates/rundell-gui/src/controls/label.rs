//! Label control renderer.

use egui::{Align, Context, FontId, Layout, RichText, Ui, pos2, vec2};
use rundell_interpreter::form_registry::{Position, TextAlign};
use crate::form_runtime::{hex_to_color32, GuiEvent};

/// Render a label control. Labels produce no events.
pub fn render(
    ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    value: &str,
    text_color: &str,
    font_size: u32,
    text_align: TextAlign,
) -> Vec<GuiEvent> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let origin = ui.available_rect_before_wrap().min;
    egui::Area::new(id)
        .fixed_pos(pos2(origin.x + position.left as f32, origin.y + position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            let color = hex_to_color32(text_color);
            let layout = layout_from_text_align(text_align);
            ui.allocate_ui_with_layout(vec2(position.width as f32, position.height as f32), layout, |ui| {
                ui.colored_label(color, RichText::new(value).font(FontId::proportional(font_size as f32)));
            });
        });
    vec![]
}

fn layout_from_text_align(text_align: TextAlign) -> Layout {
    match text_align {
        TextAlign::Left => Layout::left_to_right(Align::Min),
        TextAlign::Center => Layout::left_to_right(Align::Center),
        TextAlign::Right => Layout::left_to_right(Align::Max),
    }
}
