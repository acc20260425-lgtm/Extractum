# Analysis Report Phases Refactor Design

**Date:** 2026-07-01
**Status:** implemented historical design as of 2026-07-01. Implemented by Rust refactor commit `4c2e87de`; execution record finalized in `d97dc9d1`.
**Scope:** internal Rust refactor of analysis report map/reduce phase runtime.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/report.rs` by extracting report map/reduce provider-phase execution into a focused private child module, without changing report execution behavior, event payloads, cancellation behavior, LLM scheduling, request building, persistence, snapshot capture, database schema, or Tauri command contracts.

This is the next conservative slice after extracting `analysis::report::requests`, `analysis::report::lifecycle`, and `analysis::report::capture`. It intentionally keeps the top-level pipeline and run startup orchestration in `report.rs` so the moved module owns only the provider-phase runtime.

## Current Shape

`src-tauri/src/analysis/report.rs` currently owns these provider-phase concerns:

- map phase dispatch across chunk requests;
- map worker progress, queued, started, failed, cancelled, and chunk summary events;
- reduce phase dispatch and streaming delta events;
- cancellation bridging between `CancellationToken`, `LlmSchedulerState`, and `AnalysisState`;
- ordered map result collection and missing chunk detection;
- `ReportPipelineContext` shared by map/reduce phases.

The phase cluster is currently:

- `finish_map_phase`
- `ReportPipelineContext`
- `ReducePhaseResult`
- `run_analysis_step_with_cancel`
- `run_map_phase`
- `run_reduce_phase`

Current consumers:

- `run_report_pipeline` calls `ReportPipelineContext`, `run_map_phase`, and `run_reduce_phase`;
- inline `report::tests` call `finish_map_phase` and `run_analysis_step_with_cancel` through the parent report module;
- `report/phases.rs` will consume request helpers from `report/requests.rs`;
- `report/lifecycle.rs` already consumes `RunEvent` and should not be changed by this slice.

## Proposed Architecture

Create a private child module declared from `src-tauri/src/analysis/report.rs`:

- `src-tauri/src/analysis/report/phases.rs`

Keep `src-tauri/src/analysis/report.rs` as the report workflow facade:

- add `mod phases;`;
- import phase items through private root imports:

```rust
use self::phases::{
    run_map_phase, run_reduce_phase, ReportPipelineContext,
};
#[cfg(test)]
use self::phases::{finish_map_phase, run_analysis_step_with_cancel};
```

- do not add any root re-export from `analysis/mod.rs`;
- keep `phases` private to `analysis::report`;
- keep `run_report_pipeline`, `start_analysis_report_run`, request preparation, persistence, capture, lifecycle, and startup cancellation handling in their current modules.

Move these items from `report.rs` to `report/phases.rs`:

- `finish_map_phase`
- `ReportPipelineContext`
- `ReducePhaseResult`
- `run_analysis_step_with_cancel`
- `run_map_phase`
- `run_reduce_phase`

Keep these items in `report.rs` for this slice:

- `StartAnalysisReportRequest`
- `resolve_analysis_telegram_history_scope`
- `ReportRunError`
- `RunEvent`
- `ReportRunInput`
- `validate_report_preflight`
- `run_report_pipeline`
- `start_analysis_report_run`
- all current inline tests.

The inline test module stays in `report.rs` for this slice. Moving phase tests can be a later test-only refactor if needed.

Current `report::tests` should keep using `super::finish_map_phase` and `super::run_analysis_step_with_cancel` through private root imports in `report.rs`; tests should not import `super::phases::...` directly in this slice.

## Visibility

`report/phases.rs` should expose only the helper surface consumed by `report.rs` and current inline tests:

```rust
pub(super) struct ReportPipelineContext {
    pub(super) handle: AppHandle,
    pub(super) pool: Pool<Sqlite>,
    pub(super) resolved_profile: ResolvedLlmProfile,
    pub(super) run_id: i64,
}

pub(super) struct ReducePhaseResult {
    pub(super) request_id: String,
    pub(super) completion: LlmCompletion,
}

impl ReportPipelineContext {
    pub(super) async fn ensure_not_cancelled(&self) -> Result<(), ReportRunError>;

