# Analysis Report Phases Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract analysis report map/reduce provider-phase runtime from `src-tauri/src/analysis/report.rs` into a focused private child module without changing behavior.

**Architecture:** Add `src-tauri/src/analysis/report/phases.rs` as a private child module of `analysis::report`. Keep `report.rs` as the workflow facade: it still owns `run_report_pipeline`, `start_analysis_report_run`, `ReportRunError`, `RunEvent`, `ReportRunInput`, capture, lifecycle, persistence, and startup orchestration.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, Tokio, tokio-util cancellation tokens, Cargo.

## Global Constraints

- Run commands from the repository root.
- Use the `--manifest-path src-tauri/Cargo.toml` form for all Cargo commands from the repository root.
- Behavioral implementation-owned files are limited to `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/phases.rs`.
- Do not move `run_report_pipeline`, `start_analysis_report_run`, `ReportRunError`, `RunEvent`, `ReportRunInput`, request helpers, capture helpers, or lifecycle helpers.
- Do not change LLM scheduling, request kinds, priorities, request IDs, event payloads, cancellation behavior, snapshot persistence, database schema, migrations, frontend, Tauri command payloads, or event payloads.
- Do not add any root re-export from `analysis/mod.rs`.
- Keep `phases` private to `analysis::report`.
- Keep `ReportRunError`, `ReportRunInput`, and `CANCELLED_RUN_MESSAGE` private in `report.rs`.
- Do not make `RunEvent` or its methods `pub(crate)` or `pub`.
- Expected implementation commit message: `refactor: extract analysis report phases`.

---

## File Structure

- Modify: `src-tauri/src/analysis/report.rs`
  - Declare `mod phases;`.
  - Add private root imports for phase items.
  - Remove the moved phase runtime cluster.
  - Remove imports used only by the moved phase helpers.
  - Keep inline `report::tests` in this file and keep their `super::finish_map_phase` / `super::run_analysis_step_with_cancel` access through root private imports.
- Create: `src-tauri/src/analysis/report/phases.rs`
  - Own provider-phase runtime imports.
  - Define `ReportPipelineContext`, `ReducePhaseResult`, `finish_map_phase`, `run_analysis_step_with_cancel`, `run_map_phase`, and `run_reduce_phase`.

---

### Task 1: Baseline And Dirty Worktree Guard

**Files:**
- Read: `src-tauri/src/analysis/report.rs`
- Read if present: `src-tauri/src/analysis/report/phases.rs`
- No edits.

**Interfaces:**
- Consumes: approved design spec at `docs/superpowers/specs/2026-07-01-analysis-report-phases-refactor-design.md`.
- Produces: baseline status snapshot under `$env:TEMP` and pre-edit test evidence.

- [x] **Step 1: Capture pre-edit status**

Run:

```powershell
$env:ANALYSIS_REPORT_PHASES_STATUS_TAG = "{0}-{1}" -f (Get-Date -Format 'yyyyMMddHHmmss'), $PID
$preEditStatusPath = Join-Path $env:TEMP "analysis-report-phases-$env:ANALYSIS_REPORT_PHASES_STATUS_TAG-pre-edit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preEditStatusPath
Get-Content -Raw -LiteralPath $preEditStatusPath
```

Expected: review the output before editing. If the output is empty, the worktree baseline is clean. Keep `$env:ANALYSIS_REPORT_PHASES_STATUS_TAG` for later status comparisons in this plan run.

- [x] **Step 2: Inspect target-file drift if present**

Run:

```powershell
git diff -- src-tauri/src/analysis/report.rs
```

Expected: no output unless `report.rs` already had pre-existing user changes. If there is output, stop and decide how to isolate it before editing.

Run:

```powershell
git diff --cached -- src-tauri/src/analysis/report.rs
```

Expected: no output unless `report.rs` already had staged pre-existing changes. If there is output, stop and decide how to isolate it before editing.

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/report/phases.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/report/phases.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/report/phases.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/report/phases.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/report/phases.rs'
}
```

Expected: no output today because `phases.rs` should not exist before implementation. If it exists, inspect it and stop before overwriting it.

- [x] **Step 3: Run focused map-finish baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::finish_map_phase
```

