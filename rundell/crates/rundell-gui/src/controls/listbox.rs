//! Listbox (data-bound multi-column list) control renderer.

use egui::{Context, FontId, RichText, ScrollArea, Ui, pos2, vec2};
use rundell_interpreter::form_registry::Position;
use crate::form_runtime::GuiEvent;

/// Render a listbox. Fires `"change"` on selection change, `"select"` on
/// double-click.
#[allow(clippy::too_many_arguments)]
pub fn render(
    ui: &mut Ui,
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
    selected_indices: &mut Vec<usize>,
    font: &str,
    font_size: u32,
    enabled: bool,
) -> Vec<GuiEvent> {
    let id = egui::Id::new(format!("{form_name}_{ctrl_name}"));
    let mut events = Vec::new();
    let origin = ui.available_rect_before_wrap().min;

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
        .fixed_pos(pos2(origin.x + position.left as f32, origin.y + position.top as f32))
        .show(ctx, |ui| {
            ui.set_min_size(vec2(position.width as f32, position.height as f32));
            ui.add_enabled_ui(enabled, |ui| {
                let font_id = font_id(font, font_size);
                egui::Frame::none()
                    .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                    .show(ui, |ui| {
                        if header_visible && !columns.is_empty() {
                            ui.horizontal(|ui| {
                                for col in columns {
                                    ui.label(RichText::new(col).strong().font(font_id.clone()));
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

                                        let row_label = RichText::new(&row_text).font(font_id.clone());
                                        let resp = ui.selectable_label(is_selected, row_label);
                                        if resp.clicked() {
                                            selected_indices.clear();
                                            selected_indices.push(row_idx);
                                            events.push(GuiEvent {
                                                form: form_name.to_string(),
                                                control: ctrl_name.to_string(),
                                                event: "change".to_string(),
                                                value: None,
                                            });
                                        }
                                        if resp.double_clicked() {
                                            events.push(GuiEvent {
                                                form: form_name.to_string(),
                                                control: ctrl_name.to_string(),
                                                event: "select".to_string(),
                                                value: None,
                                            });
                                        }
                                    });
                                }
                            });
                    });
            });
        });

    events
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
