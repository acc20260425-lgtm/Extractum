# Telegram Session Storage Design

## Goal

Protect persisted Telegram session data that is currently stored as plaintext app-data JSON files named `telegram_<account_id>.session.json`.

The goal is to keep the existing session-file model, but encrypt file contents with account-scoped keys stored in the operating system credential store. Existing plaintext session files should migrate lazily without forcing users to sign in again.

## Current State

`src-tauri/src/telegram.rs` persists `SavedSession` as JSON in the Tauri app data directory.

The saved shape contains:

- `home_dc`
- `dc_options`
- `updates_state`

The app already stores LLM API keys and Telegram `api_hash` values through `src-tauri/src/secret_store.rs`, backed by the Rust `keyring` crate and service name `org.ai.extractum`.

## Decisions

- Keep one session file per Telegram account.
- Keep the existing file path: `telegram_<account_id>.session.json`.
- Replace plaintext file contents with an encrypted JSON envelope.
- Store a random 256-bit account session key in OS secure storage.
- Use secret ID `telegram.account.<account_id>.session_key`.
- Generate the account session key on first encrypted save or legacy migration.
- Use authenticated encryption, not unauthenticated encryption.
- Fail closed if an encrypted session file exists but its key cannot be read.
- Migrate legacy plaintext JSON lazily after a successful parse and successful keyring write.
- Delete both the session file and session key when account runtime is cleared with session deletion.
- Do not store Telegram session JSON directly in keyring.

## Encryption Format

The encrypted file remains JSON so it is easy to identify and inspect structurally without exposing the secret data.

```json
{
  "version": 1,
  "algorithm": "XChaCha20-Poly1305",
  "nonce": "<base64-url-no-pad nonce>",
  "ciphertext": "<base64-url-no-pad ciphertext>"
}
```

The plaintext before encryption is the existing serialized `SavedSession` JSON. The encryption key is a 32-byte random key stored as base64-url-no-pad text in keyring.

Associated data should bind the ciphertext to this app, account, and envelope version:

```text
org.ai.extractum.telegram.session.v1.account.<account_id>
```

This prevents an encrypted session file for one account from being silently accepted for another account.

## Backend Shape

Add a small session storage boundary near Telegram runtime code instead of spreading encryption logic through login and restore flows.

Recommended module:

```text
src-tauri/src/telegram_session_store.rs
```

Responsibilities:

- derive the existing session path;
- load encrypted session envelopes;
- detect and parse legacy plaintext `SavedSession`;
- convert between `SavedSession` and `MemorySession`;
- encrypt and write sessions;
- lazily migrate plaintext files after successful load;
- delete session files and session keys together.

`telegram.rs` should call this boundary instead of reading and writing session JSON directly.

## Load Behavior

Loading should become fallible:

```rust
async fn load_session(...) -> AppResult<Arc<MemorySession>>
```

Behavior:

1. If no session file exists, return a default `MemorySession`.
2. If the file parses as encrypted envelope, read the account session key from keyring and decrypt it.
3. If keyring returns no key for an encrypted file, return a clear auth/storage error and do not fall back to an empty session.
4. If the file parses as legacy plaintext `SavedSession`, create or read the account session key, encrypt the same session data, atomically replace the file, and return the loaded session.
5. If legacy migration cannot write the key or encrypted file, return an error and leave the plaintext file untouched.
6. If the file is neither a valid envelope nor a valid legacy session, return a restore error.

Restore errors should set the account runtime status to `restore_failed` and emit the existing restore-failure event. They should not silently create an empty unauthorized session when persisted session data exists but cannot be read safely.

## Save Behavior

Saving should always write the encrypted envelope.

Behavior:

1. Convert `MemorySession` into `SavedSession`.
2. Read the existing account session key.
3. If no key exists, generate a random 32-byte key and write it to keyring before writing the session file.
4. Encrypt the serialized `SavedSession` JSON with a fresh nonce.
5. Write to a temporary file in the same directory.
6. Atomically replace the session file.

If the keyring write succeeds but the file write fails for a newly generated key, the implementation may leave the unused key in keyring. A later save can reuse it. Account deletion still removes it.

## Delete Behavior

Any flow that removes account session state should delete:

- `telegram_<account_id>.session.json`
- `telegram.account.<account_id>.session_key`

Missing file and missing key are successful no-ops. Other secure-storage errors should be surfaced by account deletion after local database/runtime cleanup has run.

## Dependency Choice

Use RustCrypto crates:

```toml
chacha20poly1305 = { version = "0.10", features = ["std"] }
rand_core = { version = "0.6", features = ["getrandom"] }
```

`XChaCha20Poly1305` avoids nonce-size footguns and is a good fit for randomly generated nonces. The code should use `OsRng` for keys and nonces.

## Tests

Rust tests should use the existing in-memory `SecretStoreState` test helper and temporary directories. Tests must not require a real OS keyring.

Required coverage:

- saving a session writes an encrypted envelope, not plaintext `SavedSession` JSON;
- encrypted session load round-trips into `MemorySession`;
- legacy plaintext session load migrates the file to encrypted envelope only after keyring write succeeds;
- legacy plaintext remains unchanged when keyring write fails;
- encrypted session load fails closed when the session key is missing;
- encrypted session load fails when associated data uses the wrong account ID;
- session deletion removes both the file and `telegram.account.<id>.session_key`;
- restore paths surface session storage failures as `restore_failed` instead of silently replacing the session with default state.

## Documentation Updates

Update current-state docs and backlog after implementation:

- `docs/backlog.md`
- `docs/project.md`
- `docs/design-document.md`
- `docs/architecture-deep-dive.md`
- `docs/database-schema.md`

The docs should state that Telegram session files remain app-data files, but their contents are encrypted with OS-keyring-protected account keys.

## Out Of Scope

- Replacing `grammers_session::MemorySession` with a database-backed session store.
- Moving full session blobs into keyring.
- Adding Tauri Stronghold.
- Adding a user-facing session management UI.
- Migrating or renaming the existing session file path.
