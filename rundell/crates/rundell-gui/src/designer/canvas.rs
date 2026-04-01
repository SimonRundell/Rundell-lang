//! Drag-and-drop design canvas for the form designer.

use egui::{Color32, Rect, Stroke, Ui, pos2, vec2};
use super::DesignControl;

/// The design canvas manages control placement and selection.
pub struct DesignCanvas {
    /// Whether we are currently dragging a control.
    dragging: Option<usize>,
    /// Mouse position when drag started.
    drag_start_mouse: egui::Pos2,
    /// Control position when drag started.
    drag_start_pos: egui::Pos2,
}

impl DesignCanvas {
    pub fn new() -> Self {
        DesignCanvas {
            dragging: None,
            drag_start_mouse: egui::Pos2::ZERO,
            drag_start_pos: egui::Pos2::ZERO,
        }
    }

    /// Render the canvas, updating control positions on drag.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        controls: &mut Vec<DesignControl>,
        selected: &mut Option<usize>,
    ) {
        let available = ui.available_rect_before_wrap();

        // Draw canvas background
        ui.painter().rect_filled(available, 0.0, Color32::from_rgb(162, 162, 162));

        for (idx, ctrl) in controls.iter_mut().enumerate() {
            let pos = ctrl.state.get_position();
            let rect = Rect::from_min_size(
                pos2(available.min.x + pos.left as f32, available.min.y + pos.top as f32),
                vec2(pos.width as f32, pos.height as f32),
            );
            let is_selected = *selected == Some(idx);

            // Draw control placeholder
            let fill = if is_selected {
                Color32::from_rgba_premultiplied(100, 160, 255, 180)
            } else {
                Color32::from_rgba_premultiplied(200, 200, 200, 200)
            };
            ui.painter().rect_filled(rect, 2.0, fill);
            ui.painter().rect_stroke(rect, 2.0, Stroke::new(
                if is_selected { 2.0 } else { 1.0 },
                if is_selected { Color32::BLUE } else { Color32::DARK_GRAY },
            ));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &ctrl.name,
                egui::FontId::default(),
                Color32::BLACK,
            );

            // Interaction
            let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
            if resp.clicked() {
                *selected = Some(idx);
            }
            if resp.drag_started() {
                self.dragging = Some(idx);
                self.drag_start_mouse = resp.interact_pointer_pos().unwrap_or(rect.min);
                self.drag_start_pos = pos2(pos.left as f32, pos.top as f32);
                *selected = Some(idx);
            }
            if resp.dragged() {
                if self.dragging == Some(idx) {
                    if let Some(mouse_pos) = resp.interact_pointer_pos() {
                        let delta = mouse_pos - self.drag_start_mouse;
                        let new_left = (self.drag_start_pos.x + delta.x).max(0.0) as u32;
                        let new_top  = (self.drag_start_pos.y + delta.y).max(0.0) as u32;
                        ctrl.state.set_position(new_top, new_left, pos.width, pos.height);
                    }
                }
            }
            if resp.drag_stopped() {
                self.dragging = None;
            }
        }
    }
}

// Helper to get Position from ControlState without ownership.
trait GetPosition {
    fn get_position(&self) -> rundell_interpreter::form_registry::Position;
}

impl GetPosition for rundell_interpreter::form_registry::ControlState {
    fn get_position(&self) -> rundell_interpreter::form_registry::Position {
        use rundell_interpreter::form_registry::ControlState;
        match self {
            ControlState::Label       { position, .. } => position.clone(),
            ControlState::Textbox     { position, .. } => position.clone(),
            ControlState::Button      { position, .. } => position.clone(),
            ControlState::Radiobutton { position, .. } => position.clone(),
            ControlState::Checkbox    { position, .. } => position.clone(),
            ControlState::Switch      { position, .. } => position.clone(),
            ControlState::Select      { position, .. } => position.clone(),
            ControlState::Listbox     { position, .. } => position.clone(),
        }
    }
}
