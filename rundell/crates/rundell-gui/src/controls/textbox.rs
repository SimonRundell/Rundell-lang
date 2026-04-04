//! Textbox (single-line text input) control renderer.

use egui::{Align, Context, FontId, TextEdit, Ui, pos2, vec2};
use rundell_interpreter::form_registry::{Position, TextAlign};
use crate::form_runtime::{hex_to_color32, GuiEvent};

/// Render a textbox. Fires `("change", ...)` when text is edited.
pub fn render(
    ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    value: &mut String,
    text_color: &str,
    _text_background: &str,
    font: &str,
    font_size: u32,
    readonly: bool,
    placeholder: &str,
    text_align: TextAlign,
) -> Vec<GuiEvent> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut text = value.clone();
    let mut events = Vec::new();
    let origin = ui.available_rect_before_wrap().min;

    egui::Area::new(id)
        .fixed_pos(pos2(origin.x + position.left as f32, origin.y + position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            let mut edit = TextEdit::singleline(&mut text)
                .desired_width(position.width as f32)
                .hint_text(placeholder)
                .font(font_id(font, font_size))
                .horizontal_align(align_from_text_align(text_align));
            if readonly {
                edit = edit.interactive(false);
            }
            let response = ui.add(edit);
            if response.changed() {
                *value = text.clone();
                events.push(GuiEvent {
                    form: form_name.to_string(),
                    control: ctrl_name.to_string(),
                    event: "change".to_string(),
                    value: Some(text.clone()),
                });
            }
        });

    // Suppress unused import warning - text_color is accepted for API consistency
    let _ = hex_to_color32(text_color);

    events
}

fn align_from_text_align(text_align: TextAlign) -> Align {
    match text_align {
        TextAlign::Left => Align::Min,
        TextAlign::Center => Align::Center,
        TextAlign::Right => Align::Max,
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
