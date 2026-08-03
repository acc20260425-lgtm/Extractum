use std::fs;
use std::path::{Path, PathBuf};

use secrecy::ExposeSecret;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::secret_store::{telegram_account_session_key_secret, SecretStoreState};
use crate::telegram_impl::{
    decode_session_json, encode_session_json, session_json_requires_existing_key,
    SessionEncryptionKey, TelegramSession,
};

pub(crate) fn session_path(handle: &AppHandle, account_id: i64) -> AppResult<PathBuf> {
    let app_dir = handle
        .path()
        .app_data_dir()
        .map_err(|error| AppError::internal(error.to_string()))?;
    session_path_from_app_data_root(&app_dir, account_id)
}

fn session_path_from_app_data_root(app_data_root: &Path, account_id: i64) -> AppResult<PathBuf> {
    fs::create_dir_all(app_data_root).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(app_data_root.join(format!("telegram_{account_id}.session.json")))
}

pub(crate) fn session_exists(handle: &AppHandle, account_id: i64) -> bool {
    session_path(handle, account_id)
        .map(|path| path.exists())
        .unwrap_or(false)
}

async fn read_session_key(
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<Option<SessionEncryptionKey>> {
    let key = telegram_account_session_key_secret(account_id);
    match secret_store.get_secret(key).await? {
        Some(encoded) => SessionEncryptionKey::try_from_encoded(encoded).map(Some),
        None => Ok(None),
    }
}

async fn ensure_session_key(
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<SessionEncryptionKey> {
    if let Some(key) = read_session_key(secret_store, account_id).await? {
        return Ok(key);
    }
    let (key, encoded) = SessionEncryptionKey::generate();
    secret_store
        .set_secret(
            telegram_account_session_key_secret(account_id),
            encoded.expose_secret(),
        )
        .await?;
    Ok(key)
}

fn session_temp_path(path: &std::path::Path) -> std::path::PathBuf {
    path.with_extension("session.json.tmp")
}

fn write_atomic(path: &Path, contents: &str) -> AppResult<()> {
    write_atomic_with(
        path,
        contents,
        |path, contents| fs::write(path, contents),
        |from, to| fs::rename(from, to),
    )
}

fn write_atomic_with<Write, Rename>(
    path: &Path,
    contents: &str,
    write: Write,
    rename: Rename,
) -> AppResult<()>
where
    Write: FnOnce(&Path, &str) -> std::io::Result<()>,
    Rename: FnOnce(&Path, &Path) -> std::io::Result<()>,
{
    let tmp_path = session_temp_path(path);
    write(&tmp_path, contents).map_err(|error| AppError::internal(error.to_string()))?;
    if let Err(error) = rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(AppError::internal(error.to_string()));
    }
    Ok(())
}

async fn write_encrypted_session_file(
    path: &Path,
    secret_store: &SecretStoreState,
    account_id: i64,
    session: &TelegramSession,
) -> AppResult<()> {
    let key = ensure_session_key(secret_store, account_id).await?;
    let json = encode_session_json(session, account_id, &key).await?;
    write_atomic(path, &json)
}

pub(crate) async fn load_session(
    handle: &AppHandle,
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<TelegramSession> {
    let path = session_path(handle, account_id)?;
    load_session_from_path(&path, secret_store, account_id).await
}

pub(crate) async fn save_session(
    handle: &AppHandle,
    secret_store: &SecretStoreState,
    account_id: i64,
    session: &TelegramSession,
) -> AppResult<()> {
    let path = session_path(handle, account_id)?;
    write_encrypted_session_file(&path, secret_store, account_id, session).await
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
) -> AppResult<TelegramSession> {
    if !path.exists() {
        return Ok(TelegramSession::empty());
    }

    let json = fs::read_to_string(path).map_err(|error| AppError::internal(error.to_string()))?;
    let requires_existing_key = session_json_requires_existing_key(&json)?;

    if requires_existing_key {
        let key = read_session_key(secret_store, account_id).await?;
        return decode_session_json(&json, account_id, key.as_ref());
    }

    let session = decode_session_json(&json, account_id, None)?;
    let key = ensure_session_key(secret_store, account_id).await?;
    let encrypted = encode_session_json(&session, account_id, &key).await?;
    write_atomic(path, &encrypted)?;
    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret_store::{
        telegram_account_api_hash_secret, telegram_account_session_key_secret,
        tests::InMemorySecretStore, SECRET_SERVICE_NAME,
    };
    use extractum_core::error::AppErrorKind;
    use secrecy::SecretString;
    use std::sync::Arc;

    const LEGACY_SESSION_JSON: &str = r#"{"home_dc":2,"dc_options":{},"updates_state":{"pts":0,"qts":0,"date":0,"seq":0,"channels":[]}}"#;

    fn memory_secret_store() -> (Arc<InMemorySecretStore>, SecretStoreState) {
        let store = Arc::new(InMemorySecretStore::new());
        let state = SecretStoreState::new(store.clone());
        (store, state)
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

    fn key_error(result: AppResult<SessionEncryptionKey>, context: &str) -> AppError {
        match result {
            Ok(_) => panic!("{context}"),
            Err(error) => error,
        }
    }

    fn session_error(result: AppResult<TelegramSession>, context: &str) -> AppError {
        match result {
            Ok(_) => panic!("{context}"),
            Err(error) => error,
        }
    }

    #[test]
    fn session_path_uses_app_data_root_and_account_filename() {
        let app_data_root = tempfile::tempdir().expect("app data root");

        let path = session_path_from_app_data_root(app_data_root.path(), 42)
            .expect("resolve private session path");

        assert_eq!(path.parent(), Some(app_data_root.path()));
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("telegram_42.session.json")
        );
        assert!(app_data_root.path().is_dir());
    }

    #[test]
    fn atomic_session_write_outcome_and_error_contract_is_exact() {
        let success_root = tempfile::tempdir().expect("success root");
        let success_path = success_root.path().join("telegram_7.session.json");
        fs::write(&success_path, "old").expect("write old target");
        write_atomic_with(
            &success_path,
            "new",
            |path, contents| fs::write(path, contents),
            |from, to| fs::rename(from, to),
        )
        .expect("replace session atomically");
        assert_eq!(
            fs::read_to_string(&success_path).expect("read replacement"),
            "new"
        );
        assert!(!session_temp_path(&success_path).exists());

        let write_root = tempfile::tempdir().expect("write failure root");
        let write_path = write_root.path().join("telegram_8.session.json");
        let write_error = write_atomic_with(
            &write_path,
            "new",
            |_, _| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "scripted write failure",
                ))
            },
            |_, _| panic!("rename must not run after write failure"),
        )
        .expect_err("write failure must be internal");
        assert_error_contract(
            write_error,
            AppErrorKind::Internal,
            "scripted write failure",
            r#"{"kind":"internal","message":"scripted write failure"}"#,
        );
        assert!(!write_path.exists());
        assert!(!session_temp_path(&write_path).exists());

        let rename_root = tempfile::tempdir().expect("rename failure root");
        let rename_path = rename_root.path().join("telegram_9.session.json");
        fs::write(&rename_path, "old").expect("write retained target");
        let rename_error = write_atomic_with(
            &rename_path,
            "new",
            |path, contents| fs::write(path, contents),
            |_, _| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "scripted rename failure",
                ))
            },
        )
        .expect_err("rename failure must be internal");
        assert_error_contract(
            rename_error,
            AppErrorKind::Internal,
            "scripted rename failure",
            r#"{"kind":"internal","message":"scripted rename failure"}"#,
        );
        assert_eq!(
            fs::read_to_string(&rename_path).expect("read retained target"),
            "old"
        );
        assert!(!session_temp_path(&rename_path).exists());
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
            PathBuf::from("telegram_7.session.session.json.tmp")
        );

        assert_error_contract(
            key_error(
                SessionEncryptionKey::try_from_encoded(SecretString::new("AA".to_string())),
                "invalid key length must fail",
            ),
            AppErrorKind::Internal,
            "Invalid Telegram session key length",
            r#"{"kind":"internal","message":"Invalid Telegram session key length"}"#,
        );
        assert_error_contract(
            key_error(
                SessionEncryptionKey::try_from_encoded(SecretString::new("*".to_string())),
                "malformed key encoding must fail",
            ),
            AppErrorKind::Internal,
            "Invalid encrypted Telegram session encoding: Invalid symbol 42, offset 0.",
            r#"{"kind":"internal","message":"Invalid encrypted Telegram session encoding: Invalid symbol 42, offset 0."}"#,
        );
        assert_error_contract(
            AppError::internal("Failed to encrypt Telegram session"),
            AppErrorKind::Internal,
            "Failed to encrypt Telegram session",
            r#"{"kind":"internal","message":"Failed to encrypt Telegram session"}"#,
        );

        let temp = tempfile::tempdir().expect("tempdir");
        let malformed_path = temp.path().join("telegram_8.session.json");
        fs::write(&malformed_path, "{}").expect("write malformed envelope");
        let (malformed_store, malformed_secret_store) = memory_secret_store();
        malformed_store.fail_get("secure store should not be read");
        assert_error_contract(
            session_error(
                load_session_from_path(&malformed_path, &malformed_secret_store, 8).await,
                "malformed envelope must fail before key lookup",
            ),
            AppErrorKind::Internal,
            "Telegram session file is not a supported format",
            r#"{"kind":"internal","message":"Telegram session file is not a supported format"}"#,
        );

        let unsupported_path = temp.path().join("telegram_9.session.json");
        fs::write(
            &unsupported_path,
            r#"{"version":2,"algorithm":"XChaCha20-Poly1305","nonce":"AA","ciphertext":"AQI"}"#,
        )
        .expect("write unsupported envelope");
        let (_unsupported_store, unsupported_secret_store) = memory_secret_store();
        let (_, encoded) = SessionEncryptionKey::generate();
        unsupported_secret_store
            .set_secret(
                telegram_account_session_key_secret(9),
                encoded.expose_secret(),
            )
            .await
            .expect("store key");
        assert_error_contract(
            session_error(
                load_session_from_path(&unsupported_path, &unsupported_secret_store, 9).await,
                "unsupported envelope must fail after key lookup",
            ),
            AppErrorKind::Internal,
            "Unsupported encrypted Telegram session format",
            r#"{"kind":"internal","message":"Unsupported encrypted Telegram session format"}"#,
        );

        let legacy_path = temp.path().join("telegram_10.session.json");
        fs::write(&legacy_path, LEGACY_SESSION_JSON).expect("write legacy session");
        let (legacy_store, legacy_secret_store) = memory_secret_store();
        legacy_store.fail_set("secure store unavailable");
        assert_error_contract(
            session_error(
                load_session_from_path(&legacy_path, &legacy_secret_store, 10).await,
                "legacy encryption key write must fail",
            ),
            AppErrorKind::Internal,
            "secure store unavailable",
            r#"{"kind":"internal","message":"secure store unavailable"}"#,
        );
        assert_eq!(
            fs::read_to_string(&legacy_path).expect("legacy file remains readable"),
            LEGACY_SESSION_JSON
        );

        let delete_path = temp.path().join("telegram_11.session.json");
        let (delete_store, delete_secret_store) = memory_secret_store();
        write_encrypted_session_file(
            &delete_path,
            &delete_secret_store,
            11,
            &TelegramSession::empty(),
        )
        .await
        .expect("write session before deletion");
        delete_store.fail_delete("secure delete unavailable");
        assert_error_contract(
            delete_session_from_path(&delete_path, &delete_secret_store, 11)
                .await
                .expect_err("secret deletion must fail after file deletion"),
            AppErrorKind::Internal,
            "secure delete unavailable",
            r#"{"kind":"internal","message":"secure delete unavailable"}"#,
        );
        assert!(!delete_path.exists());
        assert!(delete_secret_store
            .get_secret(telegram_account_session_key_secret(11))
            .await
            .expect("read retained key")
            .is_some());
    }

    #[tokio::test]
    async fn legacy_plaintext_session_migrates_to_encrypted_file() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (_store, secret_store) = memory_secret_store();
        fs::write(&path, LEGACY_SESSION_JSON).expect("write legacy session");

        load_session_from_path(&path, &secret_store, 7)
            .await
            .expect("load legacy session");

        let migrated = fs::read_to_string(&path).expect("read migrated session");
        assert!(session_json_requires_existing_key(&migrated).expect("classify migrated envelope"));
        assert_ne!(migrated, LEGACY_SESSION_JSON);
    }

    #[tokio::test]
    async fn legacy_plaintext_session_remains_when_keyring_write_fails() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (store, secret_store) = memory_secret_store();
        fs::write(&path, LEGACY_SESSION_JSON).expect("write legacy session");
        store.fail_set("secure store unavailable");

        let error = session_error(
            load_session_from_path(&path, &secret_store, 7).await,
            "migration should fail",
        );

        assert_eq!(error.message, "secure store unavailable");
        assert_eq!(
            fs::read_to_string(&path).expect("read legacy session"),
            LEGACY_SESSION_JSON
        );
    }

    #[tokio::test]
    async fn encrypted_session_load_fails_when_key_is_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (_writer_store, writer_secret_store) = memory_secret_store();
        let (_reader_store, reader_secret_store) = memory_secret_store();

        write_encrypted_session_file(&path, &writer_secret_store, 7, &TelegramSession::empty())
            .await
            .expect("write encrypted session");

        let error = session_error(
            load_session_from_path(&path, &reader_secret_store, 7).await,
            "missing key should fail",
        );

        assert_eq!(error.kind, AppErrorKind::Auth);
        assert_eq!(
            error.message,
            "Telegram session key for account 7 is missing from secure storage. Sign in again."
        );
    }

    #[tokio::test]
    async fn delete_session_from_path_removes_file_and_key() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("telegram_7.session.json");
        let (_store, secret_store) = memory_secret_store();

        write_encrypted_session_file(&path, &secret_store, 7, &TelegramSession::empty())
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
