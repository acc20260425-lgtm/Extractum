# Takeout Export-DC Fallback Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic code-backed validation for Takeout shifted export-DC fallback provenance while keeping natural live fallback evidence caveated.

**Architecture:** Keep the production Takeout flow unchanged. Add a small internal test seam in the Rust export-DC helper so tests can simulate shifted-DC local errors and Telegram RPC errors without live Telegram calls. Then add provenance/docs coverage that proves the existing durable warning path records `export_dc_fallback` safely.

**Tech Stack:** Rust, Tauri backend, grammers `InvocationError`, sqlx SQLite test fixtures, Markdown docs.

**Execution note:** Use the existing checkout and a normal git branch only if isolation is needed. Do not create a git worktree in this repository.

---

## File Structure

- Modify: `src-tauri/src/takeout_import/export_dc.rs`
  - Responsibility: export-DC aliasing, fallback decision, production grammers invocation, and deterministic fallback-unit tests.
- Modify: `src-tauri/src/takeout_import/mod.rs`
  - Responsibility: Takeout job orchestration and durable fallback provenance wrapper tests.
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
  - Responsibility: record code-backed shifted export-DC fallback validation and keep the natural-live caveat visible.
- Modify: `docs/backlog.md`
  - Responsibility: close the export-DC validation backlog row only after tests/docs are complete.
- Modify: `docs/superpowers/plans/2026-05-25-takeout-export-dc-fallback-validation.md`
  - Responsibility: track task completion checkboxes during execution.

---

### Task 1: Deterministic Export-DC Fallback Invocation Tests

**Files:**
- Modify: `src-tauri/src/takeout_import/export_dc.rs`
- Modify: `docs/superpowers/plans/2026-05-25-takeout-export-dc-fallback-validation.md`

- [x] **Step 1: Add RED tests for shifted fallback invocation**

In `src-tauri/src/takeout_import/export_dc.rs`, update the test imports at the bottom of the file.

Replace:

```rust
    use super::{
        export_dc_id_for_home_dc, should_fallback_export_dc_error,
        takeout_init_request_for_source_subtype, ExportDcAttemptState, TAKEOUT_FILE_MAX_SIZE,
    };
    use crate::sources::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};
    use grammers_mtsender::{InvocationError, RpcError};
```

With:

```rust
    use super::{
        export_dc_id_for_home_dc, export_dc_invoke_with, should_fallback_export_dc_error,
        takeout_init_request_for_source_subtype, ExportDcAlias, ExportDcAttemptState,
        TAKEOUT_FILE_MAX_SIZE,
    };
    use crate::error::AppErrorKind;
    use crate::sources::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};
    use grammers_mtsender::{InvocationError, RpcError};
    use std::sync::{Arc, Mutex};
```

Then add these tests before `export_dc_fallback_is_only_for_local_transport_errors`:

```rust
    #[tokio::test]
    async fn export_dc_invoke_falls_back_to_home_dc_on_local_error() {
        let alias = ExportDcAlias {
            home_dc_id: 2,
            export_dc_id: 40_002,
        };
        let calls = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let shifted_calls = Arc::clone(&calls);
        let home_calls = Arc::clone(&calls);
        let mut warnings = Vec::new();
        let mut fallback_used = false;

        let result = export_dc_invoke_with(
            &alias,
            &mut warnings,
            &mut fallback_used,
            || async move {
                shifted_calls
                    .lock()
                    .expect("lock shifted calls")
                    .push("shifted");
                Err::<i32, InvocationError>(InvocationError::InvalidDc)
            },
            || async move {
                home_calls.lock().expect("lock home calls").push("home");
                Ok(42_i32)
            },
        )
        .await
        .expect("fallback should use home DC");

        assert_eq!(result, 42);
        assert!(fallback_used);
        assert_eq!(*calls.lock().expect("lock calls"), vec!["shifted", "home"]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Export DC 40002 failed"));
        assert!(warnings[0].contains("falling back to home DC 2"));
    }

    #[tokio::test]
    async fn export_dc_invoke_uses_home_dc_directly_after_fallback() {
        let alias = ExportDcAlias {
            home_dc_id: 2,
            export_dc_id: 40_002,
        };
        let calls = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let shifted_calls = Arc::clone(&calls);
        let home_calls = Arc::clone(&calls);
        let mut warnings = Vec::new();
        let mut fallback_used = true;

        let result = export_dc_invoke_with(
            &alias,
            &mut warnings,
            &mut fallback_used,
            || async move {
                shifted_calls
                    .lock()
                    .expect("lock shifted calls")
                    .push("shifted");
                Err::<i32, InvocationError>(InvocationError::InvalidDc)
            },
            || async move {
                home_calls.lock().expect("lock home calls").push("home");
                Ok(7_i32)
            },
        )
        .await
        .expect("already-fallback mode should use home DC");

        assert_eq!(result, 7);
        assert!(fallback_used);
        assert!(warnings.is_empty());
        assert_eq!(*calls.lock().expect("lock calls"), vec!["home"]);
    }

    #[tokio::test]
    async fn export_dc_invoke_does_not_fallback_for_rpc_errors() {
        let alias = ExportDcAlias {
            home_dc_id: 2,
            export_dc_id: 40_002,
        };
        let calls = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let shifted_calls = Arc::clone(&calls);
        let home_calls = Arc::clone(&calls);
        let mut warnings = Vec::new();
        let mut fallback_used = false;

        let error = export_dc_invoke_with(
            &alias,
            &mut warnings,
            &mut fallback_used,
            || async move {
                shifted_calls
                    .lock()
                    .expect("lock shifted calls")
                    .push("shifted");
                Err::<i32, InvocationError>(InvocationError::Rpc(RpcError {
                    code: 400,
                    name: "TAKEOUT_INVALID".to_string(),
                    value: None,
                    caused_by: None,
                }))
            },
            || async move {
                home_calls.lock().expect("lock home calls").push("home");
                Ok(99_i32)
            },
        )
        .await
        .expect_err("RPC errors should not use export-DC fallback");

        assert_eq!(error.kind, AppErrorKind::Network);
        assert!(error.message.contains("TAKEOUT_INVALID"));
        assert!(!fallback_used);
        assert!(warnings.is_empty());
        assert_eq!(*calls.lock().expect("lock calls"), vec!["shifted"]);
    }
```

- [x] **Step 2: Run RED test command**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml export_dc_invoke_falls_back_to_home_dc_on_local_error
```

Expected: compile failure because `export_dc_invoke_with` does not exist yet. The failure should mention an unresolved import or missing function named `export_dc_invoke_with`.

- [x] **Step 3: Add the internal helper and route production through it**

In `src-tauri/src/takeout_import/export_dc.rs`, replace the first line:

```rust
use std::sync::Arc;
```

With:

```rust
use std::{future::Future, sync::Arc};
```

Then replace the body of `export_dc_invoke` with a call to the helper, and add the helper immediately below `export_dc_invoke`.

Use this complete replacement for `export_dc_invoke` and the new helper:

```rust
pub(crate) async fn export_dc_invoke<R: tl::RemoteCall>(
    client: &Client,
    alias: &ExportDcAlias,
    request: &R,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<R::Return> {
    export_dc_invoke_with(
        alias,
        warnings,
        fallback_used,
        || client.invoke_in_dc(alias.export_dc_id, request),
        || client.invoke(request),
    )
    .await
}

async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
    alias: &ExportDcAlias,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    shifted_invoke: Shifted,
    home_invoke: Home,
) -> AppResult<R>
where
    Shifted: FnOnce() -> ShiftedFuture,
    Home: FnOnce() -> HomeFuture,
    ShiftedFuture: Future<Output = Result<R, InvocationError>>,
    HomeFuture: Future<Output = Result<R, InvocationError>>,
{
    if !*fallback_used {
        match shifted_invoke().await {
            Ok(response) => return Ok(response),
            Err(error) if should_fallback_export_dc_error(&error) => {
                *fallback_used = true;
                warnings.push(format!(
                    "Export DC {} failed with local transport error; falling back to home DC {}: {error}",
                    alias.export_dc_id, alias.home_dc_id
                ));
            }
            Err(error) => return Err(AppError::network(error.to_string())),
        }
    }

    home_invoke()
        .await
        .map_err(|error| AppError::network(error.to_string()))
}
```

- [x] **Step 4: Run GREEN targeted export-DC tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml export_dc_invoke
```

