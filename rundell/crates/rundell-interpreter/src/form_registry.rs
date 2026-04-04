//! Runtime form and control state for the Rundell GUI system.

use std::collections::HashMap;
use serde_json::Value as JsonValue;

/// A pixel dimension value.
#[derive(Debug, Clone)]
pub struct PixelValue(pub u32);

/// A position rectangle: top, left, width, height.
#[derive(Debug, Clone)]
pub struct Position {
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for Position {
    fn default() -> Self {
        Position { top: 0, left: 0, width: 100, height: 30 }
    }
}

/// Text alignment for control content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl TextAlign {
    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "left" => Some(TextAlign::Left),
            "center" => Some(TextAlign::Center),
            "right" => Some(TextAlign::Right),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Right => "right",
        }
    }
}

/// Live runtime state for a single control.
#[derive(Debug, Clone)]
pub enum ControlState {
    Label {
        value: String,
        visible: bool,
        enabled: bool,
        position: Position,
        text_color: String,
        font: String,
        font_size: u32,
        text_align: TextAlign,
    },
    Textbox {
        value: String,
        visible: bool,
        enabled: bool,
        position: Position,
        text_color: String,
        text_background: String,
        font: String,
        font_size: u32,
        text_align: TextAlign,
        readonly: bool,
        max_length: Option<u32>,
        placeholder: String,
        autorefresh: bool,
        on_change: Option<String>,
    },
    Button {
        caption: String,
        visible: bool,
        enabled: bool,
        position: Position,
        text_color: String,
        background_color: String,
        font: String,
        font_size: u32,
        text_align: TextAlign,
        on_click: Option<String>,
    },
    Radiobutton {
        caption: String,
        group: String,
        checked: bool,
        visible: bool,
        enabled: bool,
        position: Position,
        font: String,
        font_size: u32,
        text_align: TextAlign,
        on_change: Option<String>,
    },
    Checkbox {
        caption: String,
        checked: bool,
        visible: bool,
        enabled: bool,
        position: Position,
        font: String,
        font_size: u32,
        text_align: TextAlign,
        on_change: Option<String>,
    },
    Switch {
        caption: String,
        checked: bool,
        visible: bool,
        enabled: bool,
        position: Position,
        font: String,
        font_size: u32,
        text_align: TextAlign,
        on_change: Option<String>,
    },
    Select {
        items: Vec<String>,
        selected_index: Option<usize>,
        visible: bool,
        enabled: bool,
        position: Position,
        font: String,
        font_size: u32,
        text_align: TextAlign,
        on_change: Option<String>,
    },
    Listbox {
        data_source: Option<JsonValue>,
        columns: Vec<String>,
        image_column: Option<String>,
        multi_select: bool,
        selected_indices: Vec<usize>,
        visible: bool,
        enabled: bool,
        position: Position,
        font: String,
        font_size: u32,
        row_height: u32,
        header_visible: bool,
        on_change: Option<String>,
        on_select: Option<String>,
    },
}

