//! Inter-thread communication types between the interpreter and GUI layer.

/// Commands sent from the interpreter thread to the egui thread.
#[derive(Debug, Clone)]
pub enum GuiCommand {
    /// Show a form window.
    ShowForm { name: String, modal: bool },
    /// Close a form window.
    CloseForm { name: String },
    /// Update a specific control's display state.
    UpdateControl { form: String, control: String },
    /// Quit the GUI.
    Quit,
}

/// Responses sent from the egui thread back to the interpreter.
#[derive(Debug, Clone)]
pub enum GuiResponse {
    /// A form was closed by the user.
    FormClosed { name: String },
    /// A control event fired.
    EventFired { form: String, control: String, event: String },
    /// GUI is ready.
    Ready,
}
