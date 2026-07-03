# Analysis Store Tests Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** implemented historical execution record. Implemented by `79e065e2 refactor: split analysis store tests` and recorded complete by `125f76f1 docs: complete analysis store tests plan`.

**Goal:** Move the inline `#[cfg(test)] mod tests` body out of `src-tauri/src/analysis/store.rs` into focused nested store test modules without changing production behavior or test assertions.

**Architecture:** Keep `store.rs` as the production facade for `read_model`, `runs`, `setup`, and `snapshot`, with only `#[cfg(test)] mod tests;` for tests. Create `src-tauri/src/analysis/store/tests/` with thematic modules for read-model, setup, snapshot, and runs behavior. Keep tests exercising the parent store facade through `super::super` imports rather than private production child modules.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, Cargo tests with `--manifest-path src-tauri/Cargo.toml`, PowerShell on Windows.

## Global Constraints

- This is a test-only Rust refactor; do not change production behavior, facade re-exports, SQL, fixtures, assertions, test coverage, or external consumer paths.
- Do not move or edit production code in `store/read_model.rs`, `store/runs.rs`, `store/setup.rs`, or `store/snapshot.rs`.
- Keep production `store.rs` module declarations and facade re-exports unchanged.
- Keep production visibility unchanged; do not widen production item visibility for this test split.
- Shared test helpers may use `pub(super)` only; do not use `pub(crate)` or `pub` in `src-tauri/src/analysis/store/tests/`.
- Tests must exercise the parent store facade, not private child modules such as `store::runs` or `store::snapshot`.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Run each `cargo`, `git`, and guard command separately or through a wrapper that stops on non-zero `$LASTEXITCODE`; plain multi-command PowerShell blocks can hide failures.
- `rg` returns exit code `1` for expected no-match guards; treat that as success only where the step explicitly says no matches are expected.
- Target files must be clean before editing. If `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/tests/` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline commit before starting.
- Do not stage unrelated dirty files, including `.claude/settings.local.json`.

---

## File Structure

- Modify: `src-tauri/src/analysis/store.rs`
  - Keep production module declarations and re-exports unchanged.
  - Replace the inline test module body with `#[cfg(test)] mod tests;`.

- Create: `src-tauri/src/analysis/store/tests/mod.rs`
  - Declare thematic test modules.

- Create: `src-tauri/src/analysis/store/tests/harness.rs`
  - Keep as the shared-helper location.
  - It may be empty except for comments if the implementation keeps all helpers module-local.

- Create: `src-tauri/src/analysis/store/tests/read_model.rs`
  - Own run-list tests, read-model mapping tests, and their private fixtures.

- Create: `src-tauri/src/analysis/store/tests/setup.rs`
  - Own setup error tests and their private SQLite pool helpers.

- Create: `src-tauri/src/analysis/store/tests/snapshot.rs`
  - Own snapshot sanitization, capture, validation, and capture-failure tests.

- Create: `src-tauri/src/analysis/store/tests/runs.rs`
  - Own run insertion, duplicate lookup, status update, and saved-run deletion tests.

---

### Task 1: Split Store Tests Into Nested Modules

**Files:**
- Modify: `src-tauri/src/analysis/store.rs`
- Create: `src-tauri/src/analysis/store/tests/mod.rs`
- Create: `src-tauri/src/analysis/store/tests/harness.rs`
- Create: `src-tauri/src/analysis/store/tests/read_model.rs`
- Create: `src-tauri/src/analysis/store/tests/setup.rs`
- Create: `src-tauri/src/analysis/store/tests/snapshot.rs`
- Create: `src-tauri/src/analysis/store/tests/runs.rs`

**Interfaces:**
- Consumes:
  - Parent store facade imports through explicit `use super::super::{name_one, name_two};` lists inside thematic test modules.
  - Model imports from `crate::analysis::models::{AnalysisPromptTemplate, AnalysisRunDetail, AnalysisRunRow, CorpusMessage}` where needed.
  - `crate::error::AppErrorKind` where tests assert typed errors.
- Produces:
  - `#[cfg(test)] mod tests;` in `src-tauri/src/analysis/store.rs`.
  - `mod harness; mod read_model; mod runs; mod setup; mod snapshot;` in `tests/mod.rs`.
  - Same test functions under new paths such as `analysis::store::tests::read_model::list_analysis_run_summaries_applies_query_before_limit`.

