//! GUI control rendering for all eight Rundell control types.
//!
//! Each control is rendered using `egui::Area` with absolute positioning so
//! that the Rundell `position` tuple maps directly to screen coordinates.

use egui::{Context, Ui};

use rundell_interpreter::form_registry::ControlState;

use crate::form_runtime::GuiEvent;

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
    state: &mut ControlState,
) -> Vec<GuiEvent> {
    match state {
        ControlState::Label { visible, position, value, text_color, font, font_size, text_align, .. } => {
            if *visible {
                label::render(ui, ctx, form_name, ctrl_name, position, value, text_color, font,
                    *font_size, *text_align)
            } else {
                vec![]
            }
        }
        ControlState::Textbox { visible, position, value, text_color, text_background,
            readonly, placeholder, font, font_size, text_align, .. } => {
            if *visible {
                textbox::render(ui, ctx, form_name, ctrl_name, position, value,
                    text_color, text_background, font, *font_size, *readonly, placeholder, *text_align)
            } else {
                vec![]
            }
        }
        ControlState::Button { visible, enabled, position, caption, text_color,
            background_color, font, font_size, text_align, .. } => {
            if *visible {
                button::render(ui, ctx, form_name, ctrl_name, position, caption,
                    text_color, background_color, font, *font_size, *enabled, *text_align)
            } else {
                vec![]
            }
        }
        ControlState::Radiobutton { visible, enabled, position, caption, checked, font, font_size, text_align, .. } => {
            if *visible {
                radiobutton::render(ui, ctx, form_name, ctrl_name, position, caption,
                    checked, font, *font_size, *enabled, *text_align)
            } else {
                vec![]
            }
        }
        ControlState::Checkbox { visible, enabled, position, caption, checked, font, font_size, text_align, .. } => {
            if *visible {
                checkbox::render(ui, ctx, form_name, ctrl_name, position, caption,
                    checked, font, *font_size, *enabled, *text_align)
            } else {
                vec![]
            }
        }
        ControlState::Switch { visible, enabled, position, caption, checked, font, font_size, text_align, .. } => {
            if *visible {
                switch::render(ui, ctx, form_name, ctrl_name, position, caption,
                    checked, font, *font_size, *enabled, *text_align)
            } else {
                vec![]
            }
        }
        ControlState::Select { visible, enabled, position, items, selected_index, font, font_size, text_align, .. } => {
            if *visible {
                select::render(ui, ctx, form_name, ctrl_name, position, items,
                    selected_index, font, *font_size, *enabled, *text_align)
            } else {
                vec![]
            }
        }
        ControlState::Listbox { visible, enabled, position, data_source, columns,
            image_column, multi_select, row_height, header_visible, selected_indices,
            font, font_size, .. } => {
            if *visible {
                listbox::render(ui, ctx, form_name, ctrl_name, position, data_source,
                    columns, image_column.as_deref(), *multi_select, *row_height,
                    *header_visible, selected_indices, font, *font_size, *enabled)
            } else {
                vec![]
            }
        }
    }
}
