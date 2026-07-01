# Analysis Report Requests Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract pure analysis report request-building, chunking, and chunk-summary parsing helpers from `src-tauri/src/analysis/report.rs` into `src-tauri/src/analysis/report/requests.rs` without changing runtime behavior.

**Status:** active approved implementation plan; not implemented as of 2026-07-01 because `src-tauri/src/analysis/report/requests.rs` does not exist. After the Rust refactor commit is verified, update this line to `implemented; historical execution record` in a separate docs commit.

**Architecture:** `report.rs` remains the report workflow facade and owns orchestration, cancellation, events, snapshot capture, run lifecycle, and command entry points. A new private nested module, declared as `mod requests;`, owns only pure chunk sizing, map request, JSON parsing, and reduce request helpers. The module is private to `analysis::report`; there is no root re-export from `analysis/mod.rs`.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, existing `crate::llm` request types, existing `analysis::models` DTOs.

## Global Constraints

- Run commands from the repository root, `G:\Develop\Extractum`.
- Use `cargo --manifest-path src-tauri/Cargo.toml` because the Rust manifest is under `src-tauri/`.
- Behavioral Rust refactor edits are limited to `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/requests.rs`.
- Do not change prompts, request IDs, JSON formats, scheduler metadata, cancellation behavior, event payloads, database writes, or Tauri command payloads.
- Do not move report tests in this slice; the inline `#[cfg(test)] mod tests` stays in `report.rs`.
- Do not add root re-exports from `src-tauri/src/analysis/mod.rs`.
- Preserve current error strings from JSON extraction and chunk-summary parsing exactly.
- Preserve the existing `ANALYSIS_CHUNK_*` constant values exactly.
- Keep `format_chunk_corpus` and `summarize_chunk_for_reduce` private inside `requests.rs`.
- If there are pre-existing target-file changes, inspect and preserve them; do not stage unrelated user work.
- Capture the pre-edit `git status --short --untracked-files=all` output in execution notes and compare final status against that baseline.

---

## File Structure

- Create `src-tauri/src/analysis/report/requests.rs`
  - Owns pure request helper imports.
  - Owns `ANALYSIS_CHUNK_*` constants.
  - Owns `chunk_messages`, `format_chunk_corpus`, `build_map_request`, `extract_json_payload`, `parse_chunk_summary`, `summarize_chunk_for_reduce`, `ReduceRequestParams`, `build_reduce_request`, and `chunk_target_chars_for_model_input_limit`.

- Modify `src-tauri/src/analysis/report.rs`
  - Adds `mod requests;`.
  - Imports the public helper surface with explicit `self::requests::{...}`.
  - Removes moved helper definitions and imports used only by those helpers.
  - Keeps workflow structs/functions and all tests in place.

- Do not modify `src-tauri/src/analysis/mod.rs`.

---

### Task 1: Pre-Edit Baseline And Worktree Guard

**Files:**
- Inspect: `src-tauri/src/analysis/report.rs`
- Inspect if present: `src-tauri/src/analysis/report/requests.rs`

**Interfaces:**
- Consumes: approved design spec `docs/superpowers/specs/2026-07-01-analysis-report-requests-refactor-design.md`
- Produces: clean baselines proving current report and corpus regression slices pass before extraction

- [x] **Step 1: Inspect the worktree before editing**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: either clean output, or unrelated existing changes that are clearly outside the implementation-owned files. Save this output in execution notes as `PRE_EDIT_STATUS`. If `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/requests.rs` appears, continue with Steps 2-3 before editing.

- [x] **Step 2: Inspect any dirty tracked target file**

Run these separately if `report.rs` or `report/requests.rs` is already modified or staged:

```powershell
git diff -- src-tauri/src/analysis/report.rs
```

```powershell
git diff --cached -- src-tauri/src/analysis/report.rs
```

```powershell
git diff -- src-tauri/src/analysis/report/requests.rs
```

```powershell
git diff --cached -- src-tauri/src/analysis/report/requests.rs
```

Expected: you understand every pre-existing target-file change. Stop and ask before continuing if any pre-existing target-file change overlaps the request-helper extraction.

- [x] **Step 3: Inspect and fingerprint a pre-existing untracked `requests.rs`**

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/report/requests.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/report/requests.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/report/requests.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/report/requests.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/report/requests.rs'
}
```

Expected: if the file already exists, you capture status, length, SHA-256 hash, and contents because normal `git diff` does not show untracked file content. Stop and ask before continuing if this file contains pre-existing user work that is not the request-helper extraction.

- [x] **Step 4: Establish the focused report baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at plan writing is expected to be about `22 passed`, but do not require the exact count if nearby tests changed before execution.

- [x] **Step 5: Establish the focused constant baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::chunk_target_chars_are_derived_from_model_input_limit_with_fallback
```

