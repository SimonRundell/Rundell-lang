//! Radiobutton control renderer.

use egui::{Align, Context, FontId, Layout, RadioButton, RichText, Ui, pos2, vec2};
use rundell_interpreter::form_registry::{Position, TextAlign};
use crate::form_runtime::GuiEvent;

/// Render a radiobutton. Fires `"change"` when checked state changes.
pub fn render(
    ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    caption: &str,
    checked: &mut bool,
    font: &str,
    font_size: u32,
    enabled: bool,
    text_align: TextAlign,
) -> Vec<GuiEvent> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut events = Vec::new();
    let current = *checked;
    let origin = ui.available_rect_before_wrap().min;

    egui::Area::new(id)
        .fixed_pos(pos2(origin.x + position.left as f32, origin.y + position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            let layout = layout_from_text_align(text_align);
            ui.allocate_ui_with_layout(vec2(position.width as f32, position.height as f32), layout, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    let font_id = font_id(font, font_size);
                    let caption_text = RichText::new(caption).font(font_id);
                    if ui.add(RadioButton::new(current, caption_text)).clicked() {
                        *checked = true;
                        events.push(GuiEvent {
                            form: form_name.to_string(),
                            control: ctrl_name.to_string(),
                            event: "change".to_string(),
                            value: Some("true".to_string()),
                        });
                    }
                });
            });
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

fn font_id(font: &str, font_size: u32) -> FontId {
    let trimmed = font.trim();
    let family = match trimmed.to_ascii_lowercase().as_str() {
        "" | "default" | "proportional" => egui::FontFamily::Proportional,
        "monospace" => egui::FontFamily::Monospace,
        _ => egui::FontFamily::Name(trimmed.into()),
    };
    FontId::new(font_size as f32, family)
}
