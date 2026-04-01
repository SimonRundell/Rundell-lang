//! `.rundell.env` file format: one `KEY=<base64-blob>` per line.

use std::collections::HashMap;
use std::path::Path;

use crate::EnvError;

/// Read all raw (encrypted) entries from the env file.
///
/// Lines starting with `#` or blank lines are ignored.
/// Returns an empty map if the file does not exist.
pub fn read_all(env_path: &Path) -> Result<HashMap<String, String>, EnvError> {
    if !env_path.exists() {
        return Ok(HashMap::new());
    }

    let contents = std::fs::read_to_string(env_path)
        .map_err(|e| EnvError::Io(e.to_string()))?;

    let mut map = HashMap::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        }
    }
    Ok(map)
}

/// Write all entries to the env file, sorted by key.
///
/// Writes a header comment, then one `KEY=<base64-blob>` per line.
pub fn write_all(env_path: &Path, map: &HashMap<String, String>) -> Result<(), EnvError> {
    // TODO: On Unix, set file permissions to 0o600 for security

    let mut lines = Vec::new();
    lines.push("# Rundell environment file \u{2014} do not edit by hand".to_string());

    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    for key in keys {
        let value = &map[key];
        lines.push(format!("{key}={value}"));
    }

    let contents = lines.join("\n") + "\n";
    std::fs::write(env_path, contents).map_err(|e| EnvError::Io(e.to_string()))?;
    Ok(())
}
