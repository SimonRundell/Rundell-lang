//! The main eframe `App` implementation for Rundell GUI.
//!
//! [`RundellApp`] hosts all open form windows and dispatches GUI commands
//! received from the interpreter thread.

use std::sync::mpsc;

use egui::Context;

use rundell_interpreter::gui_channel::{GuiCommand, GuiResponse};

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
                GuiCommand::ShowForm { name, modal } => {
                    self.form_runtime.show_form(name, modal);
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
                GuiCommand::Quit => {
                    self.should_quit = true;
                    ctx.request_repaint();
                }
            }
        }
    }
}

impl eframe::App for RundellApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Drain any new commands from the interpreter.
        self.process_commands(ctx);

        // Render all open forms and collect any events.
        let events = self.form_runtime.render_all(ctx);

        // Forward events to the interpreter.
        for (form, control, event) in events {
            let _ = self.resp_tx.send(GuiResponse::EventFired { form, control, event });
        }

        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Request repaint to keep the GUI responsive while interpreter runs.
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}
