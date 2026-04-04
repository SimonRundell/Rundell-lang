//! Dropdown select (ComboBox) control renderer.

use egui::{Align, ComboBox, Context, Layout, Ui, pos2, vec2};
use rundell_interpreter::form_registry::{Position, TextAlign};
use crate::form_runtime::GuiEvent;

/// Render a dropdown select. Fires `"change"` when selection changes.
pub fn render(
    ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    items: &[String],
    selected_index: &mut Option<usize>,
    enabled: bool,
    text_align: TextAlign,
) -> Vec<GuiEvent> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut events = Vec::new();
    let mut current = selected_index.unwrap_or(0);
    let label = items.get(current).cloned().unwrap_or_default();
    let origin = ui.available_rect_before_wrap().min;

    egui::Area::new(id)
        .fixed_pos(pos2(origin.x + position.left as f32, origin.y + position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            let layout = layout_from_text_align(text_align);
            ui.allocate_ui_with_layout(vec2(position.width as f32, position.height as f32), layout, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    let combo = ComboBox::from_id_source(format!("{form_name}_{ctrl_name}_combo"))
                        .selected_text(&label)
                        .width(position.width as f32);
                    combo.show_ui(ui, |ui: &mut egui::Ui| {
                        for (i, item) in items.iter().enumerate() {
                            if ui.selectable_value(&mut current, i, item).clicked() {
                                *selected_index = Some(current);
                                let selected_text = items.get(current).cloned().unwrap_or_default();
                                events.push(GuiEvent {
                                    form: form_name.to_string(),
                                    control: ctrl_name.to_string(),
                                    event: "change".to_string(),
                                    value: Some(selected_text),
                                });
                            }
                        }
                    });
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
