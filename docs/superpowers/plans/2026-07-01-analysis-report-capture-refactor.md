# Analysis Report Capture Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract analysis report corpus capture and snapshot-freezing logic from `src-tauri/src/analysis/report.rs` into a focused private nested module without changing runtime behavior.

**Status:** Implemented historical execution record as of 2026-07-01. Rust refactor commit: `5db04dea`; execution record finalized in `4a168963`.

**Architecture:** Add `src-tauri/src/analysis/report/capture.rs` as a private child module of `analysis::report`. Keep `report.rs` as the workflow facade and expose `capture_report_corpus` to the parent and existing inline tests through a private root import only.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, Tokio tests, Cargo.

## Global Constraints

- Run commands from the repository root.
- Use `cargo test --manifest-path src-tauri/Cargo.toml ...`, `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, and `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`.
- Behavioral implementation-owned files are limited to `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/capture.rs`.
- Do not move map/reduce runtime orchestration, lifecycle helpers, request helpers, `ReportRunError`, `RunEvent`, `run_report_pipeline`, or `start_analysis_report_run`.
- Do not change snapshot persistence behavior, database schema, migrations, frontend, Tauri command payloads, or event payloads.
- Do not add any root re-export from `analysis/mod.rs`.
- Keep `ReportRunError` private in `report.rs`; do not widen it to `pub(super)`, `pub(crate)`, or `pub`.
- Keep `capture` private to `analysis::report`.
- Preserve error strings exactly: `Corpus preload failed` and `Snapshot capture failed`.
- Do not stage pre-existing target-file changes into the capture extraction commit.
- Expected implementation commit message: `refactor: extract analysis report capture helper`.

---

## File Structure

- Modify: `src-tauri/src/analysis/report.rs`
  - Declare `mod capture;`.
  - Add private root import `use self::capture::capture_report_corpus;`.
  - Remove `SNAPSHOT_CAPTURE_FAILED_MESSAGE` and the `capture_report_corpus` function body.
  - Remove imports used only by the moved capture helper.
  - Keep inline `report::tests` in this file and keep their `super::capture_report_corpus` access unchanged through the private root import.
- Create: `src-tauri/src/analysis/report/capture.rs`
  - Own capture-only imports.
  - Define private `SNAPSHOT_CAPTURE_FAILED_MESSAGE`.
  - Define `pub(super) async fn capture_report_corpus(...)`.

---

### Task 1: Baseline And Dirty Worktree Guard

**Files:**
- Read: `src-tauri/src/analysis/report.rs`
- Read if present: `src-tauri/src/analysis/report/capture.rs`
- No edits.

**Interfaces:**
- Consumes: approved design spec at `docs/superpowers/specs/2026-07-01-analysis-report-capture-refactor-design.md`.
- Produces: baseline status snapshot under `$env:TEMP` and pre-edit test evidence.

- [x] **Step 1: Capture pre-edit status**

Run:

```powershell
$env:ANALYSIS_REPORT_CAPTURE_STATUS_TAG = "{0}-{1}" -f (Get-Date -Format 'yyyyMMddHHmmss'), $PID
$preEditStatusPath = Join-Path $env:TEMP "analysis-report-capture-$env:ANALYSIS_REPORT_CAPTURE_STATUS_TAG-pre-edit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preEditStatusPath
Get-Content -Raw -LiteralPath $preEditStatusPath
```

Expected: review the output before editing. If the output is empty, the worktree baseline is clean. Keep `$env:ANALYSIS_REPORT_CAPTURE_STATUS_TAG` for later status comparisons in this plan run.

- [x] **Step 2: Inspect target-file drift if present**

Run:

```powershell
git diff -- src-tauri/src/analysis/report.rs
git diff --cached -- src-tauri/src/analysis/report.rs
if (Test-Path -LiteralPath 'src-tauri/src/analysis/report/capture.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/report/capture.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/report/capture.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/report/capture.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/report/capture.rs'
}
```

Expected: if `report.rs` or `report/capture.rs` already has user changes, stop and decide how to isolate them before editing. Do not overwrite or stage pre-existing target-file changes.

- [x] **Step 3: Run focused capture baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
```

Expected: PASS with `1 passed`, not a green `0 tests` run. If this fails, stop because the capture behavior baseline is already red.

- [x] **Step 4: Run full report test baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. If this fails, stop and record the pre-existing failure before editing.

- [x] **Step 5: Run corpus boundary baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. This protects the recently split corpus facade while capture imports move. If this fails, stop and record the pre-existing failure before editing.