Expected: PASS with `1 passed`, not a green `0 tests` run.

- [x] **Step 6: Establish the focused corpus baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. Stop before editing if this baseline is red; do not discover a pre-existing corpus failure only after the request-helper extraction.

---

### Task 2: Extract The Requests Module

**Files:**
- Create: `src-tauri/src/analysis/report/requests.rs`
- Modify: `src-tauri/src/analysis/report.rs`

**Interfaces:**
- Consumes from `report.rs`: `AnalysisPromptTemplate`, `ChunkSummary`, `CorpusMessage`, `now_secs`, `ANALYSIS_FALLBACK_CHUNK_TARGET_CHARS`, and `normalize_ref` behavior.
- Produces for `report.rs` and its inline tests:

```rust
pub(super) fn chunk_messages(
    messages: &[CorpusMessage],
    max_chars: usize,
) -> Vec<Vec<CorpusMessage>>;

pub(super) fn build_map_request(
    run_id: i64,
    profile_id: String,
    chunk_index: usize,
    total_chunks: usize,
    messages: &[CorpusMessage],
) -> LlmChatRequest;

pub(super) fn extract_json_payload(text: &str) -> Result<&str, String>;

pub(super) fn parse_chunk_summary(text: &str) -> Result<ChunkSummary, String>;

pub(super) struct ReduceRequestParams<'a> {
    pub(super) run_id: i64,
    pub(super) profile_id: String,
    pub(super) scope_label: &'a str,
    pub(super) output_language: &'a str,
    pub(super) prompt_template: &'a AnalysisPromptTemplate,
    pub(super) period_from: i64,
    pub(super) period_to: i64,
    pub(super) chunk_summaries: &'a [ChunkSummary],
    pub(super) model_override: Option<String>,
}

pub(super) fn build_reduce_request(params: ReduceRequestParams<'_>) -> LlmChatRequest;

pub(super) fn chunk_target_chars_for_model_input_limit(
    model_input_token_limit: Option<usize>,
) -> usize;
```

- [x] **Step 1: Create the nested module file with imports and constants**

Create `src-tauri/src/analysis/report/requests.rs` with this header and constant block:

```rust
use crate::llm::{LlmChatRequest, LlmMessage};

use super::super::models::{AnalysisPromptTemplate, ChunkSummary, CorpusMessage};
use super::super::trace::normalize_ref;
use super::super::{now_secs, ANALYSIS_FALLBACK_CHUNK_TARGET_CHARS};

const ANALYSIS_CHUNK_PROMPT_OVERHEAD_TOKENS: usize = 1_500;
const ANALYSIS_CHUNK_OUTPUT_RESERVE_TOKENS: usize = 2_000;
const ANALYSIS_CHUNK_SAFETY_PERCENT: usize = 80;
const ANALYSIS_CHUNK_ESTIMATED_CHARS_PER_TOKEN: usize = 3;
const ANALYSIS_CHUNK_MIN_TARGET_CHARS: usize = 2_000;
```

- [x] **Step 2: Move `chunk_messages` byte-for-byte and widen only module visibility**

Cut the existing `chunk_messages` function from `report.rs` and paste it below the constants in `requests.rs`. Change only the signature visibility:

```rust
pub(super) fn chunk_messages(messages: &[CorpusMessage], max_chars: usize) -> Vec<Vec<CorpusMessage>> {
```

The body stays byte-for-byte the same.

- [x] **Step 3: Move `format_chunk_corpus` byte-for-byte and keep it private**

Cut the existing `format_chunk_corpus` function from `report.rs` and paste it below `chunk_messages` in `requests.rs`. Keep this signature private:

```rust
fn format_chunk_corpus(messages: &[CorpusMessage]) -> String {
```

The body stays byte-for-byte the same.

- [x] **Step 4: Move `build_map_request` byte-for-byte and widen only module visibility**

Cut the existing `build_map_request` function from `report.rs` and paste it below `format_chunk_corpus` in `requests.rs`. Change only the signature visibility:

```rust
pub(super) fn build_map_request(
    run_id: i64,
    profile_id: String,
    chunk_index: usize,
    total_chunks: usize,
    messages: &[CorpusMessage],
) -> LlmChatRequest {
```

The prompt strings, request ID format, profile assignment, model override, and message structure stay byte-for-byte the same.

