use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use grammers_session::types::{DcOption, UpdatesState};
use grammers_session::{storages::MemorySession, Session, SessionData};
use rand_core::RngCore;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::secret_store::{telegram_account_session_key_secret, SecretStoreState};

const ENVELOPE_VERSION: u8 = 1;
const ENVELOPE_ALGORITHM: &str = "XChaCha20-Poly1305";
const SESSION_KEY_BYTES: usize = 32;

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedSession {
    home_dc: i32,
    dc_options: HashMap<i32, DcOption>,
    updates_state: UpdatesState,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EncryptedSessionEnvelope {
    version: u8,
    algorithm: String,
    nonce: String,
    ciphertext: String,
}

pub(crate) fn session_path(handle: &AppHandle, account_id: i64) -> AppResult<PathBuf> {
    let app_dir = handle
        .path()
        .app_data_dir()
        .map_err(|error| AppError::internal(error.to_string()))?;
    fs::create_dir_all(&app_dir).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(app_dir.join(format!("telegram_{account_id}.session.json")))
}

pub(crate) fn session_exists(handle: &AppHandle, account_id: i64) -> bool {
    session_path(handle, account_id)
        .map(|path| path.exists())
        .unwrap_or(false)
}

fn associated_data(account_id: i64) -> String {
    format!("org.ai.extractum.telegram.session.v1.account.{account_id}")
}

async fn memory_session_to_saved(session: &Arc<MemorySession>) -> SavedSession {
    let home_dc = session.home_dc_id();
    let updates_state = session.updates_state().await;
    let mut dc_options = HashMap::new();
    for dc_id in 1..=5i32 {
        if let Some(dc) = session.dc_option(dc_id) {
            dc_options.insert(dc_id, dc);
        }
    }
    SavedSession {
        home_dc,
        dc_options,
        updates_state,
    }
}

fn saved_to_memory_session(saved: SavedSession) -> Arc<MemorySession> {
    let session_data = SessionData {
        home_dc: saved.home_dc,
        dc_options: saved.dc_options,
        peer_infos: HashMap::new(),
        updates_state: saved.updates_state,
    };
    Arc::new(MemorySession::from(session_data))
}

fn encode_base64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn decode_base64(value: &str) -> AppResult<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(value).map_err(|error| {
        AppError::internal(format!(
            "Invalid encrypted Telegram session encoding: {error}"
        ))
    })
}

fn encrypt_saved_session(
    account_id: i64,
    key_bytes: &[u8],
    saved: &SavedSession,
) -> AppResult<EncryptedSessionEnvelope> {
    if key_bytes.len() != SESSION_KEY_BYTES {
        return Err(AppError::internal("Invalid Telegram session key length"));
    }
    let plaintext =
        serde_json::to_vec(saved).map_err(|error| AppError::internal(error.to_string()))?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key_bytes));
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: &plaintext,
                aad: associated_data(account_id).as_bytes(),
            },
        )
        .map_err(|_| AppError::internal("Failed to encrypt Telegram session"))?;
    Ok(EncryptedSessionEnvelope {
        version: ENVELOPE_VERSION,
        algorithm: ENVELOPE_ALGORITHM.to_string(),
        nonce: encode_base64(&nonce),
        ciphertext: encode_base64(&ciphertext),
    })
}

fn decrypt_saved_session(
    account_id: i64,
    key_bytes: &[u8],
    envelope: &EncryptedSessionEnvelope,
) -> AppResult<SavedSession> {
    if envelope.version != ENVELOPE_VERSION || envelope.algorithm != ENVELOPE_ALGORITHM {
        return Err(AppError::internal(
            "Unsupported encrypted Telegram session format",
        ));
    }
    if key_bytes.len() != SESSION_KEY_BYTES {
        return Err(AppError::internal("Invalid Telegram session key length"));
    }
    let nonce_bytes = decode_base64(&envelope.nonce)?;
    if nonce_bytes.len() != 24 {
        return Err(AppError::internal(
            "Invalid encrypted Telegram session nonce length",
        ));
    }
    let ciphertext = decode_base64(&envelope.ciphertext)?;
    let nonce = XNonce::from_slice(&nonce_bytes);
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key_bytes));
    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: &ciphertext,
                aad: associated_data(account_id).as_bytes(),
            },
        )
        .map_err(|_| AppError::internal("Failed to decrypt Telegram session"))?;
    serde_json::from_slice::<SavedSession>(&plaintext)
        .map_err(|error| AppError::internal(error.to_string()))
}

fn generate_session_key() -> String {
    let mut key = [0u8; SESSION_KEY_BYTES];
    OsRng.fill_bytes(&mut key);
    encode_base64(&key)
}

