# Typed Error Conversion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace risky string-classified backend error conversions with explicit `AppError` mappings at DB, Telegram, LLM, and validation command boundaries.

**Architecture:** Keep the existing minimal `AppError` model and frontend wire shape. Add a few shared helper constructors in `src-tauri/src/error.rs`, then migrate one backend boundary at a time so each task is testable and committable on its own.

**Tech Stack:** Rust 2021, Tauri 2 commands, SQLx SQLite, grammers Telegram client, reqwest-backed LLM providers, Cargo tests, Svelte/Vitest verification.

---

## File Structure

- Create `docs/superpowers/specs/2026-05-07-typed-error-conversion-design.md` as the active design/spec for this workstream.
- Create `docs/superpowers/plans/2026-05-07-typed-error-conversion.md` as this executable implementation plan.
- Modify `src-tauri/src/error.rs` to add explicit conversion helpers and helper tests.
- Modify `src-tauri/src/accounts.rs` for account SQL error mapping.
- Modify `src-tauri/src/analysis/templates.rs`, `src-tauri/src/analysis/groups.rs`, `src-tauri/src/analysis/mod.rs`, and `src-tauri/src/analysis/chat.rs` for command-facing validation and SQL mapping.
- Modify `src-tauri/src/telegram.rs` for runtime auth, Telegram client, DB, session path, and session persistence mapping.
- Modify `src-tauri/src/llm/mod.rs`, `src-tauri/src/llm/profiles.rs`, and `src-tauri/src/llm/runner.rs` for LLM command-boundary validation, profile, and model-listing mapping.
- Modify `docs/code-review-results-2026-05-03.md` and `docs/session-context-2026-05-03.md` when implementation is complete.

## Task 1: Persist Active Workstream Docs

**Files:**
- Create: `docs/superpowers/specs/2026-05-07-typed-error-conversion-design.md`
- Create: `docs/superpowers/plans/2026-05-07-typed-error-conversion.md`

- [ ] **Step 1: Confirm the working tree is clean**

Run:

```powershell
git status --short --branch
```

Expected: the output starts with `## main` and has no modified files.

- [ ] **Step 2: Add the design/spec document**

Create `docs/superpowers/specs/2026-05-07-typed-error-conversion-design.md` with the agreed boundary-first design:

```markdown
# Boundary-First Typed Error Conversion Design

## Summary

Tighten backend typed error conversion for the remaining DB, Telegram, LLM, and
validation paths without rewriting the whole error system.

The chosen scope is boundary-first: command/service boundaries and directly
adjacent helpers should return `AppResult<T>` with explicit `AppError`
constructors. Internal parser, compression, streaming, and provider-event paths
may keep `Result<T, String>` when the string is intentionally user-facing event
text or is not yet part of a command boundary.
```

- [ ] **Step 3: Add this implementation plan**

Create `docs/superpowers/plans/2026-05-07-typed-error-conversion.md` with this plan content.

- [ ] **Step 4: Review docs for forbidden placeholders**

Run:

```powershell
rg -n "T[B]D|TO[D]O|implement late[r]|fill in detail[s]|appropriate error handlin[g]|add validatio[n]|handle edge case[s]|Similar to Tas[k]" docs/superpowers/specs/2026-05-07-typed-error-conversion-design.md docs/superpowers/plans/2026-05-07-typed-error-conversion.md
```

Expected: no output and exit code 1.

- [ ] **Step 5: Verify markdown-only diff**

Run:

```powershell
git diff -- docs/superpowers/specs/2026-05-07-typed-error-conversion-design.md docs/superpowers/plans/2026-05-07-typed-error-conversion.md
```

Expected: only the two new docs are shown.

- [ ] **Step 6: Commit Task 1**

Run:

```powershell
git add docs/superpowers/specs/2026-05-07-typed-error-conversion-design.md docs/superpowers/plans/2026-05-07-typed-error-conversion.md
git commit -m "docs(error): add typed error conversion plan"
```

## Task 2: Add Error Helper Foundation

**Files:**
- Modify: `src-tauri/src/error.rs`
- Test: `src-tauri/src/error.rs`

- [ ] **Step 1: Add failing helper tests**

In `src-tauri/src/error.rs`, extend the existing test module imports:

```rust
use super::{classify_message, AppError, AppErrorKind};
```

Add these tests after the existing classification tests:

```rust
#[test]
fn database_helper_maps_to_internal() {
    let error = AppError::database("connection closed");

    assert_eq!(error.kind, AppErrorKind::Internal);
    assert_eq!(error.message, "Database error: connection closed");
}

#[test]
fn telegram_network_helper_maps_to_network() {
    let error = AppError::telegram_network("transport disconnected");

    assert_eq!(error.kind, AppErrorKind::Network);
    assert_eq!(error.message, "Telegram request failed: transport disconnected");
}

#[test]
fn llm_network_helper_maps_to_network() {
    let error = AppError::llm_network("timeout");

    assert_eq!(error.kind, AppErrorKind::Network);
    assert_eq!(error.message, "LLM request failed: timeout");
}
```

