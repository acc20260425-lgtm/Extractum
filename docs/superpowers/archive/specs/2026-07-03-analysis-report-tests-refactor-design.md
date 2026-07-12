# Analysis Report Tests Refactor Design

**Date:** 2026-07-03
**Status:** active spec; implementation not started as of 2026-07-03 because `src-tauri/src/analysis/report/tests/` does not exist.
**Scope:** internal Rust test-only refactor of `src-tauri/src/analysis/report.rs` into nested report test modules.

## Goal

Reduce the size and review burden of `src-tauri/src/analysis/report.rs` by moving its large inline `#[cfg(test)] mod tests` body into focused nested test modules, without changing production behavior, public or crate-visible caller paths, debug command paths, run lifecycle behavior, report request payloads, provider phase behavior, capture behavior, validation messages, test assertions, or test coverage.

This is the next conservative report slice after extracting production request, lifecycle, capture, and provider phase helpers. The remaining `report.rs` size is now dominated by tests.

## Current Shape

`src-tauri/src/analysis/report.rs` currently owns:

- production facade wiring for:
  - `capture`
  - `lifecycle`
  - `phases`
  - `requests`
- public lifecycle re-export `cleanup_interrupted_analysis_runs`;
- crate-visible lifecycle re-exports used by report command consumers;
- `StartAnalysisReportRequest`;
- `resolve_analysis_telegram_history_scope`;
- `ReportRunError`;
- `RunEvent`;
- `ReportRunInput`;
- `validate_report_preflight`;
- `run_report_pipeline`;
- `start_analysis_report_run`;
- a large inline `#[cfg(test)] mod tests`.

The inline test module mixes these concerns:

- shared fixtures:
  - `SAMPLE_JSON`
  - `sample_chunk_summary`
  - `sample_prompt_template`
  - `sample_corpus_message`
  - `sample_resolved_profile`
  - `request_cancel_pool_with_runs`
  - `insert_cancel_request_run`
- report input and scope request shape tests;
- migrated Telegram history policy tests;
- chunk target sizing test;
- capture behavior test;
- lifecycle cleanup and cancellation request tests;
- JSON extraction and chunk summary parsing tests;
- map/reduce request construction tests;
- phase helper and cancellation wrapper tests;
- report preflight validation tests;
- architecture guard that Tauri command adapters do not live in `report.rs`.

## Proposed Architecture

Move only test code into a nested test tree:

- `src-tauri/src/analysis/report/tests/mod.rs`
- `src-tauri/src/analysis/report/tests/harness.rs`
- `src-tauri/src/analysis/report/tests/scope.rs`
- `src-tauri/src/analysis/report/tests/capture.rs`
- `src-tauri/src/analysis/report/tests/lifecycle.rs`
- `src-tauri/src/analysis/report/tests/requests.rs`
- `src-tauri/src/analysis/report/tests/phases.rs`
- `src-tauri/src/analysis/report/tests/preflight.rs`
- `src-tauri/src/analysis/report/tests/architecture.rs`

Keep production code in `src-tauri/src/analysis/report.rs`.

Replace the inline test module in `report.rs` with:

```rust
#[cfg(test)]
mod tests;
```

`tests/mod.rs` should declare the thematic modules:

```rust
mod architecture;
mod capture;
mod harness;
mod lifecycle;
mod phases;
mod preflight;
mod requests;
mod scope;
```

`tests/harness.rs` owns helpers shared by more than one thematic module. The thematic modules import helpers through `super::harness`.

This split keeps the existing Cargo test path prefix under `analysis::report::tests::`. Individual tests gain one more module segment, such as `analysis::report::tests::requests::build_reduce_request_keeps_run_scoped_request_and_profile`. The implementation plan must update verification filters accordingly and must prevent green `0 tests` runs.

## File Responsibilities

`src-tauri/src/analysis/report.rs`

- Keep all production definitions, module declarations, imports, and re-exports.
- Keep only `#[cfg(test)] mod tests;` for tests.
- Do not gain new test helper code.
- Do not change production visibility or external consumer imports.

`src-tauri/src/analysis/report/tests/mod.rs`

- Declare child test modules.
- Avoid owning helper logic or test functions directly.

`src-tauri/src/analysis/report/tests/harness.rs`

- Own shared fixture constructors:
  - `sample_chunk_summary`
  - `sample_prompt_template`
  - `sample_corpus_message`
  - `sample_resolved_profile`
