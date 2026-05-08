# Secure Secret Storage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Store saved LLM API keys and Telegram `api_hash` values in OS secure storage instead of SQLite.

**Architecture:** Add a backend-owned `secret_store` abstraction over the Rust `keyring` crate, register it as Tauri state, and pass it into LLM/account/Telegram flows. SQLite remains the source of truth for non-secret profile/account metadata and keeps legacy columns only for migration compatibility.

**Tech Stack:** Tauri 2, Rust, `keyring` v3, SQLite via `sqlx`, Svelte 5, Vitest.

---

## Context And Locked Decisions

- Scope is LLM API keys plus Telegram `api_hash`.
- Telegram session JSON remains a follow-up and must stay documented as remaining debt.
- Backend uses OS keyring through Rust `keyring`.
- Service name is `org.ai.extractum`.
- Secret IDs are:
  - `llm.profile.<profile_id>.api_key`
  - `telegram.account.<account_id>.api_hash`
- Existing plaintext values migrate lazily and automatically.
- Migration writes keyring first, then deletes or blanks SQLite plaintext.
- Secure storage failures fail closed; do not write secrets back to SQLite.
- Settings UI shows configured status, not saved secret values.
- LLM clear-key UI is in scope; Telegram `api_hash` is cleared only by deleting the account.
- Current-state docs are updated in the same implementation task.

## Task 1: Add Secure Store Backend

**Files:**

- Create: `src-tauri/src/secret_store.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] Add the `keyring` dependency.

Use:

```toml
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }
```

- [ ] Create `src-tauri/src/secret_store.rs`.

Required behavior:

```rust
use std::sync::Arc;

use crate::error::{AppError, AppResult};

pub(crate) const SECRET_SERVICE_NAME: &str = "org.ai.extractum";

pub(crate) fn llm_profile_api_key_secret(profile_id: &str) -> String {
    format!("llm.profile.{profile_id}.api_key")
}

pub(crate) fn telegram_account_api_hash_secret(account_id: i64) -> String {
    format!("telegram.account.{account_id}.api_hash")
}

pub(crate) trait SecretStore: Send + Sync {
    fn get_secret(&self, key: &str) -> AppResult<Option<String>>;
    fn set_secret(&self, key: &str, value: &str) -> AppResult<()>;
    fn delete_secret(&self, key: &str) -> AppResult<()>;
}