- [x] **Step 5: Move `extract_json_payload` byte-for-byte and widen only module visibility**

Cut the existing `extract_json_payload` function from `report.rs` and paste it below `build_map_request` in `requests.rs`. Change only the signature visibility:

```rust
pub(super) fn extract_json_payload(text: &str) -> Result<&str, String> {
```

The body stays byte-for-byte the same, including these exact error strings:

```rust
"LLM response contained malformed JSON boundaries"
"LLM response did not contain a valid JSON object"
"LLM response did not contain JSON"
```

- [x] **Step 6: Move `parse_chunk_summary` byte-for-byte and widen only module visibility**

Cut the existing `parse_chunk_summary` function from `report.rs` and paste it below `extract_json_payload` in `requests.rs`. Change only the signature visibility:

```rust
pub(super) fn parse_chunk_summary(text: &str) -> Result<ChunkSummary, String> {
```

The body stays byte-for-byte the same, including this exact error prefix:

```rust
"Failed to parse chunk summary JSON: {e}"
```

- [x] **Step 7: Move `summarize_chunk_for_reduce` byte-for-byte and keep it private**

Cut the existing `summarize_chunk_for_reduce` function from `report.rs` and paste it below `parse_chunk_summary` in `requests.rs`. Keep this signature private:

```rust
fn summarize_chunk_for_reduce(summary: &ChunkSummary) -> String {
```

The body stays byte-for-byte the same, including the `normalize_ref` filtering.

- [x] **Step 8: Move `ReduceRequestParams` and expose fields only to the parent module**

Cut the existing `ReduceRequestParams` struct from `report.rs` and paste it below `summarize_chunk_for_reduce` in `requests.rs`. Change the struct and every field to `pub(super)`:

```rust
pub(super) struct ReduceRequestParams<'a> {
    pub(super) run_id: i64,
    pub(super) profile_id: String,
    pub(super) scope_label: &'a str,
    pub(super) output_language: &'a str,
    pub(super) prompt_template: &'a AnalysisPromptTemplate,
    pub(super) period_from: i64,
    pub(super) period_to: i64,
    pub(super) chunk_summaries: &'a [ChunkSummary],
    pub(super) model_override: Option<String>,
}
```

- [x] **Step 9: Move `build_reduce_request` byte-for-byte and widen only module visibility**

Cut the existing `build_reduce_request` function from `report.rs` and paste it below `ReduceRequestParams` in `requests.rs`. Change only the signature visibility:

```rust
pub(super) fn build_reduce_request(params: ReduceRequestParams<'_>) -> LlmChatRequest {
```

The prompt strings, request ID format, profile assignment, model override, chunk summary formatting, and message structure stay byte-for-byte the same.

- [x] **Step 10: Move `chunk_target_chars_for_model_input_limit` byte-for-byte and widen only module visibility**

Cut the existing `chunk_target_chars_for_model_input_limit` function from `report.rs` and paste it at the bottom of `requests.rs`. Change only the signature visibility:

```rust
pub(super) fn chunk_target_chars_for_model_input_limit(
    model_input_token_limit: Option<usize>,
) -> usize {
```

The body stays byte-for-byte the same. The constants used by this function live in `requests.rs`.

- [x] **Step 11: Declare and import the nested module from `report.rs`**

Add this module declaration near the top of `report.rs`, after the `use` block and before constants:

```rust
mod requests;
```

Add this explicit helper import block near the other `super::...` imports:

```rust
use self::requests::{
    build_map_request, build_reduce_request, chunk_messages,
    chunk_target_chars_for_model_input_limit, parse_chunk_summary, ReduceRequestParams,
};
```

If `extract_json_payload` is imported only for tests, do not import it in this production import block; import it inside the test module in Step 13.

- [x] **Step 12: Remove imports from `report.rs` that moved to `requests.rs`**

Change the `crate::llm` import in `report.rs` from:

```rust
use crate::llm::{
    resolve_effective_model, resolve_model_input_token_limit_for_backend,
    resolve_profile_for_backend, run_llm_collect_with_profile, run_llm_stream_with_profile,
    LlmChatRequest, LlmCompletion, LlmMessage, LlmRequestError, LlmRequestKind, LlmRequestMetadata,
    LlmRequestPriority, LlmSchedulerState, ResolvedLlmProfile,
};
```

to:

```rust
use crate::llm::{
    resolve_effective_model, resolve_model_input_token_limit_for_backend,
    resolve_profile_for_backend, run_llm_collect_with_profile, run_llm_stream_with_profile,
    LlmCompletion, LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority,
    LlmSchedulerState, ResolvedLlmProfile,
};
```

