# Analysis Store Snapshot Refactor Design

**Date:** 2026-07-03
**Status:** active spec; implementation not started as of 2026-07-03 because `src-tauri/src/analysis/store/snapshot.rs` does not exist
**Scope:** internal Rust refactor of analysis snapshot capture, persistence, reload, failure marking, and error sanitization logic.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/store.rs` by extracting analysis run snapshot capture and snapshot error sanitization into a focused private child module, without changing database writes, transaction order, snapshot validation, error sanitization, facade imports, report behavior, corpus tests, or saved-run cleanup behavior.

This is the next conservative backend slice after the read-model extraction. It intentionally avoids moving prompt-template storage, source existence checks, source-group loading, duplicate-run lookup, run insertion, status mutation, saved-run deletion, read-model queries, or store tests.

## Current Shape

`src-tauri/src/analysis/store.rs` currently owns:

- prompt-template initialization and source existence checks;
- source-group loading;
- duplicate-run lookup and analysis run insertion;
- read-model facade re-exports from `store/read_model.rs`;
- snapshot error sanitization, snapshot message validation, snapshot capture, snapshot persistence, and capture failure marking;
- run status mutation and saved-run deletion;
- inline tests for all of the above.

The snapshot cluster currently lives directly in `store.rs`:

- `sanitize_snapshot_error`
- `sanitize_provider_error`
- `validate_snapshot_message`
- `load_run_snapshot_messages_on_transaction`
- `capture_run_snapshot`
- `persist_run_snapshot`
- `mark_run_capture_failed`

Current consumers:

- `analysis/report/capture.rs` imports `capture_run_snapshot` and `sanitize_snapshot_error` through `analysis::store`;
- `analysis/report/lifecycle.rs` imports `mark_run_capture_failed` and `sanitize_provider_error` through `analysis::store`;
- `analysis/corpus/tests/snapshot.rs` imports `persist_run_snapshot` through `analysis::store`;
- `analysis/corpus/tests/source_resolution.rs` imports `persist_run_snapshot` through `analysis::store`;
- `store.rs` tests call `capture_run_snapshot`, `mark_run_capture_failed`, `sanitize_snapshot_error`, and `sanitize_provider_error` through the parent facade.

## Proposed Architecture

Create a private child module declared from `src-tauri/src/analysis/store.rs`:

- `src-tauri/src/analysis/store/snapshot.rs`

Keep `src-tauri/src/analysis/store.rs` as the store facade:

- add `mod snapshot;`;
- re-export the existing snapshot API from `store.rs`:

```rust
pub(crate) use self::snapshot::{
    capture_run_snapshot, mark_run_capture_failed, persist_run_snapshot, sanitize_provider_error,
    sanitize_snapshot_error,
};
```

- do not change imports in external consumers in this slice;
- keep `snapshot` private to `analysis::store`;
- keep store tests in `store.rs` for this slice.

Move these items from `store.rs` to `store/snapshot.rs`:

- `sanitize_snapshot_error`
- `sanitize_provider_error`
- `validate_snapshot_message`
- `load_run_snapshot_messages_on_transaction`
- `capture_run_snapshot`
- `persist_run_snapshot`
- `mark_run_capture_failed`

Keep these items in `store.rs` for this slice:

- `builtin_report_template_exists`
- `ensure_builtin_report_template`
- `ensure_sources_exist`
- `fetch_prompt_template`
- `fetch_source_group`
- `DuplicateRunLookup`
- `find_active_duplicate_run`
- `AnalysisRunInsert`
- `insert_analysis_run`
- read-model facade declarations and re-exports
- `set_run_status`
- `delete_saved_run`
- all current tests.

The inline test module stays in `store.rs` for this slice. Moving store tests can be a later test-only refactor.

## Visibility

`store/snapshot.rs` should expose only the existing snapshot API consumed through `analysis::store`:

```rust
pub(crate) fn sanitize_snapshot_error(category: &str, raw: &str) -> String;