- [x] **Step 1: Capture pre-edit worktree status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected:

- `src-tauri/src/analysis/store.rs` is not modified or staged.
- `src-tauri/src/analysis/store/tests/` does not exist, or it is not modified/staged/untracked.
- Unrelated local files such as `.claude/settings.local.json` may exist, but must remain unstaged throughout this task.

- [x] **Step 2: Persist a pre-edit status snapshot**

Run:

```powershell
$tag = "analysis-store-tests-refactor-" + (Get-Date -Format "yyyyMMddHHmmss")
$preEditStatusPath = Join-Path $env:TEMP "$tag-status-before.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $preEditStatusPath
$preEditStatusPath
```

Expected: PowerShell prints a temp-file path such as `C:\Users\<you>\AppData\Local\Temp\analysis-store-tests-refactor-20260703120000-status-before.txt`. Save that path in the execution notes; later status comparison uses the exact printed path.

- [x] **Step 3: Inspect target-file baseline**

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
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/tests') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/tests
    Get-ChildItem -Recurse -Force -LiteralPath 'src-tauri/src/analysis/store/tests'
}
```

Expected: no output if the directory does not exist. If it exists or shows any status, stop and make a separate baseline commit before continuing.

- [x] **Step 4: Run baseline tests**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::list_analysis_run_summaries_applies_query_before_limit
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::capture_run_snapshot_marks_captured_after_reload_and_replaces_rows
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::insert_analysis_run_persists_youtube_corpus_mode
```

Expected: pass and not a green `0 tests` run.

If any baseline test fails, stop. Record the failure as pre-existing and do not edit production code in this task.

- [x] **Step 5: Create test module directory and module root**

Create `src-tauri/src/analysis/store/tests/mod.rs`:

```rust
mod harness;
mod read_model;
mod runs;
mod setup;
mod snapshot;
```

Create `src-tauri/src/analysis/store/tests/harness.rs`:

```rust
// Shared store test helpers live here when more than one thematic module needs them.
// Keep single-module helpers private in their thematic module.
```

Do not add `pub(crate)` or `pub` helpers.

- [x] **Step 6: Move read-model tests into `read_model.rs`**

Create `src-tauri/src/analysis/store/tests/read_model.rs`.

Move these items from the current inline test module in `store.rs` into `read_model.rs`:

- `sample_run_row`
- `sample_run`
- `RunListFixture`
- `impl RunListFixture`
- `run_list_pool`
- `insert_run_list_fixture`
- `list_analysis_run_summaries_applies_query_before_limit`
- `list_analysis_run_summaries_combines_scope_and_field_filters`
- `list_analysis_run_summaries_filters_source_groups_and_template_names`
- `list_analysis_run_summaries_filters_project_runs`
- `list_analysis_run_summaries_rejects_both_scope_ids`
- `list_analysis_run_summaries_filters_status_and_dates`
- `list_analysis_run_summaries_escapes_literal_like_characters`
- `list_analysis_run_summaries_matches_all_query_terms_across_any_field`
- `resolve_run_scope_label_prefers_frozen_value`
- `map_run_summary_exposes_frozen_scope_label`
- `map_run_summary_exposes_captured_snapshot_state`
- `completed_run_without_capture_marker_is_capture_failed`
- `map_run_summary_exposes_capture_failed_snapshot_state`
- `map_run_summary_exposes_null_snapshot_state_for_active_runs_before_capture`
- `failed_terminal_run_without_capture_marker_is_capture_failed`
- `map_run_summary_exposes_youtube_corpus_mode`
- `map_run_detail_exposes_youtube_corpus_mode`

At the top of `read_model.rs`, add imports for the parent facade and model types:

```rust
use super::super::{
    list_analysis_run_summaries, map_run_detail, map_run_summary, resolve_run_scope_label,
    AnalysisRunListFilters,
};
use crate::analysis::models::{AnalysisRunDetail, AnalysisRunRow};
use crate::error::AppErrorKind;
```

Keep helper visibility private in this file.

- [x] **Step 7: Move setup tests into `setup.rs`**

Create `src-tauri/src/analysis/store/tests/setup.rs`.

