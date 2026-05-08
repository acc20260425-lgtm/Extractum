# Secure Secret Storage Design

## Goal

Move saved LLM API keys and Telegram `api_hash` values out of SQLite-backed storage and into the operating system credential store.

This design covers only:

- LLM profile API keys.
- Telegram account `api_hash` values.

Telegram session JSON files stay as a documented follow-up. They are not part of this implementation slice.

## Decisions

- Use the Rust `keyring` crate from the Tauri backend.
- Use OS credential stores, not a plaintext SQLite fallback.
- Use service name `org.ai.extractum`.
- Use stable secret IDs:
  - `llm.profile.<profile_id>.api_key`
  - `telegram.account.<account_id>.api_hash`
- Auto-migrate existing plaintext values:
  - write the old plaintext value to keyring first;
  - delete or blank the SQLite plaintext value only after successful keyring write;
  - if keyring fails, return an error and leave the old SQLite value untouched.
- Fail closed when secure storage is unavailable.
- Keep `accounts.api_hash` in the schema as a legacy `NOT NULL` placeholder for now, but write `""` for migrated and newly created accounts.
- Keep `app_settings` as the non-secret LLM/profile settings table, but remove `llm.profile.*.api_key` rows after migration.

## Backend Shape

Add a focused backend module at `src-tauri/src/secret_store.rs`.

The module owns:

- `SecretStoreState`, registered as Tauri managed state.
- `SecretStore` trait for tests and production code.
- `SystemSecretStore`, backed by `keyring::Entry`.
- async wrappers around synchronous keyring operations using `tauri::async_runtime::spawn_blocking`.

Expected trait shape:

```rust
pub(crate) trait SecretStore: Send + Sync {
    fn get_secret(&self, key: &str) -> AppResult<Option<String>>;
    fn set_secret(&self, key: &str, value: &str) -> AppResult<()>;
    fn delete_secret(&self, key: &str) -> AppResult<()>;
}
```

`keyring::Error::NoEntry` maps to `Ok(None)`. All other keyring errors map to an `AppError` that clearly says secure storage failed.

`Cargo.toml` should add:

```toml
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }
```

The implementation must not require frontend-side Tauri plugins or permissions.

## LLM Profile Behavior

Frontend-visible profile records must no longer contain API key values.

Replace:

```ts
api_key: string
```

with:

```ts
api_key_configured: boolean
```

Save semantics:

- `apiKey: null` or empty string means keep the existing saved secret.
- non-empty `apiKey` replaces the saved secret.
- a new backend command `clear_llm_profile_api_key(profile_id)` deletes the saved secret.

Model refresh and LLM execution resolve the key on the backend:

- if a temporary unsaved key is provided, use it for that request;
- otherwise use the saved key from keyring for the selected profile;
- if no key is available, keep the existing provider-specific "API key required" errors.

Settings UI behavior:

- the password field stays empty after loading a profile;
- a configured-key indicator tells the user whether a key exists;
- saving with an empty field preserves the existing key;
- `Clear API key` removes only the LLM profile secret.

## Telegram Account Behavior

`create_account` stores `api_hash` in keyring and writes only non-secret account data to SQLite.

Recommended create flow:

1. Validate label, `api_id`, and non-empty `api_hash`.
2. Start a SQLite transaction.
3. Insert the account with `api_hash = ""` and get the new account ID.
4. Write `telegram.account.<account_id>.api_hash` to keyring.
5. Commit the transaction.
6. If the keyring write fails, roll back the transaction.
7. If commit fails after keyring write, best-effort delete the newly written secret and return the database error.

Runtime behavior:

- `restore_telegram_accounts` reads `api_hash` from keyring.
- `tg_init` reads `api_hash` from keyring.
- `tg_send_code` uses the in-memory `api_hash` loaded during init.
- `delete_account` deletes the keyring secret after removing account runtime and database rows.

Legacy migration:

- if `accounts.api_hash` is non-empty, write it to `telegram.account.<id>.api_hash`;
- only after successful keyring write, set `accounts.api_hash = ""`;
- if a migrated account has no keyring secret and no legacy plaintext value, restore/init should return a clear auth error telling the user to recreate the account credentials.

## Documentation Updates

The implementation task must update current-state docs in the same branch:

- `README.md`
- `docs/project.md`
- `docs/database-schema.md`
- `docs/design-document.md`
- `docs/architecture-deep-dive.md`
- `docs/backlog.md`

The docs should state that LLM API keys and Telegram `api_hash` values use OS secure storage. They should keep Telegram session JSON as remaining follow-up debt.

## Test Strategy

Use a mock secret store in Rust tests. Tests must not depend on a real OS credential store.

Required coverage:

- LLM save/load stores API keys outside `app_settings`.
- LLM profile output exposes `api_key_configured`, not the key.
- Empty LLM save preserves an existing secret.
- `clear_llm_profile_api_key` deletes the profile secret.
- Legacy LLM `app_settings` key migrates and is deleted only after a successful secure write.
- Legacy Telegram `accounts.api_hash` migrates and is blanked only after a successful secure write.
- Secure storage failure fails closed and keeps legacy plaintext untouched.
- Account delete removes the Telegram secret.
- Settings UI preserves existing keys and calls the clear command.

Verification commands:

```powershell
npm.cmd test
npm.cmd run check
cargo test llm::
cargo test accounts::
cargo test telegram::
cargo test migrations::
git diff --check
```

## Out Of Scope

- Encrypting or migrating Telegram session JSON.
- Adding a new user-facing secret management page.
- Introducing Tauri Stronghold.
- Adding a SQLite fallback for secrets after secure storage is introduced.
