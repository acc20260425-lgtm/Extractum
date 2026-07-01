# Analysis Report Lifecycle Refactor Design

**Date:** 2026-07-01
**Status:** active spec; implementation not started as of 2026-07-01 because `src-tauri/src/analysis/report/lifecycle.rs` does not exist; not an implementation handoff until a separate plan is written
**Scope:** internal Rust refactor of analysis report run lifecycle, cancellation, and terminal status helpers.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/report.rs` by extracting report lifecycle side-effect helpers into a focused private nested module, without changing report execution, cancellation semantics, event payloads, database status updates, scheduler behavior, or Tauri command contracts.

This is the next conservative slice after extracting `analysis::report::requests`. It intentionally avoids moving map/reduce pipeline code so the refactor remains mostly about status persistence and cancellation request handling.

## Current Shape

`src-tauri/src/analysis/report.rs` currently owns both report pipeline orchestration and lifecycle side effects:

- terminal failure persistence for provider/runtime errors;
- terminal failure persistence for pre-snapshot capture errors;
- terminal cancellation persistence;
- app-start cleanup of queued/running interrupted runs;
- user-requested cancellation for queued/running report runs;
- report event emission for failed, cancelled, interrupted, and cancellation-requested states;
- map/reduce pipeline functions and tests.

The lifecycle cluster is currently:

- `fail_run`
- `fail_capture_run`
- `cancel_run`
- `mark_interrupted_analysis_runs`
- `cleanup_interrupted_analysis_runs`
- `request_analysis_run_cancel`

Current consumers:

- `start_analysis_report_run` calls `fail_run`, `fail_capture_run`, and `cancel_run` from the spawned report task;
- `cleanup_interrupted_analysis_runs` calls `mark_interrupted_analysis_runs`;
- `src-tauri/src/analysis/mod.rs` re-exports `report::cleanup_interrupted_analysis_runs`;
- `src-tauri/src/lib.rs` calls `analysis::cleanup_interrupted_analysis_runs` during startup cleanup;
- `src-tauri/src/analysis/report_commands.rs` calls `report::request_analysis_run_cancel`;
- inline report tests call `mark_interrupted_analysis_runs`.

## Proposed Architecture

Create a private nested module declared from `src-tauri/src/analysis/report.rs`:

- `src-tauri/src/analysis/report/lifecycle.rs`

Keep `src-tauri/src/analysis/report.rs` as the report workflow facade:

- add `mod lifecycle;`;
- import lifecycle helpers through explicit `self::lifecycle` imports;
- preserve the existing root re-export in `analysis/mod.rs`: `pub use self::report::cleanup_interrupted_analysis_runs;`;
- keep `lifecycle` private to `analysis::report`;
- keep map/reduce orchestration, `start_analysis_report_run`, `ReportRunError`, `ReportRunInput`, `ReportPipelineContext`, `RunEvent`, and request helpers in their current modules.

Move these functions from `report.rs` to `report/lifecycle.rs`:

- `fail_run`
- `fail_capture_run`
- `cancel_run`
- `mark_interrupted_analysis_runs`
- `cleanup_interrupted_analysis_runs`
- `request_analysis_run_cancel`

Keep these items in `report.rs` for this slice:

- `StartAnalysisReportRequest`
- `resolve_analysis_telegram_history_scope`
- `ReportRunError`
- `capture_report_corpus`
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

The inline test module stays in `report.rs` for this slice. Moving report lifecycle tests can be a later test-only refactor if needed.

## Visibility

`report/lifecycle.rs` should expose only the helper surface consumed by `report.rs`, `report_commands.rs`, and current inline tests through the parent `analysis::report` module:

```rust
pub(super) async fn fail_run(handle: &AppHandle, run_id: i64, error: String);

pub(super) async fn fail_capture_run(handle: &AppHandle, run_id: i64, error: String);

pub(super) async fn cancel_run(handle: &AppHandle, run_id: i64, message: String);

