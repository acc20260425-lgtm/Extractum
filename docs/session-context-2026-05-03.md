# Session Context - 2026-05-08

This file captures enough context to resume the current Extractum session without relying on chat history.

## Environment

- Workspace: `G:\Develop\Extractum`
- Shell: PowerShell
- Current user timezone during session: `Europe/Minsk`
- User language preference in this session: Russian
- Important developer rules active in the session:
  - use `rg` for search;
  - use `apply_patch` for manual file edits;
  - do not revert user changes;
  - git index operations may require escalation because `.git/index.lock` can be blocked by sandbox permissions;
  - when using Superpowers skills, follow their gates exactly.

## Git State At Capture Time

Latest commits:

```text
4b57262 docs(security): design encrypted telegram sessions
d14774a fix(db): tolerate migration line ending checksums
d22aaf1 feat(security): store secrets in OS keyring
cd9ce01 docs(session): capture secure storage planning context
b026dc4 docs(security): plan secure secret storage
fddd817 docs: refresh provider readiness state
3fd0e63 fix(db): register source subtype migration
b8728c1 docs(sources): complete provider readiness verification
```

Working tree at the time of capture:

```text
?? docs/superpowers/plans/2026-05-08-telegram-session-storage.md
```

This file is being overwritten after that status was collected, so after saving it the working tree should also include `docs/session-context-2026-05-03.md`.

## Completed Work In This Session

Secure storage implementation was completed and committed before this context capture:

- `d22aaf1 feat(security): store secrets in OS keyring`
- `d14774a fix(db): tolerate migration line ending checksums`
- `4b57262 docs(security): design encrypted telegram sessions`

Implemented secure storage behavior:

- `src-tauri/src/secret_store.rs` was added.
- Backend uses Rust `keyring` through `SecretStoreState`.
- Service name: `org.ai.extractum`.
- LLM API keys moved out of `app_settings`.
- Telegram account `api_hash` values moved out of SQLite.
- Stable secret IDs currently implemented:
  - `llm.profile.<profile_id>.api_key`
  - `telegram.account.<account_id>.api_hash`
- Legacy plaintext values migrate lazily:
  - write to keyring first;
  - blank/delete plaintext only after secure write succeeds;
  - fail closed and leave legacy plaintext untouched if secure storage fails.
- Frontend LLM profile records expose `api_key_configured`, not the saved key.
- LLM settings password input stays empty on load.
- `clear_llm_profile_api_key` command exists.
- Account creation writes `api_hash` to keyring and stores `accounts.api_hash = ""`.
- Account deletion removes the Telegram `api_hash` secret.

Migration checksum fix:

- The Tauri SQL plugin panicked because migration 15 had been modified by line-ending changes.
- Fix commit `d14774a` added tolerance for CRLF/LF-only migration checksum differences.
- `.gitattributes` now pins SQL migrations to LF.
- `cargo run` no longer panicked on migration 15 after the fix.

Verification already completed earlier for secure storage:

```powershell
cargo test secret_store::
cargo test llm::
cargo test accounts::
cargo test telegram::
cargo test migrations::
npm.cmd test
npm.cmd run check
git diff --check
cargo run
```

## Current Open Security Tail

The remaining security item is Telegram session JSON storage.

Current code state before implementing the new plan:

- `src-tauri/src/telegram.rs` still owns session persistence directly.
- `SavedSession` contains:
  - `home_dc`
  - `dc_options`
  - `updates_state`
- Current file path helper stores sessions in Tauri app data:
  - `telegram_<account_id>.session.json`
- Current `load_session` reads plaintext JSON and falls back to `MemorySession::default()` on read/parse failure.
- Current `save_session` serializes plaintext `SavedSession` JSON.
- `clear_account_runtime` deletes the plaintext session file directly.

Relevant current functions in `src-tauri/src/telegram.rs`:

- `session_path(handle, account_id)`
- `session_exists(handle, account_id)`
- `load_session(handle, account_id) -> Arc<MemorySession>`
- `save_session(handle, account_id, session)`
- `clear_account_runtime(handle, state, account_id, sign_out)`
- `init_account_client(handle, state, account_id, api_id, api_hash)`
- `restore_telegram_accounts(handle)`
- `tg_init(...)`
- `tg_sign_in(...)`
- `tg_logout(...)`

## Superpowers Process State

Skills used in the current flow:

- `superpowers:brainstorming`
- `superpowers:writing-plans`

Brainstorming was followed:

1. Explored code/docs.
2. Proposed three approaches for Telegram session JSON:
   - recommended: encrypted app-data envelope with per-account key in OS keyring;
   - store the full session blob directly in keyring;
   - keep plaintext and document the risk.
3. User approved the recommended approach.
4. Design spec was written and committed.

Writing-plans was then used:

- Plan was written to `docs/superpowers/plans/2026-05-08-telegram-session-storage.md`.
- Plan passed a self-review for placeholders and consistency.
- User had not yet chosen execution mode when this context file was requested.
- The plan file is currently uncommitted.

If continuing the Superpowers workflow, next ask or infer execution mode:

- `Subagent-Driven` via `superpowers:subagent-driven-development` is recommended by the plan.
- `Inline Execution` via `superpowers:executing-plans` is the other option.

## Approved Telegram Session Storage Design

Design file:

- `docs/superpowers/specs/2026-05-08-telegram-session-storage-design.md`

Design commit:

- `4b57262 docs(security): design encrypted telegram sessions`

Approved target:

- Keep one session file per Telegram account.
- Keep existing file path: `telegram_<account_id>.session.json`.
- Replace plaintext contents with encrypted JSON envelope.
- Store random 256-bit per-account session key in OS secure storage.
- New secret ID:
  - `telegram.account.<account_id>.session_key`
- Generate session key on first encrypted save or legacy plaintext migration.
- Use authenticated encryption.
- Fail closed if encrypted file exists but key cannot be read.
- Migrate plaintext JSON lazily after successful parse and successful keyring write.
- Delete both session file and session key when account runtime is cleared with session deletion.
- Do not store full Telegram session JSON directly in keyring.

Encrypted file format:

```json
{
  "version": 1,
  "algorithm": "XChaCha20-Poly1305",
  "nonce": "<base64-url-no-pad nonce>",
  "ciphertext": "<base64-url-no-pad ciphertext>"
}
```

Associated data:

```text
org.ai.extractum.telegram.session.v1.account.<account_id>
```

Dependency decision:

```toml
chacha20poly1305 = { version = "0.10", features = ["std"] }
rand_core = { version = "0.6", features = ["getrandom"] }
```

## Implementation Plan Summary

Plan file:

- `docs/superpowers/plans/2026-05-08-telegram-session-storage.md`

Plan status:

- Created.
- Self-reviewed.
- Not committed yet at the time of capture.

Plan goal:

- Encrypt persisted Telegram session files while keeping the existing per-account app-data file model and lazily migrating plaintext session JSON.

Architecture:

- Add `src-tauri/src/telegram_session_store.rs`.
- Move session path handling, `SavedSession` conversion, encryption, legacy migration, and deletion into that module.
- `telegram.rs` should delegate all session file operations to the new module.
- Session load should become fallible and surface persisted-session read/decrypt failures as `restore_failed`, not silently create an empty session.

Planned tasks:

1. Add crypto dependency and stable session key secret ID.
   - Modify `src-tauri/Cargo.toml`.
   - Modify `src-tauri/src/secret_store.rs`.
   - Add `telegram_account_session_key_secret(account_id) -> "telegram.account.<id>.session_key"`.
   - Commit message in plan: `feat(security): add telegram session key id`.

2. Create Telegram session store with encrypted round-trip tests.
   - Create `src-tauri/src/telegram_session_store.rs`.
   - Modify `src-tauri/src/lib.rs` to add `mod telegram_session_store;`.
   - Implement envelope, base64-url-no-pad helpers, `XChaCha20Poly1305`, key generation, atomic write, load/save/delete APIs.
   - Commit message in plan: `feat(security): encrypt telegram session files`.