Expected: all `export_dc_invoke_*` tests pass.

- [x] **Step 5: Run the existing export-DC test group**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml export_dc
```

Expected: all export-DC-related tests pass.

- [x] **Step 6: Format Rust**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: exit code 0.

- [x] **Step 7: Commit Task 1**

Run:

```powershell
git add src-tauri/src/takeout_import/export_dc.rs docs/superpowers/plans/2026-05-25-takeout-export-dc-fallback-validation.md
git commit -m "test: cover takeout export dc fallback invocation"
```

Expected: commit succeeds.

---

### Task 2: Durable Export-DC Fallback Provenance Test

**Files:**
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `docs/superpowers/plans/2026-05-25-takeout-export-dc-fallback-validation.md`

- [x] **Step 1: Add provenance imports**

In `src-tauri/src/takeout_import/mod.rs`, update the `#[cfg(test)] mod tests`
imports.

Replace the first import block inside `#[cfg(test)] mod tests` that starts with
`use super::{` and ends with `};` with:

```rust
    use super::{
        create_locked_takeout_start_records, is_channel_private_error, load_takeout_source_subtype,
        raw_parse, record_channel_private_fallback_if_supported,
        record_export_dc_attempt_if_needed, record_export_dc_fallback_if_needed,
        record_only_my_messages_fallback_if_needed, supports_only_my_messages_fallback,
        ExportDcAlias, ExportDcAttemptState, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
        TELEGRAM_KIND_SUPERGROUP,
    };
```

Replace the ingest provenance import:

```rust
    use crate::ingest_provenance::{create_telegram_takeout_batch, CreateTelegramTakeoutBatch};
```

With:

```rust
    use crate::ingest_provenance::{
        create_telegram_takeout_batch, finalize_ingest_batch, CreateTelegramTakeoutBatch,
        TerminalBatchStatus,
    };
```

- [x] **Step 2: Add the provenance characterization test**

Add this test after `channel_private_count_probe_records_fallback_before_search_continuation`:

```rust
    #[tokio::test]
    async fn export_dc_fallback_provenance_records_once_before_finalize() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_item_source(&pool, 1).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: TELEGRAM_KIND_SUPERGROUP.to_string(),
            },
        )
        .await
        .expect("create takeout batch");
        let alias = ExportDcAlias {
            home_dc_id: 2,
            export_dc_id: 40_002,
        };
        let mut attempts = ExportDcAttemptState::new();
        let mut warnings = vec![
            "Export DC 40002 failed with local transport error; falling back to home DC 2: invalid DC"
                .to_string(),
        ];

        record_export_dc_attempt_if_needed(&pool, batch_id, &alias, &mut attempts)
            .await
            .expect("record export DC attempt");
        record_export_dc_fallback_if_needed(
            &pool,
            batch_id,
            &warnings,
            false,
            true,
            &mut attempts,
        )
        .await
        .expect("record first export DC fallback");
        warnings.push("second fallback detail should not create a second warning".to_string());
        record_export_dc_fallback_if_needed(
            &pool,
            batch_id,
            &warnings,
            false,
            true,
            &mut attempts,
        )
        .await
        .expect("record duplicate fallback idempotently");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize batch");

        let state: (Option<i64>, i64, i64, i64) = sqlx::query_as(
            "SELECT t.export_dc_id, t.used_export_dc, t.fallback_used, b.warning_count
             FROM telegram_takeout_batches t
             JOIN ingest_batches b ON b.id = t.batch_id
             WHERE t.batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load export DC provenance");
        assert_eq!(state, (Some(40_002), 1, 1, 1));

        let warning_codes: Vec<String> =
            sqlx::query_scalar("SELECT code FROM ingest_batch_warnings WHERE batch_id = ?")
                .bind(batch_id)
                .fetch_all(&pool)
                .await
                .expect("load warning codes");
        assert_eq!(warning_codes, vec!["export_dc_fallback".to_string()]);
    }
```

