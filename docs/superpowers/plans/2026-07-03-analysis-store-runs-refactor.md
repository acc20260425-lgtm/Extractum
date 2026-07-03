# Analysis Store Runs Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract analysis store run write/status/delete logic from `src-tauri/src/analysis/store.rs` into private `src-tauri/src/analysis/store/runs.rs` without behavior or facade changes.

**Architecture:** Keep `store.rs` as the public `analysis::store` facade and inline test owner. Move only `DuplicateRunLookup`, `find_active_duplicate_run`, `AnalysisRunInsert`, `insert_analysis_run`, `set_run_status`, and `delete_saved_run` into `store/runs.rs`, then re-export them from `store.rs` through `pub(crate) use self::runs::`. External consumers continue importing through `analysis::store`.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, Cargo tests with `--manifest-path src-tauri/Cargo.toml`, PowerShell on Windows.

## Global Constraints

- This is a move-only Rust refactor; do not change SQL, DTO mappings, timestamp handling, status constants, transaction boundaries, database schema, migrations, frontend code, Tauri command payloads, or event payloads.
- Do not change imports in external consumers: `analysis/report.rs`, `analysis/report/lifecycle.rs`, `analysis/fixtures.rs`, and `analysis/mod.rs` must keep using `analysis::store`.
- Keep `store/runs.rs` private: use `mod runs;`, not `pub mod runs;` or `pub(crate) mod runs;`.
- Keep all current inline store tests in `src-tauri/src/analysis/store.rs`.
- Keep all fields on `DuplicateRunLookup` and `AnalysisRunInsert` as `pub(crate)`.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Run each `cargo`, `git`, and guard command separately or through a wrapper that stops on non-zero `$LASTEXITCODE`; plain multi-command PowerShell blocks can hide failures.
- `rg` returns exit code `1` for expected no-match guards; treat that as success only where the step explicitly says no matches are expected.
- Target implementation files must be clean before editing. If `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/runs.rs` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline commit before starting.
- Do not stage unrelated dirty files, including `.claude/settings.local.json`.

---

## File Structure

- Modify: `src-tauri/src/analysis/store.rs`
  - Add private `mod runs;`.
  - Add the `pub(crate) use self::runs::` facade re-export for the six moved public run items.
  - Remove moved run write/status/delete definitions.
  - Remove production imports used only by the moved definitions.
  - Keep existing inline `#[cfg(test)] mod tests`.

- Create: `src-tauri/src/analysis/store/runs.rs`
  - Own duplicate-run lookup, run insertion, status update, and saved-run deletion.
  - Own imports required by those functions.
  - Expose only the existing `pub(crate)` facade API consumed through `store.rs`.

- Test: `src-tauri/src/analysis/store.rs`
  - Keep current tests where they are.
  - Tests must continue importing moved items through the parent `super` facade, not `super::runs` or `runs::`.

---

### Task 1: Extract Runs Store Module

**Files:**
- Modify: `src-tauri/src/analysis/store.rs`
- Create: `src-tauri/src/analysis/store/runs.rs`

**Interfaces:**
- Consumes:
  - `super::super::corpus::YoutubeCorpusMode`
  - `super::super::models::AnalysisPromptTemplate`
  - `super::super::{now_secs, ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING}`
  - `crate::error::{AppError, AppResult}`
  - `sqlx::{Pool, Sqlite}`
- Produces:
  - `pub(crate) struct DuplicateRunLookup<'a>`
  - `pub(crate) async fn find_active_duplicate_run(pool: &Pool<Sqlite>, lookup: &DuplicateRunLookup<'_>) -> AppResult<Option<i64>>`
  - `pub(crate) struct AnalysisRunInsert<'a>`
  - `pub(crate) async fn insert_analysis_run(pool: &Pool<Sqlite>, insert: &AnalysisRunInsert<'_>) -> AppResult<i64>`
  - `pub(crate) async fn set_run_status(pool: &Pool<Sqlite>, run_id: i64, status: &str, result_markdown: Option<&str>, trace_data_zstd: Option<&[u8]>, error: Option<&str>, completed_at: Option<i64>) -> AppResult<()>`
  - `pub(crate) async fn delete_saved_run(pool: &Pool<Sqlite>, run_id: i64) -> AppResult<()>`