pub(crate) fn sanitize_provider_error(category: &str, raw: &str) -> String;

pub(crate) async fn capture_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> AppResult<Vec<CorpusMessage>>;

#[allow(dead_code)]
pub(crate) async fn persist_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> AppResult<()>;

pub(crate) async fn mark_run_capture_failed(
    pool: &Pool<Sqlite>,
    run_id: i64,
    snapshot_error: &str,
    completed_at: i64,
) -> AppResult<()>;
```

Private helpers stay private inside `snapshot.rs`:

- `validate_snapshot_message`
- `load_run_snapshot_messages_on_transaction`

Preserve `#[allow(dead_code)]` on `persist_run_snapshot` unless implementation adds a non-test production reader. Its current non-production consumers are corpus tests, so removing the attribute can create warning debt in normal non-test builds.

Expected production API changes outside `analysis::store`: none.

Expected root re-export changes in `analysis/mod.rs`: none.

## Imports

`store/snapshot.rs` should own imports needed by snapshot logic:

- `sqlx::{Pool, Sqlite}`
- `super::super::models::{CorpusMessage, StoredRunSnapshotRow}`
- `super::super::ANALYSIS_STATUS_FAILED`
- `crate::compression::{compress_text, decompress_text}`
- `crate::error::{internal_error, AppError, AppResult}`

`store.rs` should remove imports that only moved snapshot helpers use after extraction:

- `CorpusMessage`, if only snapshot helpers and tests use it in production scope;
- `StoredRunSnapshotRow`, if only `snapshot.rs` uses it;
- `compress_text`, if only `snapshot.rs` uses it;
- `decompress_text`, if only `snapshot.rs` uses it;
- `internal_error`, if only `snapshot.rs` uses it.

Keep in `store.rs` imports needed by prompt-template, source existence, source-group, duplicate-run, insert, status, delete, read-model facade, and tests. Test-only imports can remain inside the inline `#[cfg(test)] mod tests`.

The implementation plan must verify unused imports after the move and keep the diff free of new warning debt in `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/snapshot.rs`.

## Data Flow

No runtime data flow changes:

1. `analysis/report/capture.rs` still calls `store::capture_run_snapshot` and `store::sanitize_snapshot_error` through the same paths.
2. `analysis/report/lifecycle.rs` still calls `store::mark_run_capture_failed` and `store::sanitize_provider_error` through the same paths.
3. `analysis/corpus/tests/snapshot.rs` and `analysis/corpus/tests/source_resolution.rs` still call `store::persist_run_snapshot` through the same path.
4. `capture_run_snapshot` still rejects empty corpus before opening a transaction.
5. `capture_run_snapshot` still validates every `CorpusMessage` before opening a transaction.
6. `capture_run_snapshot` still clears `scope_label_snapshot`, `snapshot_captured_at`, and `snapshot_error` state before replacing rows.
7. `capture_run_snapshot` still deletes prior `analysis_run_messages`, inserts new compressed rows, reloads through `load_run_snapshot_messages_on_transaction`, rejects an empty reload, marks `snapshot_captured_at = datetime('now')`, clears `snapshot_error`, commits, and returns the reloaded `Vec<CorpusMessage>`.
8. `persist_run_snapshot` still delegates to `capture_run_snapshot` and discards the returned messages.
9. `mark_run_capture_failed` still sanitizes with category `Snapshot capture failed`, writes `ANALYSIS_STATUS_FAILED`, mirrors the sanitized string into both `error` and `snapshot_error`, preserves the supplied `completed_at`, and does not modify `snapshot_captured_at`.
10. `sanitize_snapshot_error` and `sanitize_provider_error` keep the same redaction, control-character cleanup, whitespace compaction, secret detection, URL/path redaction, and 512-character bounding behavior.

## Error Handling