This is a characterization test for existing provenance behavior. It may pass immediately. If it fails, fix only the fallback provenance guard or warning insertion path needed to satisfy the test.

- [x] **Step 3: Run targeted provenance test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml export_dc_fallback_provenance_records_once_before_finalize
```

Expected: pass. If it fails because fallback warning count is greater than one, inspect `record_export_dc_fallback_if_needed` and `ExportDcAttemptState::mark_fallback` before changing database helpers.

- [x] **Step 4: Run validation diagnostics fallback summary test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_validation_batch_summary_is_durable_and_sanitized
```

Expected: pass. This confirms validation diagnostics continue exposing `used_export_dc`, `fallback_used`, and distinct warning code `export_dc_fallback` without exposing warning bodies.

- [x] **Step 5: Format Rust**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: exit code 0.

- [x] **Step 6: Commit Task 2**

Run:

```powershell
git add src-tauri/src/takeout_import/mod.rs docs/superpowers/plans/2026-05-25-takeout-export-dc-fallback-validation.md
git commit -m "test: cover takeout export dc fallback provenance"
```

Expected: commit succeeds.

---

### Task 3: Record Code-Backed Validation In Docs

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Modify: `docs/superpowers/plans/2026-05-25-takeout-export-dc-fallback-validation.md`

- [x] **Step 1: Update validation matrix summary**

In `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`, replace:

```markdown
Current matrix summary: `8 passed`, `1 needs follow-up`, `1 blocked`,
`0 not run`.
```

With:

```markdown
Current matrix summary: `9 passed`, `1 needs follow-up`, `0 blocked`,
`0 not run`.
```

- [x] **Step 2: Update covered highlights**

In the same file, add this bullet after the `source 115` migrated smoke bullet:

```markdown
- deterministic code-backed shifted export-DC fallback coverage proving local
  fallback transition, non-fallback Telegram RPC behavior, durable
  `export_dc_fallback` warning provenance, and sanitized diagnostics; natural
  live fallback remains unobserved in the current environment;
```

- [x] **Step 3: Update shifted export-DC matrix row**

Replace the matrix row:

```markdown
| Shifted export DC fallback | blocked |  |  | export DC attempted/fallback flags, `export_dc_fallback` warning code, typed/coarse terminal outcome if present | Requires an environment that naturally triggers local transport/session fallback |
```

With:

```markdown
| Shifted export DC fallback | passed | n/a | n/a | deterministic Rust fallback/provenance tests, validation diagnostic warning-code coverage | Code-backed validation proves shifted export-DC local-error fallback, non-fallback Telegram RPC behavior, one durable `export_dc_fallback` warning, and sanitized diagnostics; natural live fallback remains unobserved |
```

- [x] **Step 4: Add dated validation note**

Add this note directly below `## Run Notes` and before the existing dated notes:

```markdown
### 2026-05-25 Shifted Export DC Fallback Code-Backed Validation

This note records deterministic code-backed validation, not natural live
Telegram fallback evidence.

Evidence:

- `cargo test --manifest-path src-tauri/Cargo.toml export_dc_invoke` passed
  after adding fake-invoker coverage for shifted local-error fallback,
  direct-home behavior after fallback, and Telegram RPC non-fallback.
- `cargo test --manifest-path src-tauri/Cargo.toml export_dc_fallback_provenance_records_once_before_finalize`
  passed, proving `export_dc_id`, `used_export_dc = 1`, `fallback_used = 1`,
  and exactly one durable `export_dc_fallback` warning before batch
  finalization.
- `cargo test --manifest-path src-tauri/Cargo.toml takeout_validation_batch_summary_is_durable_and_sanitized`
  passed, confirming validation diagnostics expose fallback flags and warning
  codes without warning bodies or private source data.

Caveat: no local live run naturally triggered shifted export-DC fallback in
Telegram transport. Future live evidence can strengthen this row without
reopening the local warning/provenance mechanics.
```

