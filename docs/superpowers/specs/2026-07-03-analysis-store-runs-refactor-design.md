# Analysis Store Runs Refactor Design

**Date:** 2026-07-03
**Status:** active spec; implementation not started as of 2026-07-03 because `src-tauri/src/analysis/store/runs.rs` does not exist.
**Scope:** internal Rust refactor of analysis run duplicate lookup, run insertion, status mutation, and saved-run deletion logic.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/store.rs` by extracting the remaining run write/status/delete logic into a focused private child module, without changing duplicate detection, run insertion fields, status updates, saved-run cleanup order, facade imports, report behavior, fixture behavior, command behavior, tests, database schema, or user-facing error messages.

This is the next conservative backend slice after the read-model, snapshot, and setup extractions. It intentionally avoids moving read-model logic, setup logic, snapshot logic, inline store tests, report lifecycle logic, or Tauri command handlers.

## Current Shape

`src-tauri/src/analysis/store.rs` currently owns:

- facade declarations and re-exports for `store/read_model.rs`;
- facade declarations and re-exports for `store/setup.rs`;
- facade declarations and re-exports for `store/snapshot.rs`;
- duplicate-run lookup and analysis run insertion;
- run status mutation and saved-run deletion;
- inline tests for store read-model, setup, snapshot, and run write behavior.

The run write/status/delete cluster currently lives directly in `store.rs`:

- `DuplicateRunLookup`
- `find_active_duplicate_run`
- `AnalysisRunInsert`
- `insert_analysis_run`
- `set_run_status`
- `delete_saved_run`

Current consumers:

- `analysis/report.rs` imports `DuplicateRunLookup`, `find_active_duplicate_run`, `AnalysisRunInsert`, `insert_analysis_run`, and `set_run_status` through `analysis::store`;
- `analysis/report/lifecycle.rs` imports `set_run_status` through `analysis::store`;
- `analysis/fixtures.rs` imports `set_run_status` through `analysis::store`;
- `analysis/mod.rs` imports `delete_saved_run` through `analysis::store` for the delete saved run command;
- `store.rs` tests call the moved items through the parent facade.

## Proposed Architecture

Create a private child module declared from `src-tauri/src/analysis/store.rs`:

- `src-tauri/src/analysis/store/runs.rs`

Keep `src-tauri/src/analysis/store.rs` as the store facade:

- add `mod runs;`;
- re-export the existing run API from `store.rs`:

```rust
pub(crate) use self::runs::{
    delete_saved_run, find_active_duplicate_run, insert_analysis_run, set_run_status,
    AnalysisRunInsert, DuplicateRunLookup,
};
```

- do not change imports in external consumers in this slice;
- keep `runs` private to `analysis::store`;
- keep store tests in `store.rs` for this slice.

Move these items from `store.rs` to `store/runs.rs`:

- `DuplicateRunLookup`
- `find_active_duplicate_run`
- `AnalysisRunInsert`
- `insert_analysis_run`
- `set_run_status`
- `delete_saved_run`

Keep these items in `store.rs` for this slice:

- `mod read_model;`, `mod setup;`, and `mod snapshot;`;
- facade re-exports for read-model, setup, snapshot, and runs;
- all current tests.

The inline test module stays in `store.rs` for this slice. Moving store tests can be a later test-only refactor.

## Visibility

`store/runs.rs` should expose only the existing run API consumed through `analysis::store`.

```rust
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
) -> AppResult<Option<i64>>;

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
) -> AppResult<i64>;

pub(crate) async fn set_run_status(
    pool: &Pool<Sqlite>,
    run_id: i64,
    status: &str,
    result_markdown: Option<&str>,
    trace_data_zstd: Option<&[u8]>,
    error: Option<&str>,
    completed_at: Option<i64>,
) -> AppResult<()>;