3. Cover legacy migration and fail-closed behavior.
   - Add tests for legacy plaintext migration, migration failure leaving plaintext unchanged, missing key fail-closed, wrong account associated-data failure, delete file+key.
   - Commit message in plan: `test(security): cover telegram session migration`.

4. Wire encrypted sessions into Telegram runtime.
   - Modify `src-tauri/src/telegram.rs`.
   - Modify `src-tauri/src/accounts.rs`.
   - Remove local session helpers from `telegram.rs`.
   - Pass `SecretStoreState` into session load/save/delete paths.
   - Make `clear_account_runtime` return `AppResult<()>`.
   - Update `tg_sign_in` to accept `secret_store` and save encrypted session.
   - Update account deletion to surface secure-storage cleanup errors after DB/runtime cleanup.
   - Commit message in plan: `feat(security): use encrypted telegram sessions`.

5. Update documentation and backlog.
   - Modify:
     - `docs/backlog.md`
     - `docs/project.md`
     - `docs/design-document.md`
     - `docs/architecture-deep-dive.md`
     - `docs/database-schema.md`
   - State that Telegram session files remain in app data but contents are encrypted with per-account OS-keyring-protected session keys.
   - Commit message in plan: `docs(security): document encrypted telegram sessions`.

6. Final verification.
   - Run:

```powershell
cargo test secret_store::
cargo test telegram_session_store::
cargo test telegram::
cargo test accounts::
cargo test migrations::
cargo test
npm.cmd test
npm.cmd run check
cargo fmt --check
git diff --check
cargo run
git status --short
git log --oneline -5
```

## Important Code Notes For Implementation

Existing `src-tauri/src/secret_store.rs`:

- `SecretStoreState` wraps sync keyring calls in `tauri::async_runtime::spawn_blocking`.
- Tests use `InMemorySecretStore`.
- `InMemorySecretStore` supports:
  - `fail_get(message)`
  - `fail_set(message)`
  - `fail_delete(message)`
- Reuse this for session store tests.

Current `src-tauri/src/accounts.rs` deletion flow:

```rust
delete_account_row_from_pool(&pool, account_id).await?;
clear_account_runtime(&handle, &state, account_id, true).await;
secret_store
    .delete_secret(telegram_account_api_hash_secret(account_id))
    .await
```

Planned behavior:

- Delete DB row first.
- Clear runtime and delete session file/session key.
- Delete API hash secret.
- Missing file/key should be no-op.
- Non-missing secure-storage errors should surface after DB/runtime cleanup.

Current `src-tauri/src/lib.rs`:

- Already has `mod secret_store;`.
- Already manages `SecretStoreState::system()`.
- Add `mod telegram_session_store;` near `mod telegram;`.
- No new frontend invoke handler is needed for session encryption.

Potential implementation gotchas captured during plan self-review:

- Do not use `UpdatesState::default()` in tests unless verified available. The plan uses `MemorySession::default()` and `memory_session_to_saved(&session).await` for sample data.
- Check XChaCha nonce length before `XNonce::from_slice`.
- Keep base64 as `URL_SAFE_NO_PAD`.
- The new encrypted file stays JSON but must not contain plaintext keys like `home_dc` or `updates_state`.
- `load_session_from_path` should parse encrypted envelope first, then legacy plaintext `SavedSession`.
- If legacy migration cannot write the key or encrypted file, return error and leave plaintext file unchanged.
- If encrypted file exists and key is missing, return auth/storage error; do not fall back to `MemorySession::default()`.

## User Decisions

User approved:

- Finish the Telegram session JSON security tail next.
- Use design option 1: encrypted envelope in app-data file with per-account key in OS keyring.
- Design spec is OK.

Latest user request before this file was written:

- Overwrite `docs/session-context-2026-05-03.md` with all information needed to restore the current session context.
- Form a commit message.

Suggested commit message for this context update:

```text
docs(session): capture encrypted session storage context
```