Move these items from the current inline test module in `store.rs` into `setup.rs`:

- `template_store_pool`
- `source_store_pool`
- `ensure_sources_exist_returns_typed_not_found_error`
- `fetch_prompt_template_returns_typed_not_found_error`

At the top of `setup.rs`, add:

```rust
use super::super::{ensure_sources_exist, fetch_prompt_template};
use crate::error::AppErrorKind;
```

Keep helper visibility private in this file.

- [x] **Step 8: Move snapshot tests into `snapshot.rs`**

Create `src-tauri/src/analysis/store/tests/snapshot.rs`.

Move these items from the current inline test module in `store.rs` into `snapshot.rs`:

- `snapshot_store_pool`
- `strict_snapshot_message`
- `sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens`
- `sanitize_provider_error_redacts_provider_payloads`
- `capture_run_snapshot_marks_captured_after_reload_and_replaces_rows`
- `capture_run_snapshot_rejects_missing_required_fields_without_marker`
- `mark_run_capture_failed_sets_snapshot_error`

At the top of `snapshot.rs`, add:

```rust
use super::super::{
    capture_run_snapshot, mark_run_capture_failed, sanitize_provider_error, sanitize_snapshot_error,
};
use crate::analysis::models::CorpusMessage;
```

Keep helper visibility private in this file unless Step 9 deliberately imports `snapshot_store_pool` from `super::harness`. The recommended approach is to keep status-update tests in `runs.rs` with a local minimal status schema, so `harness.rs` remains empty.

- [x] **Step 9: Move run-operation tests into `runs.rs`**

Create `src-tauri/src/analysis/store/tests/runs.rs`.

Move these tests from the current inline test module in `store.rs` into `runs.rs`:

- `delete_saved_run_returns_typed_not_found_error`
- `provider_failure_status_update_does_not_write_snapshot_error`
- `cancellation_after_capture_does_not_write_snapshot_error`
- `insert_analysis_run_persists_youtube_corpus_mode`
- `duplicate_lookup_matches_telegram_history_scope`
- `duplicate_lookup_keeps_project_and_source_group_scopes_separate`
- `delete_saved_run_removes_run_and_saved_children`

At the top of `runs.rs`, add:

```rust
use super::super::{
    delete_saved_run, find_active_duplicate_run, insert_analysis_run, set_run_status,
    AnalysisRunInsert, DuplicateRunLookup,
};
use crate::analysis::corpus::YoutubeCorpusMode;
use crate::analysis::models::AnalysisPromptTemplate;
use crate::error::AppErrorKind;
```

For `provider_failure_status_update_does_not_write_snapshot_error` and `cancellation_after_capture_does_not_write_snapshot_error`, do not import `snapshot_store_pool` from `snapshot.rs`. Add a private local helper in `runs.rs`:

```rust
async fn status_update_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query(
        r#"
        CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scope_label_snapshot TEXT,
            snapshot_captured_at TEXT,
            snapshot_error TEXT,
            status TEXT,
            result_markdown TEXT,
            trace_data_zstd BLOB,
            error TEXT,
            completed_at INTEGER
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create runs");
    sqlx::query("INSERT INTO analysis_runs (id, status) VALUES (1, 'running')")
        .execute(&pool)
        .await
        .expect("insert run");
    pool
}
```

Then change those two tests to call `status_update_pool().await` instead of `snapshot_store_pool().await`.

Keep all other test bodies, SQL, fixture values, and assertions unchanged.

- [x] **Step 10: Replace inline tests in `store.rs`**

In `src-tauri/src/analysis/store.rs`, replace the entire inline module that starts with `#[cfg(test)] mod tests {` and currently contains helpers such as `sample_run_row`, `run_list_pool`, and `delete_saved_run_removes_run_and_saved_children` with:

```rust
#[cfg(test)]
mod tests;
```

Do not change the production module declarations or facade re-exports above it.

- [x] **Step 11: Run rustfmt**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command exits 0. If unrelated Rust files changed, inspect them before proceeding and resolve drift before staging.

- [x] **Step 12: Run focused post-change store tests**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::read_model::list_analysis_run_summaries_applies_query_before_limit
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::setup::ensure_sources_exist_returns_typed_not_found_error
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::snapshot::capture_run_snapshot_marks_captured_after_reload_and_replaces_rows
```

Expected: pass and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::runs::insert_analysis_run_persists_youtube_corpus_mode
```

