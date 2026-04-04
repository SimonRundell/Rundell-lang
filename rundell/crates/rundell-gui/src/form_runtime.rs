//! Runtime rendering state for all open Rundell forms.
//!
//! [`FormRuntime`] owns the live state snapshot of every form and drives the
//! egui rendering each frame via [`render_all`].

use std::collections::HashMap;

use egui::{Color32, Context, Vec2};

use rundell_interpreter::form_registry::FormInstance;

use crate::controls;

/// A GUI event emitted when the user interacts.
#[derive(Debug, Clone)]
pub struct GuiEvent {
    pub form: String,
    pub control: String,
    pub event: String,
    pub value: Option<String>,
}

/// Render output for a single frame.
pub struct RenderResult {
    pub events: Vec<GuiEvent>,
    pub closed_forms: Vec<String>,
}

/// Rendering metadata for a single open form window.
struct OpenForm {
    /// Mirror of the form instance from the interpreter.
    instance: FormInstance,
    /// Whether the form should be displayed as a modal egui overlay.
    modal: bool,
    /// Controls whose display state has changed and needs re-render.
    dirty: bool,
}

/// Central rendering manager for all forms.
pub struct FormRuntime {
    /// All registered form states, keyed by form name.
    forms: HashMap<String, OpenForm>,
    /// Names of forms currently open (in display order).
    open_order: Vec<String>,
}

impl FormRuntime {
    /// Create an empty runtime.
    pub fn new() -> Self {
        FormRuntime {
            forms: HashMap::new(),
            open_order: Vec::new(),
        }
    }

    /// Register a form as open.
    pub fn show_form(&mut self, name: String, modal: bool, instance: FormInstance) {
        let entry = self.forms.entry(name.clone()).or_insert_with(|| OpenForm {
            instance: instance.clone(),
            modal,
            dirty: true,
        });
        entry.instance = instance;
        entry.instance.is_open = true;
        entry.instance.is_modal = modal;
        entry.modal = modal;
        entry.dirty = true;
        if !self.open_order.contains(&name) {
            self.open_order.push(name);
        }
    }

    /// Close a form.
    pub fn close_form(&mut self, name: &str) {
        if let Some(form) = self.forms.get_mut(name) {
            form.instance.is_open = false;
        }
        self.open_order.retain(|n| n != name);
    }

    /// Mark a control as needing a repaint.
    pub fn mark_dirty(&mut self, form: &str, _control: &str) {
        if let Some(f) = self.forms.get_mut(form) {
            f.dirty = true;
        }
    }

    /// Update this runtime's copy of a form instance from the interpreter's
    /// live data.  Called by the interpreter thread via a shared-state
    /// mechanism (or simply on each `ShowForm` command with the full state).
    #[allow(dead_code)]
    pub fn update_form_instance(&mut self, name: &str, instance: FormInstance) {
        if let Some(f) = self.forms.get_mut(name) {
            f.instance = instance;
            f.dirty = true;
        } else {
            self.forms.insert(name.to_string(), OpenForm {
                instance,
                modal: false,
                dirty: true,
            });
        }
    }

    /// Render all open forms.  Returns a list of GUI events that occurred
    /// this frame.
    pub fn render_all(&mut self, ctx: &Context) -> RenderResult {
        let mut events = Vec::new();
        let mut closed_forms = Vec::new();

        let open_names: Vec<String> = self.open_order.clone();

        for name in &open_names {
            let mut closed = false;
            {
                let Some(open) = self.forms.get_mut(name) else { continue };
                if !open.instance.is_open { continue; }

                let props = open.instance.properties.clone();
                let control_names: Vec<String> = open.instance.controls.keys().cloned().collect();
                let _modal = open.modal;
                let form_name = name.clone();

                let bg = hex_to_color32(&props.background_color);
                let w = props.width as f32;
                let h = props.height as f32;
                let title = props.title.clone();
                let resizable = props.resizable;

                let mut win_open = true;

                let mut window = egui::Window::new(&title)
                    .id(egui::Id::new(&form_name))
                    .resizable(resizable)
                    .collapsible(false)
                    .default_size(Vec2::new(w, h))
                    .open(&mut win_open);

                if resizable {
                    window = window.min_size(Vec2::new(200.0, 150.0));
                } else {
                    window = window.min_size(Vec2::new(w, h)).max_size(Vec2::new(w, h));
                }

                let resp = window.show(ctx, |ui| {
                    let mut frame_events: Vec<GuiEvent> = Vec::new();

                    // Fill the background.
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(rect, 0.0, bg);

                    for ctrl_name in &control_names {
                        if let Some(ctrl_state) = open.instance.controls.get_mut(ctrl_name) {
                            let evts = controls::render_control(
                                ui,
                                ctx,
                                &form_name,
                                ctrl_name,
                                ctrl_state,
                            );
                            frame_events.extend(evts);
                        }
                    }
                    frame_events
                });

                if let Some(inner) = resp {
                    if let Some(evts) = inner.inner {
                        events.extend(evts);
                    }
                }

                if !win_open {
                    open.instance.is_open = false;
                    closed = true;
                }
            }

            if closed {
                self.open_order.retain(|n| n != name);
                closed_forms.push(name.clone());
            }
        }

        RenderResult { events, closed_forms }
    }

    pub fn has_open_forms(&self) -> bool {
        !self.open_order.is_empty()
    }
}

/// Parse a `"#RRGGBB"` hex string into an egui [`Color32`].
///
/// Returns [`Color32::BLACK`] for malformed input and logs a warning.
pub fn hex_to_color32(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        eprintln!("[WARN] malformed colour '{hex}' — defaulting to black");
        return Color32::BLACK;
    }
    let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or_else(|_| {
        eprintln!("[WARN] cannot parse colour component '{s}'");
        0
    });
    let r = parse(&hex[0..2]);
    let g = parse(&hex[2..4]);
    let b = parse(&hex[4..6]);
    Color32::from_rgb(r, g, b)
}