#[derive(Clone)]
pub(crate) struct SecretStoreState {
    store: Arc<dyn SecretStore>,
}
```

The public async methods on `SecretStoreState` should clone the `Arc`, move the key/value into `tauri::async_runtime::spawn_blocking`, and return `AppResult`.

- [ ] Add `SystemSecretStore`.

Use `keyring::Entry::new(SECRET_SERVICE_NAME, key)` for each operation.

Map errors as follows:

- `keyring::Error::NoEntry` from `get_secret` returns `Ok(None)`.
- `keyring::Error::NoEntry` from delete returns `Ok(())`.
- all other errors return `AppError::internal(format!("Secure storage error: {error}"))`.

- [ ] Add a test-only in-memory store.

The in-memory store needs:

- successful get/set/delete;
- configurable failing get/set/delete for migration tests;
- no OS keyring access.

- [ ] Register secure store state in `src-tauri/src/lib.rs`.

Add:

```rust
mod secret_store;
use secret_store::SecretStoreState;
```

and register:

```rust
.manage(SecretStoreState::system())
```

- [ ] Run the narrow check.

```powershell
cargo test secret_store::
```

Expected: secret store unit tests pass without touching OS credentials.

## Task 2: Move LLM API Keys To Secure Store

**Files:**

- Modify: `src-tauri/src/llm/types.rs`
- Modify: `src-tauri/src/llm/profiles.rs`
- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: existing LLM Rust tests in `src-tauri/src/llm/profiles.rs` and `src-tauri/src/llm/mod.rs`

- [ ] Change frontend-facing Rust profile type.

Replace `LlmProfile.api_key: String` with:

```rust
pub api_key_configured: bool,
```

Keep `ResolvedLlmProfile.api_key: String` unchanged because providers still need the real key on the backend.

- [ ] Update profile loading to accept `&SecretStoreState`.

`load_profile_from_pool` should:

1. normalize the profile ID;
2. read non-secret provider/model/base URL from `app_settings`;
3. migrate legacy plaintext key from `app_settings` if present and non-empty;
4. remove the legacy `app_settings` API key row after successful secure write;
5. return `api_key_configured` based on keyring lookup.

- [ ] Add helper functions for legacy key migration.

Use existing key name:

```rust
llm.profile.<profile_id>.api_key
```

Migration rule:

- if legacy row is missing or empty, do nothing;
- if legacy row exists and keyring write succeeds, delete that `app_settings` row;
- if keyring write fails, return the secure storage error and leave the row untouched.

- [ ] Update save behavior.

Change `save_llm_profile` to receive `api_key: Option<String>`.

Save rule:

- `None` or empty/whitespace: preserve existing key;
- non-empty string: write it to `llm.profile.<profile_id>.api_key`;
- never write API key values to `app_settings`.

- [ ] Add `clear_llm_profile_api_key`.

Backend command:

```rust
#[tauri::command]
pub async fn clear_llm_profile_api_key(
    handle: AppHandle,
    secret_store: tauri::State<'_, SecretStoreState>,
    profile_id: String,
) -> AppResult<LlmProfilesState>
```

Behavior:

- validate profile ID;
- delete the keyring secret;
- return current `LlmProfilesState`;
- do not delete profile metadata.

- [ ] Update model listing and backend profile resolution.

`list_llm_provider_models` should continue accepting a temporary unsaved `api_key`. If omitted or blank, resolve the saved key through `SecretStoreState`.

`resolve_profile_for_backend` should pass the secure store into profile resolution so analysis/report/chat/provider-test code keeps working without frontend seeing the secret.

- [ ] Register the new Tauri command.

Add `clear_llm_profile_api_key` to imports and `tauri::generate_handler!` in `src-tauri/src/lib.rs`.

- [ ] Update LLM tests.

Required Rust tests:

- profile settings roundtrip returns `api_key_configured = true` and does not store `llm.profile.default.api_key` in `app_settings`;
- active profile resolution loads the key from mock secure store;
- empty save preserves existing secret;
- clear command deletes the secret and returns `api_key_configured = false`;
- legacy `app_settings` key migrates and is deleted after successful secure write;
- legacy key remains in SQLite when secure write fails.

- [ ] Run LLM tests.

```powershell
cargo test llm::
```

Expected: all LLM tests pass.

## Task 3: Move Telegram `api_hash` To Secure Store

**Files:**

- Modify: `src-tauri/src/accounts.rs`
- Modify: `src-tauri/src/telegram.rs`
- Test: add focused tests in those modules

- [ ] Update account creation.

`create_account` should accept `SecretStoreState` as Tauri state and keep the same frontend command shape.

Creation flow:

1. trim and validate non-empty `api_hash`;
2. start a SQLite transaction;
3. insert account with `api_hash = ""`;
4. write `telegram.account.<new_id>.api_hash` to secure store;
5. commit transaction;
6. if keyring write fails, roll back and return the error;
7. if commit fails after keyring write, best-effort delete the new secret before returning the database error.

- [ ] Add Telegram credential migration helpers.

In `telegram.rs`, account credential loading should read `id`, `api_id`, and legacy `api_hash`.

When legacy `api_hash` is non-empty:

1. write it to `telegram.account.<id>.api_hash`;
2. set `accounts.api_hash = ""` only after successful secure write;
3. use the migrated value for the current restore/init call.

When legacy `api_hash` is empty:

1. read `telegram.account.<id>.api_hash` from secure store;
2. if missing, return `AppError::auth(format!("Telegram API hash for account {id} is missing from secure storage. Recreate the account credentials."))`.

- [ ] Wire secure store into Telegram restore and init.

Update:

- `restore_telegram_accounts`
- `tg_init`
- internal credential-loading helpers

`restore_telegram_accounts(handle)` can get the state through `handle.state::<SecretStoreState>()`.

- [ ] Update account deletion.

`delete_account` should delete `telegram.account.<account_id>.api_hash` after removing the account row and clearing runtime state. If secret deletion fails because the entry is missing, still succeed. If another secure storage error happens, return it after database deletion has completed.

- [ ] Update Telegram/account tests.

Required Rust tests:

- creating an account writes the secret and stores `accounts.api_hash = ""`;
- create account fails without inserting a row when secure store write fails;
- legacy `accounts.api_hash` migrates and blanks the database column;
- legacy plaintext remains when secure store migration fails;
- missing secure-store secret for a blank legacy account returns an auth error;
- deleting an account removes the Telegram secret.

- [ ] Run account and Telegram tests.

```powershell
cargo test accounts::
cargo test telegram::
```

Expected: all account and Telegram tests pass.

## Task 4: Update Frontend Types, API Wrappers, And Settings UI

**Files:**

- Modify: `src/lib/types/llm.ts`
- Modify: `src/lib/api/llm.ts`
- Modify: `src/lib/api/llm.test.ts`
- Modify: `src/routes/settings/+page.svelte`

- [ ] Update TypeScript LLM types.

Change:

```ts
api_key: string;
```

to:

```ts
api_key_configured: boolean;
```

Update `SaveLlmProfileInput`:

```ts
apiKey: string | null;
```

- [ ] Add clear-key API wrapper.

Add:

```ts
export function clearLlmProfileApiKey(profileId: string) {
  return invoke<LlmProfilesState>("clear_llm_profile_api_key", { profileId });
}
```

- [ ] Update Settings page state.

When applying a profile:

- set `apiKey = ""`;
- track configured state from `profile.api_key_configured`;
- keep provider/model/base URL behavior unchanged.

When saving:

- send `apiKey: apiKey.trim() ? apiKey : null`;
- after successful save, clear the local password field.

Model refresh button should be enabled when either:

- a temporary `apiKey` is typed; or
- the selected saved profile has `api_key_configured = true`.

- [ ] Add LLM clear-key UI.

Add a secondary danger-soft or secondary button near the API key field:

- label: `Clear API key`;
- disabled when no saved key is configured, saving is active, or creating a new profile;
- on click, call `clearLlmProfileApiKey(selectedProfileId)`;
- sync returned profile state;
- show status `Cleared API key for '<profile_id>'.`

- [ ] Update frontend tests.

Required Vitest coverage:

- `saveLlmProfile` sends `apiKey: null` when called that way;
- `clearLlmProfileApiKey` invokes `clear_llm_profile_api_key`;
- wrapper tests no longer expect `api_key` in profile fixtures.

- [ ] Run frontend tests.

```powershell
npm.cmd test -- src/lib/api/llm.test.ts
```

Expected: LLM API wrapper tests pass.

## Task 5: Update Current-State Documentation

**Files:**

- Modify: `README.md`
- Modify: `docs/project.md`
- Modify: `docs/database-schema.md`
- Modify: `docs/design-document.md`
- Modify: `docs/architecture-deep-dive.md`
- Modify: `docs/backlog.md`

- [ ] Update README.

Replace the statement that LLM API keys and Telegram `api_hash` still live in SQLite. State that saved LLM API keys and Telegram `api_hash` use OS secure storage. Keep Telegram session JSON as remaining follow-up debt.

- [ ] Update project docs.

In `docs/project.md`, move secure storage for LLM keys and Telegram `api_hash` from "not implemented" to implemented/current state. Keep Telegram session JSON under remaining constraints.

- [ ] Update database schema docs.

Document:

- `app_settings` stores non-secret LLM profile metadata only;
- old `llm.profile.*.api_key` rows are legacy migration inputs;
- `accounts.api_hash` is a legacy placeholder column and should be empty for migrated/new rows;
- OS keyring owns saved LLM API keys and Telegram `api_hash`.

- [ ] Update design and architecture docs.

Describe:

- backend-owned `secret_store`;
- keyring-backed secret IDs;
- LLM profile flow with `api_key_configured`;
- Telegram restore/init reading `api_hash` from secure storage;
- fail-closed migration policy.

- [ ] Update backlog.

Mark Phase 4.2 complete for:

- LLM API keys;
- Telegram `api_hash`.

Leave Telegram session JSON as open follow-up debt.

- [ ] Check docs for stale claims.

```powershell
rg -n "API keys are still stored|api_hash still|SQLite-backed credentials|app_settings still contains secrets|secure storage replaces" README.md docs
```

Expected: no stale claim says LLM API keys or Telegram `api_hash` still live in SQLite.

## Task 6: Full Verification And Commit

**Files:**

- All files changed by Tasks 1-5

- [ ] Run focused backend tests.

```powershell
cargo test llm::
cargo test accounts::
cargo test telegram::
cargo test migrations::
```

Expected: all selected Rust tests pass.

- [ ] Run frontend verification.

```powershell
npm.cmd test
npm.cmd run check
```

Expected: all Vitest tests pass and Svelte check reports 0 errors.

- [ ] Run whitespace check.

```powershell
git diff --check
```

Expected: no whitespace errors.

- [ ] Inspect the final diff.

```powershell
git status --short
git diff --stat
```

Expected: only secure secret storage implementation, tests, and docs are changed.

- [ ] Commit.

Recommended commit message:

```text
feat(security): store secrets in OS keyring
```

## Acceptance Criteria

- Saved LLM API keys are not persisted in SQLite.
- Saved Telegram `api_hash` values are not persisted in SQLite for new or migrated accounts.
- Existing plaintext secrets migrate automatically after successful secure-store write.
- Secure-store failures do not silently fall back to plaintext persistence.
- Frontend never receives saved secret values.
- `/settings` can preserve, replace, and clear LLM API keys.
- Telegram account restore/init works using secure-store `api_hash`.
- Docs match the implemented current state.