async fn read_session_key(
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<Option<Vec<u8>>> {
    let key = telegram_account_session_key_secret(account_id);
    match secret_store.get_secret(key).await? {
        Some(value) => decode_base64(&value).map(Some),
        None => Ok(None),
    }
}

async fn ensure_session_key(
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<Vec<u8>> {
    if let Some(key) = read_session_key(secret_store, account_id).await? {
        return Ok(key);
    }
    let encoded = generate_session_key();
    secret_store
        .set_secret(telegram_account_session_key_secret(account_id), encoded.clone())
        .await?;
    decode_base64(&encoded)
}

fn write_atomic(path: &Path, contents: &str) -> AppResult<()> {
    let tmp_path = path.with_extension("session.json.tmp");
    fs::write(&tmp_path, contents).map_err(|error| AppError::internal(error.to_string()))?;
    fs::rename(&tmp_path, path).map_err(|error| AppError::internal(error.to_string()))
}

async fn write_encrypted_session_file(
    path: &Path,
    secret_store: &SecretStoreState,
    account_id: i64,
    saved: &SavedSession,
) -> AppResult<()> {
    let key = ensure_session_key(secret_store, account_id).await?;
    let envelope = encrypt_saved_session(account_id, &key, saved)?;
    let json =
        serde_json::to_string(&envelope).map_err(|error| AppError::internal(error.to_string()))?;
    write_atomic(path, &json)
}

pub(crate) async fn load_session(
    handle: &AppHandle,
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<Arc<MemorySession>> {
    let path = session_path(handle, account_id)?;
    load_session_from_path(&path, secret_store, account_id).await
}

pub(crate) async fn save_session(
    handle: &AppHandle,
    secret_store: &SecretStoreState,
    account_id: i64,
    session: &Arc<MemorySession>,
) -> AppResult<()> {
    let path = session_path(handle, account_id)?;
    let saved = memory_session_to_saved(session).await;
    write_encrypted_session_file(&path, secret_store, account_id, &saved).await
}

pub(crate) async fn delete_session(
    handle: &AppHandle,
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<()> {
    if let Ok(path) = session_path(handle, account_id) {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(AppError::internal(error.to_string())),
        }
    }
    secret_store
        .delete_secret(telegram_account_session_key_secret(account_id))
        .await
}

async fn load_session_from_path(
    path: &Path,
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<Arc<MemorySession>> {
    if !path.exists() {
        return Ok(Arc::new(MemorySession::default()));
    }

    let json = fs::read_to_string(path).map_err(|error| AppError::internal(error.to_string()))?;

    if let Ok(envelope) = serde_json::from_str::<EncryptedSessionEnvelope>(&json) {
        let key = read_session_key(secret_store, account_id)
            .await?
            .ok_or_else(|| {
                AppError::auth(format!(
                    "Telegram session key for account {account_id} is missing from secure storage. Sign in again."
                ))
            })?;
        let saved = decrypt_saved_session(account_id, &key, &envelope)?;
        return Ok(saved_to_memory_session(saved));
    }

    if let Ok(saved) = serde_json::from_str::<SavedSession>(&json) {
        write_encrypted_session_file(path, secret_store, account_id, &saved).await?;
        return Ok(saved_to_memory_session(saved));
    }

    Err(AppError::internal(
        "Telegram session file is not a supported format",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret_store::tests::InMemorySecretStore;
    use std::fs;

    fn memory_secret_store() -> (Arc<InMemorySecretStore>, SecretStoreState) {
        let store = Arc::new(InMemorySecretStore::new());
        let state = SecretStoreState::new(store.clone());
        (store, state)
    }

    async fn sample_saved_session() -> SavedSession {
        let session = Arc::new(MemorySession::default());
        memory_session_to_saved(&session).await
    }

    #[tokio::test]
    async fn saving_session_writes_encrypted_envelope_not_plaintext() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (_store, secret_store) = memory_secret_store();
        let saved = sample_saved_session().await;

        write_encrypted_session_file(&path, &secret_store, 7, &saved)
            .await
            .expect("write encrypted session");

        let json = fs::read_to_string(&path).expect("read encrypted session");
        assert!(serde_json::from_str::<EncryptedSessionEnvelope>(&json).is_ok());
        assert!(!json.contains("home_dc"));
        assert!(!json.contains("updates_state"));
    }

    #[tokio::test]
    async fn encrypted_session_load_round_trips() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (_store, secret_store) = memory_secret_store();
        let saved = sample_saved_session().await;

        write_encrypted_session_file(&path, &secret_store, 7, &saved)
            .await
            .expect("write encrypted session");
        let loaded = load_session_from_path(&path, &secret_store, 7)
            .await
            .expect("load encrypted session");

        assert_eq!(loaded.home_dc_id(), saved.home_dc);
    }
}
