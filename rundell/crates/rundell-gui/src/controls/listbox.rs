//! Listbox (data-bound multi-column list) control renderer.

use egui::{Context, ScrollArea, Ui, pos2, vec2};
use rundell_interpreter::form_registry::Position;
use crate::form_runtime::EventTuple;

/// Render a listbox. Fires `"change"` on selection change, `"select"` on
/// double-click.
#[allow(clippy::too_many_arguments)]
pub fn render(
    _ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    position: &Position,
    data_source: &Option<serde_json::Value>,
    columns: &[String],
    image_column: Option<&str>,
    multi_select: bool,
    row_height: u32,
    header_visible: bool,
    selected_indices: &[usize],
    enabled: bool,
) -> Vec<EventTuple> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut events = Vec::new();

    // Suppress unused variable warnings for parameters used only for future features
    let _ = multi_select;
    let _ = row_height;

    // Extract rows from data_source.
    let rows: Vec<&serde_json::Value> = data_source
        .as_ref()
        .and_then(|ds| {
            ds.get("rows")
                .or_else(|| ds.get("records"))
                .and_then(|v| v.as_array())
                .or_else(|| ds.as_array())
        })
        .map(|arr| arr.iter().collect())
        .unwrap_or_default();

    egui::Area::new(id)
        .fixed_pos(pos2(position.left as f32, position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            ui.add_enabled_ui(enabled, |ui| {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                    .show(ui, |ui| {
                        if header_visible && !columns.is_empty() {
                            ui.horizontal(|ui| {
                                for col in columns {
                                    ui.label(egui::RichText::new(col).strong());
                                    ui.separator();
                                }
                            });
                            ui.separator();
                        }

                        ScrollArea::vertical()
                            .max_height(position.height as f32)
                            .show(ui, |ui| {
                                for (row_idx, row) in rows.iter().enumerate() {
                                    let is_selected = selected_indices.contains(&row_idx);
                                    ui.horizontal(|ui| {
                                        // Image column
                                        if let Some(img_col) = image_column {
                                            if let Some(b64) = row.get(img_col).and_then(|v| v.as_str()) {
                                                use base64::Engine as _;
                                                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
                                                    // Placeholder — full image rendering via egui_extras would go here
                                                    ui.label("🖼");
                                                    let _ = bytes;
                                                }
                                            }
                                        }

                                        // Data columns
                                        let row_text = if columns.is_empty() {
                                            row.to_string()
                                        } else {
                                            columns.iter()
                                                .map(|c| row.get(c)
                                                    .map(|v| match v {
                                                        serde_json::Value::String(s) => s.clone(),
                                                        other => other.to_string(),
                                                    })
                                                    .unwrap_or_default())
                                                .collect::<Vec<_>>()
                                                .join("  |  ")
                                        };

                                        let resp = ui.selectable_label(is_selected, &row_text);
                                        if resp.clicked() {
                                            events.push((
                                                form_name.to_string(),
                                                ctrl_name.to_string(),
                                                "change".to_string(),
                                            ));
                                        }
                                        if resp.double_clicked() {
                                            events.push((
                                                form_name.to_string(),
                                                ctrl_name.to_string(),
                                                "select".to_string(),
                                            ));
                                        }
                                    });
                                }
                            });
                    });
            });
        });

    events
}