- Own lifecycle cancel helper database setup:
  - `request_cancel_pool_with_runs`
  - `insert_cancel_request_run`
- Keep `SAMPLE_JSON` here if both JSON extraction and parse tests use it.

`src-tauri/src/analysis/report/tests/scope.rs`

- Own report input and scope shape tests:
  - `report_run_input_carries_resolved_profile_snapshot`
  - `telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match`
  - `migrated_history_opt_in_rejects_non_telegram_analysis`
  - `report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape`
  - `chunk_target_chars_are_derived_from_model_input_limit_with_fallback`

`src-tauri/src/analysis/report/tests/capture.rs`

- Own `capture_report_corpus_returns_reloaded_snapshot_before_provider_phases`.

`src-tauri/src/analysis/report/tests/lifecycle.rs`

- Own lifecycle cleanup and cancel-request tests:
  - `interrupted_cleanup_preserves_captured_snapshot_state_marker`
  - `request_analysis_run_cancel_missing_run_keeps_not_found_message`
  - `request_analysis_run_cancel_completed_run_keeps_conflict_message`
  - `request_analysis_run_cancel_running_but_inactive_keeps_conflict_message`

`src-tauri/src/analysis/report/tests/requests.rs`

- Own request/JSON tests:
  - `extracts_json_with_text_before_and_after`
  - `extracts_json_inside_markdown_fence`
  - `parse_chunk_summary_ignores_non_json_prefix_with_braces`
  - `parse_chunk_summary_rejects_malformed_payload`
  - `build_map_request_keeps_run_scoped_request_and_profile`
  - `build_reduce_request_keeps_run_scoped_request_and_profile`

`src-tauri/src/analysis/report/tests/phases.rs`

- Own phase helper tests:
  - `analysis_step_cancel_wrapper_allows_completed_future`
  - `analysis_step_cancel_wrapper_interrupts_pending_future`
  - `finish_map_phase_preserves_chunk_order_by_original_index`
  - `finish_map_phase_rejects_missing_chunk_before_reduce`
  - `finish_map_phase_propagates_map_error_without_starting_reduce`

`src-tauri/src/analysis/report/tests/preflight.rs`

- Own report preflight validation tests:
  - `validate_report_preflight_rejects_empty_corpus`
  - `validate_report_preflight_rejects_oversized_runs`
  - `validate_report_preflight_allows_runs_within_limits`

`src-tauri/src/analysis/report/tests/architecture.rs`

- Own `analysis_report_workflow_file_has_no_tauri_command_adapters`.

## Visibility

This refactor is test-only, but moving tests out of the inline child module changes helper paths.

Use this visibility strategy:

1. Keep production visibility unchanged.
2. Keep thematic helper functions private to their module when only one module uses them.
3. Move only genuinely shared helpers into `tests/harness.rs`.
4. Mark shared harness helpers `pub(super)`, never `pub(crate)` or `pub`.
5. Do not widen production item visibility solely for this test split.

Expected production visibility changes: none.

Expected shared test-helper visibility:

```rust
pub(super) const SAMPLE_JSON: &str = r#"{"summary":"Brief","topics":["sync"],"notable_points":["Point"],"candidate_refs":["s1-i2"]}"#;
pub(super) fn sample_chunk_summary(label: &str) -> ChunkSummary;
pub(super) fn sample_prompt_template() -> AnalysisPromptTemplate;
pub(super) fn sample_corpus_message() -> CorpusMessage;
pub(super) fn sample_resolved_profile() -> ResolvedLlmProfile;
pub(super) async fn request_cancel_pool_with_runs() -> sqlx::SqlitePool;
pub(super) async fn insert_cancel_request_run(
    pool: &sqlx::SqlitePool,
    run_id: i64,
    status: &str,
);
```

If a listed helper ends up used by only one thematic module after the split, the implementation may keep it private in that thematic module instead of exporting it from `harness.rs`. Do not widen any helper beyond `pub(super)`.

Moved test modules may access private parent report items through `super::super`, because they are descendants of `analysis::report`. This includes:

- `capture_report_corpus`
- `mark_interrupted_analysis_runs`
- `request_analysis_run_cancel_for_pool`
- `resolve_analysis_telegram_history_scope`
- `run_analysis_step_with_cancel`
- `finish_map_phase`
- `build_map_request`
- `build_reduce_request`
- `parse_chunk_summary`
- `chunk_target_chars_for_model_input_limit`
- `validate_report_preflight`
- `ReportRunError`
- `ReportRunInput`
- `ReduceRequestParams`
- `StartAnalysisReportRequest`

Do not expose these parent items as `pub(super)`, `pub(crate)`, or `pub` solely for tests.

Facade coverage is intentional:

- tests should import production helpers through explicit parent facade imports from `super::super`;
- tests should not import private production child modules directly, such as `super::super::requests::extract_json_payload`, `super::super::phases::finish_map_phase`, `super::super::capture::capture_report_corpus`, or `super::super::lifecycle::request_analysis_run_cancel_for_pool`;
- the only direct child-module import that currently exists is `super::requests::extract_json_payload` inside inline tests; after the split it should be replaced by parent facade access if the parent keeps a private `#[cfg(test)] use self::requests::extract_json_payload;` import. If that import is missing today, the implementation should add it as `#[cfg(test)] use self::requests::extract_json_payload;` in `report.rs`, not widen `requests.rs`.

## Data Flow

No runtime data flow changes:

1. Production `report.rs` still exposes the same report workflow and lifecycle facade.
2. Test-only SQLite setup still creates the same schemas and rows.
3. Capture tests still exercise `capture_report_corpus` through the report parent facade.
4. Lifecycle tests still exercise interrupted cleanup and cancel-request helpers through parent report imports.
5. Request tests still exercise map/reduce request builders and parsing helpers through parent report imports.
6. Phase tests still exercise `finish_map_phase` and `run_analysis_step_with_cancel` through parent report imports.
7. Preflight tests still exercise `validate_report_preflight` with the same limit inputs.

Only test module paths change. The full command:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

must continue to run all report tests.

## Error Handling

Preserve current test assertions and error expectations exactly:

- migrated Telegram history non-Telegram opt-in still returns `AppErrorKind::Validation`;
- empty corpus preflight still returns message `No synced source documents were found for the selected analysis scope and period`;
- oversized preflight still contains `Analysis scope is too large`;
- cancel missing run still returns `Analysis run {run_id} not found`;
- cancel completed run still returns `Analysis run {run_id} is not queued or running`;
- cancel inactive running run still returns `Analysis run {run_id} is no longer active`;
- interrupted cleanup still preserves `2026-05-18T10:00:00Z`;
- malformed chunk summary still checks parse failure text;
- phase cancellation still preserves `Analysis run cancelled.`;
- architecture test still asserts Tauri command adapters are absent from `src/analysis/report.rs`.

The implementation plan must include source guards for these assertion markers after the move:

```powershell
$requiredMarkers = @{
    "scope.rs" = @("ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED", "include_migrated_history")
    "preflight.rs" = @("No synced source documents were found for the selected analysis scope and period", "Analysis scope is too large")
    "lifecycle.rs" = @("Analysis run {run_id} not found", "Analysis run {run_id} is not queued or running", "Analysis run {run_id} is no longer active", "2026-05-18T10:00:00Z")
    "requests.rs" = @("Failed to parse chunk summary JSON", "analysis-map-55-2-", "analysis-reduce-77-")
    "phases.rs" = @("Some chunk summaries were not collected", "Analysis run cancelled.")
    "architecture.rs" = @("Analysis report command adapters should live outside src/analysis/report.rs")
}
foreach ($entry in $requiredMarkers.GetEnumerator()) {
    foreach ($marker in $entry.Value) {
        $path = "src-tauri/src/analysis/report/tests/$($entry.Key)"
        rg -n -F $marker $path
        if ($LASTEXITCODE -ne 0) {
            throw "missing report assertion marker '$marker' in $path"
        }
    }
}
```

Expected: all moved assertion markers are present in the thematic test modules.

## Non-Goals

This slice does not:

- move or edit production code in `report/capture.rs`, `report/lifecycle.rs`, `report/phases.rs`, or `report/requests.rs`;
- split or redesign production report workflow code;
- move `ReportRunError`, `RunEvent`, `ReportRunInput`, or `StartAnalysisReportRequest`;
- change external consumer imports;
- change SQL, schemas, fixture data, request payloads, event payloads, validation messages, lifecycle behavior, cancellation behavior, trace behavior, database migrations, frontend code, or Tauri command payloads;
- add new behavior tests beyond path/coverage guards needed for the move;
- delete or weaken any current report test.

