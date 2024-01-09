use std::{
    fs,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305};
use shielder::app_state::AppState;
use tracing::info;

/// Try to get `AppState` from `path`. If `path` describes non-existing file, the default `AppState`
/// will be created, saved to `path` and returned.
///
/// `path` will be decrypted with `password`.
pub fn get_app_state(path: &PathBuf, password: &str) -> Result<AppState> {
    match path.exists() {
        true => {
            info!("File with state was found. Reading the state from {path:?}.");
            read_from(path, password)
        }
        false => {
            info!("File with state not found. Creating the default state in {path:?}.");
            create_and_save_default_state(path, password)
        }
    }
}

/// Save `app_state` to `path`.
///
/// `path` will be encrypted with `password`.
pub fn save_app_state(app_state: &AppState, path: &PathBuf, password: &str) -> Result<()> {
    let serialized =
        serde_json::to_string_pretty(app_state).map_err(|e| anyhow!("Failed to serialize: {e}"))?;
    fs::write(path, encrypt(&serialized, password)?)
        .map_err(|e| anyhow!("Failed to save application state: {e}"))
}

/// Read `AppState` from `path`.
fn read_from(path: &Path, password: &str) -> Result<AppState> {
    let file_content = fs::read(path).map_err(|e| anyhow!("Failed to read file content: {e}"))?;
    let decrypted_content = decrypt(&file_content, password)?;
    serde_json::from_str::<AppState>(&decrypted_content)
        .map_err(|e| anyhow!("Failed to deserialize application state: {e}"))
}

/// Create the default `AppState`, save it to `path` and return it.
fn create_and_save_default_state(path: &PathBuf, password: &str) -> Result<AppState> {
    File::create(path).map_err(|e| anyhow!("Failed to create {path:?}: {e}"))?;

    let state = AppState::default();
    save_app_state(&state, path, password)
        .map_err(|e| anyhow!("Failed to save state to {path:?}: {e}"))?;

    Ok(state)
}

const SALT: [u8; 32] = [41u8; 32];
const NONCE: [u8; 24] = [41u8; 24];

fn get_cipher(password: &str) -> Result<XChaCha20Poly1305> {
    let key = argon2::hash_raw(password.as_bytes(), &SALT, &Default::default())
        .map_err(|e| anyhow!("Failed to derive key: {e}"))?;
    Ok(XChaCha20Poly1305::new(key.as_slice().into()))
}

fn encrypt(content: &str, password: &str) -> Result<Vec<u8>> {
    get_cipher(password)?
        .encrypt(NONCE.as_slice().into(), content.as_bytes())
        .map_err(|e| anyhow!("Failed to encrypt data: {e}"))
}

fn decrypt(content: &[u8], password: &str) -> Result<String> {
    let decrypted = get_cipher(password)?
        .decrypt(NONCE.as_slice().into(), content)
        .map_err(|e| anyhow!("Failed to decrypt data - probably the password is incorrect: {e}"))?;
    String::from_utf8(decrypted)
        .map_err(|e| anyhow!("Failed to decrypt data - probably the password is incorrect: {e}"))
}
