//! Inter-thread communication types between the interpreter and GUI layer.

use crate::form_registry::FormInstance;
use rundell_parser::ast::MessageKind;

/// Dialog request payload with fully-evaluated arguments.
#[derive(Debug, Clone)]
pub enum DialogRequest {
    OpenFile { title: String, filter: String },
    SaveFile { title: String, filter: String },
    Message { title: String, message: String, kind: MessageKind },
    ColorPicker { initial: String },
}

/// Commands sent from the interpreter thread to the egui thread.
#[derive(Debug, Clone)]
pub enum GuiCommand {
    /// Show a form window with its current instance state.
    ShowForm { name: String, modal: bool, instance: FormInstance },
    /// Close a form window.
    CloseForm { name: String },
    /// Update a specific control's display state.
    UpdateControl { form: String, control: String },
    /// Update a form's full instance state.
    UpdateForm { name: String, instance: FormInstance },
    /// Execute a dialog request and return the result.
    DialogCall { id: u64, request: DialogRequest },
    /// Quit the GUI.
    Quit,
}

/// Responses sent from the egui thread back to the interpreter.
#[derive(Debug, Clone)]
pub enum GuiResponse {
    /// A form was closed by the user.
    FormClosed { name: String },
    /// A control event fired.
    EventFired { form: String, control: String, event: String, value: Option<String> },
    /// Dialog result for a specific request ID.
    DialogResult { id: u64, value: String },
    /// GUI is ready.
    Ready,
}