- [ ] **Step 2: Run the focused RED test**

Run:

```powershell
cd src-tauri
cargo test error
```

Expected: FAIL because `AppError::database`, `AppError::telegram_network`, and `AppError::llm_network` do not exist.

- [ ] **Step 3: Add explicit helper constructors**

In the `impl AppError` block in `src-tauri/src/error.rs`, after `internal`, add:

```rust
pub fn database(error: impl std::fmt::Display) -> Self {
    Self::internal(format!("Database error: {error}"))
}

pub fn telegram_network(error: impl std::fmt::Display) -> Self {
    Self::network(format!("Telegram request failed: {error}"))
}

pub fn llm_network(error: impl std::fmt::Display) -> Self {
    Self::network(format!("LLM request failed: {error}"))
}
```

- [ ] **Step 4: Run focused GREEN verification**

Run:

```powershell
cd src-tauri
cargo test error
```

Expected: PASS for the `error` tests.

- [ ] **Step 5: Commit Task 2**

Run:

```powershell
git add src-tauri/src/error.rs
git commit -m "refactor(error): add typed conversion helpers"
```

## Task 3: Type Account Database Failures

**Files:**
- Modify: `src-tauri/src/accounts.rs`

- [ ] **Step 1: Import `AppError`**

Change the error import in `src-tauri/src/accounts.rs`:

```rust
use crate::error::{AppError, AppResult};
```

- [ ] **Step 2: Convert account SQL mappings**

Replace every account command SQL conversion of this form:

```rust
.map_err(|e| e.to_string())?
```

with:

```rust
.map_err(AppError::database)?
```

Apply this in `list_accounts`, `get_account`, `create_account`,
`set_account_phone`, `clear_account_phone`, and `delete_account`.

- [ ] **Step 3: Verify no raw account SQL string mapping remains**

Run:

```powershell
rg -n "map_err\\(\\|e\\| e\\.to_string\\(\\)\\)" src-tauri/src/accounts.rs
```

Expected: no output and exit code 1.

- [ ] **Step 4: Run focused account verification**

Run:

```powershell
cd src-tauri
cargo test accounts
```

Expected: PASS or no account-specific tests matched, with compilation passing.

- [ ] **Step 5: Commit Task 3**

Run:

```powershell
git add src-tauri/src/accounts.rs
git commit -m "refactor(error): type account database failures"
```

## Task 4: Type Analysis Validation and Store Boundaries

**Files:**
- Modify: `src-tauri/src/analysis/templates.rs`
- Modify: `src-tauri/src/analysis/groups.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/chat.rs`

- [ ] **Step 1: Convert template validators to `AppResult`**

In `src-tauri/src/analysis/templates.rs`, change validator signatures:

```rust
pub(crate) fn validate_template_kind(template_kind: &str) -> AppResult<String>
```

```rust
fn validate_template_input(
    name: &str,
    template_kind: &str,
    body: &str,
) -> AppResult<(String, String, String)>
```

Use `AppError::validation(...)` for unsupported kind, empty name, and empty body:

```rust
_ => Err(AppError::validation(format!(
    "Unsupported template kind '{template_kind}'"
))),
```

```rust
return Err(AppError::validation("Template name cannot be empty"));
```

```rust
return Err(AppError::validation("Template body cannot be empty"));
```

- [ ] **Step 2: Convert template SQL mappings**

In `src-tauri/src/analysis/templates.rs`, replace command-path SQL conversions:

```rust
.map_err(|e| e.to_string())?
```

with:

```rust
.map_err(AppError::database)?
```

- [ ] **Step 3: Convert source group validator to `AppResult`**

In `src-tauri/src/analysis/groups.rs`, change:

```rust
pub(crate) fn normalize_source_group_input(
    name: &str,
    source_ids: Vec<i64>,
) -> AppResult<(String, Vec<i64>)>
```

Map empty name and empty source set with `AppError::validation(...)`.

- [ ] **Step 4: Convert source group SQL mappings**

In `src-tauri/src/analysis/groups.rs`, replace command-path SQL conversions:

```rust
.map_err(|e| e.to_string())?
```

with:

```rust
.map_err(AppError::database)?
```

Keep existing `AppError::not_found(...)` calls unchanged.

- [ ] **Step 5: Convert chat role validators**

In `src-tauri/src/analysis/mod.rs`, change:

```rust
fn validate_chat_turns(history: &[AnalysisChatTurn]) -> AppResult<()>
```