impl ControlState {
    /// Set a property on this control by name. Returns Ok(()) if property
    /// is recognised, or a warning string if it is unrecognised (non-fatal).
    pub fn set_property(&mut self, prop: &str, value: &str) -> Result<(), String> {
        match self {
            ControlState::Label { value: v, text_color, font, font_size, text_align, visible, enabled, .. } => {
                match prop {
                    "value" => *v = value.to_string(),
                    "textcolor" => *text_color = value.to_string(),
                    "font" => *font = value.to_string(),
                    "fontsize" => *font_size = value.parse().unwrap_or(*font_size),
                    "textalign" => {
                        *text_align = TextAlign::parse(value)
                            .ok_or_else(|| format!(
                                "[WARN] invalid textalign '{}'; use left, center, or right",
                                value
                            ))?;
                    }
                    "visible" => *visible = value == "true",
                    "enabled" => *enabled = value == "true",
                    _ => return Err(format!("[WARN] unrecognised property '{}' on label", prop)),
                }
            }
            ControlState::Textbox { value: v, text_color, text_background, readonly,
                max_length, placeholder, autorefresh, on_change, font, font_size,
                text_align, visible, enabled, .. } => {
                match prop {
                    "value" => *v = value.to_string(),
                    "textcolor" => *text_color = value.to_string(),
                    "textbackground" => *text_background = value.to_string(),
                    "font" => *font = value.to_string(),
                    "fontsize" => *font_size = value.parse().unwrap_or(*font_size),
                    "textalign" => {
                        *text_align = TextAlign::parse(value)
                            .ok_or_else(|| format!(
                                "[WARN] invalid textalign '{}'; use left, center, or right",
                                value
                            ))?;
                    }
                    "readonly" => *readonly = value == "true",
                    "maxlength" => *max_length = value.parse().ok(),
                    "placeholder" => *placeholder = value.to_string(),
                    "autorefresh" => *autorefresh = value == "true",
                    "change" => *on_change = Some(value.to_string()),
                    "visible" => *visible = value == "true",
                    "enabled" => *enabled = value == "true",
                    _ => return Err(format!("[WARN] unrecognised property '{}' on textbox", prop)),
                }
            }
            ControlState::Button { caption, text_color, background_color, font, font_size,
                text_align, on_click, visible, enabled, .. } => {
                match prop {
                    "caption" => *caption = value.to_string(),
                    "textcolor" => *text_color = value.to_string(),
                    "backgroundcolor" => *background_color = value.to_string(),
                    "font" => *font = value.to_string(),
                    "fontsize" => *font_size = value.parse().unwrap_or(*font_size),
                    "textalign" => {
                        *text_align = TextAlign::parse(value)
                            .ok_or_else(|| format!(
                                "[WARN] invalid textalign '{}'; use left, center, or right",
                                value
                            ))?;
                    }
                    "click" => *on_click = Some(value.to_string()),
                    "visible" => *visible = value == "true",
                    "enabled" => *enabled = value == "true",
                    _ => return Err(format!("[WARN] unrecognised property '{}' on button", prop)),
                }
            }
            ControlState::Radiobutton { caption, group, checked, on_change, font, font_size,
                text_align, visible, enabled, .. } => {
                match prop {
                    "caption" => *caption = value.to_string(),
                    "group" => *group = value.to_string(),
                    "checked" => *checked = value == "true",
                    "font" => *font = value.to_string(),
                    "fontsize" => *font_size = value.parse().unwrap_or(*font_size),
                    "textalign" => {
                        *text_align = TextAlign::parse(value)
                            .ok_or_else(|| format!(
                                "[WARN] invalid textalign '{}'; use left, center, or right",
                                value
                            ))?;
                    }
                    "change" => *on_change = Some(value.to_string()),
                    "visible" => *visible = value == "true",
                    "enabled" => *enabled = value == "true",
                    _ => return Err(format!("[WARN] unrecognised property '{}' on radiobutton", prop)),
                }
            }
            ControlState::Checkbox { caption, checked, on_change, font, font_size,
                text_align, visible, enabled, .. } => {
                match prop {
                    "caption" => *caption = value.to_string(),
                    "checked" => *checked = value == "true",
                    "font" => *font = value.to_string(),
                    "fontsize" => *font_size = value.parse().unwrap_or(*font_size),
                    "textalign" => {
                        *text_align = TextAlign::parse(value)
                            .ok_or_else(|| format!(
                                "[WARN] invalid textalign '{}'; use left, center, or right",
                                value
                            ))?;
                    }
                    "change" => *on_change = Some(value.to_string()),
                    "visible" => *visible = value == "true",
                    "enabled" => *enabled = value == "true",
                    _ => return Err(format!("[WARN] unrecognised property '{}' on checkbox", prop)),
                }
            }
            ControlState::Switch { caption, checked, on_change, font, font_size,
                text_align, visible, enabled, .. } => {
                match prop {
                    "caption" => *caption = value.to_string(),
                    "checked" => *checked = value == "true",
                    "font" => *font = value.to_string(),
                    "fontsize" => *font_size = value.parse().unwrap_or(*font_size),
                    "textalign" => {
                        *text_align = TextAlign::parse(value)
                            .ok_or_else(|| format!(
                                "[WARN] invalid textalign '{}'; use left, center, or right",
                                value
                            ))?;
                    }
                    "change" => *on_change = Some(value.to_string()),
                    "visible" => *visible = value == "true",
                    "enabled" => *enabled = value == "true",
                    _ => return Err(format!("[WARN] unrecognised property '{}' on switch", prop)),
                }
            }
            ControlState::Select { items, selected_index, on_change, font, font_size,
                text_align, visible, enabled, .. } => {
                match prop {
                    "items" => {
                        // Try to parse as JSON array; fall back to csv
                        if let Ok(arr) = serde_json::from_str::<serde_json::Value>(value) {
                            if let Some(a) = arr.as_array() {
                                *items = a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                            }
                        } else {
                            *items = value.split(',').map(|s| s.trim().to_string()).collect();
                        }
                    }
                    "value" => {
                        *selected_index = items.iter().position(|s| s == value);
                    }
                    "font" => *font = value.to_string(),
                    "fontsize" => *font_size = value.parse().unwrap_or(*font_size),
                    "textalign" => {
                        *text_align = TextAlign::parse(value)
                            .ok_or_else(|| format!(
                                "[WARN] invalid textalign '{}'; use left, center, or right",
                                value
                            ))?;
                    }
                    "change" => *on_change = Some(value.to_string()),
                    "visible" => *visible = value == "true",
                    "enabled" => *enabled = value == "true",
                    _ => return Err(format!("[WARN] unrecognised property '{}' on select", prop)),
                }
            }
            ControlState::Listbox { data_source, columns, image_column, multi_select,
                row_height, header_visible, on_change, on_select, font, font_size,
                visible, enabled, .. } => {
                match prop {
                    "datasource" => {
                        *data_source = serde_json::from_str(value).ok();
                    }
                    "columns" => {
                        if let Ok(arr) = serde_json::from_str::<serde_json::Value>(value) {
                            if let Some(a) = arr.as_array() {
                                *columns = a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                            }
                        }
                    }
                    "imagecolumn" => *image_column = if value.is_empty() { None } else { Some(value.to_string()) },
                    "font" => *font = value.to_string(),
                    "fontsize" => *font_size = value.parse().unwrap_or(*font_size),
                    "multiselect" => *multi_select = value == "true",
                    "rowheight" => *row_height = value.parse().unwrap_or(*row_height),
                    "headervisible" => *header_visible = value == "true",
                    "change" => *on_change = Some(value.to_string()),
                    "select" => *on_select = Some(value.to_string()),
                    "visible" => *visible = value == "true",
                    "enabled" => *enabled = value == "true",
                    _ => return Err(format!("[WARN] unrecognised property '{}' on listbox", prop)),
                }
            }
        }
        Ok(())
    }

