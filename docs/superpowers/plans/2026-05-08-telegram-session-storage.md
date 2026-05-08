# Telegram Session Storage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Encrypt persisted Telegram session files while keeping the existing per-account app-data file model and lazily migrating plaintext session JSON.

**Architecture:** Add a focused `telegram_session_store` backend module that owns session path handling, `SavedSession` conversion, encryption, legacy plaintext migration, and deletion. `telegram.rs` delegates all session file operations to this module and treats persisted-session read failures as restore failures instead of silently falling back to an empty session.

**Tech Stack:** Tauri 2, Rust, `grammers-session`, `keyring` through existing `SecretStoreState`, RustCrypto `chacha20poly1305`, `rand_core`, `base64`, `tempfile`, `tokio`.

---

## File Structure

- Create `src-tauri/src/telegram_session_store.rs`
  - Owns encrypted envelope format, key generation, file reads/writes, `SavedSession` conversion, and tests.
- Modify `src-tauri/src/lib.rs`
  - Registers the new module with `mod telegram_session_store;`.
- Modify `src-tauri/src/telegram.rs`
  - Removes direct session JSON persistence helpers and calls the new module.
  - Changes session load to fallible and passes `SecretStoreState` into init flows.
  - Surfaces session storage failures as `restore_failed`.
- Modify `src-tauri/src/accounts.rs`
  - Deletes `telegram.account.<id>.session_key` during account deletion.
- Modify `src-tauri/src/secret_store.rs`
  - Adds stable secret ID helper `telegram_account_session_key_secret`.
- Modify `src-tauri/Cargo.toml`
  - Adds `chacha20poly1305` and `rand_core`.
- Modify docs after implementation:
  - `docs/backlog.md`
  - `docs/project.md`
  - `docs/design-document.md`
  - `docs/architecture-deep-dive.md`
  - `docs/database-schema.md`

---

### Task 1: Add Crypto Dependency And Session Key Secret ID

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/secret_store.rs`

- [ ] **Step 1: Add stable session key helper test first**

In `src-tauri/src/secret_store.rs`, extend `secret_ids_are_stable`:

```rust
assert_eq!(
    telegram_account_session_key_secret(42),
    "telegram.account.42.session_key"
);
```

- [ ] **Step 2: Run the focused test and verify it fails before implementation if the helper is absent**

Run:

```powershell
cargo test secret_store::tests::secret_ids_are_stable
```

Expected before implementation: compile failure mentioning `telegram_account_session_key_secret` if the helper was not added yet.

- [ ] **Step 3: Implement the helper**

Add this near `telegram_account_api_hash_secret` in `src-tauri/src/secret_store.rs`:

```rust
pub(crate) fn telegram_account_session_key_secret(account_id: i64) -> String {
    format!("telegram.account.{account_id}.session_key")
}
```

- [ ] **Step 4: Add crypto dependencies**

In `src-tauri/Cargo.toml`, add:

```toml
chacha20poly1305 = { version = "0.10", features = ["std"] }
rand_core = { version = "0.6", features = ["getrandom"] }
```

- [ ] **Step 5: Verify helper test passes**

Run:

```powershell
cargo test secret_store::tests::secret_ids_are_stable
```

Expected: `test result: ok`.

- [ ] **Step 6: Commit**

Run:

```powershell
git add src-tauri/Cargo.toml src-tauri/src/secret_store.rs
git commit -m "feat(security): add telegram session key id"
```

---

### Task 2: Create Telegram Session Store With Encrypted Round-Trip Tests

**Files:**
- Create: `src-tauri/src/telegram_session_store.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Register the module**

In `src-tauri/src/lib.rs`, add near `mod telegram;`:

```rust
mod telegram_session_store;
```

- [ ] **Step 2: Create the module skeleton**

Create `src-tauri/src/telegram_session_store.rs` with public boundaries and types:

```rust
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
```

- [ ] **Step 3: Add pure conversion and crypto helpers**

Add helpers in the same module:

```rust
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
    URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|error| AppError::internal(format!("Invalid encrypted Telegram session encoding: {error}")))
}

fn encrypt_saved_session(account_id: i64, key_bytes: &[u8], saved: &SavedSession) -> AppResult<EncryptedSessionEnvelope> {
    if key_bytes.len() != SESSION_KEY_BYTES {
        return Err(AppError::internal("Invalid Telegram session key length"));
    }
    let plaintext = serde_json::to_vec(saved).map_err(|error| AppError::internal(error.to_string()))?;
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

fn decrypt_saved_session(account_id: i64, key_bytes: &[u8], envelope: &EncryptedSessionEnvelope) -> AppResult<SavedSession> {
    if envelope.version != ENVELOPE_VERSION || envelope.algorithm != ENVELOPE_ALGORITHM {
        return Err(AppError::internal("Unsupported encrypted Telegram session format"));
    }
    if key_bytes.len() != SESSION_KEY_BYTES {
        return Err(AppError::internal("Invalid Telegram session key length"));
    }
    let nonce_bytes = decode_base64(&envelope.nonce)?;
    if nonce_bytes.len() != 24 {
        return Err(AppError::internal("Invalid encrypted Telegram session nonce length"));
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
    serde_json::from_slice::<SavedSession>(&plaintext).map_err(|error| AppError::internal(error.to_string()))
}
```

- [ ] **Step 4: Add key helpers**

Add:

```rust
fn generate_session_key() -> String {
    let mut key = [0u8; SESSION_KEY_BYTES];
    OsRng.fill_bytes(&mut key);
    encode_base64(&key)
}

async fn read_session_key(secret_store: &SecretStoreState, account_id: i64) -> AppResult<Option<Vec<u8>>> {
    let key = telegram_account_session_key_secret(account_id);
    match secret_store.get_secret(key).await? {
        Some(value) => decode_base64(&value).map(Some),
        None => Ok(None),
    }
}

async fn ensure_session_key(secret_store: &SecretStoreState, account_id: i64) -> AppResult<Vec<u8>> {
    if let Some(key) = read_session_key(secret_store, account_id).await? {
        return Ok(key);
    }
    let encoded = generate_session_key();
    secret_store
        .set_secret(telegram_account_session_key_secret(account_id), encoded.clone())
        .await?;
    decode_base64(&encoded)
}
```

- [ ] **Step 5: Add file helpers and public API**

Add:

```rust
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
    let json = serde_json::to_string(&envelope).map_err(|error| AppError::internal(error.to_string()))?;
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
```

- [ ] **Step 6: Add load helper used by tests**

Add:

```rust
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
        let key = read_session_key(secret_store, account_id).await?.ok_or_else(|| {
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

    Err(AppError::internal("Telegram session file is not a supported format"))
}
```

- [ ] **Step 7: Add tests for encrypted save and load**

In `#[cfg(test)] mod tests`, add test scaffolding:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret_store::tests::InMemorySecretStore;
    use std::sync::Arc;

    fn memory_secret_store() -> (Arc<InMemorySecretStore>, SecretStoreState) {
        let store = Arc::new(InMemorySecretStore::new());
        let state = SecretStoreState::new(store.clone());
        (store, state)
    }

    async fn sample_saved_session() -> SavedSession {
        let session = Arc::new(MemorySession::default());
        memory_session_to_saved(&session).await
    }
}
```

Add tests:

```rust
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
```

- [ ] **Step 8: Run module tests**

Run:

```powershell
cargo test telegram_session_store::
```

Expected: compile and pass after fixes.

- [ ] **Step 9: Commit**

Run:

```powershell
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src-tauri/src/telegram_session_store.rs
git commit -m "feat(security): encrypt telegram session files"
```

---

### Task 3: Cover Legacy Migration And Fail-Closed Behavior

**Files:**
- Modify: `src-tauri/src/telegram_session_store.rs`

- [ ] **Step 1: Add legacy plaintext migration test**

Add:

```rust
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
```

- [ ] **Step 2: Add migration failure test**

Add:

```rust
#[tokio::test]
async fn legacy_plaintext_session_remains_when_keyring_write_fails() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("telegram_7.session.json");
    let (store, secret_store) = memory_secret_store();
    let legacy_json = serde_json::to_string(&sample_saved_session().await).expect("legacy json");
    fs::write(&path, &legacy_json).expect("write legacy session");
    store.fail_set("secure store unavailable");

    let error = load_session_from_path(&path, &secret_store, 7)
        .await
        .expect_err("migration should fail");

    assert_eq!(error.message, "secure store unavailable");
    assert_eq!(fs::read_to_string(&path).expect("read legacy session"), legacy_json);
}
```

- [ ] **Step 3: Add missing-key fail-closed test**

Add:

```rust
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

    let error = load_session_from_path(&path, &reader_secret_store, 7)
        .await
        .expect_err("missing key should fail");

    assert!(error.message.contains("Telegram session key for account 7 is missing"));
}
```

- [ ] **Step 4: Add associated-data account binding test**

Add:

```rust
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
        .set_secret(telegram_account_session_key_secret(8), key)
        .await
        .expect("copy key to wrong account");

    let error = load_session_from_path(&path, &secret_store, 8)
        .await
        .expect_err("wrong account aad should fail");

    assert_eq!(error.message, "Failed to decrypt Telegram session");
}
```

- [ ] **Step 5: Add deletion test**

Add:

```rust
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
    assert_eq!(
        secret_store
            .get_secret(telegram_account_session_key_secret(7))
            .await
            .expect("read session key"),
        None
    );
}
```

Add this helper used by the test and by `delete_session`:

```rust
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
```

Then update `delete_session` to call `delete_session_from_path`.

- [ ] **Step 6: Run module tests**

Run:

```powershell
cargo test telegram_session_store::
```

Expected: all session-store tests pass.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src-tauri/src/telegram_session_store.rs
git commit -m "test(security): cover telegram session migration"
```