```rust
fn validate_chat_role(role: &str) -> AppResult<()>
```

Use:

```rust
return Err(AppError::validation(format!(
    "Unsupported chat turn role '{other}'"
)));
```

```rust
return Err(AppError::validation("Chat turns cannot be empty"));
```

```rust
other => Err(AppError::validation(format!(
    "Unsupported chat role '{other}'"
))),
```

- [ ] **Step 6: Convert chat command SQL mappings**

In `src-tauri/src/analysis/chat.rs`, change `load_chat_messages_from_pool` and
`persist_chat_exchange` to return `AppResult`. Replace SQL conversions in those
helpers and `clear_analysis_chat_messages` with `AppError::database`.

- [ ] **Step 7: Run focused analysis verification**

Run:

```powershell
cd src-tauri
cargo test analysis
```

Expected: PASS.

- [ ] **Step 8: Commit Task 4**

Run:

```powershell
git add src-tauri/src/analysis/templates.rs src-tauri/src/analysis/groups.rs src-tauri/src/analysis/mod.rs src-tauri/src/analysis/chat.rs
git commit -m "refactor(error): type analysis validation failures"
```

## Task 5: Type Telegram Failures

**Files:**
- Modify: `src-tauri/src/telegram.rs`

- [ ] **Step 1: Convert session path and save helpers**

Change `session_path` and `save_session` signatures:

```rust
fn session_path(handle: &AppHandle, account_id: i64) -> AppResult<PathBuf>
```

```rust
async fn save_session(
    handle: &AppHandle,
    account_id: i64,
    session: &Arc<MemorySession>,
) -> AppResult<()>
```

Map path, directory, JSON serialization, and write failures to
`AppError::internal(...)`.

- [ ] **Step 2: Convert account credential helpers**

Change:

```rust
async fn list_account_credentials(handle: &AppHandle) -> AppResult<Vec<AccountCredentials>>
```

```rust
async fn get_account_credentials(
    handle: &AppHandle,
    account_id: i64,
) -> AppResult<AccountCredentials>
```

Map SQL failures to `AppError::database(...)` and missing credentials to:

```rust
AppError::not_found(format!("Account {account_id} not found"))
```

- [ ] **Step 3: Convert Telegram API id validation**

Change:

```rust
fn telegram_api_id(api_id: i64) -> AppResult<i32>
```

Map out-of-range values to:

```rust
AppError::validation("Telegram API ID is out of range")
```

- [ ] **Step 4: Convert runtime initialization**

Change:

```rust
async fn init_account_client(
    handle: &AppHandle,
    state: &TelegramState,
    account_id: i64,
    api_id: i32,
    api_hash: String,
) -> AppResult<bool>
```

Map `client.is_authorized().await` to:

```rust
.map_err(AppError::telegram_network)?
```

- [ ] **Step 5: Convert command Telegram client calls**

In `tg_is_authenticated`, `tg_send_code`, `tg_sign_in`, and
`get_authorized_runtime`, replace Telegram client `.map_err(|e| e.to_string())?`
with:

```rust
.map_err(AppError::telegram_network)?
```

Keep existing `AppError::auth(...)` for missing runtime and login-token flows.

- [ ] **Step 6: Keep best-effort cleanup best-effort**

Leave `clear_account_runtime` swallowing sign-out and remove-file failures. It is
best-effort cleanup and should not become a user-visible command failure.

- [ ] **Step 7: Run focused Telegram verification**

Run:

```powershell
cd src-tauri
cargo test telegram
```

Expected: PASS or no Telegram-specific tests matched, with compilation passing.

- [ ] **Step 8: Commit Task 5**

Run:

```powershell
git add src-tauri/src/telegram.rs
git commit -m "refactor(error): type telegram failures"
```

## Task 6: Type LLM Command Failures

**Files:**
- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `src-tauri/src/llm/profiles.rs`
- Modify: `src-tauri/src/llm/runner.rs`

- [ ] **Step 1: Import `AppError` and `AppResult` in LLM helpers**

In `src-tauri/src/llm/profiles.rs`, add:

```rust
use crate::error::{AppError, AppResult};
```

In `src-tauri/src/llm/runner.rs`, add:

```rust
use crate::error::{AppError, AppResult};
```

- [ ] **Step 2: Convert provider and base URL validation**

In `src-tauri/src/llm/mod.rs`, change `ProviderKind::parse` and
`normalize_base_url` to return `AppResult`. Map unsupported provider, invalid
base URL, and non-http(s) scheme to `AppError::validation(...)`.

- [ ] **Step 3: Convert profile storage helpers**

In `src-tauri/src/llm/profiles.rs`, convert these helpers to `AppResult`:

```rust
fn normalize_profile_id(raw_profile_id: &str) -> AppResult<String>
async fn read_setting(pool: &Pool<Sqlite>, key: &str) -> AppResult<Option<String>>
async fn write_setting(pool: &Pool<Sqlite>, key: &str, value: &str) -> AppResult<()>
async fn list_profile_ids_from_pool(pool: &Pool<Sqlite>) -> AppResult<Vec<String>>
async fn load_profile_from_pool(pool: &Pool<Sqlite>, profile_id: &str) -> AppResult<LlmProfile>
pub(super) async fn save_profile_to_pool(...) -> AppResult<()>
pub(super) async fn load_profiles_state_from_pool(...) -> AppResult<LlmProfilesState>
pub(super) async fn resolve_profile_from_pool(...) -> AppResult<ResolvedLlmProfile>
pub(super) async fn set_active_profile_in_pool(...) -> AppResult<()>
pub(super) fn validate_profile_id(profile_id: &str) -> AppResult<String>
pub(super) fn validate_profile_input(...) -> AppResult<(String, ProviderKind, String, String)>
```

Map SQL failures to `AppError::database(...)`, invalid input to
`AppError::validation(...)`, and missing active profile to:

```rust
AppError::not_found(format!("Profile '{profile_id}' was not found"))
```

- [ ] **Step 4: Convert command-boundary request validation**

In `src-tauri/src/llm/runner.rs`, change:

```rust
pub(crate) fn validate_request(request: &LlmChatRequest) -> AppResult<()>
```

```rust
pub(crate) fn resolve_effective_model(
    profile: &ResolvedLlmProfile,
    model_override: Option<&str>,
) -> AppResult<String>
```

Map empty request id, missing messages, all-empty messages, and empty model to
`AppError::validation(...)`.

- [ ] **Step 5: Preserve streaming string errors**

Keep these signatures unchanged:

```rust
pub(crate) async fn run_llm_collect_with_profile(...) -> Result<LlmCompletion, String>
pub(crate) async fn run_llm_stream_with_profile<F>(...) -> Result<LlmCompletion, String>
```

Inside them, call `validate_request(request).map_err(String::from)?;` so
scheduler and streamed event payloads continue receiving user-facing text.

- [ ] **Step 6: Map LLM model listing failures at command boundary**

In `list_llm_provider_models`, map model-list timeout to:

```rust
Err(AppError::llm_network(format!(
    "Loading {} models timed out after {timeout_secs} seconds",
    provider_kind.display_name()
)))
```

For `Ok(models)`, convert provider errors with:

```rust
models.map_err(AppError::llm_network)
```

- [ ] **Step 7: Run focused LLM verification**

Run:

```powershell
cd src-tauri
cargo test llm
```

Expected: PASS.

- [ ] **Step 8: Commit Task 6**

Run:

```powershell
git add src-tauri/src/llm/mod.rs src-tauri/src/llm/profiles.rs src-tauri/src/llm/runner.rs
git commit -m "refactor(error): type llm command failures"
```

## Task 7: Refresh Review Docs and Session Handoff

**Files:**
- Modify: `docs/code-review-results-2026-05-03.md`
- Modify: `docs/session-context-2026-05-03.md`

- [ ] **Step 1: Update the review document**

In `docs/code-review-results-2026-05-03.md`, update the typed error finding to
say the boundary-first DB, Telegram, LLM, and validation conversion is resolved.
Keep any remaining lower-level `Result<T, String>` cleanup as a lower-priority
future hardening note rather than the first recommended follow-up.

- [ ] **Step 2: Refresh the session handoff**

In `docs/session-context-2026-05-03.md`, record:

- completed typed error conversion commits;
- verification commands and results;
- remaining recommended follow-up from the review document;
- current branch and clean/dirty state;
- that the no-worktree, one-task-per-turn workflow remains active.

- [ ] **Step 3: Run final verification**

Run:

```powershell
cd src-tauri
cargo test
cd ..
npm.cmd test
npm.cmd run check
git diff --check
```

Expected:

- `cargo test` passes;
- `npm.cmd test` passes;
- `npm.cmd run check` reports 0 errors and 0 warnings;
- `git diff --check` exits 0.

- [ ] **Step 4: Commit Task 7**

Run:

```powershell
git add docs/code-review-results-2026-05-03.md docs/session-context-2026-05-03.md
git commit -m "docs(session): refresh typed error cleanup handoff"
```

## Execution Notes

- Use the existing workflow preference: no git worktree and inline execution.
- Execute exactly one top-level task per user turn.
- Commit at the end of each top-level task.
- If Cargo or npm verification fails with sandbox-related process or file-lock
  errors, rerun the same command with approval outside the sandbox.
- Do not remove the completed report-actions plan/spec files in this workstream;
  that is a separate docs cleanup decision.