## Implementation Notes

The implementation plan should:

1. Require a pre-edit worktree snapshot with target-file checks:
   - `git status --short --untracked-files=all`
   - `git diff -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/tests`
   - `git diff --cached -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/tests`
   - `git ls-files src-tauri/src/analysis/report/tests`
   - if `src-tauri/src/analysis/report/tests/` exists in any form, print its contents with `Get-ChildItem -Recurse`; for untracked files, also print `Get-Content -Raw` for each file before continuing.
2. Stop before editing if `report.rs` is dirty, if any target `report/tests/*` file is dirty, or if `report/tests/` already exists tracked or untracked without an explicit baseline decision.
3. Run baseline verification before editing:
   - `cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::`
   - `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`
4. Replace the inline test module with `#[cfg(test)] mod tests;`.
5. Move helpers and tests into thematic files without changing test bodies except imports and module paths.
6. Keep imports explicit in each test module. Avoid glob imports from `crate`, and do not use `use super::super::*`; import parent report functions, types, and helpers by name.
7. Preserve parent facade access for production child-module helpers. Add private root `#[cfg(test)] use` imports in `report.rs` when needed instead of widening production child modules.
8. Run `cargo fmt --manifest-path src-tauri/Cargo.toml` only if needed, then run `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
9. Inspect `git status --short --untracked-files=all` and changed file list after formatting. Behavioral diff should be limited to:
   - `src-tauri/src/analysis/report.rs`
   - `src-tauri/src/analysis/report/tests/mod.rs`
   - `src-tauri/src/analysis/report/tests/harness.rs`
   - `src-tauri/src/analysis/report/tests/scope.rs`
   - `src-tauri/src/analysis/report/tests/capture.rs`
   - `src-tauri/src/analysis/report/tests/lifecycle.rs`
   - `src-tauri/src/analysis/report/tests/requests.rs`
   - `src-tauri/src/analysis/report/tests/phases.rs`
   - `src-tauri/src/analysis/report/tests/preflight.rs`
   - `src-tauri/src/analysis/report/tests/architecture.rs`
10. Stage only intended files. Existing unrelated files, including ignored/generated files, must not be staged.

Use per-symbol checks when verifying required moved tests or modules; do not rely on a single alternation `rg` command whose success only proves one match.

## Source Guards

Run source guards after the move.

`report.rs` should keep only test module wiring:

```powershell
$lines = Get-Content src-tauri/src/analysis/report.rs
$cfgIndexes = @(
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\s*#\[cfg\(test\)\]\s*$') { $i }
    }
)
$modIndexes = @(
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\s*mod tests;\s*$') { $i }
    }
)
if ($cfgIndexes.Count -ne 1 -or $modIndexes.Count -ne 1 -or $modIndexes[0] -ne ($cfgIndexes[0] + 1)) {
    throw "report.rs must contain exactly one adjacent #[cfg(test)] / mod tests; pair"
}
$lines[$cfgIndexes[0]]
$lines[$modIndexes[0]]
```

Expected: exactly two adjacent lines are printed: `#[cfg(test)]` followed by `mod tests;`.

No inline test bodies remain in `report.rs`:

```powershell
$inlineTestMatches = @(rg -n "#\[tokio::test\]|#\[test\]|^mod tests \{|use super::\*|^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn) (sample_chunk_summary|sample_prompt_template|sample_corpus_message|sample_resolved_profile|request_cancel_pool_with_runs|insert_cancel_request_run)\b" src-tauri/src/analysis/report.rs)
if ($inlineTestMatches.Count -ne 0) {
    $inlineTestMatches
    throw "inline report test body or helper remains in report.rs"
}
```

Expected: no output and no throw.

Production visibility remains unchanged:

```powershell
$widenedMatches = @(rg -n "^\s*pub(\([^)]*\))?\s+(enum ReportRunError|struct ReportRunInput|fn validate_report_preflight|async fn run_report_pipeline)\b" src-tauri/src/analysis/report.rs)
if ($widenedMatches.Count -ne 0) {
    $widenedMatches
    throw "report production item visibility was widened for tests"
}
```

Expected: no output and no throw.

Required test files exist:

```powershell
foreach ($file in @(
    "mod.rs",
    "harness.rs",
    "scope.rs",
    "capture.rs",
    "lifecycle.rs",
    "requests.rs",
    "phases.rs",
    "preflight.rs",
    "architecture.rs"
)) {
    $path = "src-tauri/src/analysis/report/tests/$file"
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "missing report test module: $path"
    }
}
```

`tests/mod.rs` declares every thematic module independently:

```powershell
foreach ($module in @("architecture", "capture", "harness", "lifecycle", "phases", "preflight", "requests", "scope")) {
    rg -n "^mod $module;$" src-tauri/src/analysis/report/tests/mod.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing report test module declaration: $module"
    }
}
```

No test module imports private production child modules directly:

```powershell
$privateChildMatches = @(rg -n "super::super::(capture|lifecycle|phases|requests)::|crate::analysis::report::(capture|lifecycle|phases|requests)::" src-tauri/src/analysis/report/tests)
if ($privateChildMatches.Count -ne 0) {
    $privateChildMatches
    throw "report tests must use parent facade access, not private child module paths"
}
```

Expected: no output and no throw.

`tests/mod.rs` stays wiring-only:

```powershell
$testModBodyMatches = @(rg -n "#\[tokio::test\]|#\[test\]|^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn)\b|use super::|use crate::" src-tauri/src/analysis/report/tests/mod.rs)
if ($testModBodyMatches.Count -ne 0) {
    $testModBodyMatches
    throw "report tests/mod.rs must contain module declarations only"
}
```

Expected: no output and no throw.

Test modules avoid parent and crate glob imports:

```powershell
$globImportMatches = @(rg -n "use\s+super::super::\*|use\s+crate::.*::\*" src-tauri/src/analysis/report/tests)
if ($globImportMatches.Count -ne 0) {
    $globImportMatches
    throw "report test modules must use explicit imports, not parent or crate glob imports"
}
```

Expected: no output and no throw.

Required tests moved to expected modules:

```powershell
$requiredTests = @{
    "scope.rs" = @(
        "report_run_input_carries_resolved_profile_snapshot",
        "telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match",
        "migrated_history_opt_in_rejects_non_telegram_analysis",
        "report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape",
        "chunk_target_chars_are_derived_from_model_input_limit_with_fallback"
    )
    "capture.rs" = @("capture_report_corpus_returns_reloaded_snapshot_before_provider_phases")
    "lifecycle.rs" = @(
        "interrupted_cleanup_preserves_captured_snapshot_state_marker",
        "request_analysis_run_cancel_missing_run_keeps_not_found_message",
        "request_analysis_run_cancel_completed_run_keeps_conflict_message",
        "request_analysis_run_cancel_running_but_inactive_keeps_conflict_message"
    )
    "requests.rs" = @(
        "extracts_json_with_text_before_and_after",
        "extracts_json_inside_markdown_fence",
        "parse_chunk_summary_ignores_non_json_prefix_with_braces",
        "parse_chunk_summary_rejects_malformed_payload",
        "build_map_request_keeps_run_scoped_request_and_profile",
        "build_reduce_request_keeps_run_scoped_request_and_profile"
    )
    "phases.rs" = @(
        "analysis_step_cancel_wrapper_allows_completed_future",
        "analysis_step_cancel_wrapper_interrupts_pending_future",
        "finish_map_phase_preserves_chunk_order_by_original_index",
        "finish_map_phase_rejects_missing_chunk_before_reduce",
        "finish_map_phase_propagates_map_error_without_starting_reduce"
    )
    "preflight.rs" = @(
        "validate_report_preflight_rejects_empty_corpus",
        "validate_report_preflight_rejects_oversized_runs",
        "validate_report_preflight_allows_runs_within_limits"
    )
    "architecture.rs" = @("analysis_report_workflow_file_has_no_tauri_command_adapters")
}
foreach ($entry in $requiredTests.GetEnumerator()) {
    foreach ($testName in $entry.Value) {
        $path = "src-tauri/src/analysis/report/tests/$($entry.Key)"
        $content = Get-Content -Raw $path
        $pattern = "(?s)#\[(tokio::test|test)\]\s*(async\s+fn|fn) $([regex]::Escape($testName))\b"
        if ($content -notmatch $pattern) {
            throw "missing report test $testName with test attribute in $path"
        }
    }
}
```

Expected: every required test is checked independently; the command throws on the first missing test.

## Testing