---

### Task 4: Wire Encrypted Sessions Into Telegram Runtime

**Files:**
- Modify: `src-tauri/src/telegram.rs`
- Modify: `src-tauri/src/accounts.rs`

- [ ] **Step 1: Replace direct session imports and helpers**

In `src-tauri/src/telegram.rs`, remove these imports:

```rust
use std::fs;
use std::path::PathBuf;
use grammers_session::types::{DcOption, UpdatesState};
use grammers_session::SessionData;
```

Keep `MemorySession` and `Session`:

```rust
use grammers_session::{storages::MemorySession, Session};
```

Add:

```rust
use crate::telegram_session_store;
```

Remove the local `SavedSession`, `session_path`, `session_exists`, `load_session`, and `save_session` definitions from `telegram.rs`.

- [ ] **Step 2: Make runtime clear delete encrypted session artifacts**

Change `clear_account_runtime` signature:

```rust
pub async fn clear_account_runtime(
    handle: &AppHandle,
    state: &TelegramState,
    secret_store: &SecretStoreState,
    account_id: i64,
    sign_out: bool,
) -> AppResult<()> {
```

Replace the file deletion block with:

```rust
telegram_session_store::delete_session(handle, secret_store, account_id).await?;
set_account_status(handle, state, account_id, STATUS_NOT_INITIALIZED, None).await;
Ok(())
```

Update callers that do not need to fail the command to handle the result explicitly:

```rust
let _ = clear_account_runtime(&handle, &state, &secret_store, account_id, true).await;
```

For `tg_logout`, return the error:

```rust
clear_account_runtime(&handle, &state, &secret_store, account_id, true).await?;
Ok(true)
```

- [ ] **Step 3: Pass secret store into init flow**

Change `init_account_client` signature:

```rust
async fn init_account_client(
    handle: &AppHandle,
    state: &TelegramState,
    secret_store: &SecretStoreState,
    account_id: i64,
    api_id: i32,
    api_hash: String,
) -> AppResult<bool> {
```

Replace session load:

```rust
let session = telegram_session_store::load_session(handle, secret_store, account_id).await?;
```

Update all `init_account_client` calls to pass `&secret_store`.

- [ ] **Step 4: Use session store in restore session existence check**

In `restore_telegram_accounts`, replace:

```rust
if !session_exists(&handle, account.id) {
```

with:

```rust
if !telegram_session_store::session_exists(&handle, account.id) {
```

- [ ] **Step 5: Use encrypted save after sign-in**

In `tg_sign_in`, add `secret_store` parameter:

```rust
secret_store: tauri::State<'_, SecretStoreState>,
```

Replace:

```rust
save_session(&handle, account_id, &session_to_save).await?;
```

with:

```rust
telegram_session_store::save_session(&handle, &secret_store, account_id, &session_to_save).await?;
```

The command is already registered by function name in `lib.rs`; no invoke-handler name change is needed.

- [ ] **Step 6: Update account deletion call**

In `src-tauri/src/accounts.rs`, update imports if needed and change:

```rust
clear_account_runtime(&handle, &state, account_id, true).await;
secret_store
    .delete_secret(telegram_account_api_hash_secret(account_id))
    .await
```

to:

```rust
let runtime_result =
    clear_account_runtime(&handle, &state, &secret_store, account_id, true).await;
let api_hash_result = secret_store
    .delete_secret(telegram_account_api_hash_secret(account_id))
    .await;

runtime_result?;
api_hash_result
```

This keeps database deletion first and then surfaces secure-storage cleanup errors.

- [ ] **Step 7: Add restore failure unit coverage for load failures if practical**

If the current restore path remains difficult to unit-test without a full Tauri app handle, add focused coverage to `telegram_session_store` instead and verify the runtime branch manually through code review. Do not introduce a brittle Tauri integration harness for this change.

- [ ] **Step 8: Run focused backend tests**

Run:

```powershell
cargo test telegram_session_store::
cargo test telegram::
cargo test accounts::
```

Expected: all pass.

- [ ] **Step 9: Commit**

Run:

```powershell
git add src-tauri/src/telegram.rs src-tauri/src/accounts.rs src-tauri/src/telegram_session_store.rs
git commit -m "feat(security): use encrypted telegram sessions"
```

---

### Task 5: Update Documentation And Backlog

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/project.md`
- Modify: `docs/design-document.md`
- Modify: `docs/architecture-deep-dive.md`
- Modify: `docs/database-schema.md`

- [ ] **Step 1: Update backlog secure storage state**

In `docs/backlog.md`, change the `Secret storage` target row and Phase 4.2 checklist so it states:

```markdown
| Secret storage | LLM keys, Telegram `api_hash`, and Telegram session file contents use OS-backed protection | avoid logging secrets in backend errors, frontend status text, or debug output |
```

Mark:

```markdown
- [x] decide whether and how to encrypt or migrate Telegram session JSON files
- [x] Telegram session files have an explicit long-term storage decision
```

Add a current note:

```markdown
- Telegram session files remain in app data as `telegram_<account_id>.session.json`, but contents are encrypted with per-account keys stored in OS secure storage.
```

- [ ] **Step 2: Update current-state docs**

In `docs/project.md`, `docs/design-document.md`, `docs/architecture-deep-dive.md`, and `docs/database-schema.md`, replace stale wording that says Telegram sessions remain plaintext/local JSON debt with:

```markdown
Telegram session files remain app-data files, but their contents are encrypted with per-account session keys stored in OS secure storage under `telegram.account.<account_id>.session_key`.
```

Keep separate wording that `accounts.api_hash` uses `telegram.account.<account_id>.api_hash`.

- [ ] **Step 3: Scan for stale claims**

Run:

```powershell
rg -n "session JSON|session files|plaintext|plain JSON|Telegram sessions remain|session storage decision" README.md docs src-tauri/src
```

Expected: no stale claim says Telegram session files are still plaintext or unresolved debt.

- [ ] **Step 4: Commit**

Run:

```powershell
git add docs/backlog.md docs/project.md docs/design-document.md docs/architecture-deep-dive.md docs/database-schema.md
git commit -m "docs(security): document encrypted telegram sessions"
```

---

### Task 6: Final Verification

**Files:**
- Verify all changed files.

- [ ] **Step 1: Run focused Rust tests**

Run:

```powershell
cargo test secret_store::
cargo test telegram_session_store::
cargo test telegram::
cargo test accounts::
```

Expected: all pass.

- [ ] **Step 2: Run wider backend tests affected by startup/session behavior**

Run:

```powershell
cargo test migrations::
cargo test
```

Expected: all pass.

- [ ] **Step 3: Run frontend checks**

Run:

```powershell
npm.cmd test
npm.cmd run check
```

Expected: Vitest passes and Svelte check reports `0 errors`.

- [ ] **Step 4: Run formatting and diff checks**

Run:

```powershell
cargo fmt --check
git diff --check
```

Expected: both pass.

- [ ] **Step 5: Run app startup smoke test**

Run:

```powershell
cargo run
```

Expected: app starts without migration panic. If an existing plaintext Telegram session file exists, startup should migrate it or mark only that account as `restore_failed` with a storage error.

- [ ] **Step 6: Inspect final diff**

Run:

```powershell
git status --short
git log --oneline -5
```

Expected: working tree contains only intentional uncommitted verification artifacts or is clean after commits.