---

### Task 2: Extract Capture Module

**Files:**
- Modify: `src-tauri/src/analysis/report.rs`
- Create: `src-tauri/src/analysis/report/capture.rs`

**Interfaces:**
- Consumes from `report.rs`:
  - `enum ReportRunError { Failed(String), CaptureFailed(String), Cancelled(String) }`
  - `CorpusLoadRequest`
  - `CorpusMessage`
- Produces:
  - `pub(super) async fn capture_report_corpus(pool: &Pool<Sqlite>, run_id: i64, scope_label: &str, request: &CorpusLoadRequest) -> Result<Vec<CorpusMessage>, ReportRunError>`
  - private constant `SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed"`

- [x] **Step 1: Create `capture.rs` with the moved helper**

Create `src-tauri/src/analysis/report/capture.rs` with this content:

```rust
use sqlx::{Pool, Sqlite};

use super::super::corpus::{load_corpus_messages, CorpusLoadRequest};
use super::super::models::CorpusMessage;
use super::super::store::{capture_run_snapshot, sanitize_snapshot_error};
use super::ReportRunError;

const SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed";

pub(super) async fn capture_report_corpus(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    request: &CorpusLoadRequest,
) -> Result<Vec<CorpusMessage>, ReportRunError> {
    let corpus = load_corpus_messages(pool, request).await.map_err(|error| {
        ReportRunError::CaptureFailed(sanitize_snapshot_error(
            "Corpus preload failed",
            &error.to_string(),
        ))
    })?;

    if corpus.is_empty() {
        return Err(ReportRunError::CaptureFailed(
            SNAPSHOT_CAPTURE_FAILED_MESSAGE.to_string(),
        ));
    }

    capture_run_snapshot(pool, run_id, scope_label, &corpus)
        .await
        .map_err(|error| {
            ReportRunError::CaptureFailed(sanitize_snapshot_error(
                SNAPSHOT_CAPTURE_FAILED_MESSAGE,
                &error.to_string(),
            ))
        })
}
```

Expected: `capture.rs` owns all imports needed by the capture helper and does not expose the module outside `analysis::report`.

- [x] **Step 2: Declare and import the capture module from `report.rs`**

In `src-tauri/src/analysis/report.rs`, update the module/import block near existing `mod lifecycle;` and `mod requests;` to include:

```rust
mod capture;
mod lifecycle;
mod requests;

use self::capture::capture_report_corpus;
```

Keep the existing lifecycle and requests imports below it:

```rust
pub use self::lifecycle::cleanup_interrupted_analysis_runs;
#[cfg(test)]
use self::lifecycle::request_analysis_run_cancel_for_pool;
use self::lifecycle::{cancel_run, fail_capture_run, fail_run};
#[allow(unused_imports)]
pub(crate) use self::lifecycle::{mark_interrupted_analysis_runs, request_analysis_run_cancel};
use self::requests::{
    build_map_request, build_reduce_request, chunk_messages,
    chunk_target_chars_for_model_input_limit, parse_chunk_summary, ReduceRequestParams,
};
```

Expected: `capture_report_corpus` is available to `run_report_pipeline` and the inline `report::tests` through `super::capture_report_corpus`.

- [x] **Step 3: Remove moved constant and helper from `report.rs`**

Delete this constant from `report.rs`:

```rust
const SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed";
```

Delete the whole `capture_report_corpus` function from `report.rs`:

```rust
async fn capture_report_corpus(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    request: &CorpusLoadRequest,
) -> Result<Vec<CorpusMessage>, ReportRunError> {
    let corpus = load_corpus_messages(pool, request).await.map_err(|error| {
        ReportRunError::CaptureFailed(sanitize_snapshot_error(
            "Corpus preload failed",
            &error.to_string(),
        ))
    })?;

    if corpus.is_empty() {
        return Err(ReportRunError::CaptureFailed(
            SNAPSHOT_CAPTURE_FAILED_MESSAGE.to_string(),
        ));
    }

    capture_run_snapshot(pool, run_id, scope_label, &corpus)
        .await
        .map_err(|error| {
            ReportRunError::CaptureFailed(sanitize_snapshot_error(
                SNAPSHOT_CAPTURE_FAILED_MESSAGE,
                &error.to_string(),
            ))
        })
}
```

Expected: `report.rs` no longer defines the capture constant or capture helper directly.

- [x] **Step 4: Remove capture-only imports from `report.rs`**

Update the `super::corpus` import in `report.rs` from:

```rust
use super::corpus::{
    load_corpus_messages, preflight_analysis_run, preflight_limit_error, resolve_analysis_sources,
    AnalysisRunPreflight, AnalysisRunPreflightLimits, AnalysisSourceResolutionError,
    CorpusLoadRequest, YoutubeCorpusMode,
};
```

to:

```rust
use super::corpus::{
    preflight_analysis_run, preflight_limit_error, resolve_analysis_sources, AnalysisRunPreflight,
    AnalysisRunPreflightLimits, AnalysisSourceResolutionError, CorpusLoadRequest,
    YoutubeCorpusMode,
};
```

Update the `super::store` import in `report.rs` from:

```rust
use super::store::{
    capture_run_snapshot, fetch_prompt_template, fetch_source_group, find_active_duplicate_run,
    insert_analysis_run, sanitize_snapshot_error, set_run_status, AnalysisRunInsert,
    DuplicateRunLookup,
};
```

to:

```rust
use super::store::{
    fetch_prompt_template, fetch_source_group, find_active_duplicate_run, insert_analysis_run,
    set_run_status, AnalysisRunInsert, DuplicateRunLookup,
};
```

Expected: `report.rs` keeps only imports it still uses after extraction.

- [x] **Step 5: Preserve inline test access**

Confirm the existing `#[cfg(test)] mod tests` import block in `report.rs` still includes `capture_report_corpus` from the parent module:

```rust
use super::{
    build_map_request, build_reduce_request, capture_report_corpus,
    chunk_target_chars_for_model_input_limit, finish_map_phase, mark_interrupted_analysis_runs,
    parse_chunk_summary, request_analysis_run_cancel_for_pool,
    resolve_analysis_telegram_history_scope, run_analysis_step_with_cancel,
    validate_report_preflight, ReduceRequestParams, ReportRunError, ReportRunInput,
    StartAnalysisReportRequest,
};
```

Expected: tests do not import `super::capture::capture_report_corpus` directly.

Run this command to verify that boundary mechanically:

```powershell
rg -n "super::capture::capture_report_corpus" src-tauri/src/analysis/report.rs
```

Expected: no matches. `rg` exit code `1` is expected for this no-match guard.

---

### Task 3: Source Guards And Focused Verification

**Files:**
- Verify: `src-tauri/src/analysis/report.rs`
- Verify: `src-tauri/src/analysis/report/capture.rs`

**Interfaces:**
- Consumes: extraction from Task 2.
- Produces: proof that the moved items are in the intended module and no visibility was widened.

- [x] **Step 1: Verify moved item locations**