Change the trace import in `report.rs` from:

```rust
use super::trace::{build_trace_data, compress_trace_data, normalize_ref};
```

to:

```rust
use super::trace::{build_trace_data, compress_trace_data};
```

Remove the moved `ANALYSIS_CHUNK_*` constants from `report.rs`. Keep these constants in `report.rs`:

```rust
const INTERRUPTED_RUN_MESSAGE: &str = "Analysis run was interrupted when the app was restarted.";
const CANCELLED_RUN_MESSAGE: &str = "Analysis run cancelled.";
const SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed";
```

- [x] **Step 13: Update the inline report test imports**

Change the test module imports so production helpers come through the parent module imports, and `extract_json_payload` comes directly from the nested module:

```rust
#[cfg(test)]
mod tests {
    use super::requests::extract_json_payload;
    use super::{
        build_map_request, build_reduce_request, capture_report_corpus,
        chunk_target_chars_for_model_input_limit, finish_map_phase, mark_interrupted_analysis_runs,
        parse_chunk_summary, resolve_analysis_telegram_history_scope, run_analysis_step_with_cancel,
        validate_report_preflight, ReduceRequestParams, ReportRunError, ReportRunInput,
        StartAnalysisReportRequest,
    };
```

Keep the remaining `crate::analysis::corpus`, `crate::analysis::models`, `crate::error`, `crate::llm`, and `tokio_util` imports unchanged unless `cargo check` proves an import is unused.

- [x] **Step 14: Confirm no moved helper definitions remain in `report.rs`**

Run:

```powershell
rg -n "^(const ANALYSIS_CHUNK|fn chunk_messages|fn format_chunk_corpus|fn build_map_request|fn extract_json_payload|fn parse_chunk_summary|fn summarize_chunk_for_reduce|struct ReduceRequestParams|fn build_reduce_request|fn chunk_target_chars_for_model_input_limit)" src-tauri/src/analysis/report.rs
```

Expected: no matches.

- [x] **Step 15: Confirm the nested module owns the moved helper definitions**

Run:

```powershell
rg -n "^(const ANALYSIS_CHUNK|pub\\(super\\) fn chunk_messages|fn format_chunk_corpus|pub\\(super\\) fn build_map_request|pub\\(super\\) fn extract_json_payload|pub\\(super\\) fn parse_chunk_summary|fn summarize_chunk_for_reduce|pub\\(super\\) struct ReduceRequestParams|pub\\(super\\) fn build_reduce_request|pub\\(super\\) fn chunk_target_chars_for_model_input_limit)" src-tauri/src/analysis/report/requests.rs
```

Expected: matches for every moved constant, every public parent-facing helper, and the two private helper functions.

- [x] **Step 16: Confirm `extract_json_payload` has exactly parent-module visibility**

Run:

```powershell
rg -n "^pub\\(super\\) fn extract_json_payload\\(text: &str\\) -> Result<&str, String>" src-tauri/src/analysis/report/requests.rs
```

Expected: exactly one match.

Run:

```powershell
rg -n "^(pub\\(crate\\) fn extract_json_payload|pub fn extract_json_payload|fn extract_json_payload)" src-tauri/src/analysis/report/requests.rs
```

Expected: no matches. This prevents both accidental API widening and accidental test-breaking private visibility.

Run:

```powershell
rg -n "extract_json_payload" src-tauri/src/analysis/report.rs
```

Expected: matches only in the test import `use super::requests::extract_json_payload;` and the two existing tests `extracts_json_with_text_before_and_after` and `extracts_json_inside_markdown_fence`. Production code should continue to call `parse_chunk_summary`, not `extract_json_payload`.

---

### Task 3: Verify Behavior, Formatting, And Compile Boundaries

**Files:**
- Verify: `src-tauri/src/analysis/report.rs`
- Verify: `src-tauri/src/analysis/report/requests.rs`

**Interfaces:**
- Consumes: extracted helper surface from Task 2.
- Produces: verified implementation ready for the refactor commit.

- [x] **Step 1: Format the Rust code**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command exits 0. After this command, inspect changed files before staging.

- [x] **Step 2: Check implementation-owned file list after formatting**

Run:

```powershell
git status --short --untracked-files=all
```

Expected implementation-owned changes:

```text
 M src-tauri/src/analysis/report.rs
?? src-tauri/src/analysis/report/requests.rs
```

Run:

```powershell
git diff --name-only
```

```powershell
git diff --cached --name-only
```

