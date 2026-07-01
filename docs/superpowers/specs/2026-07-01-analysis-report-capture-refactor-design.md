# Analysis Report Capture Refactor Design

**Date:** 2026-07-01
**Status:** active spec; implementation not started as of 2026-07-01 because `src-tauri/src/analysis/report/capture.rs` does not exist
**Scope:** internal Rust refactor of analysis report corpus capture and snapshot-freezing helper logic.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/report.rs` by extracting report corpus capture and snapshot persistence preparation into a focused private nested module, without changing report execution, snapshot behavior, error strings, map/reduce orchestration, lifecycle handling, database schema, or Tauri command contracts.

This is the next conservative slice after extracting `analysis::report::requests` and `analysis::report::lifecycle`. It intentionally avoids moving pipeline runtime code so the refactor remains centered on one pre-provider phase: load the report corpus and persist the run snapshot.

## Current Shape

`src-tauri/src/analysis/report.rs` currently still owns corpus capture concerns:

- loading corpus messages through `load_corpus_messages`;
- translating corpus load failures into `ReportRunError::CaptureFailed`;
- rejecting empty capture results with `Snapshot capture failed`;
- persisting the immutable run snapshot through `capture_run_snapshot`;
- sanitizing snapshot/capture errors with `sanitize_snapshot_error`;
- returning the reloaded captured corpus to the report pipeline before provider phases begin.

The capture cluster is currently:

- `SNAPSHOT_CAPTURE_FAILED_MESSAGE`
- `capture_report_corpus`

Current consumers:

- `run_report_pipeline` calls `capture_report_corpus` before chunking and provider phases;
- inline report tests call `capture_report_corpus` directly through the parent report module;
- store-level tests cover lower-level `capture_run_snapshot` and `mark_run_capture_failed` behavior separately.

## Proposed Architecture

Create a private nested module declared from `src-tauri/src/analysis/report.rs`:

- `src-tauri/src/analysis/report/capture.rs`

Keep `src-tauri/src/analysis/report.rs` as the report workflow facade:

- add `mod capture;`;
- import `capture_report_corpus` through an explicit `self::capture` import;
- do not add any root re-export from `analysis/mod.rs`;
- keep `capture` private to `analysis::report`;
- keep map/reduce orchestration, lifecycle helpers, request helpers, `start_analysis_report_run`, `RunEvent`, `ReportRunInput`, and `ReportPipelineContext` in their current modules.

Move these items from `report.rs` to `report/capture.rs`:

- `SNAPSHOT_CAPTURE_FAILED_MESSAGE`
- `capture_report_corpus`

Keep these items in `report.rs` for this slice:

- `StartAnalysisReportRequest`
- `resolve_analysis_telegram_history_scope`
- `ReportRunError`
- `RunEvent`
- `finish_map_phase`
- `ReportRunInput`
- `validate_report_preflight`
- `ReportPipelineContext`
- `ReducePhaseResult`
- `run_analysis_step_with_cancel`
- `run_map_phase`
- `run_reduce_phase`
- `run_report_pipeline`
- `start_analysis_report_run`
- all current tests.

The inline test module stays in `report.rs` for this slice. Moving capture tests can be a later test-only refactor if needed.

## Visibility

`report/capture.rs` should expose only the helper surface consumed by `report.rs` and current inline tests:

```rust
pub(super) async fn capture_report_corpus(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    request: &CorpusLoadRequest,
) -> Result<Vec<CorpusMessage>, ReportRunError>;
```

`ReportRunError` remains defined in `report.rs`, but the nested capture module must be able to construct `CaptureFailed` values:

```rust
pub(super) enum ReportRunError {
    Failed(String),
    CaptureFailed(String),
    Cancelled(String),
}
```

The enum remains private to the `analysis::report` module tree. Do not move `ReportRunError` and do not expose it outside `analysis::report`.

Expected production API changes outside `analysis::report`: none.

Expected root re-export changes: none.

## Imports

`report/capture.rs` should own imports needed by capture helpers:

- `sqlx::{Pool, Sqlite}`
- `super::ReportRunError`
- `super::super::corpus::{load_corpus_messages, CorpusLoadRequest}`
- `super::super::models::CorpusMessage`
- `super::super::store::{capture_run_snapshot, sanitize_snapshot_error}`

`report.rs` should remove imports that only moved capture helpers use after extraction:

- `load_corpus_messages`, if no remaining code in `report.rs` uses it;
- `capture_run_snapshot`, if no remaining code in `report.rs` uses it;
- `sanitize_snapshot_error`, if no remaining code in `report.rs` uses it.

Keep in `report.rs` imports needed by pipeline, lifecycle, startup validation, and tests.

The implementation plan must verify unused imports after the move and keep the diff free of new warning debt in `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/capture.rs`.

## Constants

Move `SNAPSHOT_CAPTURE_FAILED_MESSAGE` to `report/capture.rs` and keep it private there:

```rust
const SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed";
```

Do not duplicate this string in `report.rs`.

Keep `INTERRUPTED_RUN_MESSAGE` and `CANCELLED_RUN_MESSAGE` in `report.rs`; they are not part of capture behavior.

## Data Flow

No runtime data flow changes:

1. `run_report_pipeline` still calls `capture_report_corpus` before chunking and provider phases.
2. `capture_report_corpus` still calls `load_corpus_messages(pool, request).await`.
3. Corpus load failures still become `ReportRunError::CaptureFailed(sanitize_snapshot_error("Corpus preload failed", &error.to_string()))`.
4. Empty corpus results still become `ReportRunError::CaptureFailed("Snapshot capture failed".to_string())`.
5. Non-empty corpus results still call `capture_run_snapshot(pool, run_id, scope_label, &corpus).await`.
6. Snapshot persistence failures still become `ReportRunError::CaptureFailed(sanitize_snapshot_error("Snapshot capture failed", &error.to_string()))`.
7. Successful capture still returns the reloaded captured corpus from `capture_run_snapshot`.
8. Provider phases still do not start before capture completes successfully.

## Error Handling

Preserve current error behavior exactly:

- corpus preload failure prefix remains `Corpus preload failed`;
- empty corpus failure message remains `Snapshot capture failed`;
- snapshot capture failure prefix remains `Snapshot capture failed`;
- all capture failures still use `ReportRunError::CaptureFailed`;
- capture failure handling in `start_analysis_report_run` still routes through `fail_capture_run`;
- no new error codes, messages, or user-facing strings are introduced.

The implementation plan must include source or test guards for these literals:

```text
Corpus preload failed
Snapshot capture failed
```

## Non-Goals

This slice does not:

- move map/reduce runtime orchestration;
- move lifecycle helpers;
- move request helpers;
- move `ReportRunError`;
- move `RunEvent`;
- move `run_report_pipeline`;
- move `start_analysis_report_run`;
- split report tests into files;
- change snapshot persistence behavior;
- change database schema, migrations, frontend, Tauri command payloads, or event payloads.

## Implementation Hygiene

The implementation plan must include a pre-edit worktree guard:

```powershell
git status --short --untracked-files=all
```

If target files are already modified, staged, or untracked before this task starts, inspect the baseline before editing:

```powershell
git diff -- src-tauri/src/analysis/report.rs
git diff --cached -- src-tauri/src/analysis/report.rs
```

If `src-tauri/src/analysis/report/capture.rs` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/report/capture.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/report/capture.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/report/capture.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/report/capture.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/report/capture.rs'
}
```

Do not stage pre-existing target-file changes into the capture extraction commit.

The implementation plan must capture the pre-edit status output as `PRE_EDIT_STATUS` and compare final status against it after formatting, checks, and commit.

## Testing

Run commands from the repository root.

Before editing, establish the current baseline:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
```

Expected: PASS with `1 passed`, not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. This guards recently split corpus boundaries while capture imports move.

After editing and before committing, run the same report and corpus tests again. The post-change capture-focused test must be repeated explicitly:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
```

Expected: PASS with `1 passed`, not a green `0 tests` run.

Also run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/capture.rs` are not acceptable.

The implementation plan must include source guards:

```powershell
rg -n "async fn capture_report_corpus|const SNAPSHOT_CAPTURE_FAILED_MESSAGE" src-tauri/src/analysis/report.rs
rg -n "pub\\(super\\) async fn capture_report_corpus|const SNAPSHOT_CAPTURE_FAILED_MESSAGE" src-tauri/src/analysis/report/capture.rs
```

Expected: first command has no matches after extraction; second command has both moved items.

## Commit Shape

Expected implementation-owned files:

- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/report/capture.rs`

Expected implementation commit:

```text
refactor: extract analysis report capture helper
```

The design spec and implementation plan should be committed separately from the Rust refactor.
