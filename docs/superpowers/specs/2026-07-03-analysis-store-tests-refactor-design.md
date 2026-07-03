# Analysis Store Tests Refactor Design

**Date:** 2026-07-03
**Status:** active spec; implementation not started as of 2026-07-03 because `src-tauri/src/analysis/store/tests/` does not exist.
**Scope:** internal Rust test-only refactor of `src-tauri/src/analysis/store.rs` into nested store test modules.

## Goal

Reduce the size and review burden of `src-tauri/src/analysis/store.rs` by moving its large inline `#[cfg(test)] mod tests` body into focused nested test modules, without changing production behavior, facade re-exports, SQL, fixtures, assertions, test coverage, or external consumer paths.

This is the next conservative store slice after extracting read-model, snapshot, setup, and run operation production code. The production store facade is now small; the remaining file size is dominated by tests.

## Current Shape

`src-tauri/src/analysis/store.rs` currently owns:

- production facade wiring for `read_model`, `runs`, `setup`, and `snapshot`;
- `pub(crate)` re-exports consumed by the rest of `analysis`;
- a large inline `#[cfg(test)] mod tests`;
- shared in-memory SQLite setup helpers;
- tests for run listing/read-model behavior;
- tests for snapshot capture and error sanitization behavior;
- tests for setup error behavior;
- tests for run insertion, duplicate lookup, status update, and saved-run deletion.

The inline test module is the dominant part of the file. It mixes these concerns:

- shared fixture helpers:
  - `sample_run_row`
  - `sample_run`
  - SQLite pool builders for run-list, snapshot, template, and source tests
  - `AnalysisPromptTemplate` fixtures
  - snapshot `CorpusMessage` fixture construction
- read-model/list tests:
  - query-before-limit behavior
  - scope and field filters
  - source-group, project, template, status, date, and query filtering
  - literal SQL LIKE escaping
  - multi-term query behavior
  - run summary/detail mapping and snapshot-state mapping
- snapshot tests:
  - snapshot error sanitization
  - provider error sanitization
  - capture reload/replace behavior
  - missing required field rejection
  - capture failure marking
- setup tests:
  - missing source not-found error
  - missing prompt template not-found error
- runs tests:
  - saved-run deletion not-found behavior
  - status updates not writing snapshot error
  - insert persistence for YouTube corpus mode
  - duplicate lookup by Telegram history scope
  - duplicate lookup separation for project/source-group scopes
  - saved-run child cleanup.

## Proposed Architecture

Move only test code into a nested test tree:

- `src-tauri/src/analysis/store/tests/mod.rs`
- `src-tauri/src/analysis/store/tests/harness.rs`
- `src-tauri/src/analysis/store/tests/read_model.rs`
- `src-tauri/src/analysis/store/tests/setup.rs`
- `src-tauri/src/analysis/store/tests/snapshot.rs`
- `src-tauri/src/analysis/store/tests/runs.rs`

Keep production code in `src-tauri/src/analysis/store.rs`.

Replace the inline test module in `store.rs` with:

```rust
#[cfg(test)]
mod tests;
```

`tests/mod.rs` should declare child test modules:

```rust
mod harness;
mod read_model;
mod runs;
mod setup;
mod snapshot;
```

`tests/harness.rs` owns helpers shared by more than one thematic test module. The thematic modules import helpers through `super::harness`.

This split keeps the existing Cargo test path prefix under `analysis::store::tests::`. Individual tests gain one more module segment, such as `analysis::store::tests::runs::insert_analysis_run_persists_youtube_corpus_mode`. The implementation plan must update verification filters accordingly and must prevent green `0 tests` runs.

## File Responsibilities

`src-tauri/src/analysis/store.rs`

- Keep all production module declarations and facade re-exports.
- Keep only `#[cfg(test)] mod tests;` for tests.
- Do not gain new test helper code.
- Do not change production visibility or external consumer imports.

`src-tauri/src/analysis/store/tests/mod.rs`

- Declare child test modules.
- Avoid owning helper logic directly unless it is only module wiring.

`src-tauri/src/analysis/store/tests/harness.rs`