```powershell
git ls-files --others --exclude-standard
```

Expected: the only new implementation-owned paths are `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/requests.rs`, plus any unrelated paths already present in `PRE_EDIT_STATUS`. If `cargo fmt` changed any unrelated Rust file, inspect it and resolve that drift before committing this refactor. The refactor commit must not include unrelated rustfmt drift.

- [x] **Step 3: Run focused report tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. The output must include these tests by name or the test count must be paired with `-- --list` proof in Step 4:

```text
analysis::report::tests::chunk_target_chars_are_derived_from_model_input_limit_with_fallback
analysis::report::tests::parse_chunk_summary_ignores_non_json_prefix_with_braces
analysis::report::tests::parse_chunk_summary_rejects_malformed_payload
analysis::report::tests::build_map_request_keeps_run_scoped_request_and_profile
analysis::report::tests::build_reduce_request_keeps_run_scoped_request_and_profile
```

- [x] **Step 4: List focused report tests if the Step 3 output is too terse**

Run this if Step 3 does not print individual test names:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests:: -- --list
```

Expected: output includes the five test names listed in Step 3.

- [x] **Step 5: Run the focused constant guard**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::chunk_target_chars_are_derived_from_model_input_limit_with_fallback
```

Expected: PASS with `1 passed`, not a green `0 tests` run.

- [x] **Step 6: Run recently split corpus regression tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 7: Run formatting check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

- [x] **Step 8: Run all-target compile check**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain. New warnings mentioning `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/requests.rs` are not acceptable.

- [x] **Step 9: Inspect the final diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/requests.rs
```

Expected:

- `report.rs` has `mod requests;` and explicit `self::requests` imports.
- `report.rs` no longer contains moved helper bodies or `ANALYSIS_CHUNK_*` constants.
- `requests.rs` contains the moved helper bodies with only visibility/import changes.
- No prompt strings, request ID prefixes, JSON parse error strings, or chunk constant values changed.

- [x] **Step 10: Check whitespace**

Run:

```powershell
git diff --check -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/requests.rs
```

Expected: no whitespace errors.

---

### Task 4: Commit The Refactor

**Files:**
- Stage: `src-tauri/src/analysis/report.rs`
- Stage: `src-tauri/src/analysis/report/requests.rs`

**Interfaces:**
- Consumes: verified implementation from Task 3.
- Produces: one Rust refactor commit.

- [ ] **Step 1: Confirm only intended implementation files are dirty**

Run:

```powershell
git status --short --untracked-files=all
```

Expected implementation-owned files only, unless unrelated pre-existing changes were captured in `PRE_EDIT_STATUS` and are still intentionally left unstaged. Any file not present in `PRE_EDIT_STATUS` and not one of the two implementation-owned paths must be resolved before staging.

- [ ] **Step 2: Stage only implementation-owned Rust files**

Run:

```powershell
git add -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/requests.rs
```

Expected: only those two Rust files are staged.

- [ ] **Step 3: Verify staged files and staged whitespace**

Run:

```powershell
git diff --cached --stat
```

Expected:

```text
src-tauri/src/analysis/report.rs
src-tauri/src/analysis/report/requests.rs
```

Run:

```powershell
git diff --cached --check
```

Expected: no whitespace errors.

- [ ] **Step 4: Commit**

Run:

```powershell
git commit -m "refactor: extract analysis report request helpers"
```

Expected: commit succeeds.

- [ ] **Step 5: Confirm final status against the pre-edit baseline**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: clean output if `PRE_EDIT_STATUS` was clean. If `PRE_EDIT_STATUS` contained unrelated pre-existing changes, final status must match those unrelated entries exactly and must not include `src-tauri/src/analysis/report.rs`, `src-tauri/src/analysis/report/requests.rs`, or any new rustfmt drift.

---

## Self-Review

- Spec coverage: this plan covers the private nested module, no root re-export, helper visibility, unchanged tests, constant guard, report/corpus regression checks, fmt, all-target compile, and commit shape.
- Red-flag scan: no unfinished work items are intentionally left open.
- Type consistency: `ReduceRequestParams`, `AnalysisPromptTemplate`, `ChunkSummary`, `CorpusMessage`, `LlmChatRequest`, and `LlmMessage` match current `report.rs` usage.

---

## Execution Options

Plan complete and saved to `docs/superpowers/plans/2026-07-01-analysis-report-requests-refactor.md`. Two execution options:

1. **Subagent-Driven (recommended)** - dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** - execute tasks in this session using executing-plans, batch execution with checkpoints.