- [x] **Step 5: Update backlog priority snapshot**

In `docs/backlog.md`, replace:

```markdown
| High | Takeout source import | complete remaining export-DC validation and decide migrated-history import policy on top of persisted provenance |
```

With:

```markdown
| High | Takeout source import | decide migrated-history import policy on top of persisted provenance |
```

- [x] **Step 6: Mark the backlog export-DC validation item complete**

In `docs/backlog.md`, replace:

```markdown
- [ ] validate shifted export DC behavior and the warning path when fallback to home DC is used
```

With:

```markdown
- [x] validate shifted export DC behavior and the warning path when fallback to home DC is used
  - Code-backed validation proves local shifted export-DC fallback, Telegram RPC
    non-fallback, one durable `export_dc_fallback` warning, and sanitized
    diagnostics. Natural live fallback remains unobserved in the current
    environment.
```

- [x] **Step 7: Run docs grep sanity check**

Run:

```powershell
rg -n "Shifted export DC fallback|export_dc_fallback|complete remaining export-DC validation|natural live fallback" docs\backlog.md docs\superpowers\verification\takeout-representative-validation-and-fallback-coverage.md
```

Expected:

- the validation row status is `passed`;
- the backlog export-DC item is checked;
- `complete remaining export-DC validation` is absent;
- the natural-live caveat is visible in the verification note or row.

- [x] **Step 8: Commit Task 3**

Run:

```powershell
git add docs/backlog.md docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md docs/superpowers/plans/2026-05-25-takeout-export-dc-fallback-validation.md
git commit -m "docs: record takeout export dc fallback validation"
```

Expected: commit succeeds.

---

### Task 4: Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-05-25-takeout-export-dc-fallback-validation.md`

- [x] **Step 1: Run targeted Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml export_dc
```

Expected: exit code 0 and all export-DC-related tests pass.

- [x] **Step 2: Run Takeout validation diagnostics test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_validation_batch_summary_is_durable_and_sanitized
```

Expected: exit code 0.

- [x] **Step 3: Run cargo check**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: exit code 0.

- [x] **Step 4: Run full Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: exit code 0.

- [x] **Step 5: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected: exit code 0. Windows LF/CRLF warnings from Git are acceptable if there are no whitespace errors.

- [x] **Step 6: Check working tree**

Run:

```powershell
git status --short --branch
```

Expected: on the implementation branch or `main`, with no unstaged changes except this plan file if its completion checkboxes are not yet committed.

- [x] **Step 7: Commit completed plan checkboxes**

After every task checkbox above has been updated to `[x]`, run:

```powershell
git add docs/superpowers/plans/2026-05-25-takeout-export-dc-fallback-validation.md
git commit -m "docs: mark takeout export dc fallback validation plan complete"
```

Expected: commit succeeds.

---

## Acceptance Checklist

- [x] `export_dc_invoke_with` or equivalent internal helper has deterministic tests for local-error fallback, direct-home mode after fallback, and RPC non-fallback.
- [x] Production `export_dc_invoke` still uses the same grammers client calls and warning text.
- [x] Durable fallback provenance test proves `export_dc_id`, `used_export_dc`, `fallback_used`, and one `export_dc_fallback` warning before finalization.
- [x] Validation diagnostics warning-code coverage remains sanitized.
- [x] Verification matrix row is `passed` with a natural-live-fallback caveat.
- [x] Backlog export-DC validation item is checked and migrated-history policy remains the next Takeout priority.
- [x] `cargo check --manifest-path src-tauri/Cargo.toml` passes.
- [x] `cargo test --manifest-path src-tauri/Cargo.toml` passes.
- [x] `git diff --check` is clean.
