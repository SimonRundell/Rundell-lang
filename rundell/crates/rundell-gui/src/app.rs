//! The main eframe `App` implementation for Rundell GUI.
//!
//! [`RundellApp`] hosts all open form windows and dispatches GUI commands
//! received from the interpreter thread.

use std::sync::mpsc;

use egui::Context;

use rundell_interpreter::gui_channel::{DialogRequest, GuiCommand, GuiResponse};
use rundell_parser::ast::MessageKind;

use crate::dialogs;

use crate::form_runtime::FormRuntime;

/// The top-level egui application.
pub struct RundellApp {
    /// Receiver for commands from the interpreter thread.
    cmd_rx: mpsc::Receiver<GuiCommand>,
    /// Sender for responses back to the interpreter thread.
    resp_tx: mpsc::SyncSender<GuiResponse>,
    /// Runtime rendering state for all open forms.
    form_runtime: FormRuntime,
    /// Whether the application should quit.
    should_quit: bool,
}

impl RundellApp {
    /// Create a new application with the given inter-thread channels.
    pub fn new(
        cmd_rx: mpsc::Receiver<GuiCommand>,
        resp_tx: mpsc::SyncSender<GuiResponse>,
    ) -> Self {
        RundellApp {
            cmd_rx,
            resp_tx,
            form_runtime: FormRuntime::new(),
            should_quit: false,
        }
    }

    /// Drain pending commands from the interpreter.
    fn process_commands(&mut self, ctx: &Context) {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                GuiCommand::ShowForm { name, modal, instance } => {
                    self.form_runtime.show_form(name, modal, instance);
                    ctx.request_repaint();
                }
                GuiCommand::CloseForm { name } => {
                    self.form_runtime.close_form(&name);
                    // Notify interpreter that the form closed.
                    let _ = self.resp_tx.send(GuiResponse::FormClosed { name });
                    ctx.request_repaint();
                }
                GuiCommand::UpdateControl { form, control } => {
                    self.form_runtime.mark_dirty(&form, &control);
                    ctx.request_repaint();
                }
                GuiCommand::UpdateForm { name, instance } => {
                    self.form_runtime.update_form_instance(&name, instance);
                    ctx.request_repaint();
                }
                GuiCommand::DialogCall { id, request } => {
                    let value = handle_dialog_request(request);
                    let _ = self.resp_tx.send(GuiResponse::DialogResult { id, value });
                    ctx.request_repaint();
                }
                GuiCommand::Quit => {
                    self.should_quit = true;
                    ctx.request_repaint();
                }
            }
        }
    }
}

fn handle_dialog_request(request: DialogRequest) -> String {
    match request {
        DialogRequest::OpenFile { title, filter } => {
            let (desc, ext) = parse_filter_desc_ext(&filter);
            dialogs::open_file(&title, &desc, &ext)
        }
        DialogRequest::SaveFile { title, filter } => {
            let (desc, ext) = parse_filter_desc_ext(&filter);
            dialogs::save_file(&title, &desc, &ext)
        }
        DialogRequest::Message { title, message, kind } => {
            let kind_str = match kind {
                MessageKind::Ok => "ok",
                MessageKind::OkCancel => "okcancel",
                MessageKind::YesNo => "yesno",
            };
            dialogs::message_box(&title, &message, kind_str)
        }
        DialogRequest::ColorPicker { initial } => {
            dialogs::color_picker(&initial)
        }
    }
}

fn parse_filter_desc_ext(filter: &str) -> (String, String) {
    let mut desc = String::new();
    if let Some(start) = filter.find('(') {
        desc = filter[..start].trim().to_string();
    }
    if desc.is_empty() {
        desc = "Files".to_string();
    }

    let mut ext = String::new();
    for part in filter.split(|c| c == ';' || c == ',') {
        let trimmed = part.trim();
        if let Some(found) = trimmed.strip_prefix("*.") {
            let found = found.trim_end_matches(')');
            if !found.is_empty() && found != "*" {
                ext = found.to_string();
                break;
            }
        }
    }
    if ext.is_empty() {
        ext = "*".to_string();
    }
    (desc, ext)
}

impl eframe::App for RundellApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Drain any new commands from the interpreter.
        self.process_commands(ctx);

        // Render all open forms and collect any events.
        let render = self.form_runtime.render_all(ctx);

        // Forward events to the interpreter.
        for event in render.events {
            let _ = self.resp_tx.send(GuiResponse::EventFired {
                form: event.form,
                control: event.control,
                event: event.event,
                value: event.value,
            });
        }

        for name in render.closed_forms {
            let _ = self.resp_tx.send(GuiResponse::FormClosed { name });
        }

        if !self.form_runtime.has_open_forms() {
            self.should_quit = true;
        }

        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Request repaint to keep the GUI responsive while interpreter runs.
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}
