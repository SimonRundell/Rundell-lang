//! Dropdown select (ComboBox) control renderer.

use egui::{ComboBox, Context, Ui, pos2, vec2};
use rundell_interpreter::form_registry::Position;
use crate::form_runtime::EventTuple;

/// Render a dropdown select. Fires `"change"` when selection changes.
pub fn render(
    _ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    items: &[String],
    selected_index: Option<usize>,
    enabled: bool,
) -> Vec<EventTuple> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut events = Vec::new();
    let mut current = selected_index.unwrap_or(0);
    let label = items.get(current).cloned().unwrap_or_default();

    egui::Area::new(id)
        .fixed_pos(pos2(position.left as f32, position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            ui.add_enabled_ui(enabled, |ui| {
                let combo = ComboBox::from_id_source(format!("{form_name}_{ctrl_name}_combo"))
                    .selected_text(&label)
                    .width(position.width as f32);
                combo.show_ui(ui, |ui: &mut egui::Ui| {
                    for (i, item) in items.iter().enumerate() {
                        if ui.selectable_value(&mut current, i, item).clicked() {
                            events.push((
                                form_name.to_string(),
                                ctrl_name.to_string(),
                                "change".to_string(),
                            ));
                        }
                    }
                });
            });
        });

    events
}