- [ ] **Step 1: Capture pre-edit worktree status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected:

- `src-tauri/src/analysis/store.rs` is not modified or staged.
- `src-tauri/src/analysis/store/runs.rs` does not exist, or it is not modified/staged/untracked.
- Unrelated local files such as `.claude/settings.local.json` may exist, but must remain unstaged throughout this task.

- [ ] **Step 2: Persist a pre-edit status snapshot**

Run:

```powershell
$tag = "analysis-store-runs-refactor-" + (Get-Date -Format "yyyyMMddHHmmss")
$preEditStatusPath = Join-Path $env:TEMP "$tag-status-before.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $preEditStatusPath
$preEditStatusPath
```

Expected: PowerShell prints a temp-file path such as `C:\Users\<you>\AppData\Local\Temp\analysis-store-runs-refactor-20260703120000-status-before.txt`. Save that path in the execution notes; later status comparison uses the exact printed path.

- [ ] **Step 3: Inspect target-file baseline**

Run:

```powershell
git diff -- src-tauri/src/analysis/store.rs
```

Expected: no diff.

Run:

```powershell
git diff --cached -- src-tauri/src/analysis/store.rs
```

Expected: no staged diff.

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/runs.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/runs.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/store/runs.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/store/runs.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/runs.rs'
}
```

Expected: no output if `runs.rs` does not exist. If it exists or shows any status, stop and make a separate baseline commit before continuing.

- [ ] **Step 4: Run focused baseline tests**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::insert_analysis_run_persists_youtube_corpus_mode
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::duplicate_lookup_matches_telegram_history_scope
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::duplicate_lookup_keeps_project_and_source_group_scopes_separate
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::delete_saved_run_returns_typed_not_found_error
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::delete_saved_run_removes_run_and_saved_children
```

Expected: pass and not a green `0 tests` run.

If any baseline test fails, stop. Record the failure as pre-existing and do not edit production code in this task.

- [ ] **Step 5: Create `store/runs.rs` with the moved production code**

Create `src-tauri/src/analysis/store/runs.rs` with this content copied from the current top production block of `store.rs`:

```rust
use sqlx::{Pool, Sqlite};

use super::super::corpus::YoutubeCorpusMode;
use super::super::models::AnalysisPromptTemplate;
use super::super::{
    now_secs, ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING,
};
use crate::error::{AppError, AppResult};

pub(crate) struct DuplicateRunLookup<'a> {
    pub(crate) scope_type: &'a str,
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: &'a str,
    pub(crate) prompt_template_id: i64,
    pub(crate) provider_profile: &'a str,
    pub(crate) model: &'a str,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
    pub(crate) telegram_history_scope: &'a str,
}

pub(crate) async fn find_active_duplicate_run(
    pool: &Pool<Sqlite>,
    lookup: &DuplicateRunLookup<'_>,
) -> AppResult<Option<i64>> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM analysis_runs
        WHERE run_type = ?
          AND scope_type = ?
          AND (source_id = ? OR (source_id IS NULL AND ? IS NULL))
          AND (source_group_id = ? OR (source_group_id IS NULL AND ? IS NULL))
          AND (project_id = ? OR (project_id IS NULL AND ? IS NULL))
          AND period_from = ?
          AND period_to = ?
          AND output_language = ?
          AND prompt_template_id = ?
          AND provider_profile = ?
          AND model = ?
          AND youtube_corpus_mode = ?
          AND COALESCE(telegram_history_scope, 'current') = ?
          AND status IN (?, ?)
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(lookup.scope_type)
    .bind(lookup.source_id)
    .bind(lookup.source_id)
    .bind(lookup.source_group_id)
    .bind(lookup.source_group_id)
    .bind(lookup.project_id)
    .bind(lookup.project_id)
    .bind(lookup.period_from)
    .bind(lookup.period_to)
    .bind(lookup.output_language)
    .bind(lookup.prompt_template_id)
    .bind(lookup.provider_profile)
    .bind(lookup.model)
    .bind(lookup.youtube_corpus_mode.as_wire())
    .bind(lookup.telegram_history_scope)
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(ANALYSIS_STATUS_RUNNING)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) struct AnalysisRunInsert<'a> {
    pub(crate) scope_type: &'a str,
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: &'a str,
    pub(crate) prompt_template: &'a AnalysisPromptTemplate,
    pub(crate) provider_profile: &'a str,
    pub(crate) provider: &'a str,
    pub(crate) model: &'a str,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
    pub(crate) telegram_history_scope: &'a str,
    pub(crate) scope_label_snapshot: Option<&'a str>,
}

pub(crate) async fn insert_analysis_run(
    pool: &Pool<Sqlite>,
    insert: &AnalysisRunInsert<'_>,
) -> AppResult<i64> {
    let created_at = now_secs();
    sqlx::query_scalar(
        r#"
        INSERT INTO analysis_runs (
            run_type,
            scope_type,
            source_id,
            source_group_id,
            project_id,
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            prompt_template_version,
            provider_profile,
            provider,
            model,
            youtube_corpus_mode,
            telegram_history_scope,
            status,
            scope_label_snapshot,
            created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(insert.scope_type)
    .bind(insert.source_id)
    .bind(insert.source_group_id)
    .bind(insert.project_id)
    .bind(insert.period_from)
    .bind(insert.period_to)
    .bind(insert.output_language)
    .bind(insert.prompt_template.id)
    .bind(insert.prompt_template.version)
    .bind(insert.provider_profile)
    .bind(insert.provider)
    .bind(insert.model)
    .bind(insert.youtube_corpus_mode.as_wire())
    .bind(insert.telegram_history_scope)
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(insert.scope_label_snapshot)
    .bind(created_at)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn set_run_status(
    pool: &Pool<Sqlite>,
    run_id: i64,
    status: &str,
    result_markdown: Option<&str>,
    trace_data_zstd: Option<&[u8]>,
    error: Option<&str>,
    completed_at: Option<i64>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET
            status = ?,
            result_markdown = COALESCE(?, result_markdown),
            trace_data_zstd = COALESCE(?, trace_data_zstd),
            error = ?,
            completed_at = ?
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(result_markdown)
    .bind(trace_data_zstd)
    .bind(error)
    .bind(completed_at)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn delete_saved_run(pool: &Pool<Sqlite>, run_id: i64) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    sqlx::query("DELETE FROM analysis_chat_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    sqlx::query("DELETE FROM analysis_run_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    let deleted = sqlx::query("DELETE FROM analysis_runs WHERE id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected();

    if deleted == 0 {
        return Err(AppError::not_found(format!(
            "Analysis run {run_id} not found"
        )));
    }

    tx.commit().await.map_err(AppError::database)?;
    Ok(())
}
```

Keep this file free of test code in this slice.

- [ ] **Step 6: Update `store.rs` module declarations and facade**

At the top of `src-tauri/src/analysis/store.rs`, replace the current production imports and module block:

```rust
use sqlx::{Pool, Sqlite};

use super::corpus::YoutubeCorpusMode;
use super::models::AnalysisPromptTemplate;
use super::{now_secs, ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING};
use crate::error::{AppError, AppResult};

mod read_model;
mod setup;
mod snapshot;
```

with:

```rust
mod read_model;
mod runs;
mod setup;
mod snapshot;
```

Then add this facade re-export after the read-model re-export and before setup/snapshot re-exports:

```rust
pub(crate) use self::runs::{
    delete_saved_run, find_active_duplicate_run, insert_analysis_run, set_run_status,
    AnalysisRunInsert, DuplicateRunLookup,
};
```

Do not make the module declaration public.

- [ ] **Step 7: Remove moved definitions from `store.rs`**

In `src-tauri/src/analysis/store.rs`, delete the production block that starts at:

```rust
pub(crate) struct DuplicateRunLookup<'a> {
```

and ends after the closing brace of the `delete_saved_run` function whose signature is:

```rust
pub(crate) async fn delete_saved_run(pool: &Pool<Sqlite>, run_id: i64) -> AppResult<()> {
```

In the current code, that closing brace is immediately followed by:

```rust
#[cfg(test)]
mod tests {
```

After deletion, the existing setup/snapshot re-export blocks should remain, and the next line after the production facade section should be:

```rust
#[cfg(test)]
mod tests {
```

Do not edit the inline test bodies in this step.