Expected: PASS and not a green `0 tests` run. This covers ordered map collection, missing summaries, and first-error propagation.

- [x] **Step 4: Run focused cancellation-helper baseline**

Execution adjustment: the planned selector `analysis::report::tests::run_analysis_step_with_cancel` matched 0 tests because Cargo filters by test name, not helper function usage. The actual focused cancellation tests are `analysis_step_cancel_wrapper_*`; `cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::analysis_step_cancel_wrapper` ran 2 tests and passed.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::run_analysis_step_with_cancel
```

Expected: PASS and not a green `0 tests` run. This covers cancellation wrapper behavior.

- [x] **Step 5: Run full report test baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. If this fails, stop and record the pre-existing failure before editing.

---

### Task 2: Extract Provider Phase Module

**Files:**
- Modify: `src-tauri/src/analysis/report.rs`
- Create: `src-tauri/src/analysis/report/phases.rs`

**Interfaces:**
- Consumes from `report.rs`:
  - `enum ReportRunError { Failed(String), CaptureFailed(String), Cancelled(String) }`
  - `struct ReportRunInput`
  - `struct RunEvent`
  - `const CANCELLED_RUN_MESSAGE: &str`
- Produces in `phases.rs`:
  - `pub(super) struct ReportPipelineContext`
  - `pub(super) struct ReducePhaseResult`
  - `pub(super) fn finish_map_phase(ordered_summaries, first_error)`
  - `pub(super) async fn run_analysis_step_with_cancel(cancellation_token, future)`
  - `pub(super) async fn run_map_phase(ctx, chunks)`
  - `pub(super) async fn run_reduce_phase(ctx, input, chunk_summaries)`

- [x] **Step 1: Create `phases.rs` with imports and moved items**

Create `src-tauri/src/analysis/report/phases.rs` by moving the existing implementations of these items from `src-tauri/src/analysis/report.rs`:

```rust
fn finish_map_phase(
    ordered_summaries: Vec<Option<ChunkSummary>>,
    first_error: Option<ReportRunError>,
) -> Result<Vec<ChunkSummary>, ReportRunError>
```

```rust
struct ReportPipelineContext {
    handle: AppHandle,
    pool: Pool<Sqlite>,
    resolved_profile: ResolvedLlmProfile,
    run_id: i64,
}
```

```rust
struct ReducePhaseResult {
    request_id: String,
    completion: LlmCompletion,
}
```

```rust
async fn run_analysis_step_with_cancel<Fut, T>(
    cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> Result<T, LlmRequestError>
where
    Fut: Future<Output = Result<T, LlmRequestError>>
```

```rust
async fn run_map_phase(
    ctx: &ReportPipelineContext,
    chunks: Vec<Vec<CorpusMessage>>,
) -> Result<Vec<ChunkSummary>, ReportRunError>
```

```rust
async fn run_reduce_phase(
    ctx: &ReportPipelineContext,
    input: &ReportRunInput,
    chunk_summaries: &[ChunkSummary],
) -> Result<ReducePhaseResult, ReportRunError>
```

Use this import block at the top of `phases.rs`:

```rust
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::llm::{
    run_llm_collect_with_profile, run_llm_stream_with_profile, LlmCompletion, LlmRequestError,
    LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState,
    ResolvedLlmProfile,
};

use super::super::models::{AnalysisChunkSummaryEvent, ChunkSummary, CorpusMessage};
use super::super::state::AnalysisState;
use super::requests::{
    build_map_request, build_reduce_request, parse_chunk_summary, ReduceRequestParams,
};
use super::{ReportRunError, ReportRunInput, RunEvent, CANCELLED_RUN_MESSAGE};
```

Expected: moved function bodies are preserved byte-for-byte except for visibility, imports, module paths, and rustfmt.

- [x] **Step 2: Apply required visibility in `phases.rs`**

Update the moved declarations in `phases.rs` to this visibility shape:

```rust
pub(super) struct ReportPipelineContext {
    pub(super) handle: AppHandle,
    pub(super) pool: Pool<Sqlite>,
    pub(super) resolved_profile: ResolvedLlmProfile,
    pub(super) run_id: i64,
}

impl ReportPipelineContext {
    pub(super) async fn ensure_not_cancelled(&self) -> Result<(), ReportRunError> {
        if self
            .handle
            .state::<AnalysisState>()
            .is_report_run_cancelled(self.run_id)
            .await
        {
            return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
        }

        Ok(())
    }

    async fn cancel_children(&self) {
        self.handle
            .state::<LlmSchedulerState>()
            .cancel_run_requests(self.run_id)
            .await;
    }

    pub(super) fn emit(&self, event: RunEvent) {
        event.emit(&self.handle);
    }
}

pub(super) struct ReducePhaseResult {
    pub(super) request_id: String,
    pub(super) completion: LlmCompletion,
}
```

Also make these moved helpers `pub(super)`:

```rust
pub(super) fn finish_map_phase(
    ordered_summaries: Vec<Option<ChunkSummary>>,
    first_error: Option<ReportRunError>,
) -> Result<Vec<ChunkSummary>, ReportRunError>
```

```rust
pub(super) async fn run_analysis_step_with_cancel<Fut, T>(
    cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> Result<T, LlmRequestError>
where
    Fut: Future<Output = Result<T, LlmRequestError>>
```

```rust
pub(super) async fn run_map_phase(
    ctx: &ReportPipelineContext,
    chunks: Vec<Vec<CorpusMessage>>,
) -> Result<Vec<ChunkSummary>, ReportRunError>
```

```rust
pub(super) async fn run_reduce_phase(
    ctx: &ReportPipelineContext,
    input: &ReportRunInput,
    chunk_summaries: &[ChunkSummary],
) -> Result<ReducePhaseResult, ReportRunError>
```

Expected: parent `report.rs` can construct `ReportPipelineContext`, call `ensure_not_cancelled`, call `emit`, and read `ctx.pool`, `reduce_result.request_id`, and `reduce_result.completion`.

- [x] **Step 3: Declare and import the phases module from `report.rs`**

In `src-tauri/src/analysis/report.rs`, update the module declarations to include `phases`:

```rust
mod capture;
mod lifecycle;
mod phases;
mod requests;
```

Add private root imports after the existing capture import:

```rust
use self::capture::capture_report_corpus;
use self::phases::{run_map_phase, run_reduce_phase, ReportPipelineContext};
#[cfg(test)]
use self::phases::{finish_map_phase, run_analysis_step_with_cancel};
```

Expected: production code uses `run_map_phase`, `run_reduce_phase`, and `ReportPipelineContext` through root private imports. Inline tests keep using `super::finish_map_phase` and `super::run_analysis_step_with_cancel`.

- [x] **Step 4: Remove moved phase items from `report.rs`**

Delete these complete items from `src-tauri/src/analysis/report.rs` after they have been moved to `phases.rs`. For each item, delete from the shown signature or type declaration through its matching closing brace.

```rust
fn finish_map_phase(
    ordered_summaries: Vec<Option<ChunkSummary>>,
    first_error: Option<ReportRunError>,
) -> Result<Vec<ChunkSummary>, ReportRunError>
```

```rust
struct ReportPipelineContext
```

```rust
impl ReportPipelineContext
```

```rust
struct ReducePhaseResult
```

```rust
async fn run_analysis_step_with_cancel<Fut, T>(
    cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> Result<T, LlmRequestError>
where
    Fut: Future<Output = Result<T, LlmRequestError>>
```

```rust
async fn run_map_phase(
    ctx: &ReportPipelineContext,
    chunks: Vec<Vec<CorpusMessage>>,
) -> Result<Vec<ChunkSummary>, ReportRunError>
```

```rust
async fn run_reduce_phase(
    ctx: &ReportPipelineContext,
    input: &ReportRunInput,
    chunk_summaries: &[ChunkSummary],
) -> Result<ReducePhaseResult, ReportRunError>
```

Expected: `report.rs` still contains `ReportRunInput`, `validate_report_preflight`, `run_report_pipeline`, and `start_analysis_report_run`.

- [x] **Step 5: Remove phase-only imports from `report.rs`**

Update the top of `src-tauri/src/analysis/report.rs` by removing these root imports when they are no longer used outside `phases.rs`:

```rust
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::task::JoinSet;
```

Update the `crate::llm` import in `report.rs` from the current broader set to keep only report-start/pipeline/test-needed items:

```rust
use crate::llm::{
    resolve_effective_model, resolve_model_input_token_limit_for_backend,
    resolve_profile_for_backend, ResolvedLlmProfile,
};
```

If inline tests still need `LlmRequestError`, `LlmSchedulerState`, or `ProviderKind`, import them inside `#[cfg(test)] mod tests` as they are today:

```rust
use crate::llm::{LlmRequestError, LlmSchedulerState, ProviderKind, ResolvedLlmProfile};
```

Keep `AnalysisChunkSummaryEvent` in the `super::models` import because `RunEvent::chunk_summary` remains in `report.rs`. Update the import to keep the current model set:

```rust
use super::models::{
    AnalysisChunkSummaryEvent, AnalysisPromptTemplate, AnalysisRunEvent, ChunkSummary,
    CorpusMessage,
};
```

Keep `tokio_util::sync::CancellationToken` at the root only if production code still uses it. If only tests need it, import it inside `#[cfg(test)] mod tests` as today.

Expected: no new unused-import warnings in `report.rs` or `phases.rs`. `AnalysisChunkSummaryEvent` remains used by `RunEvent::chunk_summary` in `report.rs`.

- [x] **Step 6: Preserve non-moved private contracts in `report.rs`**

Confirm these definitions remain in `src-tauri/src/analysis/report.rs`:

```rust
const CANCELLED_RUN_MESSAGE: &str = "Analysis run cancelled.";
```

```rust
enum ReportRunError {
    Failed(String),
    CaptureFailed(String),
    Cancelled(String),
}
```

```rust
struct ReportRunInput {
    run_id: i64,
    scope_label: String,
    corpus_request: CorpusLoadRequest,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template: AnalysisPromptTemplate,
    model_override: Option<String>,
    resolved_profile: ResolvedLlmProfile,
    chunk_target_chars: usize,
    preflight: AnalysisRunPreflight,
}
```

Expected: these items are not moved and are not widened to `pub(super)`, `pub(crate)`, or `pub`.

---

### Task 3: Source Guards And Focused Verification

**Files:**
- Verify: `src-tauri/src/analysis/report.rs`
- Verify: `src-tauri/src/analysis/report/phases.rs`

**Interfaces:**
- Consumes: extraction from Task 2.
- Produces: proof that moved items and visibility landed in the intended modules.

Scheduler-backed runtime tests for `run_map_phase` and `run_reduce_phase` are intentionally not added in this move-only refactor. Because of that accepted risk, preserve the moved source byte-for-byte except for imports, module paths, visibility, and rustfmt, and treat the source guards in this task as required verification rather than optional smoke checks.

- [x] **Step 1: Verify moved item locations**

Run:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?async fn run_map_phase\(|^\s*(pub\([^)]*\)\s+|pub\s+)?async fn run_reduce_phase\(|^\s*(pub\([^)]*\)\s+|pub\s+)?async fn run_analysis_step_with_cancel(<[^>]+>)?\(|^\s*(pub\([^)]*\)\s+|pub\s+)?fn finish_map_phase\(|^\s*(pub\([^)]*\)\s+|pub\s+)?struct ReportPipelineContext|^\s*(pub\([^)]*\)\s+|pub\s+)?struct ReducePhaseResult" src-tauri/src/analysis/report.rs
```

Expected: no matches. `rg` exit code `1` is expected for this no-match guard.

Run:

```powershell
rg -n "^pub\(super\) async fn run_map_phase\(|^pub\(super\) async fn run_reduce_phase\(|^pub\(super\) async fn run_analysis_step_with_cancel(<[^>]+>)?\(|^pub\(super\) fn finish_map_phase\(|^pub\(super\) struct ReportPipelineContext|^pub\(super\) struct ReducePhaseResult" src-tauri/src/analysis/report/phases.rs
```

Expected: prints all moved phase items in `phases.rs`.

- [x] **Step 2: Verify private module and parent contracts**

Run:

```powershell
rg -n "^pub.*mod phases" src-tauri/src/analysis/report.rs
```

Expected: no matches. `rg` exit code `1` is expected for this no-match guard.

Run:

```powershell
rg -n "^mod phases;" src-tauri/src/analysis/report.rs
```

Expected: exactly one private `mod phases;` line.

Run:

```powershell
rg -n "^\s*pub(\([^)]*\))?\s+enum ReportRunError|^\s*pub(\([^)]*\))?\s+struct ReportRunInput|^\s*pub.*CANCELLED_RUN_MESSAGE" src-tauri/src/analysis/report.rs
```

Expected: no matches. `rg` exit code `1` is expected for this no-match guard.

Run:

```powershell
rg -n "^enum ReportRunError|^struct ReportRunInput|^const CANCELLED_RUN_MESSAGE" src-tauri/src/analysis/report.rs
```

Expected: exactly one match for each private parent item.

- [x] **Step 3: Verify `ReportPipelineContext` and `ReducePhaseResult` parent-visible fields**

Run:

```powershell
rg -n "^    pub\(super\) handle: AppHandle|^    pub\(super\) pool: Pool<Sqlite>|^    pub\(super\) resolved_profile: ResolvedLlmProfile|^    pub\(super\) run_id: i64|^    pub\(super\) request_id: String|^    pub\(super\) completion: LlmCompletion" src-tauri/src/analysis/report/phases.rs
```

Expected: prints all six parent-visible fields needed by `report.rs`.

Run:

```powershell
rg -n "^    pub\(super\) async fn ensure_not_cancelled|^    pub\(super\) fn emit" src-tauri/src/analysis/report/phases.rs
```

Expected: prints both parent-visible `ReportPipelineContext` methods consumed by `report.rs`.

- [x] **Step 4: Verify tests use root private imports**

Run:

```powershell
rg -n "super::phases::finish_map_phase|super::phases::run_analysis_step_with_cancel" src-tauri/src/analysis/report.rs
```

Expected: no matches. `rg` exit code `1` is expected for this no-match guard.

- [x] **Step 5: Verify provider-phase event and error strings moved intact**

Run:

```powershell
rg -n '"Dispatching .* chunk analysis request|queued at position|Analyzing chunk|summarized\.|Chunk .* failed\.|Chunk .* cancelled\.|Some chunk summaries were not collected|Chunk worker crashed:|Writing final report|Final report queued at position|Final report generation failed\.|Final report generation cancelled\."|CANCELLED_RUN_MESSAGE' src-tauri/src/analysis/report/phases.rs
```

Expected: prints provider-phase event/error literals and `CANCELLED_RUN_MESSAGE` references in `phases.rs`. The literal `"Analysis run cancelled."` remains only in the private `CANCELLED_RUN_MESSAGE` const in `report.rs`.

- [x] **Step 6: Verify provider request kinds moved intact**

Run:

```powershell
rg -n "LlmRequestKind::AnalysisReportMap|LlmRequestKind::AnalysisReportReduce" src-tauri/src/analysis/report/phases.rs
```

Expected: prints both request kinds in `phases.rs`.

- [x] **Step 7: Run focused map-finish tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::finish_map_phase
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 8: Run focused cancellation-helper tests**

Execution adjustment: as in Task 1, the non-empty focused selector is `analysis::report::tests::analysis_step_cancel_wrapper`; it ran 2 tests and passed.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::run_analysis_step_with_cancel
```

Expected: PASS and not a green `0 tests` run.

---

### Task 4: Full Verification, Format, And Commit

**Files:**
- Verify: `src-tauri/src/analysis/report.rs`
- Verify: `src-tauri/src/analysis/report/phases.rs`
- Commit: only the two implementation-owned Rust files.

**Interfaces:**
- Consumes: Task 3 source guard and focused test evidence.
- Produces: refactor commit `refactor: extract analysis report phases`.

- [ ] **Step 1: Run full report test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 2: Check formatting**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

If this fails, run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Then run:

```powershell
git status --short --untracked-files=all
```

Expected: only `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/phases.rs` have new implementation-owned formatting changes. If rustfmt touched unrelated files, stop and resolve that drift outside this refactor commit.

After any format fixes, rerun:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output before continuing to `cargo check`, staging, or commit.

- [ ] **Step 3: Check all Rust targets**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. This is a broad post-change regression gate, not a pre-edit characterization check. Existing warnings outside touched files may remain. New warnings mentioning `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/phases.rs` are not acceptable.

- [ ] **Step 4: Inspect implementation diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/phases.rs
```

Expected: diff contains only the phase extraction, visibility needed by the parent module, module declaration/imports, and import cleanup.

Run:

```powershell
git status --short --untracked-files=all
```

Expected: status contains only implementation-owned files plus any pre-existing unrelated baseline entries captured in Task 1.

- [ ] **Step 5: Compare pre-commit status against baseline**

Run:

```powershell
$preEditStatusPath = Join-Path $env:TEMP "analysis-report-phases-$env:ANALYSIS_REPORT_PHASES_STATUS_TAG-pre-edit-status.txt"
$preCommitStatusPath = Join-Path $env:TEMP "analysis-report-phases-$env:ANALYSIS_REPORT_PHASES_STATUS_TAG-pre-commit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preCommitStatusPath
Compare-Object `
    (Get-Content -LiteralPath $preEditStatusPath) `
    (Get-Content -LiteralPath $preCommitStatusPath)
```

Expected: the only intentional differences from baseline are `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/phases.rs`. If any unrelated rustfmt drift or unexpected file appears, stop before committing.

- [ ] **Step 6: Stage only implementation-owned files**

Run:

```powershell
git add -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/phases.rs
```

Expected: command exits successfully.

Run:

```powershell
git diff --cached --stat
```

Expected: staged stat contains only `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/phases.rs`.

Run:

```powershell
git diff --cached --check
```

Expected: no output. If it reports any issue, stop before commit.

- [ ] **Step 7: Commit the Rust refactor**

Run:

```powershell
git commit -m "refactor: extract analysis report phases"
```

Expected: commit succeeds and includes only the two implementation-owned Rust files.

- [ ] **Step 8: Compare final status against baseline**

Run:

```powershell
$preEditStatusPath = Join-Path $env:TEMP "analysis-report-phases-$env:ANALYSIS_REPORT_PHASES_STATUS_TAG-pre-edit-status.txt"
$finalStatusPath = Join-Path $env:TEMP "analysis-report-phases-$env:ANALYSIS_REPORT_PHASES_STATUS_TAG-final-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $finalStatusPath
Compare-Object `
    (Get-Content -LiteralPath $preEditStatusPath) `
    (Get-Content -LiteralPath $finalStatusPath)
```

Expected: if the pre-edit status was clean, `Compare-Object` has no output. If the pre-edit status had unrelated entries, any output is explained and does not include phase refactor files or new rustfmt drift.

Run:

```powershell
git log --oneline -1
```

Expected: latest commit is `refactor: extract analysis report phases`.

---

## Self-Review

- Spec coverage: the plan covers private `phases.rs`, no root re-export, preserved private parent contracts, moved phase items, parent-visible context/result fields, event/error literal guards, pre-edit status snapshot, focused tests, full report tests, `fmt --check`, and `cargo check --all-targets`.
- Placeholder scan: no placeholder tokens or open-ended test steps are present.
- Type consistency: helper signatures and field names match the current `report.rs` source and the approved phases design.