Run commands separately, not as one PowerShell block, unless using an explicit stopping wrapper that checks `$LASTEXITCODE`.

Baseline before editing:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This establishes crate-wide compile coverage before the test module-boundary refactor.

Post-change focused module slices:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::scope::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::capture::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::lifecycle::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::requests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::phases::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::preflight::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::architecture::
```

Expected for each focused module slice: pass in the default dev test profile and not a green `0 tests` run.

Post-change test inventory:

```powershell
$testList = cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests:: -- --list
if ($LASTEXITCODE -ne 0) {
    throw "report test inventory command failed"
}
foreach ($testPath in @(
    "analysis::report::tests::scope::report_run_input_carries_resolved_profile_snapshot",
    "analysis::report::tests::scope::telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match",
    "analysis::report::tests::scope::migrated_history_opt_in_rejects_non_telegram_analysis",
    "analysis::report::tests::scope::report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape",
    "analysis::report::tests::scope::chunk_target_chars_are_derived_from_model_input_limit_with_fallback",
    "analysis::report::tests::capture::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases",
    "analysis::report::tests::lifecycle::interrupted_cleanup_preserves_captured_snapshot_state_marker",
    "analysis::report::tests::lifecycle::request_analysis_run_cancel_missing_run_keeps_not_found_message",
    "analysis::report::tests::lifecycle::request_analysis_run_cancel_completed_run_keeps_conflict_message",
    "analysis::report::tests::lifecycle::request_analysis_run_cancel_running_but_inactive_keeps_conflict_message",
    "analysis::report::tests::requests::extracts_json_with_text_before_and_after",
    "analysis::report::tests::requests::extracts_json_inside_markdown_fence",
    "analysis::report::tests::requests::parse_chunk_summary_ignores_non_json_prefix_with_braces",
    "analysis::report::tests::requests::parse_chunk_summary_rejects_malformed_payload",
    "analysis::report::tests::requests::build_map_request_keeps_run_scoped_request_and_profile",
    "analysis::report::tests::requests::build_reduce_request_keeps_run_scoped_request_and_profile",
    "analysis::report::tests::phases::analysis_step_cancel_wrapper_allows_completed_future",
    "analysis::report::tests::phases::analysis_step_cancel_wrapper_interrupts_pending_future",
    "analysis::report::tests::phases::finish_map_phase_preserves_chunk_order_by_original_index",
    "analysis::report::tests::phases::finish_map_phase_rejects_missing_chunk_before_reduce",
    "analysis::report::tests::phases::finish_map_phase_propagates_map_error_without_starting_reduce",
    "analysis::report::tests::preflight::validate_report_preflight_rejects_empty_corpus",
    "analysis::report::tests::preflight::validate_report_preflight_rejects_oversized_runs",
    "analysis::report::tests::preflight::validate_report_preflight_allows_runs_within_limits",
    "analysis::report::tests::architecture::analysis_report_workflow_file_has_no_tauri_command_adapters"
)) {
    if ($testList -notmatch [regex]::Escape($testPath)) {
        throw "report test is missing from cargo test --list output: $testPath"
    }
}
```

Expected: every moved test path appears in Cargo's test inventory.

Post-change full report slice:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

Post-change compile and format:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: both pass.

Before accepting a test command as coverage, check that the output includes real tests for the intended module. A green `0 tests` run is a failure for every filtered command listed here.

## Commit Shape

Expected implementation commit:

- `refactor: split analysis report tests`

Expected files in that commit:

- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/report/tests/mod.rs`
- `src-tauri/src/analysis/report/tests/harness.rs`
- `src-tauri/src/analysis/report/tests/scope.rs`
- `src-tauri/src/analysis/report/tests/capture.rs`
- `src-tauri/src/analysis/report/tests/lifecycle.rs`
- `src-tauri/src/analysis/report/tests/requests.rs`
- `src-tauri/src/analysis/report/tests/phases.rs`
- `src-tauri/src/analysis/report/tests/preflight.rs`
- `src-tauri/src/analysis/report/tests/architecture.rs`

Do not include unrelated rustfmt drift. If `cargo fmt` changes unrelated Rust files, inspect the drift and either make a separate format-only commit or restore only implementation-owned formatting changes after review. The final implementation status should be clean except for explicitly pre-existing unrelated files.

## Open Questions

None. The design intentionally keeps production report workflow code in `report.rs` and only moves tests.
