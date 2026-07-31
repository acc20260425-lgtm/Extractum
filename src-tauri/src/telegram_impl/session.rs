use std::collections::HashMap;
use std::sync::Arc;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use grammers_session::types::{DcOption, PeerInfo, UpdatesState};
use grammers_session::{storages::MemorySession, Session, SessionData};
use rand_core::RngCore;
use secrecy::{ExposeSecret, SecretString, SecretVec};

use extractum_core::error::{AppError, AppResult};

const SESSION_KEY_BYTES: usize = 32;
const ENVELOPE_VERSION: u8 = 1;
const ENVELOPE_ALGORITHM: &str = "XChaCha20-Poly1305";

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

#[derive(Clone)]
pub struct SessionEncryptionKey(Arc<SecretVec<u8>>);

impl SessionEncryptionKey {
    pub fn try_from_encoded(encoded: SecretString) -> AppResult<Self> {
        let key = decode_base64(encoded.expose_secret())?;
        if key.len() != SESSION_KEY_BYTES {
            return Err(AppError::internal("Invalid Telegram session key length"));
        }
        Ok(Self(Arc::new(SecretVec::new(key))))
    }

    pub fn generate() -> (Self, SecretString) {
        let mut key = [0u8; SESSION_KEY_BYTES];
        OsRng.fill_bytes(&mut key);
        (
            Self(Arc::new(SecretVec::new(key.to_vec()))),
            SecretString::new(encode_base64(&key)),
        )
    }
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

fn associated_data(account_id: i64) -> String {
    format!("org.ai.extractum.telegram.session.v1.account.{account_id}")
}

async fn memory_session_to_saved(session: &TelegramSession) -> SavedSession {
    let session = session.raw_memory_session();
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

fn saved_to_telegram_session(saved: SavedSession) -> TelegramSession {
    let session_data = SessionData {
        home_dc: saved.home_dc,
        dc_options: saved.dc_options,
        peer_infos: HashMap::new(),
        updates_state: saved.updates_state,
    };
    TelegramSession {
        inner: Arc::new(MemorySession::from(session_data)),
    }
}

fn encrypt_saved_session(
    account_id: i64,
    key: &SessionEncryptionKey,
    saved: &SavedSession,
) -> AppResult<EncryptedSessionEnvelope> {
    let key_bytes = key.0.expose_secret();
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
    key: &SessionEncryptionKey,
    envelope: &EncryptedSessionEnvelope,
) -> AppResult<SavedSession> {
    if envelope.version != ENVELOPE_VERSION || envelope.algorithm != ENVELOPE_ALGORITHM {
        return Err(AppError::internal(
            "Unsupported encrypted Telegram session format",
        ));
    }
    let key_bytes = key.0.expose_secret();
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

#[derive(Clone)]
pub struct TelegramSession {
    inner: Arc<MemorySession>,
}

impl TelegramSession {
    pub fn empty() -> Self {
        Self {
            inner: Arc::new(MemorySession::default()),
        }
    }

    pub(super) fn clone_memory_session(&self) -> Arc<MemorySession> {
        Arc::clone(&self.inner)
    }

    pub(super) fn raw_memory_session(&self) -> &Arc<MemorySession> {
        &self.inner
    }

    pub(super) async fn cache_peer_infos(&self, peer_infos: &[PeerInfo]) {
        for peer_info in peer_infos {
            if peer_info.auth().is_some() {
                self.inner.cache_peer(peer_info).await;
            }
        }
    }
}

pub fn session_json_requires_existing_key(json: &str) -> AppResult<bool> {
    if serde_json::from_str::<EncryptedSessionEnvelope>(json).is_ok() {
        return Ok(true);
    }
    if serde_json::from_str::<SavedSession>(json).is_ok() {
        return Ok(false);
    }
    Err(AppError::internal(
        "Telegram session file is not a supported format",
    ))
}

pub fn decode_session_json(
    json: &str,
    account_id: i64,
    key: Option<&SessionEncryptionKey>,
) -> AppResult<TelegramSession> {
    if let Ok(envelope) = serde_json::from_str::<EncryptedSessionEnvelope>(json) {
        let key = key.ok_or_else(|| {
            AppError::auth(format!(
                "Telegram session key for account {account_id} is missing from secure storage. Sign in again."
            ))
        })?;
        let saved = decrypt_saved_session(account_id, key, &envelope)?;
        return Ok(saved_to_telegram_session(saved));
    }

    if let Ok(saved) = serde_json::from_str::<SavedSession>(json) {
        return Ok(saved_to_telegram_session(saved));
    }

    Err(AppError::internal(
        "Telegram session file is not a supported format",
    ))
}

pub async fn encode_session_json(
    session: &TelegramSession,
    account_id: i64,
    key: &SessionEncryptionKey,
) -> AppResult<String> {
    let saved = memory_session_to_saved(session).await;
    let envelope = encrypt_saved_session(account_id, key, &saved)?;
    serde_json::to_string(&envelope).map_err(|error| AppError::internal(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use extractum_core::error::AppErrorKind;

    fn test_session(home_dc: i32) -> TelegramSession {
        TelegramSession {
            inner: Arc::new(MemorySession::from(SessionData {
                home_dc,
                dc_options: HashMap::new(),
                peer_infos: HashMap::new(),
                updates_state: UpdatesState::default(),
            })),
        }
    }

    #[test]
    fn session_encryption_key_rejects_invalid_length() {
        let encoded = SecretString::new(URL_SAFE_NO_PAD.encode([7u8; SESSION_KEY_BYTES - 1]));
        match SessionEncryptionKey::try_from_encoded(encoded) {
            Ok(_) => panic!("RED: CP4 invalid session key length"),
            Err(error) => {
                assert_eq!(error.message, "Invalid Telegram session key length");
            }
        }
    }

    #[test]
    fn generated_session_key_returns_write_only_encoded_secret() {
        let (_key, encoded) = SessionEncryptionKey::generate();
        let decoded = URL_SAFE_NO_PAD
            .decode(encoded.expose_secret())
            .expect("generated key is URL-safe base64");
        assert_eq!(
            decoded.len(),
            SESSION_KEY_BYTES,
            "RED: CP4 generated session key"
        );
        SessionEncryptionKey::try_from_encoded(encoded).expect("generated key reconstructs");
    }

    #[tokio::test]
    async fn saving_session_writes_encrypted_envelope_not_plaintext() {
        let session = test_session(2);
        let key = SessionEncryptionKey::try_from_encoded(SecretString::new(
            URL_SAFE_NO_PAD.encode([7u8; SESSION_KEY_BYTES]),
        ))
        .expect("valid key");

        let json = encode_session_json(&session, 7, &key)
            .await
            .expect("encode encrypted session");

        assert!(
            serde_json::from_str::<EncryptedSessionEnvelope>(&json).is_ok(),
            "moved codec must write the canonical encrypted envelope"
        );
        assert!(!json.contains("home_dc"));
        assert!(!json.contains("updates_state"));
    }

    #[tokio::test]
    async fn encrypted_session_load_round_trips() {
        let session = test_session(2);
        let key = SessionEncryptionKey::try_from_encoded(SecretString::new(
            URL_SAFE_NO_PAD.encode([7u8; SESSION_KEY_BYTES]),
        ))
        .expect("valid key");
        let json = encode_session_json(&session, 7, &key)
            .await
            .expect("encode encrypted session");

        let loaded = decode_session_json(&json, 7, Some(&key)).expect("decode encrypted session");

        let loaded_memory_session = loaded.clone_memory_session();
        let same_memory_session = loaded.clone_memory_session();
        assert!(Arc::ptr_eq(&loaded_memory_session, &same_memory_session));
        assert_eq!(loaded_memory_session.home_dc_id(), 2);
    }

    #[tokio::test]
    async fn encrypted_session_load_fails_for_wrong_account_id() {
        let session = test_session(2);
        let key = SessionEncryptionKey::try_from_encoded(SecretString::new(
            URL_SAFE_NO_PAD.encode([7u8; SESSION_KEY_BYTES]),
        ))
        .expect("valid key");
        let json = encode_session_json(&session, 7, &key)
            .await
            .expect("encode encrypted session");

        let error = match decode_session_json(&json, 8, Some(&key)) {
            Ok(_) => panic!("wrong account AAD should fail"),
            Err(error) => error,
        };

        assert_eq!(error.message, "Failed to decrypt Telegram session");
    }

    #[tokio::test]
    async fn legacy_json_returns_rewrite_decision() {
        let session = test_session(2);
        let legacy_json = serde_json::to_string(&memory_session_to_saved(&session).await)
            .expect("serialize legacy session");

        assert!(
            !session_json_requires_existing_key(&legacy_json).expect("classify legacy session"),
            "legacy input must return the rewrite decision"
        );
    }

    #[tokio::test]
    async fn missing_encrypted_key_preserves_auth_error() {
        let session = test_session(2);
        let key = SessionEncryptionKey::try_from_encoded(SecretString::new(
            URL_SAFE_NO_PAD.encode([7u8; SESSION_KEY_BYTES]),
        ))
        .expect("valid key");
        let json = encode_session_json(&session, 7, &key)
            .await
            .expect("encode encrypted session");

        let error = match decode_session_json(&json, 7, None) {
            Ok(_) => panic!("missing encrypted key must fail"),
            Err(error) => error,
        };
        assert_eq!(error.kind, AppErrorKind::Auth);
        assert_eq!(
            error.message,
            "Telegram session key for account 7 is missing from secure storage. Sign in again."
        );
        assert_eq!(
            serde_json::to_string(&error).expect("serialize missing-key error"),
            r#"{"kind":"auth","message":"Telegram session key for account 7 is missing from secure storage. Sign in again."}"#
        );
    }
}
