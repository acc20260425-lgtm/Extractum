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
use secrecy::ExposeSecret;
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
        Some(value) => decode_base64(value.expose_secret()).map(Some),
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
        .set_secret(
            telegram_account_session_key_secret(account_id),
            encoded.clone(),
        )
        .await?;
    decode_base64(&encoded)
}

fn session_temp_path(path: &std::path::Path) -> std::path::PathBuf {
    path.with_extension("session.json.tmp")
}

fn write_atomic(path: &Path, contents: &str) -> AppResult<()> {
    let tmp_path = session_temp_path(path);
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
        delete_session_from_path(&path, secret_store, account_id).await?;
        return Ok(());
    }
    secret_store
        .delete_secret(telegram_account_session_key_secret(account_id))
        .await
}

async fn delete_session_from_path(
    path: &Path,
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<()> {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(AppError::internal(error.to_string())),
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
    use crate::secret_store::{
        telegram_account_api_hash_secret, telegram_account_session_key_secret,
        tests::InMemorySecretStore, SECRET_SERVICE_NAME,
    };
    use extractum_core::error::AppErrorKind;
    use secrecy::ExposeSecret;
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

    fn assert_error_contract(
        error: AppError,
        expected_kind: AppErrorKind,
        expected_message: &str,
        expected_json: &str,
    ) {
        assert_eq!(error.kind, expected_kind);
        assert_eq!(error.message, expected_message);
        assert_eq!(
            serde_json::to_string(&error).expect("serialize AppError"),
            expected_json
        );
    }

    #[tokio::test]
    async fn session_path_temp_path_and_error_contract_is_exact() {
        assert_eq!(SECRET_SERVICE_NAME, "org.ai.extractum");
        assert_eq!(
            telegram_account_api_hash_secret(7),
            "telegram.account.7.api_hash"
        );
        assert_eq!(
            telegram_account_session_key_secret(7),
            "telegram.account.7.session_key"
        );
        let session_file = PathBuf::from("telegram_7.session.json");
        assert_eq!(session_file.to_string_lossy(), "telegram_7.session.json");
        assert_eq!(
            session_temp_path(&session_file),
            PathBuf::from("telegram_7.session.session.json.tmp"),
            "RED: CP2 session path and error contract"
        );

        assert_eq!(ENVELOPE_VERSION, 1);
        assert_eq!(ENVELOPE_ALGORITHM, "XChaCha20-Poly1305");
        assert_eq!(SESSION_KEY_BYTES, 32);
        assert_eq!(
            associated_data(7),
            "org.ai.extractum.telegram.session.v1.account.7"
        );
        assert_eq!(encode_base64(&[0xff, 0xee]), "_-4");
        assert_eq!(
            decode_base64("_-4").expect("decode URL-safe base64"),
            vec![0xff, 0xee]
        );
        assert!(!encode_base64(&[0xff, 0xee]).contains('='));

        let deterministic_envelope = EncryptedSessionEnvelope {
            version: 1,
            algorithm: "XChaCha20-Poly1305".to_string(),
            nonce: "AA".to_string(),
            ciphertext: "AQI".to_string(),
        };
        assert_eq!(
            serde_json::to_string(&deterministic_envelope).expect("serialize envelope"),
            r#"{"version":1,"algorithm":"XChaCha20-Poly1305","nonce":"AA","ciphertext":"AQI"}"#
        );

        let saved = sample_saved_session().await;
        let key = [7u8; SESSION_KEY_BYTES];
        let encrypted =
            encrypt_saved_session(7, &key, &saved).expect("encrypt canonical saved session");
        assert_eq!(encrypted.version, 1);
        assert_eq!(encrypted.algorithm, "XChaCha20-Poly1305");
        assert_eq!(
            decode_base64(&encrypted.nonce).expect("decode nonce").len(),
            24
        );
        assert!(!encrypted.nonce.contains('='));
        assert!(!encrypted.ciphertext.contains('='));
        let decrypted =
            decrypt_saved_session(7, &key, &encrypted).expect("decrypt canonical envelope");
        assert_eq!(decrypted.home_dc, saved.home_dc);

        assert_error_contract(
            AppError::internal("Failed to encrypt Telegram session"),
            AppErrorKind::Internal,
            "Failed to encrypt Telegram session",
            r#"{"kind":"internal","message":"Failed to encrypt Telegram session"}"#,
        );
        assert_error_contract(
            decode_base64("*").expect_err("reject malformed base64"),
            AppErrorKind::Internal,
            "Invalid encrypted Telegram session encoding: Invalid symbol 42, offset 0.",
            r#"{"kind":"internal","message":"Invalid encrypted Telegram session encoding: Invalid symbol 42, offset 0."}"#,
        );
        assert_error_contract(
            match encrypt_saved_session(7, &[0; SESSION_KEY_BYTES - 1], &saved) {
                Ok(_) => panic!("invalid encryption key must fail"),
                Err(error) => error,
            },
            AppErrorKind::Internal,
            "Invalid Telegram session key length",
            r#"{"kind":"internal","message":"Invalid Telegram session key length"}"#,
        );
        assert_error_contract(
            match decrypt_saved_session(
                7,
                &key,
                &EncryptedSessionEnvelope {
                    version: 2,
                    algorithm: ENVELOPE_ALGORITHM.to_string(),
                    nonce: encrypted.nonce.clone(),
                    ciphertext: encrypted.ciphertext.clone(),
                },
            ) {
                Ok(_) => panic!("unsupported envelope must fail"),
                Err(error) => error,
            },
            AppErrorKind::Internal,
            "Unsupported encrypted Telegram session format",
            r#"{"kind":"internal","message":"Unsupported encrypted Telegram session format"}"#,
        );
        assert_error_contract(
            match decrypt_saved_session(7, &[0; SESSION_KEY_BYTES - 1], &encrypted) {
                Ok(_) => panic!("invalid decryption key must fail"),
                Err(error) => error,
            },
            AppErrorKind::Internal,
            "Invalid Telegram session key length",
            r#"{"kind":"internal","message":"Invalid Telegram session key length"}"#,
        );
        assert_error_contract(
            match decrypt_saved_session(
                7,
                &key,
                &EncryptedSessionEnvelope {
                    version: ENVELOPE_VERSION,
                    algorithm: ENVELOPE_ALGORITHM.to_string(),
                    nonce: encode_base64(&[0; 23]),
                    ciphertext: encrypted.ciphertext.clone(),
                },
            ) {
                Ok(_) => panic!("invalid nonce length must fail"),
                Err(error) => error,
            },
            AppErrorKind::Internal,
            "Invalid encrypted Telegram session nonce length",
            r#"{"kind":"internal","message":"Invalid encrypted Telegram session nonce length"}"#,
        );
        assert_error_contract(
            match decrypt_saved_session(8, &key, &encrypted) {
                Ok(_) => panic!("wrong-account AAD must reject ciphertext"),
                Err(error) => error,
            },
            AppErrorKind::Internal,
            "Failed to decrypt Telegram session",
            r#"{"kind":"internal","message":"Failed to decrypt Telegram session"}"#,
        );

        let temp = tempfile::tempdir().expect("tempdir");
        let encrypted_path = temp.path().join("telegram_7.session.json");
        fs::write(
            &encrypted_path,
            serde_json::to_string(&encrypted).expect("serialize encrypted envelope"),
        )
        .expect("write encrypted envelope");
        let (_missing_store, missing_secret_store) = memory_secret_store();
        assert_error_contract(
            match load_session_from_path(&encrypted_path, &missing_secret_store, 7).await {
                Ok(_) => panic!("missing key must fail"),
                Err(error) => error,
            },
            AppErrorKind::Auth,
            "Telegram session key for account 7 is missing from secure storage. Sign in again.",
            r#"{"kind":"auth","message":"Telegram session key for account 7 is missing from secure storage. Sign in again."}"#,
        );

        let malformed_path = temp.path().join("telegram_8.session.json");
        fs::write(&malformed_path, "{}").expect("write malformed envelope");
        let (_malformed_store, malformed_secret_store) = memory_secret_store();
        assert_error_contract(
            match load_session_from_path(&malformed_path, &malformed_secret_store, 8).await {
                Ok(_) => panic!("malformed envelope must fail"),
                Err(error) => error,
            },
            AppErrorKind::Internal,
            "Telegram session file is not a supported format",
            r#"{"kind":"internal","message":"Telegram session file is not a supported format"}"#,
        );

        let legacy_path = temp.path().join("telegram_9.session.json");
        let legacy_json = serde_json::to_string(&saved).expect("serialize legacy session");
        fs::write(&legacy_path, &legacy_json).expect("write legacy session");
        let (legacy_store, legacy_secret_store) = memory_secret_store();
        legacy_store.fail_set("secure store unavailable");
        assert_error_contract(
            match load_session_from_path(&legacy_path, &legacy_secret_store, 9).await {
                Ok(_) => panic!("legacy encryption key write must fail"),
                Err(error) => error,
            },
            AppErrorKind::Internal,
            "secure store unavailable",
            r#"{"kind":"internal","message":"secure store unavailable"}"#,
        );
        assert_eq!(
            fs::read_to_string(&legacy_path).expect("legacy file remains readable"),
            legacy_json
        );

        let delete_path = temp.path().join("telegram_10.session.json");
        let (delete_store, delete_secret_store) = memory_secret_store();
        write_encrypted_session_file(&delete_path, &delete_secret_store, 10, &saved)
            .await
            .expect("write session before deletion");
        delete_store.fail_delete("secure delete unavailable");
        assert_error_contract(
            delete_session_from_path(&delete_path, &delete_secret_store, 10)
                .await
                .expect_err("secret deletion must fail after file deletion"),
            AppErrorKind::Internal,
            "secure delete unavailable",
            r#"{"kind":"internal","message":"secure delete unavailable"}"#,
        );
        assert!(!delete_path.exists());
        assert!(delete_secret_store
            .get_secret(telegram_account_session_key_secret(10))
            .await
            .expect("read retained key")
            .is_some());
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

    #[tokio::test]
    async fn legacy_plaintext_session_migrates_to_encrypted_file() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (_store, secret_store) = memory_secret_store();
        let saved = sample_saved_session().await;
        let legacy_json = serde_json::to_string(&saved).expect("legacy json");
        fs::write(&path, &legacy_json).expect("write legacy session");

        let loaded = load_session_from_path(&path, &secret_store, 7)
            .await
            .expect("load legacy session");

        assert_eq!(loaded.home_dc_id(), saved.home_dc);
        let migrated = fs::read_to_string(&path).expect("read migrated session");
        assert!(serde_json::from_str::<EncryptedSessionEnvelope>(&migrated).is_ok());
        assert_ne!(migrated, legacy_json);
    }

    #[tokio::test]
    async fn legacy_plaintext_session_remains_when_keyring_write_fails() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (store, secret_store) = memory_secret_store();
        let legacy_json =
            serde_json::to_string(&sample_saved_session().await).expect("legacy json");
        fs::write(&path, &legacy_json).expect("write legacy session");
        store.fail_set("secure store unavailable");

        let error = match load_session_from_path(&path, &secret_store, 7).await {
            Ok(_) => panic!("migration should fail"),
            Err(error) => error,
        };

        assert_eq!(error.message, "secure store unavailable");
        assert_eq!(
            fs::read_to_string(&path).expect("read legacy session"),
            legacy_json
        );
    }

    #[tokio::test]
    async fn encrypted_session_load_fails_when_key_is_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (_writer_store, writer_secret_store) = memory_secret_store();
        let (_reader_store, reader_secret_store) = memory_secret_store();

        let saved = sample_saved_session().await;

        write_encrypted_session_file(&path, &writer_secret_store, 7, &saved)
            .await
            .expect("write encrypted session");

        let error = match load_session_from_path(&path, &reader_secret_store, 7).await {
            Ok(_) => panic!("missing key should fail"),
            Err(error) => error,
        };

        assert!(error
            .message
            .contains("Telegram session key for account 7 is missing"));
    }

    #[tokio::test]
    async fn encrypted_session_load_fails_for_wrong_account_id() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (_store, secret_store) = memory_secret_store();

        let saved = sample_saved_session().await;

        write_encrypted_session_file(&path, &secret_store, 7, &saved)
            .await
            .expect("write encrypted session");

        let key = secret_store
            .get_secret(telegram_account_session_key_secret(7))
            .await
            .expect("read session key")
            .expect("session key exists");
        secret_store
            .set_secret(telegram_account_session_key_secret(8), key.expose_secret())
            .await
            .expect("copy key to wrong account");

        let error = match load_session_from_path(&path, &secret_store, 8).await {
            Ok(_) => panic!("wrong account aad should fail"),
            Err(error) => error,
        };

        assert_eq!(error.message, "Failed to decrypt Telegram session");
    }

    #[tokio::test]
    async fn delete_session_from_path_removes_file_and_key() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (_store, secret_store) = memory_secret_store();

        let saved = sample_saved_session().await;

        write_encrypted_session_file(&path, &secret_store, 7, &saved)
            .await
            .expect("write encrypted session");

        delete_session_from_path(&path, &secret_store, 7)
            .await
            .expect("delete session");

        assert!(!path.exists());
        assert!(secret_store
            .get_secret(telegram_account_session_key_secret(7))
            .await
            .expect("read session key")
            .is_none());
    }
}
