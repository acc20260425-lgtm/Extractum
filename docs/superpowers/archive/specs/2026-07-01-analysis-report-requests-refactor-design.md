# Analysis Report Requests Refactor Design

**Date:** 2026-07-01
**Status:** implemented; historical design record as of 2026-07-01
**Scope:** internal Rust refactor of pure analysis report request, chunking, and parsing helpers.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/report.rs` by extracting pure report request-building, chunking, and map-summary parsing helpers into a focused private nested module, without changing report behavior, prompts, request IDs, event flow, cancellation, database writes, or Tauri command contracts.

This is the next conservative slice after simplifying `analysis::corpus`. It intentionally avoids the runtime-heavy map/reduce orchestration and run lifecycle code so the first `report.rs` split is mostly pure, easy to verify, and low risk.

## Current Shape

`src-tauri/src/analysis/report.rs` is currently about 1,800 lines and owns several distinct concerns:

- Tauri-facing report start/cancel entry points and request validation;
- source resolution, corpus request construction, preflight, and snapshot capture;
- report chunk sizing and corpus chunk formatting;
- map-phase LLM request construction;
- map-summary JSON extraction and parsing;
- reduce-phase request construction;
- map/reduce runtime orchestration, scheduler callbacks, cancellation, and event emission;
- run status transitions for failed, cancelled, interrupted, running, and completed runs;
- inline tests for request builders, parsing, preflight validation, cancellation wrappers, and capture behavior.

The pure request/chunking behavior is clustered around these items:

- `ANALYSIS_CHUNK_PROMPT_OVERHEAD_TOKENS`
- `ANALYSIS_CHUNK_OUTPUT_RESERVE_TOKENS`
- `ANALYSIS_CHUNK_SAFETY_PERCENT`
- `ANALYSIS_CHUNK_ESTIMATED_CHARS_PER_TOKEN`
- `ANALYSIS_CHUNK_MIN_TARGET_CHARS`
- `chunk_messages`
- `format_chunk_corpus`
- `build_map_request`
- `extract_json_payload`
- `parse_chunk_summary`
- `summarize_chunk_for_reduce`
- `ReduceRequestParams`
- `build_reduce_request`
- `chunk_target_chars_for_model_input_limit`

Current consumers are internal to `report.rs`:

- `run_map_phase` calls `build_map_request` and `parse_chunk_summary`;
- `run_reduce_phase` constructs `ReduceRequestParams` and calls `build_reduce_request`;
- `run_report_pipeline` calls `chunk_messages`;
- `start_analysis_report_run` calls `chunk_target_chars_for_model_input_limit`;
- inline tests call the same helpers directly.

## Proposed Architecture

Create a private nested module declared from `src-tauri/src/analysis/report.rs`:

- `src-tauri/src/analysis/report/requests.rs`

Keep `src-tauri/src/analysis/report.rs` as the report workflow facade:

- add `mod requests;`;
- import only the request helpers it needs through explicit `self::requests` imports;
- do not add any root re-export from `analysis/mod.rs`;
- keep `requests` private to `analysis::report`;
- keep all runtime orchestration, cancellation, event emission, snapshot capture, run lifecycle, and command entry points in `report.rs`.

Move these items from `report.rs` to `report/requests.rs`:

- chunk target constants:
  - `ANALYSIS_CHUNK_PROMPT_OVERHEAD_TOKENS`
  - `ANALYSIS_CHUNK_OUTPUT_RESERVE_TOKENS`
  - `ANALYSIS_CHUNK_SAFETY_PERCENT`
  - `ANALYSIS_CHUNK_ESTIMATED_CHARS_PER_TOKEN`
  - `ANALYSIS_CHUNK_MIN_TARGET_CHARS`
- `chunk_messages`
- `format_chunk_corpus`
- `build_map_request`
- `extract_json_payload`
- `parse_chunk_summary`
- `summarize_chunk_for_reduce`
- `ReduceRequestParams`
- `build_reduce_request`
- `chunk_target_chars_for_model_input_limit`

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
- `fail_run`
- `fail_capture_run`
- `cancel_run`
- `mark_interrupted_analysis_runs`
- `cleanup_interrupted_analysis_runs`
- `request_analysis_run_cancel`
- `start_analysis_report_run`
- all current tests.

The inline test module stays in `report.rs` for this slice. Moving tests can be a later test-only refactor if needed.

## Visibility

`report/requests.rs` should expose only the helper surface consumed by `report.rs` and its current inline tests:

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

pub(super) fn extract_json_payload(text: &str) -> Result<&str, String>;
```

Keep these private inside `requests.rs`:

```rust
fn format_chunk_corpus(messages: &[CorpusMessage]) -> String;
fn summarize_chunk_for_reduce(summary: &ChunkSummary) -> String;
```

Current test dependency inventory:

- `chunk_target_chars_are_derived_from_model_input_limit_with_fallback` calls `chunk_target_chars_for_model_input_limit`;
- `extracts_json_with_text_before_and_after` calls `extract_json_payload`;
- `extracts_json_inside_markdown_fence` calls `extract_json_payload`;
- `parse_chunk_summary_ignores_non_json_prefix_with_braces` calls `parse_chunk_summary`;
- `parse_chunk_summary_rejects_malformed_payload` calls `parse_chunk_summary`;
- `build_map_request_keeps_run_scoped_request_and_profile` calls `build_map_request`;
- `build_reduce_request_keeps_run_scoped_request_and_profile` calls `build_reduce_request` and constructs `ReduceRequestParams`;
- current tests do not call `format_chunk_corpus` or `summarize_chunk_for_reduce` directly, so those helpers should stay private.

Expected production API changes outside `analysis::report`: none.

Expected root re-export changes: none.

## Imports

`report/requests.rs` should own imports needed by pure request helpers:

- `crate::llm::{LlmChatRequest, LlmMessage}`
- `super::super::models::{AnalysisPromptTemplate, ChunkSummary, CorpusMessage}` or equivalent module-relative imports
- `super::super::trace::normalize_ref`
- `super::super::{now_secs, ANALYSIS_FALLBACK_CHUNK_TARGET_CHARS}`

`report.rs` should remove imports that only the moved helpers used:

- `LlmChatRequest`, if no remaining runtime code needs the concrete type directly after the move;
- `LlmMessage`;
- `normalize_ref`;
- chunk target constants moved to `requests.rs`.

Keep in `report.rs` imports needed by runtime orchestration:

- `LlmCompletion`
- `LlmRequestError`
- `LlmRequestKind`
- `LlmRequestMetadata`
- `LlmRequestPriority`
- `LlmSchedulerState`
- `ResolvedLlmProfile`
- `run_llm_collect_with_profile`
- `run_llm_stream_with_profile`
- `resolve_*` LLM helpers used by `start_analysis_report_run`.

The implementation plan must verify unused imports after the move and keep the diff free of new warning debt in `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/requests.rs`.

## Data Flow

No runtime data flow changes:

1. `start_analysis_report_run` still resolves the effective model and calls the same chunk-target calculation before preflight.
2. `run_report_pipeline` still chunks the captured corpus using the same `chunk_messages` algorithm.
3. `run_map_phase` still builds the same map requests, with the same request ID prefix and prompt text.
4. Map completions still parse via the same JSON extraction and `ChunkSummary` deserialization behavior.
5. `finish_map_phase` remains in `report.rs` and still preserves original chunk order.
6. `run_reduce_phase` still builds the same reduce request, with the same request ID prefix and prompt text.
7. Event kinds, phases, messages, cancellation checks, scheduler metadata, priority, and owner run IDs do not change.
8. Snapshot capture, trace generation, and final report persistence do not change.

## Error Handling

Preserve current error behavior exactly:

- malformed or missing JSON extraction errors keep the same strings;
- `parse_chunk_summary` keeps `Failed to parse chunk summary JSON: {e}`;
- runtime map parse errors still become `ReportRunError::Failed`;
- cancellation still maps to `ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string())`;
- provider failures still flow through existing runtime error handling in `report.rs`;
- no new error codes, messages, or user-facing strings are introduced.

## Non-Goals

This slice does not:

- move map/reduce runtime orchestration;
- move `RunEvent` or event emission;
- move run lifecycle status helpers;
- move `capture_report_corpus`;
- move `start_analysis_report_run` or cancel/cleanup entry points;
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

If `src-tauri/src/analysis/report/requests.rs` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/report/requests.rs') {
    git diff -- src-tauri/src/analysis/report/requests.rs
    git diff --cached -- src-tauri/src/analysis/report/requests.rs
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/report/requests.rs'
}
```

Do not stage pre-existing target-file changes into the request-helper extraction commit.

## Testing

Run commands from the repository root.

Before editing, establish the current baseline:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at spec authoring: `22 passed`; do not require this exact count if nearby tests change before execution.

After editing and before committing, run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. The output must include request-helper coverage, including these tests:

- `analysis::report::tests::chunk_target_chars_are_derived_from_model_input_limit_with_fallback`
- `analysis::report::tests::parse_chunk_summary_ignores_non_json_prefix_with_braces`
- `analysis::report::tests::parse_chunk_summary_rejects_malformed_payload`
- `analysis::report::tests::build_map_request_keeps_run_scoped_request_and_profile`
- `analysis::report::tests::build_reduce_request_keeps_run_scoped_request_and_profile`

The implementation plan must also require one focused constant-behavior guard:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::chunk_target_chars_are_derived_from_model_input_limit_with_fallback
```

Expected: PASS with `1 passed`, not a green `0 tests` run. This is the required regression guard for the moved `ANALYSIS_CHUNK_*` constants. Do not change constant values in this refactor.

Also run consumer and compile checks:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. This is a regression guard for the recently split corpus tests and shared analysis report/corpus boundaries.

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/requests.rs` are not acceptable.

## Commit Shape

Expected implementation-owned files:

- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/report/requests.rs`

Expected implementation commit:

```text
refactor: extract analysis report request helpers
```

The design spec and implementation plan should be committed separately from the Rust refactor.