pub(crate) async fn delete_saved_run(pool: &Pool<Sqlite>, run_id: i64) -> AppResult<()>;
```

The fields on `DuplicateRunLookup` and `AnalysisRunInsert` must remain `pub(crate)`. They are construction contracts for `analysis/report.rs` and store tests; making the structs `pub(crate)` while making fields private would break current callers.

Private helper additions are allowed inside `runs.rs` only if they do not change the facade API, SQL behavior, errors, or visibility of the moved items.

Expected production API changes outside `analysis::store`: none.

Expected root re-export changes in `analysis/mod.rs`: none.

## Imports

`store/runs.rs` should own imports needed by run write/status/delete logic:

- `sqlx::{Pool, Sqlite}`
- `super::super::corpus::YoutubeCorpusMode`
- `super::super::models::AnalysisPromptTemplate`
- `super::super::{now_secs, ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING}`
- `crate::error::{AppError, AppResult}`

`store.rs` should remove production imports that only moved run helpers use after extraction:

- `sqlx::{Pool, Sqlite}`, if production `store.rs` only needs those for moved code;
- `YoutubeCorpusMode`, if production `store.rs` only needs it for moved structs;
- `AnalysisPromptTemplate`, if production `store.rs` only needs it for `AnalysisRunInsert`;
- `now_secs`, if production `store.rs` only needs it for `insert_analysis_run`;
- `ANALYSIS_RUN_TYPE_REPORT`, `ANALYSIS_STATUS_QUEUED`, and `ANALYSIS_STATUS_RUNNING`, if production `store.rs` only needs them for moved code;
- `AppError` and `AppResult`, if production `store.rs` only needs them for moved functions.

Test-only imports can remain inside the inline `#[cfg(test)] mod tests`. The implementation plan must include a production-import guard that checks the section of `store.rs` before `#[cfg(test)] mod tests`; moved-only imports must not remain in the production import block.

The implementation plan must verify unused imports after the move and keep the diff free of new warning debt in `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/runs.rs`.

## Data Flow

No runtime data flow changes:

1. `analysis/report.rs` still calls duplicate lookup and run insertion through the existing `store` facade before starting the report pipeline.
2. Duplicate lookup still filters on report run type, scope ids, period, output language, prompt template, provider profile, model, YouTube corpus mode, `COALESCE(telegram_history_scope, 'current')`, and queued/running status.
3. Duplicate lookup still orders by `created_at DESC` and returns at most one active run id.
4. `insert_analysis_run` still inserts report runs with the same scope fields, template id/version, provider fields, `youtube_corpus_mode.as_wire()`, `telegram_history_scope`, queued status, optional `scope_label_snapshot`, and `now_secs()` creation timestamp.
5. `set_run_status` still updates `status`, `result_markdown`, `trace_data_zstd`, `error`, and `completed_at` for the supplied run id. `result_markdown` and `trace_data_zstd` still use the current `COALESCE(?, existing_column)` behavior.
6. `analysis/report/lifecycle.rs` and `analysis/fixtures.rs` still call `set_run_status` through the same facade path.
7. `analysis/mod.rs` still calls `delete_saved_run` through the same facade path for saved run deletion.
8. `delete_saved_run` still starts one transaction, deletes `analysis_chat_messages`, deletes `analysis_run_messages`, deletes the `analysis_runs` row, returns not-found when no run row was deleted, and commits only after all deletes succeed.

## Error Handling

Preserve current error behavior exactly:

- duplicate lookup database failures still use `AppError::database`;
- insert database failures still use `AppError::database`;
- status update database failures still use `AppError::database`;
- delete transaction begin, child deletes, run delete, and commit failures still use `AppError::database`;
- deleting a missing run still returns `AppError::not_found(format!("Analysis run {run_id} not found"))`;
- no new error codes, messages, SQL filters, DTO fields, migrations, or user-facing strings are introduced.

The implementation plan must include source guards for these literals and SQL fragments:

```powershell
rg -n -F "COALESCE(telegram_history_scope, 'current') = ?" src-tauri/src/analysis/store/runs.rs
rg -n -F "ORDER BY created_at DESC" src-tauri/src/analysis/store/runs.rs
rg -n -F "youtube_corpus_mode.as_wire()" src-tauri/src/analysis/store/runs.rs
rg -n -F "ANALYSIS_STATUS_QUEUED" src-tauri/src/analysis/store/runs.rs
rg -n -F "ANALYSIS_STATUS_RUNNING" src-tauri/src/analysis/store/runs.rs
rg -n -F "UPDATE analysis_runs" src-tauri/src/analysis/store/runs.rs
rg -n -F "DELETE FROM analysis_chat_messages WHERE run_id = ?" src-tauri/src/analysis/store/runs.rs
rg -n -F "DELETE FROM analysis_run_messages WHERE run_id = ?" src-tauri/src/analysis/store/runs.rs
rg -n -F "DELETE FROM analysis_runs WHERE id = ?" src-tauri/src/analysis/store/runs.rs
rg -n -F "Analysis run {run_id} not found" src-tauri/src/analysis/store/runs.rs
```

Expected: all duplicate lookup, insert, status, and saved-run cleanup markers are present in `runs.rs` after extraction.

## Non-Goals

This slice does not:

- move read-model logic or `store/read_model.rs`;
- move setup logic or `store/setup.rs`;
- move snapshot logic or `store/snapshot.rs`;
- split store tests into files;
- change `analysis/report.rs`, `analysis/report/lifecycle.rs`, `analysis/fixtures.rs`, or `analysis/mod.rs` import paths;
- change SQL, DTO mappings, timestamp handling, status constants, transaction boundaries, database schema, migrations, frontend code, Tauri command payloads, or event payloads.

## Implementation Hygiene

The implementation plan must include a pre-edit worktree guard:

```powershell
git status --short --untracked-files=all
```

Target files must be clean before editing. If `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/runs.rs` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline commit before starting this refactor. This is required because the implementation plan should use full-file staging for the two target Rust files.

Inspect tracked target-file diffs before editing:

```powershell
git diff -- src-tauri/src/analysis/store.rs
git diff --cached -- src-tauri/src/analysis/store.rs
```

If `src-tauri/src/analysis/store/runs.rs` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/runs.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/runs.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/store/runs.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/store/runs.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/runs.rs'
}
```

Do not stage unrelated dirty files, such as local tool settings. Unrelated dirty files must remain unstaged and must be accounted for in baseline/final status comparisons.

The implementation plan must capture pre-edit status using a unique tag and persist the paths for later PowerShell sessions:

```powershell
$tag = "analysis-store-runs-refactor-" + (Get-Date -Format "yyyyMMddHHmmss")
$preEditStatusPath = Join-Path $env:TEMP "$tag-status-before.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $preEditStatusPath
$preEditStatusPath
```

Before commit, compare the final status to the captured baseline and confirm no new unintended files or diffs exist outside:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/runs.rs`

If `cargo fmt` rewrites unrelated Rust files, resolve that drift before the refactor commit by making a separate format-only commit or restoring only implementation-owned formatting changes after review. Final status should return to the captured baseline except for the intended staged refactor files.

Stage only implementation-owned files for this refactor. Do not stage local tool settings or unrelated docs.

## Source Guards

The implementation plan must include source guards after the move.

Private module declaration:

```powershell
rg -n "^mod runs;" src-tauri/src/analysis/store.rs
rg -n "^pub.*mod runs" src-tauri/src/analysis/store.rs
```

Expected: first command has one match; second command has no matches. `rg` exit code `1` is expected for no-match guards.

Facade re-export:

```powershell
rg -n "^pub\(crate\) use self::runs::" src-tauri/src/analysis/store.rs
rg -n "delete_saved_run|find_active_duplicate_run|insert_analysis_run|set_run_status|AnalysisRunInsert|DuplicateRunLookup" src-tauri/src/analysis/store.rs
```

Expected: `store.rs` has a `pub(crate) use self::runs::` facade and the six public run items are present in that facade or store tests, not as production definitions.