    async fn cancel_children(&self);

    pub(super) fn emit(&self, event: RunEvent);
}

pub(super) fn finish_map_phase(
    ordered_summaries: Vec<Option<ChunkSummary>>,
    first_error: Option<ReportRunError>,
) -> Result<Vec<ChunkSummary>, ReportRunError>;

pub(super) async fn run_analysis_step_with_cancel<Fut, T>(
    cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> Result<T, LlmRequestError>
where
    Fut: Future<Output = Result<T, LlmRequestError>>;

pub(super) async fn run_map_phase(
    ctx: &ReportPipelineContext,
    chunks: Vec<Vec<CorpusMessage>>,
) -> Result<Vec<ChunkSummary>, ReportRunError>;

pub(super) async fn run_reduce_phase(
    ctx: &ReportPipelineContext,
    input: &ReportRunInput,
    chunk_summaries: &[ChunkSummary],
) -> Result<ReducePhaseResult, ReportRunError>;
```

`ReportRunError` remains defined in `report.rs` with its current private visibility. Because `report/phases.rs` is a child module of `report.rs`, it can construct all variants without widening the enum:

```rust
enum ReportRunError {
    Failed(String),
    CaptureFailed(String),
    Cancelled(String),
}
```

`ReportRunInput` remains defined in `report.rs` with its current private visibility. Because `report/phases.rs` is a child module of `report.rs`, it can read the private type and fields through `super::ReportRunInput` without widening the type. Do not move it and do not make it `pub(super)`, `pub(crate)`, or `pub` in this slice.

`CANCELLED_RUN_MESSAGE` remains defined in `report.rs` with private visibility. Because `report/phases.rs` is a child module of `report.rs`, it can read the private constant through `super::CANCELLED_RUN_MESSAGE` without widening it. Do not make it `pub(super)`, `pub(crate)`, or `pub`.

`RunEvent` remains defined in `report.rs`. `report/phases.rs` currently needs these builder methods:

- `RunEvent::new`
- `request_id`
- `queue_position`
- `message`
- `progress`
- `delta`
- `chunk_summary`
- `error`
- `emit`

Keep the current visibility wherever possible. Because `report/phases.rs` is a child module of `report.rs`, it can call private `RunEvent` builder methods without widening them. Do not make `RunEvent` or its methods `pub(crate)` or `pub`.

The implementation plan must include source guards:

```powershell
rg -n "^pub.*mod phases" src-tauri/src/analysis/report.rs
rg -n "^mod phases;" src-tauri/src/analysis/report.rs
rg -n "^\s*pub(\([^)]*\))?\s+enum ReportRunError" src-tauri/src/analysis/report.rs
rg -n "^enum ReportRunError" src-tauri/src/analysis/report.rs
rg -n "^\s*pub(\([^)]*\))?\s+struct ReportRunInput" src-tauri/src/analysis/report.rs
rg -n "^struct ReportRunInput" src-tauri/src/analysis/report.rs
rg -n "^\s*pub.*CANCELLED_RUN_MESSAGE" src-tauri/src/analysis/report.rs
rg -n "^const CANCELLED_RUN_MESSAGE" src-tauri/src/analysis/report.rs
rg -n "^    pub\(super\) async fn ensure_not_cancelled" src-tauri/src/analysis/report/phases.rs
rg -n "^    pub\(super\) fn emit" src-tauri/src/analysis/report/phases.rs
```

Expected: first, third, fifth, and seventh commands have no matches; `rg` exit code `1` is expected for those no-match guards. Second command prints exactly one private `mod phases;` line. Fourth command prints exactly one private `enum ReportRunError` line. Sixth command prints exactly one private `struct ReportRunInput` line. Eighth command prints exactly one private `const CANCELLED_RUN_MESSAGE` line. Ninth and tenth commands print the `pub(super)` `ReportPipelineContext` methods consumed by `report.rs`.

Expected production API changes outside `analysis::report`: none.

Expected root re-export changes: none.

## Imports

`report/phases.rs` should own imports needed by provider-phase runtime:

- `std::future::Future`
- `std::sync::atomic::{AtomicUsize, Ordering}`
- `std::sync::Arc`
- `sqlx::{Pool, Sqlite}`
- `tauri::{AppHandle, Manager}`
- `tokio::task::JoinSet`
- `tokio_util::sync::CancellationToken`
- `crate::llm::{run_llm_collect_with_profile, run_llm_stream_with_profile, LlmCompletion, LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState, ResolvedLlmProfile}`
- `super::super::models::{AnalysisChunkSummaryEvent, ChunkSummary, CorpusMessage}`
- `super::super::state::AnalysisState`
- `super::requests::{build_map_request, build_reduce_request, parse_chunk_summary, ReduceRequestParams}`
- `super::{ReportRunError, ReportRunInput, RunEvent, CANCELLED_RUN_MESSAGE}`

`report.rs` should remove imports that only moved phase helpers use after extraction:

- `std::future::Future`
- `std::sync::atomic::{AtomicUsize, Ordering}`
- `std::sync::Arc`
- `tokio::task::JoinSet`
- `tokio_util::sync::CancellationToken`, if no inline tests or remaining code still need the unqualified import;
- `run_llm_collect_with_profile`, `run_llm_stream_with_profile`, `LlmCompletion`, `LlmRequestError`, `LlmRequestKind`, `LlmRequestMetadata`, `LlmRequestPriority`, `LlmSchedulerState`, if no remaining code or inline tests still need them through root imports;
- `AnalysisChunkSummaryEvent`, if only phase events use it.

Keep in `report.rs` imports needed by request preparation, top-level pipeline, lifecycle, capture, trace persistence, and inline tests. Test-only imports can remain inside `#[cfg(test)] mod tests`.

The implementation plan must verify unused imports after the move and keep the diff free of new warning debt in `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/phases.rs`.

## Data Flow

No runtime data flow changes:

1. `run_report_pipeline` still performs cancellation checks, loads DB pool, marks run running, emits load/chunking events, captures corpus, chunks corpus, persists final status, and emits completed/persist events.
2. `run_report_pipeline` still creates `ReportPipelineContext` after chunking.
3. `run_map_phase` still dispatches one LLM map request per chunk through `LlmSchedulerState::run_request`.
4. Map requests still use `build_map_request` and parse completions through `parse_chunk_summary`.
5. Map phase still emits queued, started, progress, failed, cancelled, and `AnalysisChunkSummaryEvent` payloads with the same kind/phase/message/progress fields.
6. On the first map error or worker crash, `run_map_phase` still asks the context to cancel child LLM requests and waits for workers to finish before returning.
7. `finish_map_phase` still propagates the first map error before checking for missing summaries.
8. `run_reduce_phase` still builds the final reduce request through `build_reduce_request`.
9. Reduce phase still emits queued, started, streaming delta, failed, and cancelled events with the same kind/phase/message fields.
10. `run_analysis_step_with_cancel` still returns `LlmRequestError::Cancelled` when the token is already cancelled or becomes cancelled while awaiting the provider future.
11. `run_report_pipeline` still uses the returned `ReducePhaseResult` to build trace data and persist the completed run.

## Error Handling

Preserve current error behavior exactly:

- map parse failures still become `ReportRunError::Failed` with the parse error string;
- map provider failures still emit `"Chunk N of M failed."` and return `ReportRunError::Failed`;
- map cancellation still emits `"Chunk N of M cancelled."` and returns `ReportRunError::Cancelled("Analysis run cancelled.")`;
- missing map summaries still fail with `"Some chunk summaries were not collected"`;
- map worker join failures still fail with `"Chunk worker crashed: {error}"`;
- reduce provider failures still emit `"Final report generation failed."` and return `ReportRunError::Failed`;
- reduce cancellation still emits `"Final report generation cancelled."` and returns `ReportRunError::Cancelled("Analysis run cancelled.")`;
- no new error codes, messages, event kinds, phases, or user-facing strings are introduced.

The implementation plan must include source or test guards for these literals:

```powershell
rg -n '"Dispatching .* chunk analysis request|queued at position|Analyzing chunk|summarized\.|Chunk .* failed\.|Chunk .* cancelled\.|Some chunk summaries were not collected|Chunk worker crashed:|Writing final report|Final report queued at position|Final report generation failed\.|Final report generation cancelled\.|Analysis run cancelled\."' src-tauri/src/analysis/report/phases.rs
```

Expected: all provider-phase event and error literals are present in `report/phases.rs` after extraction. These source guards are required because this slice does not add new scheduler-backed runtime tests for `run_map_phase` or `run_reduce_phase`.

## Non-Goals

This slice does not:

- move `run_report_pipeline`;
- move `start_analysis_report_run`;
- move `ReportRunError`;
- move `RunEvent`;
- move request-building helpers out of `report/requests.rs`;
- move capture helpers;
- move lifecycle helpers;
- split report tests into files;
- change LLM request scheduling, priorities, request IDs, or provider profile resolution;
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

If `src-tauri/src/analysis/report/phases.rs` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/report/phases.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/report/phases.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/report/phases.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/report/phases.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/report/phases.rs'
}
```

Do not stage pre-existing target-file changes into the phases extraction commit.

The implementation plan must capture pre-edit status using a unique tag so repeated executions do not overwrite each other:

```powershell
$env:ANALYSIS_REPORT_PHASES_STATUS_TAG = "{0}-{1}" -f (Get-Date -Format 'yyyyMMddHHmmss'), $PID
$preEditStatusPath = Join-Path $env:TEMP "analysis-report-phases-$env:ANALYSIS_REPORT_PHASES_STATUS_TAG-pre-edit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preEditStatusPath
Get-Content -Raw -LiteralPath $preEditStatusPath
```

After formatting, checks, and commit, compare final status against the captured baseline.

## Testing

Run commands from the repository root.

Before editing, establish the current baseline:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::finish_map_phase
```