- [ ] **Step 8: Run rustfmt**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command exits 0. If unrelated Rust files changed, inspect them before proceeding and resolve drift before staging.

- [ ] **Step 9: Run focused post-change store tests**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::insert_analysis_run_persists_youtube_corpus_mode
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::duplicate_lookup_matches_telegram_history_scope
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::duplicate_lookup_keeps_project_and_source_group_scopes_separate
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::delete_saved_run_returns_typed_not_found_error
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::delete_saved_run_removes_run_and_saved_children
```

Expected: pass and not a green `0 tests` run.

- [ ] **Step 10: Run source guards**

Private module declaration:

```powershell
rg -n "^mod runs;" src-tauri/src/analysis/store.rs
```

Expected: exactly one match.

```powershell
rg -n "^pub.*mod runs" src-tauri/src/analysis/store.rs
```

Expected: no matches. Exit code `1` is expected.

Facade re-export:

```powershell
rg -n "^pub\(crate\) use self::runs::" src-tauri/src/analysis/store.rs
```

Expected: one match.

```powershell
rg -n "delete_saved_run|find_active_duplicate_run|insert_analysis_run|set_run_status|AnalysisRunInsert|DuplicateRunLookup" src-tauri/src/analysis/store.rs
```

Expected: matches in the `pub(crate) use self::runs::` facade and inline tests only; no production definitions should appear.

Moved definitions absent from `store.rs`:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?struct (DuplicateRunLookup|AnalysisRunInsert)\b" src-tauri/src/analysis/store.rs
```

Expected: no matches. Exit code `1` is expected.

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?async fn (find_active_duplicate_run|insert_analysis_run|set_run_status|delete_saved_run)\b" src-tauri/src/analysis/store.rs
```

Expected: no matches. Exit code `1` is expected.

Moved definitions present in `runs.rs`:

```powershell
rg -n "^pub\(crate\) struct (DuplicateRunLookup|AnalysisRunInsert)\b" src-tauri/src/analysis/store/runs.rs
```

Expected: two matches.

```powershell
rg -n "^pub\(crate\) async fn (find_active_duplicate_run|insert_analysis_run|set_run_status|delete_saved_run)\b" src-tauri/src/analysis/store/runs.rs
```

Expected: four matches.

Field visibility:

```powershell
rg -n "^\s+pub\(crate\) (scope_type|source_id|source_group_id|project_id|period_from|period_to|output_language|prompt_template_id|provider_profile|provider|model|youtube_corpus_mode|telegram_history_scope|scope_label_snapshot|prompt_template):" src-tauri/src/analysis/store/runs.rs
```

Expected: all current fields on `DuplicateRunLookup` and `AnalysisRunInsert` are reported with `pub(crate)` visibility.

Store tests must use the facade:

```powershell
rg -n "super::runs|runs::" src-tauri/src/analysis/store.rs
```

Expected: no matches. Exit code `1` is expected.

Production import cleanup:

```powershell
$beforeTests = (Get-Content -Raw src-tauri/src/analysis/store.rs).Split("#[cfg(test)]")[0]
$beforeTests | Select-String -Pattern "use sqlx::|YoutubeCorpusMode|AnalysisPromptTemplate|now_secs|ANALYSIS_RUN_TYPE_REPORT|ANALYSIS_STATUS_QUEUED|ANALYSIS_STATUS_RUNNING|AppError|AppResult"
```

Expected: no matches. If there is a match, remove the unused production import or document the concrete non-moved production use before continuing.

Behavior markers in `runs.rs`:

```powershell
rg -n -F "COALESCE(telegram_history_scope, 'current') = ?" src-tauri/src/analysis/store/runs.rs
```

Expected: one match.

```powershell
rg -n -F "ORDER BY created_at DESC" src-tauri/src/analysis/store/runs.rs
```

Expected: one match.

```powershell
rg -n -F "youtube_corpus_mode.as_wire()" src-tauri/src/analysis/store/runs.rs
```

Expected: two matches.

```powershell
rg -n -F "ANALYSIS_STATUS_QUEUED" src-tauri/src/analysis/store/runs.rs
```

Expected: import and bind usage are present.

```powershell
rg -n -F "ANALYSIS_STATUS_RUNNING" src-tauri/src/analysis/store/runs.rs
```

Expected: import and bind usage are present.

```powershell
rg -n -F "UPDATE analysis_runs" src-tauri/src/analysis/store/runs.rs
```

Expected: one match.

```powershell
rg -n -F "DELETE FROM analysis_chat_messages WHERE run_id = ?" src-tauri/src/analysis/store/runs.rs
```

Expected: one match.

```powershell
rg -n -F "DELETE FROM analysis_run_messages WHERE run_id = ?" src-tauri/src/analysis/store/runs.rs
```

Expected: one match.

```powershell
rg -n -F "DELETE FROM analysis_runs WHERE id = ?" src-tauri/src/analysis/store/runs.rs
```

Expected: one match.

```powershell
rg -n -F "Analysis run {run_id} not found" src-tauri/src/analysis/store/runs.rs
```

Expected: one match.

- [ ] **Step 11: Run full post-change verification**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run. Do not use `--release` for this fixture slice.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This covers `analysis/mod.rs`, `analysis/report.rs`, `analysis/report/lifecycle.rs`, `analysis/fixtures.rs`, and other facade consumers.

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: pass. If it fails, run `cargo fmt --manifest-path src-tauri/Cargo.toml`, inspect `git status --short --untracked-files=all`, resolve unrelated drift, and then rerun `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.