pub(crate) async fn mark_interrupted_analysis_runs(pool: &Pool<Sqlite>) -> AppResult<()>;

pub async fn cleanup_interrupted_analysis_runs(handle: AppHandle);

pub(crate) async fn request_analysis_run_cancel(
    handle: &AppHandle,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    run_id: i64,
) -> AppResult<()>;
```

`cleanup_interrupted_analysis_runs` remains `pub` because `analysis/mod.rs` currently re-exports it for startup cleanup in `lib.rs`. `request_analysis_run_cancel` and `mark_interrupted_analysis_runs` remain `pub(crate)` because they are crate-facing facade functions. `fail_run`, `fail_capture_run`, and `cancel_run` should be only `pub(super)` because they are used by `start_analysis_report_run` inside `report.rs`.

`RunEvent` remains defined in `report.rs` but must be callable from `lifecycle.rs`. Use the smallest visibility required by the nested module:

```rust
pub(super) struct RunEvent {
    event: AnalysisRunEvent,
}

impl RunEvent {
    pub(super) fn new(run_id: i64, kind: &str, phase: &str) -> Self;
    pub(super) fn message(self, message: String) -> Self;
    pub(super) fn error(self, error: String) -> Self;
    pub(super) fn emit(self, handle: &AppHandle);
}
```

This is the complete `RunEvent` lifecycle contract for this slice:

- `lifecycle.rs` may call only `RunEvent::new`, `RunEvent::message`, `RunEvent::error`, and `RunEvent::emit`;
- `lifecycle.rs` must not access `RunEvent.event` directly;
- `RunEvent.event` remains private;
- `RunEvent::request_id`, `RunEvent::queue_position`, `RunEvent::progress`, `RunEvent::delta`, and `RunEvent::chunk_summary` remain private to `report.rs`;
- `RunEvent` is not moved in this slice.

The implementation plan must include a source guard that searches `src-tauri/src/analysis/report/lifecycle.rs` for forbidden `RunEvent` usage:

```powershell
rg -n "RunEvent::(request_id|queue_position|progress|delta|chunk_summary)|\\.event" src-tauri/src/analysis/report/lifecycle.rs
```

Expected: no matches.

The implementation plan must also include source guards that protect the `RunEvent` surface in `report.rs`:

```powershell
rg -n "pub\\(super\\) struct RunEvent|pub\\(super\\) fn (new|message|error|emit)" src-tauri/src/analysis/report.rs
rg -n "pub\\(super\\) fn (request_id|queue_position|progress|delta|chunk_summary)" src-tauri/src/analysis/report.rs
```

Expected: the first command matches `RunEvent` plus exactly the four lifecycle-facing methods; the second command has no matches. This prevents widening the map/reduce event-builder API while making lifecycle compile.

`report.rs` must preserve the current facade paths after moving implementations into `lifecycle.rs`:

```rust
pub use self::lifecycle::cleanup_interrupted_analysis_runs;
pub(crate) use self::lifecycle::{mark_interrupted_analysis_runs, request_analysis_run_cancel};
```

This facade forwarding is required, not optional. The exact import grouping may differ, but these effective visibilities and paths must remain true:

- `analysis::cleanup_interrupted_analysis_runs` keeps working through the existing `analysis/mod.rs` root re-export;
- `analysis::report::cleanup_interrupted_analysis_runs` keeps working as the source of that root re-export;
- `analysis::report::request_analysis_run_cancel` keeps working for `analysis/report_commands.rs`;
- `analysis::report::mark_interrupted_analysis_runs` keeps working for the current inline report tests.

The implementation plan must include source guards for these paths:

```powershell
rg -n "pub use self::lifecycle::cleanup_interrupted_analysis_runs" src-tauri/src/analysis/report.rs
rg -n "pub\\(crate\\) use self::lifecycle::\\{mark_interrupted_analysis_runs, request_analysis_run_cancel\\}" src-tauri/src/analysis/report.rs
rg -n "pub use self::report::cleanup_interrupted_analysis_runs" src-tauri/src/analysis/mod.rs
rg -n "report::request_analysis_run_cancel" src-tauri/src/analysis/report_commands.rs
```

Expected: each command has at least one match after the refactor.

Expected production API changes outside `analysis::report`: none.

Expected root re-export changes: preserve the existing `pub use self::report::cleanup_interrupted_analysis_runs;` in `analysis/mod.rs`; do not add new root re-exports.

## Imports

`report/lifecycle.rs` should own imports needed by lifecycle helpers:

- `sqlx::{Pool, Sqlite}`
- `tauri::AppHandle`
- `crate::db::get_pool`
- `crate::error::{AppError, AppResult}`
- `crate::llm::{LlmSchedulerState}`
- `super::{RunEvent, INTERRUPTED_RUN_MESSAGE}`
- `super::super::state::AnalysisState` or equivalent existing facade import
- `super::super::store::{fetch_run_row, mark_run_capture_failed, sanitize_provider_error, set_run_status}`
- `super::super::{now_secs, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING}`

`report.rs` should remove imports that only moved lifecycle helpers used after extraction:

- `fetch_run_row`, if no remaining code in `report.rs` uses it;
- `mark_run_capture_failed`, if no remaining code in `report.rs` uses it;
- `sanitize_provider_error`, if no remaining code in `report.rs` uses it.

Keep in `report.rs` imports still needed by report pipeline and startup validation, including `get_pool`, `set_run_status`, `sanitize_snapshot_error`, `capture_run_snapshot`, and LLM scheduler/runtime types.

The implementation plan must verify unused imports after the move and keep the diff free of new warning debt in `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/lifecycle.rs`.

## Constants

Keep all existing report message constants in `report.rs` for this slice:

- make `INTERRUPTED_RUN_MESSAGE` `pub(super)` so `lifecycle.rs` can bind the same interrupted cleanup message while the existing inline report test can keep asserting the same constant;
- keep `CANCELLED_RUN_MESSAGE` private because `ReportPipelineContext::ensure_not_cancelled`, map/reduce cancellation paths, and pipeline startup checks use it inside `report.rs`;
- keep `SNAPSHOT_CAPTURE_FAILED_MESSAGE` private because `capture_report_corpus` uses it inside `report.rs`.

Do not duplicate these strings in `lifecycle.rs`.

## Data Flow

No runtime data flow changes:

1. `start_analysis_report_run` still spawns the report task and maps `ReportRunError` variants to the same lifecycle helpers.
2. `fail_run` still sanitizes provider/runtime errors with `sanitize_provider_error("Report run failed", &error)`.
3. `fail_run` still writes `ANALYSIS_STATUS_FAILED` through `set_run_status` and emits the same failed event.
4. `fail_capture_run` still writes snapshot capture failure through `mark_run_capture_failed` and emits the same failed event.
5. `cancel_run` still writes `ANALYSIS_STATUS_CANCELLED` through `set_run_status` and emits the same cancelled event.
6. `mark_interrupted_analysis_runs` still updates queued/running runs to cancelled with the same interrupted message and timestamp behavior.
7. `cleanup_interrupted_analysis_runs` still best-effort runs interruption cleanup when a pool can be resolved.
8. `request_analysis_run_cancel` still fetches the run, validates queued/running status, requests state cancellation, cancels scheduler requests, handles conflict cases, and emits the same progress event.

## Error Handling

Preserve current error behavior exactly:

- `request_analysis_run_cancel` keeps the same not-found message: `Analysis run {run_id} not found`;
- non-queued/non-running cancellation keeps the same conflict message: `Analysis run {run_id} is not queued or running`;
- inactive cancellation keeps the same conflict message: `Analysis run {run_id} is no longer active`;
- lifecycle status update failures remain best-effort and ignored in terminal helpers;
- cleanup pool resolution failures remain best-effort and ignored;
- no new error codes, messages, or user-facing strings are introduced.

The implementation plan must protect the `request_analysis_run_cancel` strings with explicit test assertions before moving lifecycle code. If no focused tests exist when the plan starts, add characterization tests in `report.rs` before the extraction commit. Required assertion coverage:

```rust
assert_eq!(error.message, format!("Analysis run {run_id} not found"));
assert_eq!(error.message, format!("Analysis run {run_id} is not queued or running"));
assert_eq!(error.message, format!("Analysis run {run_id} is no longer active"));
```

Use the repository's actual `AppError` fields/accessors in the test implementation; the strings above are the required literal values. The focused tests should exercise the existing `analysis::report::request_analysis_run_cancel` facade path, not a direct `lifecycle` module path.

## Non-Goals

This slice does not:

- move map/reduce runtime orchestration;
- move `ReportRunError`;
- move `RunEvent`;
- move `capture_report_corpus`;
- move `start_analysis_report_run`;
- split report tests into files;
- change prompts, JSON formats, request IDs, LLM metadata, scheduler behavior, or cancellation behavior;
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

If `src-tauri/src/analysis/report/lifecycle.rs` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/report/lifecycle.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/report/lifecycle.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/report/lifecycle.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/report/lifecycle.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/report/lifecycle.rs'
}
```