- Own shared fixtures used by more than one thematic module.
- Own shared SQLite schema helpers only when the same schema is used across modules.
- Avoid becoming a dumping ground for helpers used by a single module.

`src-tauri/src/analysis/store/tests/read_model.rs`

- Own run-list tests and read-model mapping tests.
- Own `RunListFixture`, `run_list_pool`, and `insert_run_list_fixture`.
- Own `sample_run_row` and `sample_run` unless another thematic module needs them during implementation.

`src-tauri/src/analysis/store/tests/setup.rs`

- Own setup error tests for `ensure_sources_exist` and `fetch_prompt_template`.
- Own `template_store_pool` and `source_store_pool` unless another module needs them during implementation.

`src-tauri/src/analysis/store/tests/snapshot.rs`

- Own snapshot error sanitization tests.
- Own provider error sanitization tests.
- Own snapshot capture, required-field rejection, and capture failure marking tests.
- Own `snapshot_store_pool` and `strict_snapshot_message` unless `runs.rs` keeps status-update snapshot-error tests and shares that pool through `harness.rs`.

`src-tauri/src/analysis/store/tests/runs.rs`

- Own run insertion, duplicate lookup, status update, and saved-run deletion tests.
- Own local run-operation schemas and `AnalysisPromptTemplate` fixtures.
- If status-update snapshot-error tests keep using the snapshot schema, import that helper from `harness.rs`; otherwise keep a local minimal schema helper private to `runs.rs`.

## Visibility

This refactor is test-only, but moving tests out of the inline child module changes privacy boundaries.

Implementation should use this visibility strategy:

1. Keep production visibility unchanged.
2. Keep thematic helper functions private to their module when only one module uses them.
3. Move only genuinely shared helpers into `tests/harness.rs`.
4. Mark shared harness helpers `pub(super)`, never `pub(crate)` or `pub`.
5. Do not widen production item visibility solely for this test split.

Expected production visibility changes: none.

Expected test-helper visibility changes: helpers shared from `harness.rs` become `pub(super)`.

Expected `tests/harness.rs` helper contract is intentionally small:

```rust
pub(super) async fn snapshot_store_pool() -> sqlx::SqlitePool;
```

This helper is only required if `snapshot.rs` and `runs.rs` both need the current snapshot/status schema. If the implementation keeps all snapshot-schema tests in `snapshot.rs` or gives `runs.rs` a local minimal status schema, `harness.rs` may be limited to module declarations and contain no helpers. Do not export single-module helpers from `harness.rs`.

Expected facade access from test modules:

- Thematic test modules should import production store API through explicit parent facade imports, for example `use super::super::{list_analysis_run_summaries, AnalysisRunListFilters};`.
- They should not import `crate::analysis::store::read_model`, `crate::analysis::store::runs`, `crate::analysis::store::setup`, or `crate::analysis::store::snapshot` directly.
- The tests must continue exercising the parent store facade, not private child modules.

## Data Flow

No runtime data flow changes:

1. Production `store.rs` still exposes the same read-model, setup, snapshot, and run operation facade.
2. In-memory SQLite setup still creates the same schemas.
3. Seed helpers still insert the same rows and metadata.
4. Read-model tests still exercise `list_analysis_run_summaries`, `map_run_summary`, `map_run_detail`, and `resolve_run_scope_label` through the store facade.
5. Setup tests still exercise `ensure_sources_exist` and `fetch_prompt_template` through the store facade.
6. Snapshot tests still exercise `capture_run_snapshot`, `mark_run_capture_failed`, `sanitize_snapshot_error`, and `sanitize_provider_error` through the store facade.
7. Runs tests still exercise `insert_analysis_run`, `find_active_duplicate_run`, `set_run_status`, and `delete_saved_run` through the store facade.

Only test module paths change. The full command:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

must continue to run all store tests.

## Error Handling

Preserve current test assertions and error expectations exactly:

- missing source still asserts `AppErrorKind::NotFound` and `Source 7 not found`;
- missing prompt template still asserts `AppErrorKind::NotFound` and `Analysis prompt template 99 not found`;
- missing saved run still asserts `AppErrorKind::NotFound` and `Analysis run 42 not found`;
- snapshot validation still asserts missing `item_kind` without writing the capture marker;
- sanitization assertions still reject paths, URLs, tokens, provider payloads, and empty sanitized output;
- status-update tests still assert provider failure and cancellation updates do not write `snapshot_error`;
- no test assertion, fixture value, SQL table definition, SQL insert, or user-facing error string is changed.