Moved definitions must not remain in `store.rs`:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?struct (DuplicateRunLookup|AnalysisRunInsert)\b" src-tauri/src/analysis/store.rs
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?async fn (find_active_duplicate_run|insert_analysis_run|set_run_status|delete_saved_run)\b" src-tauri/src/analysis/store.rs
```

Expected: no matches. `rg` exit code `1` is expected.

Moved definitions must exist in `runs.rs`:

```powershell
rg -n "^pub\(crate\) struct (DuplicateRunLookup|AnalysisRunInsert)\b" src-tauri/src/analysis/store/runs.rs
rg -n "^pub\(crate\) async fn (find_active_duplicate_run|insert_analysis_run|set_run_status|delete_saved_run)\b" src-tauri/src/analysis/store/runs.rs
```

Expected: all moved public run API items exist in `runs.rs`.

Field visibility must remain `pub(crate)`:

```powershell
rg -n "^\s+pub\(crate\) (scope_type|source_id|source_group_id|project_id|period_from|period_to|output_language|prompt_template_id|provider_profile|provider|model|youtube_corpus_mode|telegram_history_scope|scope_label_snapshot|prompt_template):" src-tauri/src/analysis/store/runs.rs
```

Expected: every current field on `DuplicateRunLookup` and `AnalysisRunInsert` remains constructible by existing callers.

Store tests should keep covering the facade rather than the private child module:

```powershell
rg -n "super::runs|runs::" src-tauri/src/analysis/store.rs
```

Expected: no matches. `rg` exit code `1` is expected. Inline tests should continue importing moved items through the parent `super` facade.

Production import cleanup guard:

```powershell
$beforeTests = (Get-Content -Raw src-tauri/src/analysis/store.rs).Split("#[cfg(test)]")[0]
$beforeTests | Select-String -Pattern "use sqlx::|YoutubeCorpusMode|AnalysisPromptTemplate|now_secs|ANALYSIS_RUN_TYPE_REPORT|ANALYSIS_STATUS_QUEUED|ANALYSIS_STATUS_RUNNING|AppError|AppResult"
```

Expected: no moved-only production imports remain in `store.rs`. If a future edit gives production `store.rs` a real non-moved use for one of these names, the plan must document that reason explicitly.

## Testing

Run required commands from the repository root with `--manifest-path src-tauri/Cargo.toml`. Run each command separately or through a wrapper that stops on non-zero `$LASTEXITCODE`; do not place multiple `cargo` commands in one plain PowerShell block.

Baseline before editing:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::insert_analysis_run_persists_youtube_corpus_mode
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::duplicate_lookup_matches_telegram_history_scope
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::duplicate_lookup_keeps_project_and_source_group_scopes_separate
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::delete_saved_run_returns_typed_not_found_error
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::delete_saved_run_removes_run_and_saved_children
```

Expected: every focused baseline command passes and is not a green `0 tests` run.

Post-change verification:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::insert_analysis_run_persists_youtube_corpus_mode
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::duplicate_lookup_matches_telegram_history_scope
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::duplicate_lookup_keeps_project_and_source_group_scopes_separate
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::delete_saved_run_returns_typed_not_found_error
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::delete_saved_run_removes_run_and_saved_children
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected:

- every focused store run test passes and is not a green `0 tests` run;
- `analysis::store::tests::` passes and is not a green `0 tests` run;
- `analysis::report::tests::` passes and is not a green `0 tests` run, covering report duplicate lookup, insertion, and status consumers through the facade;
- `analysis::fixtures::tests::` passes in the default dev test profile and is not a green `0 tests` run, covering the debug fixture status-update consumer;
- `cargo check --all-targets` passes, covering `analysis/mod.rs`, `analysis/report.rs`, `analysis/report/lifecycle.rs`, and other facade consumers;
- `cargo fmt -- --check` passes after any formatting fix. If formatting fixes are required, run `cargo fmt`, inspect changed files with `git status --short --untracked-files=all`, resolve unrelated drift, then run `cargo fmt -- --check` again before staging.

There is no separate runtime command-handler test required for `analysis/mod.rs` delete behavior in this slice. The behavior is covered by store-level `delete_saved_run` tests, while `cargo check --all-targets` covers the command import and facade path. This is an accepted coverage boundary for a move-only refactor.

## Commit Shape

The implementation should produce one focused refactor commit that contains only:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/runs.rs`

Documentation hardening commits may be separate, as in prior store refactor slices.

Before committing:

```powershell
git status --short --untracked-files=all
git diff -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/runs.rs
git diff --cached --check
```

Run the git commands separately or through a stopping wrapper. Do not rely on a plain multi-command PowerShell block for failure handling.