Preserve current error behavior exactly:

- empty corpus still returns `internal_error("Snapshot capture failed: empty corpus")`;
- empty reloaded snapshot still returns `internal_error("Snapshot capture failed: reloaded snapshot is empty")`;
- missing `ref`, `content`, `item_kind`, `source_type`, or required `source_subtype` still return the same `Snapshot message ref is required`, `Snapshot message {ref} content is required`, `Snapshot message {ref} item_kind is required`, `Snapshot message {ref} source_type is required`, and `Snapshot message {ref} source_subtype is required for {source_type}` messages;
- compression and decompression failures still map through `internal_error`;
- database failures still use `AppError::database`;
- `sanitize_snapshot_error` still falls back to the supplied category when the sanitized string is empty or secret-like;
- `sanitize_provider_error` still falls back to the supplied category when raw provider payload markers are present;
- no new error codes, messages, SQL filters, DTO fields, migrations, or user-facing strings are introduced.

The implementation plan must include source guards for these literals and SQL fragments:

```powershell
rg -n -F "Snapshot capture failed: empty corpus" src-tauri/src/analysis/store/snapshot.rs
rg -n -F "Snapshot capture failed: reloaded snapshot is empty" src-tauri/src/analysis/store/snapshot.rs
rg -n -F "source_subtype is required for" src-tauri/src/analysis/store/snapshot.rs
rg -n -F "DELETE FROM analysis_run_messages WHERE run_id = ?" src-tauri/src/analysis/store/snapshot.rs
rg -n -F "UPDATE analysis_runs SET snapshot_captured_at = datetime('now'), snapshot_error = NULL WHERE id = ?" src-tauri/src/analysis/store/snapshot.rs
rg -n -F "Snapshot capture failed" src-tauri/src/analysis/store/snapshot.rs
```

Expected: all snapshot validation, persistence, and failure markers are present in `snapshot.rs` after extraction.

## Non-Goals

This slice does not:

- move prompt-template initialization;
- move source existence checks;
- move source-group loading;
- move duplicate-run lookup;
- move analysis run insertion;
- move run status mutation or saved-run deletion;
- move read-model logic or `store/read_model.rs`;
- split store tests into files;
- change SQL, DTO mappings, compression format, metadata handling, validation rules, sanitization rules, transaction boundaries, database schema, migrations, frontend code, Tauri command payloads, or event payloads.

## Implementation Hygiene

The implementation plan must include a pre-edit worktree guard:

```powershell
git status --short --untracked-files=all
```

Target files must be clean before editing. If `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/snapshot.rs` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline commit before starting this refactor. This is required because the implementation plan should use full-file staging for the two target Rust files.

Inspect tracked target-file diffs before editing:

```powershell
git diff -- src-tauri/src/analysis/store.rs
git diff --cached -- src-tauri/src/analysis/store.rs
```

If `src-tauri/src/analysis/store/snapshot.rs` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/snapshot.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs'
}
```

Do not stage unrelated dirty files, such as local tool settings. Unrelated dirty files must remain unstaged and must be accounted for in baseline/final status comparisons.

The implementation plan must capture pre-edit status using a unique tag and persist the paths for later PowerShell sessions:

```powershell
$env:ANALYSIS_STORE_SNAPSHOT_STATUS_TAG = "{0}-{1}" -f (Get-Date -Format 'yyyyMMddHHmmss'), $PID
$statusPointerPath = Join-Path $env:TEMP 'analysis-store-snapshot-latest-status-paths.txt'
$preEditStatusPath = Join-Path $env:TEMP "analysis-store-snapshot-$env:ANALYSIS_STORE_SNAPSHOT_STATUS_TAG-pre-edit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preEditStatusPath
"ANALYSIS_STORE_SNAPSHOT_STATUS_TAG=$env:ANALYSIS_STORE_SNAPSHOT_STATUS_TAG" | Set-Content -Encoding utf8 -LiteralPath $statusPointerPath
"PRE_EDIT_STATUS_PATH=$preEditStatusPath" | Add-Content -Encoding utf8 -LiteralPath $statusPointerPath
Get-Content -Raw -LiteralPath $preEditStatusPath
Get-Content -Raw -LiteralPath $statusPointerPath
```

After formatting, checks, and commit, compare final status against the captured baseline by reloading the pointer file.

## Testing

Run commands from the repository root.

Before editing, establish the current baseline:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::sanitize_snapshot_error
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::sanitize_provider_error
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::capture_run_snapshot
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::mark_run_capture_failed
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::snapshot
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::source_resolution
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

After editing and before committing, run each command separately with the same non-zero expectations. Do not paste these as one PowerShell block unless the block explicitly checks `$LASTEXITCODE` after every native command and stops on failure.

Also run consumer compile coverage:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. This covers production imports in `analysis/report/capture.rs`, `analysis/report/lifecycle.rs`, and crate test consumers. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/snapshot.rs` are not acceptable.

