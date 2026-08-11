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

pub(crate) fn session_error(error: impl std::fmt::Display) -> AppError {
    AppError::internal(error.to_string())
}

fn associated_data(account_id: i64) -> String {
    format!("org.ai.extractum.telegram.session.v1.account.{account_id}")
}

async fn memory_session_to_saved(session: &TelegramSession) -> AppResult<SavedSession> {
    let session = session.clone_memory_session();
    let home_dc = session.home_dc_id().map_err(session_error)?;
    let updates_state = session.updates_state().await.map_err(session_error)?;
    let mut dc_options = HashMap::new();
    for dc_id in 1..=5i32 {
        if let Some(dc) = session.dc_option(dc_id).map_err(session_error)? {
            dc_options.insert(dc_id, dc);
        }
    }
    Ok(SavedSession {
        home_dc,
        dc_options,
        updates_state,
    })
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
    encrypt_saved_session_with(account_id, key, saved, |cipher, nonce, payload| {
        cipher.encrypt(nonce, payload)
    })
}

fn encrypt_saved_session_with<F>(
    account_id: i64,
    key: &SessionEncryptionKey,
    saved: &SavedSession,
    encrypt: F,
) -> AppResult<EncryptedSessionEnvelope>
where
    F: FnOnce(
        &XChaCha20Poly1305,
        &XNonce,
        Payload<'_, '_>,
    ) -> Result<Vec<u8>, chacha20poly1305::aead::Error>,
{
    let key_bytes = key.0.expose_secret();
    if key_bytes.len() != SESSION_KEY_BYTES {
        return Err(AppError::internal("Invalid Telegram session key length"));
    }
    let plaintext =
        serde_json::to_vec(saved).map_err(|error| AppError::internal(error.to_string()))?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key_bytes));
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let associated_data = associated_data(account_id);
    let ciphertext = encrypt(
        &cipher,
        &nonce,
        Payload {
            msg: &plaintext,
            aad: associated_data.as_bytes(),
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

    pub(super) async fn cache_peer_infos(&self, peer_infos: &[PeerInfo]) -> AppResult<()> {
        for peer_info in peer_infos {
            if peer_info.auth().is_some() {
                self.inner
                    .cache_peer(peer_info)
                    .await
                    .map_err(session_error)?;
            }
        }
        Ok(())
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
    let saved = memory_session_to_saved(session).await?;
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

    fn expect_error<T>(result: AppResult<T>, context: &str) -> AppError {
        match result {
            Ok(_) => panic!("{context}"),
            Err(error) => error,
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
        assert_eq!(
            loaded_memory_session
                .home_dc_id()
                .expect("read loaded home DC"),
            2
        );
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
    async fn encrypted_session_error_contract_is_exact() {
        let session = test_session(2);
        let key = SessionEncryptionKey::try_from_encoded(SecretString::new(
            URL_SAFE_NO_PAD.encode([7u8; SESSION_KEY_BYTES]),
        ))
        .expect("valid key");
        let saved = memory_session_to_saved(&session)
            .await
            .expect("save memory session");

        let encryption_error = expect_error(
            encrypt_saved_session_with(7, &key, &saved, |_, _, _| {
                Err(chacha20poly1305::aead::Error)
            }),
            "scripted encryption failure must preserve the error contract",
        );
        assert_eq!(encryption_error.kind, AppErrorKind::Internal);
        assert_eq!(
            encryption_error.message,
            "Failed to encrypt Telegram session"
        );

        let invalid_key_error = expect_error(
            SessionEncryptionKey::try_from_encoded(SecretString::new("!".to_owned())),
            "invalid encoded key must fail through the codec",
        );
        assert_eq!(invalid_key_error.kind, AppErrorKind::Internal);
        assert_eq!(
            invalid_key_error.message,
            "Invalid encrypted Telegram session encoding: Invalid symbol 33, offset 0."
        );

        let json = encode_session_json(&session, 7, &key)
            .await
            .expect("encode real encrypted session");
        let envelope: EncryptedSessionEnvelope =
            serde_json::from_str(&json).expect("decode real encrypted envelope");
        let valid_nonce = envelope.nonce.clone();

        let mut invalid_base64 = envelope;
        invalid_base64.nonce = "!".to_owned();
        let invalid_base64_error = expect_error(
            decode_session_json(
                &serde_json::to_string(&invalid_base64).expect("serialize invalid base64 envelope"),
                7,
                Some(&key),
            ),
            "invalid nonce encoding must fail through the codec",
        );
        assert_eq!(invalid_base64_error.kind, AppErrorKind::Internal);
        assert_eq!(
            invalid_base64_error.message,
            "Invalid encrypted Telegram session encoding: Invalid symbol 33, offset 0."
        );

        let mut invalid_nonce = invalid_base64;
        invalid_nonce.nonce = URL_SAFE_NO_PAD.encode([9u8; 23]);
        let invalid_nonce_error = expect_error(
            decode_session_json(
                &serde_json::to_string(&invalid_nonce).expect("serialize invalid nonce envelope"),
                7,
                Some(&key),
            ),
            "invalid nonce material must fail",
        );
        assert_eq!(invalid_nonce_error.kind, AppErrorKind::Internal);
        assert_eq!(
            invalid_nonce_error.message,
            "Invalid encrypted Telegram session nonce length"
        );

        let mut decrypt_failure = invalid_nonce;
        decrypt_failure.nonce = valid_nonce;
        decrypt_failure.ciphertext = URL_SAFE_NO_PAD.encode([0u8; 1]);
        let decrypt_error = expect_error(
            decode_session_json(
                &serde_json::to_string(&decrypt_failure)
                    .expect("serialize decrypt-failure envelope"),
                7,
                Some(&key),
            ),
            "invalid ciphertext must fail through the real decrypt path",
        );
        assert_eq!(decrypt_error.kind, AppErrorKind::Internal);
        assert_eq!(decrypt_error.message, "Failed to decrypt Telegram session");

        let mut unsupported_envelope = decrypt_failure;
        unsupported_envelope.version = ENVELOPE_VERSION + 1;
        let unsupported_envelope_error = expect_error(
            decode_session_json(
                &serde_json::to_string(&unsupported_envelope)
                    .expect("serialize unsupported envelope"),
                7,
                Some(&key),
            ),
            "unsupported envelope format must fail",
        );
        assert_eq!(unsupported_envelope_error.kind, AppErrorKind::Internal);
        assert_eq!(
            unsupported_envelope_error.message,
            "Unsupported encrypted Telegram session format"
        );

        let unsupported_file_error = expect_error(
            decode_session_json("{}", 7, Some(&key)),
            "unsupported session file format must fail",
        );
        assert_eq!(unsupported_file_error.kind, AppErrorKind::Internal);
        assert_eq!(
            unsupported_file_error.message,
            "Telegram session file is not a supported format"
        );
    }

    #[tokio::test]
    async fn legacy_json_returns_rewrite_decision() {
        let session = test_session(2);
        let saved = memory_session_to_saved(&session)
            .await
            .expect("save memory session");
        let legacy_json = serde_json::to_string(&saved).expect("serialize legacy session");

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