Expected: pass and not a green `0 tests` run.

- [x] **Step 13: Run source guards**

Production facade and external test module declaration:

```powershell
rg -n "^#\[cfg\(test\)\]$|^mod tests;" src-tauri/src/analysis/store.rs
```

Expected: two matches, one for `#[cfg(test)]` and one for `mod tests;`.

```powershell
rg -n "mod read_model;|mod runs;|mod setup;|mod snapshot;" src-tauri/src/analysis/store.rs
```

Expected: four matches.

Inline test body absent from `store.rs`:

```powershell
rg -n "sample_run_row|run_list_pool|snapshot_store_pool|template_store_pool|source_store_pool|strict_snapshot_message|insert_analysis_run_persists_youtube_corpus_mode|delete_saved_run_removes_run_and_saved_children" src-tauri/src/analysis/store.rs
```

Expected: no matches. Exit code `1` is expected.

Test module files:

```powershell
Get-ChildItem src-tauri/src/analysis/store/tests -Filter *.rs | Select-Object Name
```

Expected names:

```text
harness.rs
mod.rs
read_model.rs
runs.rs
setup.rs
snapshot.rs
```

Module declarations:

```powershell
rg -n "^mod (harness|read_model|runs|setup|snapshot);" src-tauri/src/analysis/store/tests/mod.rs
```

Expected: five matches.

Tests exercise the store facade rather than private production child modules:

```powershell
rg -n "store::(read_model|runs|setup|snapshot)|super::super::(read_model|runs|setup|snapshot)|super::(read_model|runs|setup|snapshot)" src-tauri/src/analysis/store/tests
```

Expected: no matches. Exit code `1` is expected.

Harness visibility remains test-tree local:

```powershell
rg -n "pub\(crate\)|pub fn|pub async fn" src-tauri/src/analysis/store/tests
```

Expected: no matches. Exit code `1` is expected.

Core tests are in thematic files:

```powershell
rg -n "list_analysis_run_summaries_applies_query_before_limit|map_run_summary_exposes_captured_snapshot_state" src-tauri/src/analysis/store/tests/read_model.rs
```

Expected: both test names are present.

```powershell
rg -n "ensure_sources_exist_returns_typed_not_found_error|fetch_prompt_template_returns_typed_not_found_error" src-tauri/src/analysis/store/tests/setup.rs
```

Expected: both test names are present.

```powershell
rg -n "sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens|capture_run_snapshot_marks_captured_after_reload_and_replaces_rows|mark_run_capture_failed_sets_snapshot_error" src-tauri/src/analysis/store/tests/snapshot.rs
```

Expected: all three test names are present.

```powershell
rg -n "insert_analysis_run_persists_youtube_corpus_mode|duplicate_lookup_matches_telegram_history_scope|delete_saved_run_removes_run_and_saved_children" src-tauri/src/analysis/store/tests/runs.rs
```

Expected: all three test names are present.

Assertion markers:

```powershell
rg -n -F "Source 7 not found" src-tauri/src/analysis/store/tests/setup.rs
```

Expected: one match.

```powershell
rg -n -F "Analysis prompt template 99 not found" src-tauri/src/analysis/store/tests/setup.rs
```

Expected: one match.

```powershell
rg -n -F "Analysis run 42 not found" src-tauri/src/analysis/store/tests/runs.rs
```

Expected: one match.

```powershell
rg -n -F "item_kind" src-tauri/src/analysis/store/tests/snapshot.rs
```

Expected: at least one match.

```powershell
rg -n -F "Provider request failed" src-tauri/src/analysis/store/tests/snapshot.rs
```

Expected: one match.

```powershell
rg -n -F "Analysis run cancelled." src-tauri/src/analysis/store/tests/runs.rs
```

Expected: one match.

