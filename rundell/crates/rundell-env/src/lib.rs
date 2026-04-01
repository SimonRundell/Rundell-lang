//! `rundell-env` — encrypted credential storage for the Rundell language.
//!
//! Credentials are stored in a `.rundell.env` file as AES-256-GCM encrypted
//! values, one per line in the format `KEY=<base64-blob>`.  The encryption
//! key is derived from the machine hostname and OS username via HKDF-SHA256,
//! meaning credentials stored on one machine cannot be decrypted on another.

pub mod crypto;
pub mod store;

use std::path::Path;

use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during env-file operations.
#[derive(Debug, Error)]
pub enum EnvError {
    /// The requested key was not found in the environment file.
    #[error("Key '{0}' not found in environment file")]
    KeyNotFound(String),
    /// Decryption failed — wrong machine, corrupt data, or bad base64.
    #[error("Decryption failed for key '{0}'")]
    DecryptionFailed(String),
    /// An I/O error occurred reading or writing the env file.
    #[error("IO error: {0}")]
    Io(String),
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Retrieve a stored credential value by key.
///
/// Reads the env file at `env_path`, decrypts the value for `key`, and
/// returns the plaintext string.
pub fn env_get(env_path: &Path, key: &str) -> Result<String, EnvError> {
    let map = store::read_all(env_path)?;
    let encrypted = map.get(key).ok_or_else(|| EnvError::KeyNotFound(key.to_string()))?;
    let machine_key = crypto::derive_machine_key();
    crypto::decrypt_value(&machine_key, encrypted, key)
}

/// Store (or update) a credential value.
///
/// Reads the existing env file (if any), encrypts `value` under `key`, and
/// writes the updated file back.
pub fn env_set(env_path: &Path, key: &str, value: &str) -> Result<(), EnvError> {
    let mut map = store::read_all(env_path)?;
    let machine_key = crypto::derive_machine_key();
    let encrypted = crypto::encrypt_value(&machine_key, value);
    map.insert(key.to_string(), encrypted);
    store::write_all(env_path, &map)
}

/// List all key names stored in the env file.
///
/// Returns a sorted list of key names (not values).
pub fn env_list(env_path: &Path) -> Result<Vec<String>, EnvError> {
    let map = store::read_all(env_path)?;
    let mut keys: Vec<String> = map.into_keys().collect();
    keys.sort();
    Ok(keys)
}

/// Delete a credential by key.
///
/// Returns `EnvError::KeyNotFound` if the key does not exist.
pub fn env_delete(env_path: &Path, key: &str) -> Result<(), EnvError> {
    let mut map = store::read_all(env_path)?;
    if map.remove(key).is_none() {
        return Err(EnvError::KeyNotFound(key.to_string()));
    }
    store::write_all(env_path, &map)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    /// Helper: write then read back a value, asserting round-trip correctness.
    fn roundtrip(plaintext: &str) {
        let tmp = NamedTempFile::new().expect("tmp file");
        let path = tmp.path();
        env_set(path, "TEST_KEY", plaintext).expect("env_set");
        let got = env_get(path, "TEST_KEY").expect("env_get");
        assert_eq!(got, plaintext, "round-trip failed for: {plaintext:?}");
    }

    #[test]
    fn roundtrip_empty_string() {
        roundtrip("");
    }

    #[test]
    fn roundtrip_unicode() {
        roundtrip("héllo wörld \u{1F600}");
    }

    #[test]
    fn roundtrip_value_with_equals() {
        // Values that contain '=' must survive the KEY=value line format.
        roundtrip("abc=def==xyz");
    }

    #[test]
    fn roundtrip_normal_string() {
        roundtrip("my_secret_password_123!");
    }

    #[test]
    fn roundtrip_long_string() {
        let long = "A".repeat(10_000);
        roundtrip(&long);
    }

    #[test]
    fn key_not_found() {
        let tmp = NamedTempFile::new().expect("tmp file");
        let result = env_get(tmp.path(), "MISSING_KEY");
        assert!(matches!(result, Err(EnvError::KeyNotFound(_))));
    }

    #[test]
    fn corrupt_data_returns_decryption_failed() {
        let tmp = NamedTempFile::new().expect("tmp file");
        // Write a syntactically valid line but with garbage base64 content.
        std::fs::write(tmp.path(), "MY_KEY=bm90dmFsaWRjaXBoZXJ0ZXh0\n")
            .expect("write");
        let result = env_get(tmp.path(), "MY_KEY");
        assert!(matches!(result, Err(EnvError::DecryptionFailed(_))));
    }

    #[test]
    fn missing_file_returns_key_not_found() {
        let path = PathBuf::from("/tmp/rundell_env_does_not_exist_xyzzy.env");
        let result = env_get(&path, "ANYTHING");
        // File doesn't exist → empty map → KeyNotFound
        assert!(matches!(result, Err(EnvError::KeyNotFound(_))));
    }

    #[test]
    fn multiple_keys_in_one_file() {
        let tmp = NamedTempFile::new().expect("tmp file");
        let path = tmp.path();

        env_set(path, "API_KEY", "secret1").expect("set API_KEY");
        env_set(path, "DB_PASS", "secret2").expect("set DB_PASS");
        env_set(path, "TOKEN", "secret3").expect("set TOKEN");

        assert_eq!(env_get(path, "API_KEY").unwrap(), "secret1");
        assert_eq!(env_get(path, "DB_PASS").unwrap(), "secret2");
        assert_eq!(env_get(path, "TOKEN").unwrap(), "secret3");

        let keys = env_list(path).unwrap();
        assert_eq!(keys, vec!["API_KEY", "DB_PASS", "TOKEN"]);

        env_delete(path, "DB_PASS").unwrap();
        let keys_after = env_list(path).unwrap();
        assert_eq!(keys_after, vec!["API_KEY", "TOKEN"]);
    }
}