Also run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

The implementation plan must include source guards:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?(fn|async fn) (sanitize_snapshot_error|sanitize_provider_error|validate_snapshot_message|load_run_snapshot_messages_on_transaction|capture_run_snapshot|persist_run_snapshot|mark_run_capture_failed)\b" src-tauri/src/analysis/store.rs
rg -n "^mod snapshot;" src-tauri/src/analysis/store.rs
rg -n "^pub.*mod snapshot" src-tauri/src/analysis/store.rs
rg -n "^pub\(crate\) use self::snapshot::" src-tauri/src/analysis/store.rs
$storeFacade = Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'
$snapshotReExport = [regex]::Match($storeFacade, "pub\(crate\) use self::snapshot::\{(?<block>[\s\S]*?)\};")
if (-not $snapshotReExport.Success) {
    throw "missing snapshot facade re-export block"
}
foreach ($name in @('capture_run_snapshot', 'mark_run_capture_failed', 'persist_run_snapshot', 'sanitize_provider_error', 'sanitize_snapshot_error')) {
    if ($snapshotReExport.Groups['block'].Value -notmatch ("\b" + [regex]::Escape($name) + "\b")) {
        throw "missing snapshot facade re-export: $name"
    }
}
rg -n "^pub\(crate\) (fn|async fn) (sanitize_snapshot_error|sanitize_provider_error|capture_run_snapshot|persist_run_snapshot|mark_run_capture_failed)\b" src-tauri/src/analysis/store/snapshot.rs
rg -n "^(fn|async fn) (validate_snapshot_message|load_run_snapshot_messages_on_transaction)\b" src-tauri/src/analysis/store/snapshot.rs
rg -n "^\s*pub(\([^)]*\))?\s+(fn|async fn) (validate_snapshot_message|load_run_snapshot_messages_on_transaction)\b" src-tauri/src/analysis/store/snapshot.rs
$snapshotSource = Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs'
if ($snapshotSource -notmatch "#\[allow\(dead_code\)\]\s*pub\(crate\) async fn persist_run_snapshot") {
    throw "persist_run_snapshot must preserve #[allow(dead_code)]"
}
```

Expected: first command has no matches; `rg` exit code `1` is expected for this no-match guard. The second command prints exactly one private module declaration. The third command has no matches; `rg` exit code `1` is expected. The facade loop completes without throwing. Public API guards print the five re-exported snapshot API items. The private-helper positive guard prints both private helper signatures. The private-helper widening guard has no matches; `rg` exit code `1` is expected. The `persist_run_snapshot` PowerShell guard confirms the dead-code allowance stayed attached to the function.

## Commit Shape

Expected implementation-owned files:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/snapshot.rs`

Expected implementation commit:

```text
refactor: extract analysis store snapshot logic
```

The design spec and implementation plan should be committed separately from the Rust refactor.
