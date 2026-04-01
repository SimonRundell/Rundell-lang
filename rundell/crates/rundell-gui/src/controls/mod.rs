//! GUI control rendering for all eight Rundell control types.
//!
//! Each control is rendered using `egui::Area` with absolute positioning so
//! that the Rundell `position` tuple maps directly to screen coordinates.

use egui::{Context, Ui};

use rundell_interpreter::form_registry::ControlState;

use crate::form_runtime::EventTuple;

mod label;
mod textbox;
mod button;
mod radiobutton;
mod checkbox;
mod switch;
mod select;
mod listbox;

/// Render a single control and return any events fired this frame.
pub fn render_control(
    ui: &mut Ui,
    ctx: &Context,
    form_name: &str,
    ctrl_name: &str,
    state: &ControlState,
) -> Vec<EventTuple> {
    match state {
        ControlState::Label { visible, position, value, text_color, font_size, .. } => {
            if *visible {
                label::render(ui, ctx, form_name, ctrl_name, position, value, text_color, *font_size)
            } else {
                vec![]
            }
        }
        ControlState::Textbox { visible, position, value, text_color, text_background,
            readonly, placeholder, .. } => {
            if *visible {
                textbox::render(ui, ctx, form_name, ctrl_name, position, value,
                    text_color, text_background, *readonly, placeholder)
            } else {
                vec![]
            }
        }
        ControlState::Button { visible, enabled, position, caption, text_color,
            background_color, .. } => {
            if *visible {
                button::render(ui, ctx, form_name, ctrl_name, position, caption,
                    text_color, background_color, *enabled)
            } else {
                vec![]
            }
        }
        ControlState::Radiobutton { visible, enabled, position, caption, checked, .. } => {
            if *visible {
                radiobutton::render(ui, ctx, form_name, ctrl_name, position, caption,
                    *checked, *enabled)
            } else {
                vec![]
            }
        }
        ControlState::Checkbox { visible, enabled, position, caption, checked, .. } => {
            if *visible {
                checkbox::render(ui, ctx, form_name, ctrl_name, position, caption,
                    *checked, *enabled)
            } else {
                vec![]
            }
        }
        ControlState::Switch { visible, enabled, position, caption, checked, .. } => {
            if *visible {
                switch::render(ui, ctx, form_name, ctrl_name, position, caption,
                    *checked, *enabled)
            } else {
                vec![]
            }
        }
        ControlState::Select { visible, enabled, position, items, selected_index, .. } => {
            if *visible {
                select::render(ui, ctx, form_name, ctrl_name, position, items,
                    *selected_index, *enabled)
            } else {
                vec![]
            }
        }
        ControlState::Listbox { visible, enabled, position, data_source, columns,
            image_column, multi_select, row_height, header_visible, selected_indices, .. } => {
            if *visible {
                listbox::render(ui, ctx, form_name, ctrl_name, position, data_source,
                    columns, image_column.as_deref(), *multi_select, *row_height,
                    *header_visible, selected_indices, *enabled)
            } else {
                vec![]
            }
        }
    }
}