Do not stage pre-existing target-file changes into the lifecycle extraction commit.

The implementation plan must capture the pre-edit status output as `PRE_EDIT_STATUS` and compare final status against it after formatting, checks, and commit. After any `cargo fmt` run, require:

```powershell
git status --short --untracked-files=all
git diff --name-only
git diff --cached --name-only
git ls-files --others --exclude-standard
```

Expected implementation-owned changed paths are only:

- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/report/lifecycle.rs`

If `PRE_EDIT_STATUS` was clean, final `git status --short --untracked-files=all` must be clean after the refactor commit. If `PRE_EDIT_STATUS` contained unrelated pre-existing changes, final status must match those unrelated entries exactly and must not include new rustfmt drift or target-file changes.

## Testing

Run commands from the repository root.

Before editing, establish the current baseline:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

Before moving lifecycle code, the implementation plan must establish or add focused `request_analysis_run_cancel` characterization coverage. If these tests already exist, run them before editing. If they do not exist, add them in a small test-only step before extraction and run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::request_analysis_run_cancel_
```

Expected: PASS and not a green `0 tests` run. The output must include focused tests covering missing run, non-queued/non-running run, and inactive active-run cancellation, with string assertions for:

- `Analysis run {run_id} not found`
- `Analysis run {run_id} is not queued or running`
- `Analysis run {run_id} is no longer active`

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::interrupted_cleanup_preserves_captured_snapshot_state_marker
```

Expected: PASS with `1 passed`, not a green `0 tests` run. This is the focused guard for `mark_interrupted_analysis_runs`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. This guards recently split analysis corpus boundaries while report imports move again.

After editing and before committing, run the same report and corpus tests again. The post-change `analysis::report::tests::` run must be PASS and not a green `0 tests` run. The post-change lifecycle-focused tests must also be repeated explicitly:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::interrupted_cleanup_preserves_captured_snapshot_state_marker
```

Expected: PASS with `1 passed`, not a green `0 tests` run. This post-change check is required even if the broader `analysis::report::tests::` slice already passed.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::request_analysis_run_cancel_
```

Expected: PASS and not a green `0 tests` run. The output must include the focused cancellation request tests added or confirmed before extraction.

Also run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/lifecycle.rs` are not acceptable.

The implementation plan must also require checking that `src-tauri/src/analysis/report_commands.rs` still compiles through the existing `report::request_analysis_run_cancel` facade path. `cargo check --all-targets` is the required compile gate for this because `report_commands.rs` is not behaviorally changed by this slice.

## Commit Shape

Expected implementation-owned files:

- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/report/lifecycle.rs`

Expected implementation commit:

```text
refactor: extract analysis report lifecycle helpers
```

The design spec and implementation plan should be committed separately from the Rust refactor.
