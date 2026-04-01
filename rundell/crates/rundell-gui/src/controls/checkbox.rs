//! Checkbox control renderer.

use egui::{Context, Ui, pos2, vec2};
use rundell_interpreter::form_registry::Position;
use crate::form_runtime::EventTuple;

/// Render a checkbox. Fires `"change"` when checked state changes.
pub fn render(
    _ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    caption: &str,
    checked: bool,
    enabled: bool,
) -> Vec<EventTuple> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut events = Vec::new();
    let mut current = checked;

    egui::Area::new(id)
        .fixed_pos(pos2(position.left as f32, position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            ui.add_enabled_ui(enabled, |ui| {
                if ui.checkbox(&mut current, caption).changed() {
                    events.push((form_name.to_string(), ctrl_name.to_string(), "change".to_string()));
                }
            });
        });

    events
}