    /// Set the position of this control.
    pub fn set_position(&mut self, top: u32, left: u32, width: u32, height: u32) {
        let pos = match self {
            ControlState::Label { position, .. } => position,
            ControlState::Textbox { position, .. } => position,
            ControlState::Button { position, .. } => position,
            ControlState::Radiobutton { position, .. } => position,
            ControlState::Checkbox { position, .. } => position,
            ControlState::Switch { position, .. } => position,
            ControlState::Select { position, .. } => position,
            ControlState::Listbox { position, .. } => position,
        };
        pos.top = top;
        pos.left = left;
        pos.width = width;
        pos.height = height;
    }

    /// Get the string value of a property. Returns None if unrecognised.
    pub fn get_property(&self, prop: &str) -> Option<String> {
        match self {
            ControlState::Label { value, text_color, font, font_size, text_align, visible, enabled, .. } => {
                match prop {
                    "value" => Some(value.clone()),
                    "textcolor" => Some(text_color.clone()),
                    "font" => Some(font.clone()),
                    "fontsize" => Some(font_size.to_string()),
                    "textalign" => Some(text_align.as_str().to_string()),
                    "visible" => Some(visible.to_string()),
                    "enabled" => Some(enabled.to_string()),
                    _ => None,
                }
            }
            ControlState::Textbox { value, text_color, text_background, readonly,
                max_length, placeholder, autorefresh, font, font_size,
                text_align, visible, enabled, .. } => {
                match prop {
                    "value" => Some(value.clone()),
                    "textcolor" => Some(text_color.clone()),
                    "textbackground" => Some(text_background.clone()),
                    "font" => Some(font.clone()),
                    "fontsize" => Some(font_size.to_string()),
                    "textalign" => Some(text_align.as_str().to_string()),
                    "readonly" => Some(readonly.to_string()),
                    "maxlength" => max_length.map(|n| n.to_string()),
                    "placeholder" => Some(placeholder.clone()),
                    "autorefresh" => Some(autorefresh.to_string()),
                    "visible" => Some(visible.to_string()),
                    "enabled" => Some(enabled.to_string()),
                    _ => None,
                }
            }
            ControlState::Button { caption, text_color, background_color, font, font_size,
                text_align, visible, enabled, .. } => {
                match prop {
                    "caption" => Some(caption.clone()),
                    "textcolor" => Some(text_color.clone()),
                    "backgroundcolor" => Some(background_color.clone()),
                    "font" => Some(font.clone()),
                    "fontsize" => Some(font_size.to_string()),
                    "textalign" => Some(text_align.as_str().to_string()),
                    "visible" => Some(visible.to_string()),
                    "enabled" => Some(enabled.to_string()),
                    _ => None,
                }
            }
            ControlState::Radiobutton { caption, group, checked, font, font_size,
                text_align, visible, enabled, .. } => {
                match prop {
                    "caption" => Some(caption.clone()),
                    "group" => Some(group.clone()),
                    "checked" => Some(checked.to_string()),
                    "font" => Some(font.clone()),
                    "fontsize" => Some(font_size.to_string()),
                    "textalign" => Some(text_align.as_str().to_string()),
                    "visible" => Some(visible.to_string()),
                    "enabled" => Some(enabled.to_string()),
                    _ => None,
                }
            }
            ControlState::Checkbox { caption, checked, font, font_size, text_align, visible, enabled, .. } => {
                match prop {
                    "caption" => Some(caption.clone()),
                    "checked" => Some(checked.to_string()),
                    "font" => Some(font.clone()),
                    "fontsize" => Some(font_size.to_string()),
                    "textalign" => Some(text_align.as_str().to_string()),
                    "visible" => Some(visible.to_string()),
                    "enabled" => Some(enabled.to_string()),
                    _ => None,
                }
            }
            ControlState::Switch { caption, checked, font, font_size, text_align, visible, enabled, .. } => {
                match prop {
                    "caption" => Some(caption.clone()),
                    "checked" => Some(checked.to_string()),
                    "font" => Some(font.clone()),
                    "fontsize" => Some(font_size.to_string()),
                    "textalign" => Some(text_align.as_str().to_string()),
                    "visible" => Some(visible.to_string()),
                    "enabled" => Some(enabled.to_string()),
                    _ => None,
                }
            }
            ControlState::Select { items, selected_index, font, font_size, text_align, visible, enabled, .. } => {
                match prop {
                    "value" => selected_index.and_then(|i| items.get(i)).cloned(),
                    "font" => Some(font.clone()),
                    "fontsize" => Some(font_size.to_string()),
                    "textalign" => Some(text_align.as_str().to_string()),
                    "visible" => Some(visible.to_string()),
                    "enabled" => Some(enabled.to_string()),
                    _ => None,
                }
            }
            ControlState::Listbox { data_source, columns, image_column, multi_select,
                row_height, header_visible, font, font_size, visible, enabled, selected_indices, .. } => {
                match prop {
                    "datasource" => data_source.as_ref().map(|v| v.to_string()),
                    "columns" => Some(serde_json::Value::Array(
                        columns.iter().map(|s| serde_json::Value::String(s.clone())).collect()
                    ).to_string()),
                    "imagecolumn" => image_column.clone().or(Some(String::new())),
                    "font" => Some(font.clone()),
                    "fontsize" => Some(font_size.to_string()),
                    "multiselect" => Some(multi_select.to_string()),
                    "rowheight" => Some(row_height.to_string()),
                    "headervisible" => Some(header_visible.to_string()),
                    "visible" => Some(visible.to_string()),
                    "enabled" => Some(enabled.to_string()),
                    "value" => {
                        // Return selected record(s) as JSON
                        if selected_indices.is_empty() {
                            Some("null".to_string())
                        } else {
                            // Try to get from datasource
                            if let Some(ds) = data_source {
                                // Look for "rows", "records", or array at root
                                let rows = ds.get("rows")
                                    .or_else(|| ds.get("records"))
                                    .and_then(|v| v.as_array())
                                    .or_else(|| ds.as_array());
                                if let Some(arr) = rows {
                                    let selected: Vec<_> = selected_indices.iter()
                                        .filter_map(|&i| arr.get(i))
                                        .cloned()
                                        .collect();
                                    return Some(serde_json::Value::Array(selected).to_string());
                                }
                            }
                            Some("null".to_string())
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}

/// Default `ControlState` for a given control type keyword.
pub fn default_control_state(ctrl_type: &rundell_parser::ast::ControlType) -> ControlState {
    use rundell_parser::ast::ControlType;
    match ctrl_type {
        ControlType::Label => ControlState::Label {
            value: String::new(),
            visible: true,
            enabled: true,
            position: Position::default(),
            text_color: "#000000".to_string(),
            font: "default".to_string(),
            font_size: 12,
            text_align: TextAlign::Left,
        },
        ControlType::Textbox => ControlState::Textbox {
            value: String::new(),
            visible: true,
            enabled: true,
            position: Position::default(),
            text_color: "#000000".to_string(),
            text_background: "#FFFFFF".to_string(),
            font: "default".to_string(),
            font_size: 12,
            text_align: TextAlign::Left,
            readonly: false,
            max_length: None,
            placeholder: String::new(),
            autorefresh: true,
            on_change: None,
        },
        ControlType::Button => ControlState::Button {
            caption: String::new(),
            visible: true,
            enabled: true,
            position: Position::default(),
            text_color: "#000000".to_string(),
            background_color: "#E0E0E0".to_string(),
            font: "default".to_string(),
            font_size: 12,
            text_align: TextAlign::Center,
            on_click: None,
        },
        ControlType::Radiobutton => ControlState::Radiobutton {
            caption: String::new(),
            group: String::new(),
            checked: false,
            visible: true,
            enabled: true,
            position: Position::default(),
            font: "default".to_string(),
            font_size: 12,
            text_align: TextAlign::Left,
            on_change: None,
        },
        ControlType::Checkbox => ControlState::Checkbox {
            caption: String::new(),
            checked: false,
            visible: true,
            enabled: true,
            position: Position::default(),
            font: "default".to_string(),
            font_size: 12,
            text_align: TextAlign::Left,
            on_change: None,
        },
        ControlType::Switch => ControlState::Switch {
            caption: String::new(),
            checked: false,
            visible: true,
            enabled: true,
            position: Position::default(),
            font: "default".to_string(),
            font_size: 12,
            text_align: TextAlign::Left,
            on_change: None,
        },
        ControlType::Select => ControlState::Select {
            items: Vec::new(),
            selected_index: None,
            visible: true,
            enabled: true,
            position: Position::default(),
            font: "default".to_string(),
            font_size: 12,
            text_align: TextAlign::Left,
            on_change: None,
        },
        ControlType::Listbox => ControlState::Listbox {
            data_source: None,
            columns: Vec::new(),
            image_column: None,
            multi_select: false,
            selected_indices: Vec::new(),
            visible: true,
            enabled: true,
            position: Position::default(),
            font: "default".to_string(),
            font_size: 12,
            row_height: 24,
            header_visible: true,
            on_change: None,
            on_select: None,
        },
    }
}

/// Form-level properties.
#[derive(Debug, Clone)]
pub struct FormProperties {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub background_color: String,
    pub text_color: String,
    pub text_background: String,
}

impl Default for FormProperties {
    fn default() -> Self {
        FormProperties {
            title: String::new(),
            width: 800,
            height: 600,
            resizable: false,
            background_color: "#A2A2A2".to_string(),
            text_color: "#000000".to_string(),
            text_background: "#FFFFFF".to_string(),
        }
    }
}

impl FormProperties {
    /// Set a form-level property by name.
    pub fn set_property(&mut self, prop: &str, value: &str) -> Result<(), String> {
        match prop {
            "title" => self.title = value.to_string(),
            "width" => self.width = value.parse().unwrap_or(self.width),
            "height" => self.height = value.parse().unwrap_or(self.height),
            "resizable" => self.resizable = value == "true",
            "backgroundcolor" => self.background_color = value.to_string(),
            "textcolor" => self.text_color = value.to_string(),
            "textbackground" => self.text_background = value.to_string(),
            _ => return Err(format!("[WARN] unrecognised form property '{}'", prop)),
        }
        Ok(())
    }

    pub fn get_property(&self, prop: &str) -> Option<String> {
        match prop {
            "title" => Some(self.title.clone()),
            "width" => Some(self.width.to_string()),
            "height" => Some(self.height.to_string()),
            "resizable" => Some(self.resizable.to_string()),
            "backgroundcolor" => Some(self.background_color.clone()),
            "textcolor" => Some(self.text_color.clone()),
            "textbackground" => Some(self.text_background.clone()),
            _ => None,
        }
    }
}

/// A live form instance registered in rootWindow.
#[derive(Debug, Clone)]
pub struct FormInstance {
    /// The form's runtime properties.
    pub properties: FormProperties,
    /// Live control state keyed by control name.
    pub controls: HashMap<String, ControlState>,
    /// Whether the form is currently open/visible.
    pub is_open: bool,
    /// Whether the form was opened modally.
    pub is_modal: bool,
}

impl FormInstance {
    pub fn new() -> Self {
        FormInstance {
            properties: FormProperties::default(),
            controls: HashMap::new(),
            is_open: false,
            is_modal: false,
        }
    }
}

impl Default for FormInstance {
    fn default() -> Self {
        Self::new()
    }
}

/// The global rootWindow — container for all registered forms.
#[derive(Debug, Default)]
pub struct RundellWindow {
    /// Registered forms, keyed by name.
    pub forms: HashMap<String, FormInstance>,
}
