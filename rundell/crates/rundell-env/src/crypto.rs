//! AES-256-GCM encryption / decryption for rundell-env credential storage.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use hkdf::Hkdf;
use sha2::Sha256;

use crate::EnvError;

/// Derives a 32-byte AES key from the machine identity.
///
/// Uses HKDF-SHA256 with machine hostname + OS username as input keying
/// material.  The info string `"rundell-env-v1"` is used as HKDF context.
/// Falls back to empty string if `hostname::get()` fails.
pub fn derive_machine_key() -> [u8; 32] {
    let host = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_default();
    let user = whoami::username();

    // IKM = hostname + ":" + username
    let mut ikm = String::new();
    ikm.push_str(&host);
    ikm.push(':');
    ikm.push_str(&user);

    let hk = Hkdf::<Sha256>::new(None, ikm.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(b"rundell-env-v1", &mut okm)
        .expect("HKDF expand should never fail for 32-byte output");
    okm
}

/// Encrypts a plaintext string value.
///
/// Returns a base64-encoded string: nonce (12 bytes) || ciphertext || GCM tag.
/// Uses a fresh random nonce for every call via [`aes_gcm::aead::OsRng`].
pub fn encrypt_value(key: &[u8; 32], plaintext: &str) -> String {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .expect("AES-GCM encryption should not fail");

    // Concatenate: nonce (12 bytes) || ciphertext+tag
    let mut blob = nonce.to_vec();
    blob.extend_from_slice(&ciphertext);
    STANDARD.encode(&blob)
}

/// Decrypts a base64-encoded encrypted value produced by [`encrypt_value`].
///
/// Returns `Err(EnvError::DecryptionFailed(key_name))` if the key is wrong,
/// the data is corrupt, or base64 decoding fails.
pub fn decrypt_value(key: &[u8; 32], encoded: &str, key_name: &str) -> Result<String, EnvError> {
    let blob = STANDARD
        .decode(encoded)
        .map_err(|_| EnvError::DecryptionFailed(key_name.to_string()))?;

    if blob.len() < 12 {
        return Err(EnvError::DecryptionFailed(key_name.to_string()));
    }

    let (nonce_bytes, ciphertext) = blob.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| EnvError::DecryptionFailed(key_name.to_string()))?;

    String::from_utf8(plaintext).map_err(|_| EnvError::DecryptionFailed(key_name.to_string()))
}