Run:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?async fn capture_report_corpus\(|^const SNAPSHOT_CAPTURE_FAILED_MESSAGE" src-tauri/src/analysis/report.rs
rg -n "^pub\(super\) async fn capture_report_corpus\(|^const SNAPSHOT_CAPTURE_FAILED_MESSAGE" src-tauri/src/analysis/report/capture.rs
```

Expected: first command has no matches. `rg` exit code `1` is expected for this no-match guard. Second command prints both the `pub(super) async fn capture_report_corpus(` line and the `const SNAPSHOT_CAPTURE_FAILED_MESSAGE` line.

- [x] **Step 2: Verify the capture module stays private**

Run:

```powershell
rg -n "^pub.*mod capture" src-tauri/src/analysis/report.rs
rg -n "^mod capture;" src-tauri/src/analysis/report.rs
```

Expected: first command has no matches. `rg` exit code `1` is expected for this no-match guard. Second command prints exactly one `mod capture;` line.

- [x] **Step 3: Verify `ReportRunError` stayed private and in `report.rs`**

Run:

```powershell
rg -n "^\s*pub(\([^)]*\))?\s+enum ReportRunError" src-tauri/src/analysis/report.rs
rg -n "^enum ReportRunError" src-tauri/src/analysis/report.rs
```

Expected: first command has no matches. `rg` exit code `1` is expected for this no-match guard. Second command prints exactly one private `enum ReportRunError` line in `report.rs`.

- [x] **Step 4: Verify error strings moved to `capture.rs` only**

Run:

```powershell
rg -n '"Corpus preload failed"|"Snapshot capture failed"' src-tauri/src/analysis/report/capture.rs
rg -n "^const SNAPSHOT_CAPTURE_FAILED_MESSAGE" src-tauri/src/analysis/report.rs
rg -n '"Corpus preload failed"|"Snapshot capture failed"' src-tauri/src/analysis/report.rs
```

Expected: first command prints both literals in `capture.rs`. The second and third commands have no matches. `rg` exit code `1` is expected for those no-match guards.

- [x] **Step 5: Run focused capture test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
```

Expected: PASS with `1 passed`, not a green `0 tests` run.

---

### Task 4: Full Verification, Format, And Commit

**Files:**
- Verify: `src-tauri/src/analysis/report.rs`
- Verify: `src-tauri/src/analysis/report/capture.rs`
- Commit: only the two implementation-owned Rust files.

**Interfaces:**
- Consumes: Task 3 source guard and focused test evidence.
- Produces: refactor commit `refactor: extract analysis report capture helper`.

**Execution adjustment:** The Rust refactor commit was created at Task 2 to honor the user instruction to commit after each task. Task 4 records the full verification, clean status comparisons, and confirms the already-created refactor commit `5db04dea`.

- [x] **Step 1: Run full report test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 2: Run corpus boundary test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 3: Check formatting**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

If this fails, run these commands separately:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Then inspect formatting drift:

```powershell
git status --short --untracked-files=all
```

Expected: only `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/capture.rs` have new implementation-owned formatting changes. If rustfmt touched unrelated files, stop and resolve that drift outside this refactor commit.

After any format fixes, rerun:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output before continuing to `cargo check`, staging, or commit.

- [x] **Step 4: Check all Rust targets**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. This is a broad post-change regression gate, not a pre-edit characterization check; if it fails, inspect whether the failure is pre-existing or caused by this refactor before committing. Existing warnings outside touched files may remain. New warnings mentioning `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/capture.rs` are not acceptable.

- [x] **Step 5: Inspect implementation diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/capture.rs
git status --short --untracked-files=all
```

Expected: diff contains only the capture extraction and import cleanup. Status contains only implementation-owned files plus any pre-existing unrelated baseline entries captured in Task 1.

- [x] **Step 6: Compare pre-commit status against baseline**

Run:

```powershell
$preEditStatusPath = Join-Path $env:TEMP "analysis-report-capture-$env:ANALYSIS_REPORT_CAPTURE_STATUS_TAG-pre-edit-status.txt"
$preCommitStatusPath = Join-Path $env:TEMP "analysis-report-capture-$env:ANALYSIS_REPORT_CAPTURE_STATUS_TAG-pre-commit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preCommitStatusPath
Compare-Object `
    (Get-Content -LiteralPath $preEditStatusPath) `
    (Get-Content -LiteralPath $preCommitStatusPath)
```

Expected: the only intentional differences from baseline are `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/capture.rs`. If any unrelated rustfmt drift or unexpected file appears, stop before committing.

- [x] **Step 7: Stage only implementation-owned files**

Run:

```powershell
git add -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/capture.rs
```

Expected: command exits successfully.

Then run:

```powershell
git diff --cached --stat
```

Expected: staged stat contains only `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/capture.rs`.

Then run:

```powershell
git diff --cached --check
```

Expected: no output. If it reports any issue, stop before commit.

- [x] **Step 8: Commit the Rust refactor**

Run:

```powershell
git commit -m "refactor: extract analysis report capture helper"
```

Expected: commit succeeds and includes only the two implementation-owned Rust files.

- [x] **Step 9: Compare final status against baseline**

Run:

```powershell
$preEditStatusPath = Join-Path $env:TEMP "analysis-report-capture-$env:ANALYSIS_REPORT_CAPTURE_STATUS_TAG-pre-edit-status.txt"
$finalStatusPath = Join-Path $env:TEMP "analysis-report-capture-$env:ANALYSIS_REPORT_CAPTURE_STATUS_TAG-final-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $finalStatusPath
Compare-Object `
    (Get-Content -LiteralPath $preEditStatusPath) `
    (Get-Content -LiteralPath $finalStatusPath)
git log --oneline -1
```

Expected: if the pre-edit status was clean, `Compare-Object` has no output. If the pre-edit status had unrelated entries, any output is explained and does not include capture refactor files or new rustfmt drift. Latest commit is `refactor: extract analysis report capture helper`.

---

## Self-Review

- Spec coverage: the plan covers private `capture.rs`, root private import, no `analysis/mod.rs` re-export, preserved `ReportRunError` privacy, moved constant/helper, import cleanup, error string guards, pre-edit status snapshot, post-change report/corpus tests, `fmt --check`, and `cargo check --all-targets`.
- Placeholder scan: no placeholder tokens or open-ended "write tests" steps are present.
- Type consistency: the produced helper signature matches the approved design and the current `run_report_pipeline` and inline `report::tests` call sites.