Expected: PASS and not a green `0 tests` run. This covers ordered map collection, missing summaries, and first-error propagation.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::run_analysis_step_with_cancel
```

Expected: PASS and not a green `0 tests` run. This covers cancellation helper behavior.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. This guards the full inline report test surface while phase helpers move.

This slice does not add new scheduler-backed runtime tests for `run_map_phase` or `run_reduce_phase`. That is an accepted risk for this move-only refactor because those paths require LLM scheduler/provider orchestration; the implementation must instead preserve the moved source byte-for-byte except for module paths/imports/visibility, run the full `analysis::report::tests::` slice, run `cargo check --all-targets`, and use the provider-phase event/error literal guards below.

After editing and before committing, run these commands explicitly:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::finish_map_phase
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::run_analysis_step_with_cancel
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

Also run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. This is a broad post-change regression gate, not a pre-edit characterization check; if it fails, inspect whether the failure is pre-existing or caused by this refactor before committing. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/phases.rs` are not acceptable.

The implementation plan must include source guards:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?async fn run_map_phase\(|^\s*(pub\([^)]*\)\s+|pub\s+)?async fn run_reduce_phase\(|^\s*(pub\([^)]*\)\s+|pub\s+)?async fn run_analysis_step_with_cancel\(|^\s*(pub\([^)]*\)\s+|pub\s+)?fn finish_map_phase\(|^\s*(pub\([^)]*\)\s+|pub\s+)?struct ReportPipelineContext|^\s*(pub\([^)]*\)\s+|pub\s+)?struct ReducePhaseResult" src-tauri/src/analysis/report.rs
rg -n "^pub\(super\) async fn run_map_phase\(|^pub\(super\) async fn run_reduce_phase\(|^pub\(super\) async fn run_analysis_step_with_cancel\(|^pub\(super\) fn finish_map_phase\(|^pub\(super\) struct ReportPipelineContext|^pub\(super\) struct ReducePhaseResult" src-tauri/src/analysis/report/phases.rs
rg -n "super::phases::finish_map_phase|super::phases::run_analysis_step_with_cancel" src-tauri/src/analysis/report.rs
```

Expected: first command has no matches; `rg` exit code `1` is expected for this no-match guard. Second command prints all moved phase items in `phases.rs`. Third command has no matches; inline tests should use root private imports, not direct child-module imports.

## Commit Shape

Expected implementation-owned files:

- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/report/phases.rs`

Expected implementation commit:

```text
refactor: extract analysis report phases
```

The design spec and implementation plan should be committed separately from the Rust refactor.