- [ ] **Step 12: Compare final worktree to the pre-edit status snapshot**

Run, replacing `<PRE_EDIT_STATUS_PATH>` with the path printed in Step 2:

```powershell
$before = Get-Content -LiteralPath '<PRE_EDIT_STATUS_PATH>'
$afterPath = Join-Path $env:TEMP "analysis-store-runs-refactor-status-after.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $afterPath
$after = Get-Content -LiteralPath $afterPath
Compare-Object -ReferenceObject $before -DifferenceObject $after
```

Expected: differences are limited to intended changes in:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/runs.rs`

Unrelated pre-existing files such as `.claude/settings.local.json` may appear in both before and after and must not be staged.

- [ ] **Step 13: Inspect implementation diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/runs.rs
```

Expected:

- `store.rs` adds `mod runs;`.
- `store.rs` adds the `pub(crate) use self::runs::` facade.
- `store.rs` removes only the moved production run write/status/delete code and moved-only production imports.
- `store.rs` inline tests are not rewritten to use `super::runs` or `runs::`.
- `runs.rs` contains the moved code with unchanged SQL, bindings, field visibility, errors, and transaction order.

Run:

```powershell
git diff --check -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/runs.rs
```

Expected: no whitespace errors.

- [ ] **Step 14: Stage implementation files only**

Run:

```powershell
git add -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/runs.rs
```

Expected: only the two implementation files are staged.

Run:

```powershell
git diff --cached --name-status
```

Expected:

```text
M       src-tauri/src/analysis/store.rs
A       src-tauri/src/analysis/store/runs.rs
```

Run:

```powershell
git diff --cached --check
```

Expected: no whitespace errors.

- [ ] **Step 15: Commit the refactor**

Run:

```powershell
git commit -m "refactor: extract analysis store run operations"
```

Expected: commit succeeds with only:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/runs.rs`

- [ ] **Step 16: Record post-commit status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: no new implementation files remain unstaged. Pre-existing unrelated files may remain untracked, but no refactor files should be dirty.

---

## Final Verification Checklist

Before reporting the implementation complete, confirm the execution log includes:

- [ ] focused baseline store run tests passed before editing and were not green `0 tests` runs;
- [ ] focused post-change store run tests passed and were not green `0 tests` runs;
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::` passed and was not a green `0 tests` run;
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::` passed and was not a green `0 tests` run;
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::` passed in the default dev profile and was not a green `0 tests` run;
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed;
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passed after any formatting fixes;
- [ ] source guards proved `runs` is private and the moved production definitions are absent from `store.rs`;
- [ ] source guards proved moved definitions, field visibility, SQL markers, status constants, and delete error string exist in `runs.rs`;
- [ ] store tests still use the parent facade, not `super::runs`;
- [ ] production import cleanup guard found no moved-only imports in `store.rs`;
- [ ] staged files were limited to `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/runs.rs`;
- [ ] post-commit `git status --short --untracked-files=all` has no dirty refactor files.