- [x] **Step 14: Run full post-change verification**

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
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This covers test-only module paths and production facade consumers.

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: pass. If it fails, run `cargo fmt --manifest-path src-tauri/Cargo.toml`, inspect `git status --short --untracked-files=all`, resolve unrelated drift, and then rerun `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.

- [x] **Step 15: Compare final worktree to the pre-edit status snapshot**

Run, replacing `<PRE_EDIT_STATUS_PATH>` with the path printed in Step 2:

```powershell
$before = Get-Content -LiteralPath '<PRE_EDIT_STATUS_PATH>'
$afterPath = Join-Path $env:TEMP "analysis-store-tests-refactor-status-after.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $afterPath
$after = Get-Content -LiteralPath $afterPath
Compare-Object -ReferenceObject $before -DifferenceObject $after
```

Expected: differences are limited to intended changes in:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/tests/mod.rs`
- `src-tauri/src/analysis/store/tests/harness.rs`
- `src-tauri/src/analysis/store/tests/read_model.rs`
- `src-tauri/src/analysis/store/tests/setup.rs`
- `src-tauri/src/analysis/store/tests/snapshot.rs`
- `src-tauri/src/analysis/store/tests/runs.rs`

Unrelated pre-existing files such as `.claude/settings.local.json` may appear in both before and after and must not be staged.

- [x] **Step 16: Inspect implementation diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/tests
```

Expected:

- `store.rs` keeps production module declarations and facade re-exports unchanged.
- `store.rs` contains only `#[cfg(test)] mod tests;` for tests.
- `tests/mod.rs` declares the five child modules.
- thematic files contain moved tests with unchanged SQL, fixture values, and assertions.
- no thematic test imports private production child modules directly.

Run:

```powershell
git diff --check -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/tests
```

Expected: no whitespace errors.

- [x] **Step 17: Stage implementation files only**

Run:

```powershell
git add -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/tests/mod.rs src-tauri/src/analysis/store/tests/harness.rs src-tauri/src/analysis/store/tests/read_model.rs src-tauri/src/analysis/store/tests/setup.rs src-tauri/src/analysis/store/tests/snapshot.rs src-tauri/src/analysis/store/tests/runs.rs
```

Expected: only the seven implementation files are staged.

Run:

```powershell
git diff --cached --name-status
```

Expected:

```text
M       src-tauri/src/analysis/store.rs
A       src-tauri/src/analysis/store/tests/harness.rs
A       src-tauri/src/analysis/store/tests/mod.rs
A       src-tauri/src/analysis/store/tests/read_model.rs
A       src-tauri/src/analysis/store/tests/runs.rs
A       src-tauri/src/analysis/store/tests/setup.rs
A       src-tauri/src/analysis/store/tests/snapshot.rs
```

Run:

```powershell
git diff --cached --check
```

Expected: no whitespace errors.

- [x] **Step 18: Commit the refactor**

Run:

```powershell
git commit -m "refactor: split analysis store tests"
```

Expected: commit succeeds with only:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/tests/harness.rs`
- `src-tauri/src/analysis/store/tests/mod.rs`
- `src-tauri/src/analysis/store/tests/read_model.rs`
- `src-tauri/src/analysis/store/tests/runs.rs`
- `src-tauri/src/analysis/store/tests/setup.rs`
- `src-tauri/src/analysis/store/tests/snapshot.rs`

- [x] **Step 19: Record post-commit status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: no new implementation files remain unstaged. Pre-existing unrelated files may remain untracked, but no refactor files should be dirty.

---

## Final Verification Checklist

Before reporting the implementation complete, confirm the execution log includes:

- [x] baseline `analysis::store::tests::` passed before editing and was not a green `0 tests` run;
- [x] baseline focused read-model, snapshot, and runs tests passed before editing and were not green `0 tests` runs;
- [x] post-change focused read-model, setup, snapshot, and runs tests passed and were not green `0 tests` runs;
- [x] post-change `cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::` passed and was not a green `0 tests` run;
- [x] post-change `cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::` passed and was not a green `0 tests` run;
- [x] `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed;
- [x] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passed after any formatting fixes;
- [x] source guards proved `store.rs` no longer contains inline test body helpers or test functions;
- [x] source guards proved thematic test files exist and hold the expected core test names;
- [x] source guards proved tests do not import private production child modules directly;
- [x] source guards proved no `pub(crate)` or public helpers were introduced in `store/tests`;
- [x] staged files were limited to `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/tests/*.rs`;
- [x] post-commit `git status --short --untracked-files=all` has no dirty refactor files.