The implementation plan must include source guards for these strings after the move:

```powershell
rg -n -F "Source 7 not found" src-tauri/src/analysis/store/tests/setup.rs
rg -n -F "Analysis prompt template 99 not found" src-tauri/src/analysis/store/tests/setup.rs
rg -n -F "Analysis run 42 not found" src-tauri/src/analysis/store/tests/runs.rs
rg -n -F "item_kind" src-tauri/src/analysis/store/tests/snapshot.rs
rg -n -F "Provider request failed" src-tauri/src/analysis/store/tests/snapshot.rs
rg -n -F "Analysis run cancelled." src-tauri/src/analysis/store/tests/runs.rs
```

Expected: all moved assertion markers are present in the thematic test modules.

## Non-Goals

This slice does not:

- move or edit production code in `store/read_model.rs`, `store/runs.rs`, `store/setup.rs`, or `store/snapshot.rs`;
- split or redesign production store facade re-exports;
- change external consumer imports;
- change SQL, schemas, fixture data, assertion messages, DTO mappings, transaction behavior, database migrations, frontend code, Tauri command payloads, or event payloads;
- add new behavior tests beyond path/coverage guards needed for the move;
- delete or weaken any current store test.

## Implementation Hygiene

The implementation plan must include a pre-edit worktree guard:

```powershell
git status --short --untracked-files=all
```

Target files must be clean before editing. If `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/tests/` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline commit before starting this refactor. This is required because the implementation plan may use full-file staging for the target Rust files.

Inspect tracked target-file diffs before editing:

```powershell
git diff -- src-tauri/src/analysis/store.rs
git diff --cached -- src-tauri/src/analysis/store.rs
```

If `src-tauri/src/analysis/store/tests/` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/tests') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/tests
    Get-ChildItem -Recurse -Force -LiteralPath 'src-tauri/src/analysis/store/tests'
}
```

Do not stage unrelated dirty files, such as local tool settings. Unrelated dirty files must remain unstaged and must be accounted for in baseline/final status comparisons.

The implementation plan must capture pre-edit status using a unique tag and persist the paths for later PowerShell sessions:

```powershell
$tag = "analysis-store-tests-refactor-" + (Get-Date -Format "yyyyMMddHHmmss")
$preEditStatusPath = Join-Path $env:TEMP "$tag-status-before.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $preEditStatusPath
$preEditStatusPath
```

Before commit, compare the final status to the captured baseline and confirm no new unintended files or diffs exist outside:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/tests/mod.rs`
- `src-tauri/src/analysis/store/tests/harness.rs`
- `src-tauri/src/analysis/store/tests/read_model.rs`
- `src-tauri/src/analysis/store/tests/setup.rs`
- `src-tauri/src/analysis/store/tests/snapshot.rs`
- `src-tauri/src/analysis/store/tests/runs.rs`

If `cargo fmt` rewrites unrelated Rust files, resolve that drift before the refactor commit by making a separate format-only commit or restoring only implementation-owned formatting changes after review. Final status should return to the captured baseline except for intended staged refactor files.

## Source Guards

The implementation plan must include source guards after the move.

`store.rs` should be a short production facade plus external test-module declaration:

```powershell
rg -n "^#\[cfg\(test\)\]$|^mod tests;" src-tauri/src/analysis/store.rs
rg -n "mod read_model;|mod runs;|mod setup;|mod snapshot;" src-tauri/src/analysis/store.rs
```

Expected: test module declaration is present, and existing production module declarations remain.

Inline test body must not remain in `store.rs`:

```powershell
rg -n "sample_run_row|run_list_pool|snapshot_store_pool|template_store_pool|source_store_pool|strict_snapshot_message|insert_analysis_run_persists_youtube_corpus_mode|delete_saved_run_removes_run_and_saved_children" src-tauri/src/analysis/store.rs
```

Expected: no matches. `rg` exit code `1` is expected.

Test module files must exist:

```powershell
Get-ChildItem src-tauri/src/analysis/store/tests -Filter *.rs | Select-Object Name
```

Expected names:

- `mod.rs`
- `harness.rs`
- `read_model.rs`
- `setup.rs`
- `snapshot.rs`
- `runs.rs`

`tests/mod.rs` must declare thematic modules:

```powershell
rg -n "^mod (harness|read_model|runs|setup|snapshot);" src-tauri/src/analysis/store/tests/mod.rs
```

Expected: five module declarations.

Tests should exercise the store facade rather than private child modules:

```powershell
rg -n "store::(read_model|runs|setup|snapshot)|super::super::(read_model|runs|setup|snapshot)|super::(read_model|runs|setup|snapshot)" src-tauri/src/analysis/store/tests
```

Expected: no matches. `rg` exit code `1` is expected.

Harness visibility must stay test-tree local:

```powershell
rg -n "pub\(crate\)|pub fn|pub async fn" src-tauri/src/analysis/store/tests
```

Expected: no matches. `rg` exit code `1` is expected. Shared helpers should use `pub(super)` only.

Core test names must be present in thematic files:

```powershell
rg -n "list_analysis_run_summaries_applies_query_before_limit|map_run_summary_exposes_captured_snapshot_state" src-tauri/src/analysis/store/tests/read_model.rs
rg -n "ensure_sources_exist_returns_typed_not_found_error|fetch_prompt_template_returns_typed_not_found_error" src-tauri/src/analysis/store/tests/setup.rs
rg -n "sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens|capture_run_snapshot_marks_captured_after_reload_and_replaces_rows|mark_run_capture_failed_sets_snapshot_error" src-tauri/src/analysis/store/tests/snapshot.rs
rg -n "insert_analysis_run_persists_youtube_corpus_mode|duplicate_lookup_matches_telegram_history_scope|delete_saved_run_removes_run_and_saved_children" src-tauri/src/analysis/store/tests/runs.rs
```

Expected: every named test exists in the expected file.

## Testing

Run required commands from the repository root with `--manifest-path src-tauri/Cargo.toml`. Run each command separately or through a wrapper that stops on non-zero `$LASTEXITCODE`; do not place multiple `cargo` commands in one plain PowerShell block.

Baseline before editing:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::list_analysis_run_summaries_applies_query_before_limit
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::capture_run_snapshot_marks_captured_after_reload_and_replaces_rows
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::insert_analysis_run_persists_youtube_corpus_mode
```

Expected: every baseline command passes and is not a green `0 tests` run.

Post-change verification:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::read_model::list_analysis_run_summaries_applies_query_before_limit
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::setup::ensure_sources_exist_returns_typed_not_found_error
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::snapshot::capture_run_snapshot_marks_captured_after_reload_and_replaces_rows
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::runs::insert_analysis_run_persists_youtube_corpus_mode
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected:

- every focused store test command passes and is not a green `0 tests` run;
- `analysis::store::tests::` passes and is not a green `0 tests` run;
- `analysis::report::tests::` passes and is not a green `0 tests` run, covering report consumers of the store facade after the test-module move;
- `cargo check --all-targets` passes, covering test-only module paths and production facade consumers;
- `cargo fmt -- --check` passes after any formatting fix. If formatting fixes are required, run `cargo fmt`, inspect changed files with `git status --short --untracked-files=all`, resolve unrelated drift, then run `cargo fmt -- --check` again before staging.

## Commit Shape

The implementation should produce one focused test-only refactor commit that contains only:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/tests/mod.rs`
- `src-tauri/src/analysis/store/tests/harness.rs`
- `src-tauri/src/analysis/store/tests/read_model.rs`
- `src-tauri/src/analysis/store/tests/setup.rs`
- `src-tauri/src/analysis/store/tests/snapshot.rs`
- `src-tauri/src/analysis/store/tests/runs.rs`

Documentation hardening commits may be separate, as in prior refactor slices.

Before committing:

```powershell
git status --short --untracked-files=all
git diff -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/tests
git diff --cached --check
```

Run the git commands separately or through a stopping wrapper. Do not rely on a plain multi-command PowerShell block for failure handling.
