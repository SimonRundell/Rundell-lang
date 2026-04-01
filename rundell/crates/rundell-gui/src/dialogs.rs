//! Native system dialog wrappers using the `rfd` crate.
//!
//! All dialogs block the calling thread until dismissed, returning the
//! user's choice as a Rundell `string` value.

#![allow(dead_code)]

/// Open the OS file-open dialog.
///
/// Returns the selected path, or an empty string if cancelled.
pub fn open_file(title: &str, filter_desc: &str, filter_ext: &str) -> String {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(filter_desc, &[filter_ext])
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Open the OS file-save dialog.
///
/// Returns the chosen path, or an empty string if cancelled.
pub fn save_file(title: &str, filter_desc: &str, filter_ext: &str) -> String {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(filter_desc, &[filter_ext])
        .save_file()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Show a modal message box.
///
/// `kind` is `"ok"`, `"okcancel"`, or `"yesno"`.
/// Returns the button label as a lowercase string.
pub fn message_box(title: &str, message: &str, kind: &str) -> String {
    use rfd::{MessageButtons, MessageDialog, MessageLevel};

    let buttons = match kind {
        "okcancel" => MessageButtons::OkCancel,
        "yesno"    => MessageButtons::YesNo,
        _           => MessageButtons::Ok,
    };

    let result = MessageDialog::new()
        .set_title(title)
        .set_description(message)
        .set_buttons(buttons)
        .set_level(MessageLevel::Info)
        .show();

    match result {
        rfd::MessageDialogResult::Ok     => "ok".to_string(),
        rfd::MessageDialogResult::Cancel => "cancel".to_string(),
        rfd::MessageDialogResult::Yes    => "yes".to_string(),
        rfd::MessageDialogResult::No     => "no".to_string(),
        rfd::MessageDialogResult::Custom(s) => s.to_lowercase(),
    }
}

/// Show a colour picker.
///
/// `initial` is a `"#RRGGBB"` string.  Returns the chosen colour in the same
/// format, or `initial` if the user cancels.
///
/// Note: `rfd` does not provide a native colour picker on all platforms.
/// This implementation falls back to an egui-based picker (see the GUI-side
/// implementation); here we just return the initial value as a stub for
/// headless / Phase-12 usage.
pub fn color_picker(initial: &str) -> String {
    // rfd does not expose a colour picker; return the initial value.
    // The actual GUI-side implementation uses an egui colour picker window.
    initial.to_string()
}
